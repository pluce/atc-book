'use client';

import { useState, useEffect, Suspense, useRef } from 'react';
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import Link from 'next/link';
import JSZip from 'jszip';
import { saveAs } from 'file-saver';
import { PDFDocument } from 'pdf-lib';
import { useTranslation } from 'react-i18next';
import '../lib/i18n';
import { fetchWithRetry, groupTags, getTagGroupKey } from '../lib/utils';
import { Chart, SavedDock } from '../types';
import { Notice } from '../lib/notices/types';
import { STATION_RULES, STATION_TAGS } from '../lib/constants';
import { ChartViewer } from '../components/ChartViewer';
import { Dock } from '../components/Dock';
import { SearchForm } from '../components/SearchForm';
import { ChartFilters } from '../components/ChartFilters';
import { ChartGrid } from '../components/ChartGrid';
import { ThemeToggle } from '../components/ThemeToggle';

function SearchPage() {
  const { t, i18n } = useTranslation();
  const searchParams = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();

  const [mounted, setMounted] = useState(false);
  const [icao, setIcao] = useState('');
  // Source is locally derived but we keep state if needed for UI later, currently automated
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [source, setSource] = useState('SIA');
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [merging, setMerging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [charts, setCharts] = useState<Chart[]>([]);
  const [notices, setNotices] = useState<Notice[]>([]);
  const [noticesLoading, setNoticesLoading] = useState(false);
  const [searchedIcao, setSearchedIcao] = useState('');
  const [selectedUrls, setSelectedUrls] = useState<Set<string>>(new Set());
  const [filterText, setFilterText] = useState('');
  const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());

  // FEATURE: Dock & Viewer State
  const [pinnedCharts, setPinnedCharts] = useState<Chart[]>([]);
  const [savedDocks, setSavedDocks] = useState<SavedDock[]>([]);
  const [activeDockId, setActiveDockId] = useState<string | null>(null);
  const [viewingChart, setViewingChart] = useState<Chart | null>(null);
  const [dockOpen, setDockOpen] = useState(false);
  const [dockSide, setDockSide] = useState<'bottom' | 'left' | 'right'>('bottom');
  
  // FEATURE: Scratchpad State
  const [scratchpadContent, setScratchpadContent] = useState('');
  
  // CACHE: Blob URLs for instant display
  const [blobCache, setBlobCache] = useState<Record<string, string>>({});
  const createdBlobUrls = useRef<Set<string>>(new Set());

  useEffect(() => {
    setMounted(true);
    // Load pinned charts from localStorage
    try {
      const saved = localStorage.getItem('pinnedCharts');
      if (saved) {
        const parsed = JSON.parse(saved);
        setPinnedCharts(parsed);
        if (Array.isArray(parsed) && parsed.length > 0) {
          setDockOpen(true);
        }
      }
      
      const savedSide = localStorage.getItem('dockSide');
      if (savedSide && ['bottom', 'left', 'right'].includes(savedSide)) {
          setDockSide(savedSide as any);
      }

      const savedNotes = localStorage.getItem('dockNotes');
      if (savedNotes) {
          setScratchpadContent(savedNotes);
      }

      const savedDocksStorage = localStorage.getItem('savedDocks');
      if (savedDocksStorage) {
        setSavedDocks(JSON.parse(savedDocksStorage));
      }

      const savedActiveDockId = localStorage.getItem('activeDockId');
      if (savedActiveDockId) {
          setActiveDockId(savedActiveDockId);
      }
    } catch (e) {
      console.error("Failed to load pinned charts or dock settings", e);
    }
  }, []);

  // Preload pinned charts into Blob Cache
  useEffect(() => {
    if (!mounted) return;
    
    pinnedCharts.forEach(async (chart) => {
        // If not already cached and not currently being viewed (viewing handles its own load if needed, but preloading ensures availability)
        if (!blobCache[chart.url]) {
            try {
                const proxyUrl = `/api/proxy?url=${encodeURIComponent(chart.url)}`;
                const res = await fetchWithRetry(proxyUrl);
                const blob = await res.blob();
                const objUrl = URL.createObjectURL(blob);
                createdBlobUrls.current.add(objUrl);
                setBlobCache(prev => ({ ...prev, [chart.url]: objUrl }));
            } catch (e) {
                console.error("Background preload failed for", chart.filename, e);
            }
        }
    });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pinnedCharts, mounted]); // Intentionally omitting blobCache to avoid loops

  // Cleanup blobs on unmount
  useEffect(() => {
      return () => {
          createdBlobUrls.current.forEach(url => URL.revokeObjectURL(url));
          createdBlobUrls.current.clear();
      };
  }, []);

  // Save pinned charts to localStorage whenever they change
  useEffect(() => {
    if (mounted) {
      localStorage.setItem('pinnedCharts', JSON.stringify(pinnedCharts));
      localStorage.setItem('dockSide', dockSide);
      localStorage.setItem('dockNotes', scratchpadContent);
      if (activeDockId) localStorage.setItem('activeDockId', activeDockId);
      else localStorage.removeItem('activeDockId');
    }
  }, [pinnedCharts, dockSide, scratchpadContent, activeDockId, mounted]);

  // AUTO-SAVE Scratchpad to Active Dock
  useEffect(() => {
      if (mounted && activeDockId) {
          setSavedDocks(prevDocks => {
              const dockIndex = prevDocks.findIndex(d => d.id === activeDockId);
              if (dockIndex === -1) return prevDocks; // Active dock deleted?

              const updatedDock = { ...prevDocks[dockIndex], notes: scratchpadContent };
              // Only update if notes actually changed to avoid unnecessary renders/loops (though React state updates handle shallow equality)
              if (prevDocks[dockIndex].notes === scratchpadContent) return prevDocks;

              const newDocks = [...prevDocks];
              newDocks[dockIndex] = updatedDock;
              localStorage.setItem('savedDocks', JSON.stringify(newDocks));
              return newDocks;
          });
      }
  }, [scratchpadContent, activeDockId, mounted]);

  // Auto-close dock when last chart is removed
  useEffect(() => {
    if (pinnedCharts.length === 0) {
      setDockOpen(false);
      // Optional: Should we clear active dock if we clear all charts?
      // User might want to keep notes context. Let's keep it for now.
    }
  }, [pinnedCharts]);

  // Keyboard Shortcuts
  useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
          if (e.key === 'Escape') closeViewer();
      };
      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
  };


  const loadAirport = async (code: string, tags?: Set<string>, filter?: string) => {
    if (code.length < 4) return;
    
    setLoading(true);
    setError(null);
    setCharts([]);
    setNotices([]);
    setNoticesLoading(true);
    setSearchedIcao('');
    setSelectedUrls(new Set());
    
    // Set filters immediately if provided, else reset them (except if tags/filter are explicitly passed as undefined it means keep them? No, we use explicit reset logic)
    // Here we assume if provided (initial load) we set them. 
    // If not provided (manual search), we reset them.
    if (tags !== undefined) setSelectedTags(tags);
    else setSelectedTags(new Set());
    
    if (filter !== undefined) setFilterText(filter);
    else setFilterText('');

    // Determine source based on ICAO prefix
    // Previously logic was in frontend, now moved to backend.
    // We just pass the ICAO and backend aggregates sources (SIA/ATLAS/UK/etc)

    try {
      const [resCharts, resNotices] = await Promise.all([
        fetch(`/api/charts?icao=${code}`),
        fetch(`/api/notices?icao=${code}`)
      ]);

      const dataCharts = await resCharts.json();
      const dataNotices = await resNotices.json();

      if (!resCharts.ok) {
        throw new Error(dataCharts.error || t('error_fetch'));
      }

      setCharts(dataCharts.charts.map((chart: Chart) => ({ ...chart, icao: code })));
      
      if (resNotices.ok && dataNotices.notices) {
        setNotices(dataNotices.notices);
      } else {
        console.error('Failed to fetch notices', dataNotices);
      }
      setNoticesLoading(false);

      // Select all by default, except those ending in _INSTR_XX.pdf
      const initialSelection = dataCharts.charts
        .filter((c: Chart) => !/_INSTR_\d{2}\.pdf$/i.test(c.filename))
        .map((c: Chart) => c.url);
      
      setSelectedUrls(new Set(initialSelection));
      setSearchedIcao(code);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    loadAirport(icao);
  };

  // Sync state to URL
  useEffect(() => {
    if (!mounted || !searchedIcao) return;

    const params = new URLSearchParams();
    params.set('icao', searchedIcao);
    
    if (selectedTags.size > 0) {
        params.set('tags', Array.from(selectedTags).join(','));
    }
    
    if (filterText) {
        params.set('q', filterText);
    }

    router.replace(`${pathname}?${params.toString()}`);
  }, [searchedIcao, selectedTags, filterText, mounted, pathname, router]);

  // Initial load from URL
  useEffect(() => {
     if (!searchParams) return;
     const urlIcao = searchParams.get('icao');
     const urlTags = searchParams.get('tags');
     const urlQ = searchParams.get('q');

     if (urlIcao && !searchedIcao && !loading) {
         const tagsSet = urlTags ? new Set(urlTags.split(',')) : new Set<string>();
         loadAirport(urlIcao, tagsSet, urlQ || '');
     }
     // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams]); // Dependent on searchParams to trigger on mount/nav

  const toggleChart = (url: string) => {
    const newSelected = new Set(selectedUrls);
    if (newSelected.has(url)) {
      newSelected.delete(url);
    } else {
      newSelected.add(url);
    }
    setSelectedUrls(newSelected);
  };

  // FEATURE: Pinning Logic
  const isPinned = (chart: Chart) => pinnedCharts.some(p => p.url === chart.url);

  const togglePin = (chart: Chart, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    
    if (isPinned(chart)) {
      setPinnedCharts(pinnedCharts.filter(p => p.url !== chart.url));
    } else {
      setPinnedCharts([...pinnedCharts, chart]);
      // Auto-open dock if closed when adding
      if (!dockOpen) setDockOpen(true);
    }
  };

  const openViewer = (chart: Chart) => {
    setViewingChart(chart);
  };

  const closeViewer = () => {
    setViewingChart(null);
  };

  const cycleDockSide = () => {
      const sides: ('bottom' | 'left' | 'right')[] = ['bottom', 'left', 'right'];
      const nextIndex = (sides.indexOf(dockSide) + 1) % sides.length;
      setDockSide(sides[nextIndex]);
  };

  const toggleGroup = (chartsInGroup: Chart[]) => {
    const allSelected = chartsInGroup.every(c => selectedUrls.has(c.url));
    const newSelected = new Set(selectedUrls);
    
    if (allSelected) {
      chartsInGroup.forEach(c => newSelected.delete(c.url));
    } else {
      chartsInGroup.forEach(c => newSelected.add(c.url));
    }
    setSelectedUrls(newSelected);
  };

  const toggleTag = (tag: string) => {
    const newTags = new Set(selectedTags);
    
    if (newTags.has(tag)) {
        newTags.delete(tag);
    } else {
        newTags.add(tag);
    }
    setSelectedTags(newTags);
  };

  // Filter charts
  const filteredCharts = charts.filter(chart => {
    const searchStr = filterText.toLowerCase();
    
    // Safety check for properties, though they should be strings
    const category = (chart.category || '').toLowerCase();
    const subtitle = (chart.subtitle || '').toLowerCase();
    const filename = (chart.filename || '').toLowerCase();

    const matchesText = category.includes(searchStr) || 
           subtitle.includes(searchStr) ||
           filename.includes(searchStr);

    // Group logic: OR within group, AND between groups
    if (selectedTags.size === 0) return matchesText;

    const activeTagsByGroup = {
      'group_stations': [] as string[],
      'group_runways': [] as string[],
      'group_approaches': [] as string[],
      'group_phases': [] as string[],
      'group_others': [] as string[]
    };

    Array.from(selectedTags).forEach(tag => {
      const g = getTagGroupKey(tag) as keyof typeof activeTagsByGroup;
      if (activeTagsByGroup[g]) activeTagsByGroup[g].push(tag);
      else activeTagsByGroup['group_others'].push(tag);
    });

    // Special handling for Station tags (filter by category, not tags)
    if (activeTagsByGroup['group_stations'].length > 0) {
        // OR logic: matches if ANY of the selected station tags allows this category
        const matchesStation = activeTagsByGroup['group_stations'].some(stationTag => {
            const allowedCategories = STATION_RULES[stationTag] || [];
            return allowedCategories.includes(chart.category);
        });
        
        if (!matchesStation) return false;
    }

    const matchesGroups = Object.keys(activeTagsByGroup).every(key => {
      const groupKey = key as keyof typeof activeTagsByGroup;
      if (groupKey === 'group_stations') return true; // Handled separately above

      const groupTags = activeTagsByGroup[groupKey];
      if (groupTags.length === 0) return true; // No filter for this group -> Pass
      // OR logic within group: chart must have AT LEAST ONE of the tags in this group
      return groupTags.some((t: string) => chart.tags?.includes(t));
    });

    return matchesText && matchesGroups;
  });

  const handleSelectVisible = (select: boolean) => {
    const newSelected = new Set(selectedUrls);
    filteredCharts.forEach(c => {
      if (select) newSelected.add(c.url);
      else newSelected.delete(c.url);
    });
    setSelectedUrls(newSelected);
  };

  const handleDownloadSelected = async () => {
    const chartsToDownload = filteredCharts.filter(c => selectedUrls.has(c.url));
    if (chartsToDownload.length === 0) return;
    
    setDownloading(true);
    try {
      const zip = new JSZip();
      
      const downloadPromises = chartsToDownload.map(async (chart) => {
        try {
          // Use our proxy to avoid CORS issues
          const proxyUrl = `/api/proxy?url=${encodeURIComponent(chart.url)}`;
          const response = await fetchWithRetry(proxyUrl);
          
          const blob = await response.blob();
          zip.file(chart.filename, blob);
        } catch (e) {
          console.error(`Failed to download ${chart.filename}`, e);
          zip.file(`${chart.filename}.error.txt`, "Could not download file.");
        }
      });

      await Promise.all(downloadPromises);
      
      const content = await zip.generateAsync({ type: 'blob' });
      saveAs(content, `Cartes_${searchedIcao}_selection.zip`);
      
    } catch (err) {
      console.error('Error creating zip:', err);
      setError(t('error_zip'));
    } finally {
      setDownloading(false);
    }
  };

  const handleMergeSelected = async () => {
    const chartsToDownload = filteredCharts.filter(c => selectedUrls.has(c.url));
    if (chartsToDownload.length === 0) return;
    
    setMerging(true);
    try {
      const mergedPdf = await PDFDocument.create();
      
      // Process strictly in order to maintain a logical document structure
      const sortedCharts = [...chartsToDownload].sort((a, b) => {
         if (a.category !== b.category) return a.category.localeCompare(b.category);
         if (a.subtitle !== b.subtitle) return a.subtitle.localeCompare(b.subtitle);
         return (a.filename || '').localeCompare(b.filename || '');
      });

      for (const chart of sortedCharts) {
        try {
          // Use our proxy to avoid CORS issues
          const proxyUrl = `/api/proxy?url=${encodeURIComponent(chart.url)}`;
          const response = await fetchWithRetry(proxyUrl);
          
          const arrayBuffer = await response.arrayBuffer();
          const pdf = await PDFDocument.load(arrayBuffer);
          const copiedPages = await mergedPdf.copyPages(pdf, pdf.getPageIndices());
          copiedPages.forEach((page) => mergedPdf.addPage(page));
        } catch (e) {
          console.error(`Error processing ${chart.filename}`, e);
        }
      }
      
      const pdfBytes = await mergedPdf.save();
      const blob = new Blob([pdfBytes as any], { type: 'application/pdf' });
      saveAs(blob, `Cartes_${searchedIcao}_complet.pdf`);

    } catch (err) {
      console.error('Error merging PDF:', err);
      setError(t('error_merge'));
    } finally {
      setMerging(false);
    }
  };

  const handlePinSelected = () => {
    const chartsToPin = filteredCharts.filter(c => selectedUrls.has(c.url));
    if (chartsToPin.length === 0) return;

    setPinnedCharts(prev => {
        const newPins = [...prev];
        chartsToPin.forEach(chart => {
            if (!newPins.some(p => p.url === chart.url)) {
                newPins.push(chart);
            }
        });
        return newPins;
    });

    if (!dockOpen) setDockOpen(true);
  };

  const handleRenameChart = (chart: Chart, newName: string) => {
    // Update local state (pinned charts)
    setPinnedCharts(prev => prev.map(p => 
      p.url === chart.url ? { ...p, customTitle: newName } : p
    ));
  };

  const handleSaveDock = (name: string) => {
    if (!name.trim() || pinnedCharts.length === 0) return;
    
    // Check if we are overwriting the active dock (by name or ID)
    // Constraint: We update the Active Dock if names match, OR create new.
    // If activeDockId is set, and user kept the name, we update.
    
    let targetId = Date.now().toString();
    
    // Find if a dock with this name already exists
    const existingIndex = savedDocks.findIndex(d => d.name === name.trim());
    
    let newDocks = [...savedDocks];
    
    if (existingIndex !== -1) {
        // Overwrite existing
        targetId = savedDocks[existingIndex].id;
        newDocks[existingIndex] = {
            ...newDocks[existingIndex],
            charts: [...pinnedCharts],
            notes: scratchpadContent, // Enforce current notes on save
            timestamp: Date.now()
        };
    } else {
        // Create new
        const newSave: SavedDock = {
          id: targetId,
          name: name.trim(),
          charts: [...pinnedCharts],
          notes: scratchpadContent,
          timestamp: Date.now()
        };
        newDocks = [newSave, ...savedDocks];
    }
    
    setSavedDocks(newDocks);
    localStorage.setItem('savedDocks', JSON.stringify(newDocks));
    setActiveDockId(targetId);
  };

  const handleDeleteDock = (id: string) => {
    const newDocks = savedDocks.filter(d => d.id !== id);
    setSavedDocks(newDocks);
    localStorage.setItem('savedDocks', JSON.stringify(newDocks));
    if (activeDockId === id) setActiveDockId(null);
  };

  // Group charts by category
  const groupedCharts = filteredCharts.reduce((groups, chart) => {
    if (!groups[chart.category]) {
      groups[chart.category] = [];
    }
    groups[chart.category].push(chart);
    return groups;
  }, {} as Record<string, Chart[]>);

  const availableTags = Array.from(new Set(charts.flatMap(c => c.tags || []))).sort();
  const groupedTags = groupTags(availableTags);

  if (!mounted) return null; // Avoid hydration mismatch

  // Layout calculations for Viewer and Main Content
  const dockVisible = pinnedCharts.length > 0 || savedDocks.length > 0;

  // Padding for main content so it doesn't get hidden by dock
  const mainContentStyle = {
      paddingBottom: (dockVisible && dockSide === 'bottom') ? (dockOpen ? '8rem' : '4rem') : '2rem',
      paddingLeft: (dockVisible && dockOpen && dockSide === 'left') ? '8rem' : '2rem', // Apply left padding if docked left
      paddingRight: (dockVisible && dockOpen && dockSide === 'right') ? '8rem' : '2rem', // Apply right padding if docked right (though content is centered usually)
  };

  // Position for Viewer
  const viewerStyle = {
      bottom: (dockVisible && dockOpen && dockSide === 'bottom') ? '8rem' : '0',
      left: (dockVisible && dockOpen && dockSide === 'left') ? '8rem' : '0',
      right: (dockVisible && dockOpen && dockSide === 'right') ? '8rem' : '0',
      top: 0
  };


  return (
    <main className="min-h-screen bg-background text-foreground font-sans relative flex flex-col transition-all duration-300"> 
      {/* Viewer Modal - Now respects dock position */}
      {viewingChart && (
        <ChartViewer 
          chart={viewingChart}
          onClose={closeViewer}
          blobUrl={blobCache[viewingChart.url]}
          style={viewerStyle}
        />
      )}

      {/* Main Content Wrapper */}
      <div 
        className="flex-1 transition-all duration-300 flex flex-col"
        style={mainContentStyle}
      >
      <div className="absolute top-4 left-4 z-40 flex gap-2 items-center">
           <ThemeToggle />
           <div className="h-6 w-px bg-border mx-1"></div>
           <button 
             onClick={() => changeLanguage('fr')} 
             data-testid="lang-fr"
             className={`text-xl px-2 py-1 rounded border transition-colors ${i18n.language === 'fr' ? 'bg-primary border-primary' : 'bg-card border-border hover:bg-secondary'}`}
             title="Français"
           >
             🇫🇷
           </button>
           <button 
             onClick={() => changeLanguage('en')} 
             data-testid="lang-en"
             className={`text-xl px-2 py-1 rounded border transition-colors ${i18n.language === 'en' ? 'bg-primary border-primary' : 'bg-card border-border hover:bg-secondary'}`}
             title="English"
           >
             🇬🇧
           </button>
      </div>

      <div className="max-w-6xl mx-auto space-y-8 w-full flex-1">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-8 pt-8">
          <header className="text-center lg:text-left space-y-4 lg:w-1/3">
            <h1 className="text-6xl font-black font-mono tracking-tighter bg-gradient-to-r from-blue-800 to-indigo-900 dark:from-blue-400 dark:to-indigo-500 bg-clip-text text-transparent drop-shadow-sm">
              ATC BOOK
            </h1>
            <p className="text-muted-foreground font-medium text-lg">
              {t('subtitle')}
            </p>
          </header>

          <div className="w-full lg:w-2/3 lg:max-w-2xl">
            <SearchForm 
              icao={icao}
              setIcao={setIcao}
              onSubmit={handleSubmit}
              loading={loading}
              error={error}
            />
          </div>
        </div>

        {searchedIcao && !loading && !error && (
          <section className="space-y-8 animate-fade-in">
            <ChartFilters 
              searchedIcao={searchedIcao}
              visibleCount={filteredCharts.length}
              selectedCount={selectedUrls.size}
              filterText={filterText}
              setFilterText={setFilterText}
              availableTags={availableTags}
              groupedTags={groupedTags}
              selectedTags={selectedTags}
              toggleTag={toggleTag}
              onSelectVisible={handleSelectVisible}
              onPinSelected={handlePinSelected}
              onMergeSelected={handleMergeSelected}
              onDownloadSelected={handleDownloadSelected}
              merging={merging}
              downloading={downloading}
            />

            <ChartGrid 
              groupedCharts={groupedCharts}
              selectedUrls={selectedUrls}
              toggleChart={toggleChart}
              toggleGroup={toggleGroup}
              isPinned={isPinned}
              togglePin={togglePin}
              openViewer={openViewer}
            />
          </section>
        )}
      </div>

      <footer className="mt-12 py-6 border-t border-border">
        <div className="container mx-auto px-4 flex flex-col md:flex-row items-center justify-between gap-4 text-muted-foreground text-sm">
          <div className="flex items-center gap-2">
            <span>{t('footer_credits')}</span>
            <a 
              href="https://youtube.com/channel/UCoeiQSBuqp3oFpK16nQT1_Q/" 
              target="_blank" 
              rel="noopener noreferrer"
              className="font-semibold text-primary hover:text-primary/80 flex items-center gap-1 transition-colors"
            >
              Stardust Citizen
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16" className="inline ml-1">
                <path d="M8.051 1.999h.089c.822.003 4.987.033 6.11.335a2.01 2.01 0 0 1 1.415 1.42c.101.38.172.883.22 1.402l.01.104.022.26.008.104c.065.914.073 1.77.074 1.957v.075c-.001.194-.01 1.108-.082 2.06l-.008.105-.009.104c-.05.572-.124 1.14-.235 1.558a2.007 2.007 0 0 1-1.415 1.42c-1.16.312-5.569.334-6.18.335h-.142c-.309 0-1.587-.006-2.927-.052l-.17-.006-.087-.004-.171-.007-.171-.007c-1.11-.049-2.167-.128-2.654-.26a2.007 2.007 0 0 1-1.415-1.419c-.111-.417-.185-.986-.235-1.558L.09 9.82l-.008-.104A31.4 31.4 0 0 1 0 7.68v-.123c.002-.215.01-.958.064-1.778l.007-.103.003-.052.008-.104.022-.26.01-.104c.048-.519.119-1.023.22-1.402a2.007 2.007 0 0 1 1.415-1.42c.487-.13 1.544-.21 2.654-.26l.17-.007.172-.006.086-.003.171-.007A99.788 99.788 0 0 1 7.858 2h.193zM6.4 5.209v4.818l4.157-2.408L6.4 5.209z"/>
              </svg>
            </a>
            <span className="text-border mx-2">|</span>
            <Link href="/help" data-testid="footer-help-link" className="hover:text-primary hover:underline transition-colors flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {t('footer_help')}
            </Link>
          </div>
          
          <div className="flex gap-4 text-xs">
             <div className="px-3 py-1 bg-card rounded-full border border-border font-mono text-primary">
                {process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'Unknown'} / {process.env.NEXT_PUBLIC_AIRAC_DATE || 'Unknown'}
             </div>
          </div>
        </div>
      </footer>
      </div>

      <Dock 
        charts={pinnedCharts}
        notices={notices}
        onRemoveChart={(chart) => setPinnedCharts(prev => prev.filter(p => p.url !== chart.url))}
        onClear={() => {
             setPinnedCharts([]);
             setActiveDockId(null);
             setScratchpadContent('');
        }}
        onRestore={(dock) => {
            setPinnedCharts(dock.charts);
            // Important: Set content, defaulting to empty string if undefined to clear previous context
            setScratchpadContent(dock.notes || '');
            setActiveDockId(dock.id);
            setDockOpen(true);
        }}
        savedDocks={savedDocks}
        onSaveDock={handleSaveDock}
        onDeleteDock={handleDeleteDock}
        isOpen={dockOpen}
        onToggleOpen={() => setDockOpen(!dockOpen)}
        side={dockSide}
        onCycleSide={cycleDockSide}
        viewingChart={viewingChart}
        onViewChart={openViewer}
        currentIcao={searchedIcao}
        currentTags={Array.from(selectedTags)}
        scratchpadContent={scratchpadContent}
        onUpdateScratchpad={setScratchpadContent}
        onRenameChart={handleRenameChart}
        activeDockId={activeDockId}
      />

    </main>
  );
}

export default function Home() {
  return (
    <Suspense fallback={<div className="min-h-screen bg-background flex items-center justify-center text-muted-foreground">Loading...</div>}>
      <SearchPage />
    </Suspense>
  );
}
