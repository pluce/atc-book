'use client';

import { useState, useEffect, Suspense } from 'react';
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import JSZip from 'jszip';
import { saveAs } from 'file-saver';
import { PDFDocument } from 'pdf-lib';
import { useTranslation } from 'react-i18next';
import '../lib/i18n';

// Helper for retrying requests with exponential backoff & jitter
async function fetchWithRetry(url: string, retries = 3, baseDelay = 1000): Promise<Response> {
  let lastError: unknown;
  
  for (let i = 0; i <= retries; i++) {
    try {
      const response = await fetch(url);
      if (response.ok) return response;
      throw new Error(`HTTP error! status: ${response.status}`);
    } catch (error) {
      lastError = error;
      if (i < retries) {
        // Backoff: base * 2^attempt + random jitter (0-1000ms)
        const delay = (baseDelay * Math.pow(2, i)) + (Math.random() * 1000);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  throw lastError;
}

type Chart = {
  category: string;
  subtitle: string;
  filename: string;
  url: string;
  page?: string;
  tags?: string[];
};

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
  const [searchedIcao, setSearchedIcao] = useState('');
  const [selectedUrls, setSelectedUrls] = useState<Set<string>>(new Set());
  const [filterText, setFilterText] = useState('');
  const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set());
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const changeLanguage = (lng: string) => {
    i18n.changeLanguage(lng);
  };

  const STATION_TAGS = ['DEL', 'GND', 'TWR', 'APP', 'DEP'];
  const STATION_RULES: Record<string, string[]> = {
    'DEL': ["PARKING", "AERODROME", "SID"],
    'GND': ["PARKING", "AERODROME", "GROUND"],
    'TWR': ["GROUND", "AERODROME", "IAC", "SID", "VAC", "VLC"],
    'APP': ["STAR", "IAC"],
    'DEP': ["SID"]
  };

  const loadAirport = async (code: string, tags?: Set<string>, filter?: string) => {
    if (code.length < 4) return;
    
    setLoading(true);
    setError(null);
    setCharts([]);
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
    const derivedSource = code.toUpperCase().startsWith('EG') ? 'UK' : 'SIA';
    setSource(derivedSource);

    try {
      const res = await fetch(`/api/charts?icao=${code}&source=${derivedSource}`);
      const data = await res.json();

      if (!res.ok) {
        throw new Error(data.error || t('error_fetch'));
      }

      setCharts(data.charts);
      // Select all by default, except those ending in _INSTR_XX.pdf
      const initialSelection = data.charts
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

  const getTagGroupKey = (tag: string) => {
    if (STATION_TAGS.includes(tag)) return 'group_stations';
    if (/^\d{2}[LRC]?$/.test(tag)) return 'group_runways';
    if (tag.startsWith('App.')) return 'group_phases';
    if (['ILS', 'LOC', 'RNAV', 'RNP', 'VPT', 'MVL', 'Nuit', 'DME'].some(t => tag.includes(t))) return 'group_approaches';
    return 'group_others';
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

  const groupTags = (tags: string[]) => {
    const groups: Record<string, string[]> = {
      'group_stations': [...STATION_TAGS],
      'group_runways': [],
      'group_approaches': [],
      'group_phases': [],
      'group_others': []
    };
    
    tags.forEach(tag => {
      const g = getTagGroupKey(tag);
      if (groups[g]) groups[g].push(tag);
      else groups['group_others'].push(tag);
    });
    
    // Sort specific groups
    groups['group_runways'].sort(); // Keep runways sorted alphanumerically
    
    return groups;
  };

  const getCategoryLabel = (category: string) => {
    const map: Record<string, string> = {
      "PARKING": "cat_parking",
      "AERODROME": "cat_aerodrome",
      "GROUND": "cat_ground_movements",
      "IAC": "cat_instrument_approach",
      "SID": "cat_sid",
      "STAR": "cat_star",
      "VAC": "VAC",
      "VLC": "VLC",
      "TEM": "TEM"
    };

    return map[category] ? t(map[category]) : category;
  };

  const getTagLabel = (tag: string) => {
    if (tag === 'App. Finale') return t('tag_app_final');
    if (tag === 'App. Initiale') return t('tag_app_initial');
    if (tag === 'Nuit') return t('tag_night');
    return tag;
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
      // Sorting: Category then Subtitle then Page
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
          // Continue with next file
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

  return (
    <main className="min-h-screen bg-slate-900 text-slate-100 p-8 font-sans relative">
      <div className="absolute top-4 left-4 z-50 flex gap-2">
           <button 
             onClick={() => changeLanguage('fr')} 
             className={`text-xl px-2 py-1 rounded border transition-colors ${i18n.language === 'fr' ? 'bg-blue-600 border-blue-500' : 'bg-slate-800 border-slate-700 hover:bg-slate-700'}`}
             title="Français"
           >
             🇫🇷
           </button>
           <button 
             onClick={() => changeLanguage('en')} 
             className={`text-xl px-2 py-1 rounded border transition-colors ${i18n.language === 'en' ? 'bg-blue-600 border-blue-500' : 'bg-slate-800 border-slate-700 hover:bg-slate-700'}`}
             title="English"
           >
             🇬🇧
           </button>
      </div>

      <div className="max-w-6xl mx-auto space-y-8">
        <header className="text-center space-y-4">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-blue-400 to-indigo-400 bg-clip-text text-transparent">
            ATC BOOK
          </h1>
          <p className="text-slate-400">
            {t('subtitle')}
          </p>
        </header>

        <section className="bg-slate-800 p-6 rounded-2xl shadow-xl border border-slate-700 max-w-2xl mx-auto">
          <form onSubmit={handleSubmit} className="flex gap-4 items-end sm:items-stretch flex-col sm:flex-row">
            <div className="flex-1 space-y-2 w-full">
              <label htmlFor="icao" className="block text-sm font-medium text-slate-300">
                {t('search_label')}
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  id="icao"
                  value={icao}
                  onChange={(e) => setIcao(e.target.value.toUpperCase())}
                  className="flex-1 w-full bg-slate-900 border border-slate-600 rounded-lg px-4 py-3 text-lg focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all placeholder-slate-600"
                  placeholder={t('search_placeholder')}
                  maxLength={4}
                  required
                />
              </div>
            </div>
            <button
              type="submit"
              disabled={loading || icao.length < 4}
              className="bg-blue-600 hover:bg-blue-500 text-white font-semibold py-3 px-8 rounded-lg shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center min-w-[150px] w-full sm:w-auto"
            >
              {loading ? (
                <span className="animate-pulse">{t('searching')}</span>
              ) : (
                t('search_button')
              )}
            </button>
          </form>

          {error && (
            <div className="mt-6 p-4 bg-red-900/50 border border-red-700 text-red-200 rounded-lg animate-fade-in text-center">
              ⚠️ {error}
            </div>
          )}
        </section>

        {searchedIcao && !loading && !error && (
          <section className="space-y-8 animate-fade-in">
            <div className="sticky top-4 z-10 flex flex-col gap-2 md:gap-4">
                <div className="bg-slate-800/90 backdrop-blur-md border border-slate-700/50 rounded-xl shadow-2xl overflow-hidden transition-all duration-300">
                    <div className="p-4 flex items-center justify-between gap-4">
                        <div className="flex-1 min-w-0">
                            <h2 className="text-xl md:text-2xl font-bold text-white truncate">
                            {t('results_title')} <span className="text-blue-400">{searchedIcao}</span>
                            </h2>
                            <div className="flex items-center gap-2 text-xs md:text-sm text-slate-400 mt-1">
                                <span>
                                {t('visible_charts_plural', { count: filteredCharts.length })}
                                </span>
                                <span className="hidden md:inline">|</span>
                                <span className="text-blue-300 font-medium">
                                {t('selected_charts_plural', { count: selectedUrls.size })}
                                </span>
                            </div>
                        </div>

                        <div className="hidden md:flex items-center gap-3">
                            <div className="flex items-center gap-2">
                                <button
                                    onClick={() => handleSelectVisible(true)}
                                    className="px-3 py-1.5 text-xs font-medium text-blue-300 bg-blue-900/30 hover:bg-blue-900/50 border border-blue-800/50 rounded-lg transition-colors whitespace-nowrap"
                                >
                                    {t('select_all')}
                                </button>
                                <button
                                    onClick={() => handleSelectVisible(false)}
                                    className="px-3 py-1.5 text-xs font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition-colors whitespace-nowrap"
                                >
                                    {t('deselect_all')}
                                </button>
                            </div>
                            
                            <div className="h-6 w-px bg-slate-700 mx-1"></div>

                            <div className="flex gap-2">
                                <button
                                    onClick={handleMergeSelected}
                                    disabled={merging || selectedUrls.size === 0}
                                    className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
                                >
                                    {merging ? (
                                        <>
                                        <svg className="animate-spin h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                        <span>{t('merging')}</span>
                                        </>
                                    ) : (
                                        <>
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                            <path fillRule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clipRule="evenodd" />
                                        </svg>
                                        <span>{t('merge_button')}</span>
                                        </>
                                    )}
                                </button>

                                <button
                                onClick={handleDownloadSelected}
                                disabled={downloading || selectedUrls.size === 0}
                                className="flex items-center gap-2 bg-emerald-600 hover:bg-emerald-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
                                >
                                {downloading ? (
                                    <>
                                    <svg className="animate-spin h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    <span>{t('zipping')}</span>
                                    </>
                                ) : (
                                    <>
                                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
                                    </svg>
                                    <span>{t('zip_button')}</span>
                                    </>
                                )}
                                </button>
                            </div>
                        </div>

                        <button 
                            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
                            className="md:hidden p-2 text-slate-400 hover:text-white bg-slate-700/50 rounded-lg hover:bg-slate-700 transition-colors"
                        >
                            {mobileMenuOpen ? (
                                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
                                </svg>
                            ) : (
                                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                                </svg>
                            )}
                        </button>
                    </div>

                    <div className={`${mobileMenuOpen ? 'max-h-[80vh] opacity-100 overflow-y-auto' : 'max-h-0 opacity-0 overflow-hidden'} md:max-h-none md:opacity-100 md:overflow-visible transition-all duration-300 ease-in-out border-t border-slate-700/50 md:border-none bg-slate-900/50 md:bg-transparent`}>
                        <div className="p-4 pt-2 md:p-4 md:pt-0 space-y-4">
                            
                            {/* Action Buttons */}
                            <div className="flex flex-wrap items-center gap-3 justify-end border-b md:border-none border-slate-700/50 pb-4 md:pb-0 md:hidden">
                                <div className="flex items-center gap-2">
                                    <button
                                        onClick={() => handleSelectVisible(true)}
                                        className="px-3 py-1.5 text-xs font-medium text-blue-300 bg-blue-900/30 hover:bg-blue-900/50 border border-blue-800/50 rounded-lg transition-colors whitespace-nowrap"
                                    >
                                        {t('select_all')}
                                    </button>
                                    <button
                                        onClick={() => handleSelectVisible(false)}
                                        className="px-3 py-1.5 text-xs font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition-colors whitespace-nowrap"
                                    >
                                        {t('deselect_all')}
                                    </button>
                                </div>
                                
                                <div className="h-6 w-px bg-slate-700 mx-1 hidden md:block"></div>

                                <div className="flex gap-2 w-full md:w-auto">
                                    <button
                                        onClick={handleMergeSelected}
                                        disabled={merging || selectedUrls.size === 0}
                                        className="flex-1 md:flex-none flex items-center justify-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
                                    >
                                        {merging ? (
                                            <>
                                            <svg className="animate-spin h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            <span>{t('merging')}</span>
                                            </>
                                        ) : (
                                            <>
                                            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                                <path fillRule="evenodd" d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2H6a2 2 0 01-2-2V4z" clipRule="evenodd" />
                                            </svg>
                                            <span>{t('merge_button')}</span>
                                            </>
                                        )}
                                    </button>

                                    <button
                                    onClick={handleDownloadSelected}
                                    disabled={downloading || selectedUrls.size === 0}
                                    className="flex-1 md:flex-none flex items-center justify-center gap-2 bg-emerald-600 hover:bg-emerald-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
                                    >
                                    {downloading ? (
                                        <>
                                        <svg className="animate-spin h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                        <span>{t('zipping')}</span>
                                        </>
                                    ) : (
                                        <>
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                            <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
                                        </svg>
                                        <span>{t('zip_button')}</span>
                                        </>
                                    )}
                                    </button>
                                </div>
                            </div>

                            {/* Filter Input */}
                            <div className="relative">
                                <input
                                    type="text"
                                    placeholder={t('filter_placeholder')}
                                    value={filterText}
                                    onChange={(e) => setFilterText(e.target.value)}
                                    className="w-full bg-slate-900/50 md:bg-slate-800/90 border border-slate-600 rounded-lg pl-10 pr-4 py-3 focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all placeholder-slate-500 text-slate-200"
                                />
                                <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                                </svg>
                                {filterText && (
                                    <button 
                                        onClick={() => setFilterText('')}
                                        className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-white"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                        </svg>
                                    </button>
                                )}
                            </div>

                            {/* Tags Filter */}
                            {(availableTags.length > 0) && (
                                <div className="flex flex-col gap-3 rounded-xl">
                                    <div className="flex flex-wrap items-center gap-2">
                                    {/* Render groups in order */}
                                    {['group_stations', 'group_runways', 'group_approaches', 'group_phases', 'group_others'].map((groupKey, idx, arr) => {
                                        const tags = groupedTags[groupKey];
                                        if (!tags || tags.length === 0) return null;
                                        
                                        return (
                                            <div key={groupKey} className="flex flex-wrap items-center gap-2">
                                                <span className="text-[10px] text-slate-500 uppercase font-bold tracking-wider mr-1">
                                                    {t(groupKey)}
                                                </span>
                                                {tags.map((tag: string) => {
                                                    const isSelected = selectedTags.has(tag);
                                                    return (
                                                        <button 
                                                        key={tag}
                                                        onClick={() => toggleTag(tag)}
                                                        className={`px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all select-none
                                                            ${isSelected 
                                                            ? 'bg-blue-600 border-blue-500 text-white shadow-lg shadow-blue-900/50' 
                                                            : 'bg-slate-800 border-slate-700 text-slate-400 hover:bg-slate-700 hover:text-slate-200 hover:border-slate-600'}
                                                        `}
                                                        >
                                                        {getTagLabel(tag)}
                                                        </button>
                                                    );
                                                })}
                                                {/* Separator if not last group */}
                                                {idx < arr.length - 1 && groupedTags[arr[idx+1]]?.length > 0 && (
                                                    <div className="w-px h-6 bg-slate-600 mx-2 hidden md:block"></div>
                                                )}
                                            </div>
                                        );
                                    })}
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {filteredCharts.length > 0 ? (
              <div className="space-y-10">
                {Object.entries(groupedCharts).map(([category, groupCharts]) => (
                  <div key={category} className="space-y-4">
                    <div className="flex items-center gap-3 border-b border-slate-700 pb-2">
                       <input 
                          type="checkbox"
                          checked={groupCharts.every(c => selectedUrls.has(c.url))}
                          onChange={() => toggleGroup(groupCharts)}
                          className="w-5 h-5 rounded border-slate-600 text-blue-600 focus:ring-blue-500 bg-slate-800 cursor-pointer"
                        />
                      <h3 className="text-xl font-semibold text-blue-300">{getCategoryLabel(category)}</h3>
                      <span className="text-sm text-slate-500 bg-slate-800 px-2 py-0.5 rounded-full">
                        {groupCharts.length}
                      </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                      {groupCharts.map((chart, index) => {
                        const isSelected = selectedUrls.has(chart.url);
                        return (
                          <div
                            key={index}
                            onClick={(e) => {
                                // Toggle selection if clicking card (unless clicking link)
                                if (!(e.target as HTMLElement).closest('a') && !(e.target as HTMLElement).closest('input')) {
                                    toggleChart(chart.url);
                                }
                            }}
                            className={`group border p-4 rounded-xl transition-all duration-200 flex flex-col justify-between h-full hover:shadow-lg cursor-pointer select-none
                              ${isSelected 
                                ? 'bg-slate-800 border-blue-500/50 shadow-blue-900/10' 
                                : 'bg-slate-800/50 border-slate-700/50 hover:bg-slate-800 hover:border-slate-600'}
                            `}
                          >
                            <div className="flex items-start gap-3 mb-2">
                              <input 
                                type="checkbox"
                                checked={isSelected}
                                onChange={() => toggleChart(chart.url)}
                                className="mt-1 w-4 h-4 rounded border-slate-600 text-blue-600 focus:ring-blue-500 bg-slate-900/50 cursor-pointer"
                              />
                              <div className="flex-1 min-w-0">
                                <div className="flex justify-between items-start gap-2">
                                  <h4 className={`font-semibold leading-snug truncate ${isSelected ? 'text-white' : 'text-slate-300'} group-hover:text-blue-200`}>
                                  {(!chart.subtitle || chart.subtitle.toLowerCase().trim() === chart.category.toLowerCase().trim()) 
                                    ? getCategoryLabel(chart.category) 
                                    : chart.subtitle}
                                  </h4>
                                  {chart.page && (
                                      <span className="text-[10px] bg-slate-700 text-slate-300 px-1.5 py-0.5 rounded font-mono border border-slate-600 whitespace-nowrap">
                                          {chart.page}
                                      </span>
                                  )}
                                </div>
                                
                                {chart.tags && chart.tags.length > 0 && (
                                  <div className="flex flex-wrap gap-1 mt-1.5">
                                    {chart.tags.map(tag => (
                                      <span key={tag} className="text-[9px] uppercase tracking-wide font-semibold bg-slate-700/50 text-slate-400 px-1.5 py-0.5 rounded border border-slate-700/50">
                                        {getTagLabel(tag)}
                                      </span>
                                    ))}
                                  </div>
                                )}

                              </div>
                            </div>
                            
                            <div className="mt-1 pt-1.5 border-t border-slate-700/50 flex items-center justify-between gap-2">
                               <div className="flex-1 flex items-center gap-2 min-w-0">
                                    <a 
                                        href={chart.url} 
                                        target="_blank" 
                                        rel="noopener noreferrer"
                                        className="text-[10px] font-mono text-slate-500 truncate hover:text-blue-400 hover:underline"
                                        title={chart.filename}
                                        onClick={(e) => e.stopPropagation()}
                                    >
                                        {chart.filename}
                                    </a>
                               </div>
                              <a 
                                href={chart.url}
                                target="_blank"
                                rel="noopener noreferrer"
                                onClick={(e) => e.stopPropagation()}
                                className="p-1 hover:bg-slate-700 rounded text-slate-400 hover:text-white transition-colors"
                              >
                                  <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                                  </svg>
                              </a>
                            </div>
                          </div>
                      );})}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-12 text-slate-500 bg-slate-800/30 rounded-xl border-2 border-dashed border-slate-700">
                <p>{t('no_results')}</p>
              </div>
            )}
          </section>
        )}
      </div>

      <footer className="mt-12 py-6 border-t border-slate-700/50">
        <div className="container mx-auto px-4 flex flex-col md:flex-row items-center justify-between gap-4 text-slate-400 text-sm">
          <div className="flex items-center gap-2">
            <span>{t('footer_credits')}</span>
            <a 
              href="https://youtube.com/channel/UCoeiQSBuqp3oFpK16nQT1_Q/" 
              target="_blank" 
              rel="noopener noreferrer"
              className="font-semibold text-blue-400 hover:text-blue-300 flex items-center gap-1 transition-colors"
            >
              Stardust Citizen
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" viewBox="0 0 16 16" className="inline ml-1">
                <path d="M8.051 1.999h.089c.822.003 4.987.033 6.11.335a2.01 2.01 0 0 1 1.415 1.42c.101.38.172.883.22 1.402l.01.104.022.26.008.104c.065.914.073 1.77.074 1.957v.075c-.001.194-.01 1.108-.082 2.06l-.008.105-.009.104c-.05.572-.124 1.14-.235 1.558a2.007 2.007 0 0 1-1.415 1.42c-1.16.312-5.569.334-6.18.335h-.142c-.309 0-1.587-.006-2.927-.052l-.17-.006-.087-.004-.171-.007-.171-.007c-1.11-.049-2.167-.128-2.654-.26a2.007 2.007 0 0 1-1.415-1.419c-.111-.417-.185-.986-.235-1.558L.09 9.82l-.008-.104A31.4 31.4 0 0 1 0 7.68v-.123c.002-.215.01-.958.064-1.778l.007-.103.003-.052.008-.104.022-.26.01-.104c.048-.519.119-1.023.22-1.402a2.007 2.007 0 0 1 1.415-1.42c.487-.13 1.544-.21 2.654-.26l.17-.007.172-.006.086-.003.171-.007A99.788 99.788 0 0 1 7.858 2h.193zM6.4 5.209v4.818l4.157-2.408L6.4 5.209z"/>
              </svg>
            </a>
          </div>
          
          <div className="flex gap-4 text-xs">
             <div className="px-3 py-1 bg-slate-800 rounded-full border border-slate-700 font-mono text-blue-300">
                {process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'Unknown'} / {process.env.NEXT_PUBLIC_AIRAC_DATE || 'Unknown'}
             </div>
          </div>
        </div>
      </footer>
    </main>
  );
}

export default function Home() {
  return (
    <Suspense fallback={<div className="min-h-screen bg-slate-900 flex items-center justify-center text-slate-400">Loading...</div>}>
      <SearchPage />
    </Suspense>
  );
}
