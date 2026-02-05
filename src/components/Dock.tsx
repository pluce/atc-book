import { useState, useRef, useEffect } from 'react';
import dynamic from 'next/dynamic';
import { useTranslation } from 'react-i18next';
import { Chart, SavedDock } from '../types';
import { Notice } from '../lib/notices/types';
import { CATEGORY_MAP } from '../lib/constants';

const RichTextEditor = dynamic(() => import('./RichTextEditor'), { 
  ssr: false,
  loading: () => <div className="h-full w-full animate-pulse bg-secondary/50 rounded-lg" />
});

interface DockProps {
  charts: Chart[];
  notices: Notice[];
  onRemoveChart: (chart: Chart) => void;
  onClear: () => void;
  onRestore: (dock: SavedDock) => void;
  savedDocks: SavedDock[];
  onSaveDock: (name: string) => void;
  onDeleteDock: (id: string) => void;
  isOpen: boolean;
  onToggleOpen: () => void;
  side: 'bottom' | 'left' | 'right';
  onCycleSide: () => void;
  viewingChart: Chart | null;
  onViewChart: (chart: Chart) => void;
  currentIcao?: string;
  currentTags?: string[];
  scratchpadContent: string;
  onUpdateScratchpad: (content: string) => void;
  activeDockId?: string | null;
  onRenameChart: (chart: Chart, newName: string) => void;
}

export function Dock({ 
  charts, 
  notices,
  onRemoveChart, 
  onClear,
  onRestore,
  savedDocks,
  onSaveDock,
  onDeleteDock,
  isOpen, 
  onToggleOpen, 
  side, 
  onCycleSide,
  viewingChart,
  onViewChart,
  currentIcao,
  currentTags,
  scratchpadContent,
  onUpdateScratchpad,
  activeDockId,
  onRenameChart
}: DockProps) {
  const { t } = useTranslation();
  const [viewMode, setViewMode] = useState<'charts' | 'notices' | 'saves' | 'scratchpad'>('charts');
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  
  // Saving State
  const [showSaveInput, setShowSaveInput] = useState(false);
  const [saveName, setSaveName] = useState('');

  // Renaming State
  const [editingChartUrl, setEditingChartUrl] = useState<string | null>(null);
  const [editValue, setEditValue] = useState('');
  const editInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingChartUrl && editInputRef.current) {
        editInputRef.current.focus();
        editInputRef.current.select();
    }
  }, [editingChartUrl]);

  const startEditing = (chart: Chart, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingChartUrl(chart.url);
    setEditValue(chart.customTitle || (chart.category === 'Instrument Approach' ? (chart.filename.replace('.pdf','')) : (chart.subtitle || chart.category)));
  };

  const cancelEditing = () => {
    setEditingChartUrl(null);
    setEditValue('');
  };

  const saveEditing = (chart: Chart) => {
    if (editValue.trim()) {
        onRenameChart(chart, editValue.trim());
    }
    setEditingChartUrl(null);
  };

  // Scratchpad Local State for Debounce
  const [localNote, setLocalNote] = useState(scratchpadContent);
  const debounceTimerProp = useRef<NodeJS.Timeout | null>(null);

  // Sync local note when prop changes (loading a save)
  useEffect(() => {
    setLocalNote(scratchpadContent);
  }, [scratchpadContent]);

  const handleNoteChange = (content: string) => {
    setLocalNote(content);
    
    if (debounceTimerProp.current) clearTimeout(debounceTimerProp.current);
    debounceTimerProp.current = setTimeout(() => {
        onUpdateScratchpad(content);
    }, 1000); // 1s debounce
  };

  const scrollRef = useRef<HTMLDivElement>(null);
  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({});

  const handleSave = () => {
    onSaveDock(saveName);
    setShowSaveInput(false);
    setSaveName('');
  };

  const deleteSave = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    onDeleteDock(id);
  };
  
  const restoreSave = (dock: SavedDock) => {
      onRestore(dock);
      setViewMode('charts');
  };

  const prepareSave = () => {
      let defaultName = '';
      
      // If we are working on an active dock, propose its name
      if (activeDockId) {
          const activeDock = savedDocks.find(d => d.id === activeDockId);
          if (activeDock) {
              defaultName = activeDock.name;
          }
      }

      // Fallback if no active dock or active dock not found
      if (!defaultName) {
         defaultName = currentIcao 
            ? `${currentIcao}${currentTags && currentTags.length > 0 ? '_' + currentTags.join('_') : ''}`
            : `Save ${new Date().toLocaleTimeString()}`;
      }

      setSaveName(defaultName);
      setShowSaveInput(true);
  };

  const getCategoryLabel = (category: string) => {
    return CATEGORY_MAP[category] ? t(CATEGORY_MAP[category]) : category;
  };

  const getNoticeLabel = (category: string | undefined) => {
      if (!category) return t('notice_cat_A'); // Default to A (General) or similar if unknown
      
      // Try exact category (e.g., "FA")
      const key = `notice_cat_${category}`;
      // @ts-ignore
      const exact = t(key);
      if (exact !== key) return exact;

      // Try parent category (e.g., "F")
      const parentKey = `notice_cat_${category.charAt(0)}`;
      // @ts-ignore
      const parent = t(parentKey);
      if (parent !== parentKey) return parent;

      return category;
  };

  const dockVisible = charts.length > 0 || (notices && notices.length > 0) || savedDocks.length > 0;
  
  // Grouping logic for charts
  const uniqueIcaos = Array.from(new Set(charts.map(c => c.icao || 'Unknown'))).filter(i => i !== 'Unknown');
  const hasMultipleAirports = uniqueIcaos.length > 1;

  const groupedCharts = charts.reduce((groups, chart) => {
    const airportKey = hasMultipleAirports ? (chart.icao || 'Other') : 'Single';
    const catKey = chart.category;

    if (!groups[airportKey]) groups[airportKey] = {};
    if (!groups[airportKey][catKey]) groups[airportKey][catKey] = [];
    
    groups[airportKey][catKey].push(chart);
    return groups;
  }, {} as Record<string, Record<string, Chart[]>>);

  // Grouping logic for notices
  const groupedNotices = (notices || []).reduce((groups, notice) => {
      // Use category set by adapter (code23)
     const cat = notice.category || 'OTHER';
     if (!groups[cat]) groups[cat] = [];
     groups[cat].push(notice);
     return groups;
  }, {} as Record<string, Notice[]>);
  
  const noticesHeightClass = side === 'bottom' ? 'h-96' : 'w-96';
  const standardSizeClass = side === 'bottom' ? 'h-32' : 'w-48';
  const isExpandedView = viewMode !== 'charts';

  return (
      <div 
        data-testid="dock-container"
        className={`fixed z-[60] transition-all duration-300 ease-in-out bg-popover/95 backdrop-blur-md border-border shadow-2xl
            ${!dockVisible ? 'translate-y-[200%] opacity-0' : 'translate-y-0 opacity-100'}
            ${side === 'bottom' 
                ? `bottom-0 left-0 right-0 border-t ${isOpen && isExpandedView ? noticesHeightClass : standardSizeClass}`
                : side === 'left'
                    ? `top-0 bottom-0 left-0 border-r ${isOpen && isExpandedView ? noticesHeightClass : standardSizeClass}`
                    : `top-0 bottom-0 right-0 border-l ${isOpen && isExpandedView ? noticesHeightClass : standardSizeClass}`
            }
            ${!isOpen && side === 'bottom' ? 'translate-y-[calc(100%-2.5rem)]' : ''}
            ${!isOpen && side === 'left' ? '-translate-x-[calc(100%-2.5rem)]' : ''}
            ${!isOpen && side === 'right' ? 'translate-x-[calc(100%-2.5rem)]' : ''}
        `}
      >
        {/* Toggle Handle & Controls - Floating Pill Design when Bottom */}
        {dockVisible && (
             <div className={`absolute flex items-center justify-center pointer-events-none z-50
                 ${side === 'bottom' 
                    ? 'top-0 left-1/2 -translate-x-1/2 -translate-y-full w-auto' 
                    : side === 'left'
                        ? 'right-0 top-1/2 -translate-y-1/2 translate-x-full'
                        : 'left-0 top-1/2 -translate-y-1/2 -translate-x-full'
                 }
             `}>
                 <div className={`bg-popover border-border flex items-center relative backdrop-blur-md pointer-events-auto rounded-full border shadow-2xl transition-all hover:scale-105 active:scale-95
                    ${side === 'bottom' 
                        ? 'px-6 py-2 flex-row gap-4 mb-4' 
                        : side === 'left'
                            ? 'py-6 px-2 flex-col gap-4 ml-4'
                            : 'py-6 px-2 flex-col gap-4 mr-4'
                    }
                 `}>
                    <button
                        onClick={() => {
                            if (!isOpen) onToggleOpen();
                            setViewMode(viewMode === 'scratchpad' ? 'charts' : 'scratchpad');
                        }}
                        className={`p-1.5 rounded-full transition-all duration-200
                            ${viewMode === 'scratchpad' ? 'text-primary bg-primary/10 shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-secondary'}
                        `}
                        title={t('dock_scratchpad_title') || "Mémo"}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                    </button>

                    <button 
                        onClick={onToggleOpen}
                        className={`text-muted-foreground hover:text-foreground flex items-center gap-2 text-xs font-semibold uppercase tracking-wider p-1 transition-colors group
                            ${side !== 'bottom' ? 'flex-col-reverse' : 'flex-row'}
                        `}
                        title={isOpen ? "Réduire" : "Agrandir"}
                    >
                        {side === 'bottom' && <span className="text-sm font-medium tracking-normal group-hover:text-primary transition-colors">{t('dock_title')} ({charts.length})</span>}
                        
                        <svg xmlns="http://www.w3.org/2000/svg" className={`h-4 w-4 transition-transform duration-300 transform group-hover:text-primary
                            ${side === 'bottom' && isOpen ? 'rotate-180' : ''}
                            ${side === 'left' && isOpen ? 'rotate-180' : ''} 
                            ${side === 'right' && !isOpen ? 'rotate-180' : ''}
                        `} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                             {side === 'bottom' 
                                ? <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
                                : <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                             }
                        </svg>

                        {side !== 'bottom' && <span className="text-[10px] font-mono font-bold">{charts.length}</span>}
                    </button>

                    {/* Notices Toggle (if notices exist) */}
                    {notices && notices.length > 0 && (
                        <button
                            onClick={() => {
                                if (!isOpen) onToggleOpen();
                                setViewMode(viewMode === 'notices' ? 'charts' : 'notices');
                            }}
                            className={`p-1.5 rounded-full transition-all duration-200 flex items-center justify-center
                                ${viewMode === 'notices' ? 'text-primary bg-primary/10 shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-secondary'}
                            `}
                            title={t('dock_notices_title')}
                        >
                            <div className="relative">
                                <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                                </svg>
                                <span className="absolute -top-1.5 -right-1.5 bg-red-500 text-white rounded-full w-3.5 h-3.5 flex items-center justify-center text-[9px] font-bold shadow-sm ring-2 ring-popover">{notices.length}</span>
                            </div>
                        </button>
                    )}

                    {/* Rotate Button */}
                    <button 
                        onClick={onCycleSide}
                        className="text-muted-foreground hover:text-primary p-1.5 rounded-full hover:bg-secondary transition-colors"
                        title="Changer la position (Bas / Gauche / Droite)"
                    >
                         <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                         </svg>
                    </button>

                    {/* Save / Load */}
                    {(charts.length > 0 || savedDocks.length > 0) && (
                        <>
                            
                            {charts.length > 0 && showSaveInput ? (
                                <div className={`flex items-center gap-1 animate-in fade-in zoom-in duration-200 
                                    ${side === 'bottom' ? 'pointer-events-auto' : 'absolute top-0 left-full ml-3 bg-popover border border-border p-1.5 rounded-lg shadow-xl flex-row z-50 w-auto'}
                                    ${side === 'right' ? '!right-full !left-auto !mr-3 !ml-0' : ''}
                                `}>
                                    <input 
                                        autoFocus
                                        data-testid="dock-save-input"
                                        type="text" 
                                        value={saveName} 
                                        onChange={(e) => setSaveName(e.target.value)}
                                        placeholder={t('dock_save_placeholder') || "Nom de la sauvegarde..."}
                                        className="h-8 w-40 px-2 text-xs bg-background border border-border rounded focus:ring-1 focus:ring-primary outline-none"
                                        onKeyDown={(e) => {
                                            if(e.key === 'Enter') handleSave();
                                            if(e.key === 'Escape') setShowSaveInput(false);
                                        }}
                                    />
                                    <button 
                                        onClick={handleSave} 
                                        className="text-green-600 hover:text-green-700 hover:bg-green-500/20 h-8 w-8 flex items-center justify-center rounded transition-colors" 
                                        data-testid="dock-save-confirm-btn"
                                        type="button"
                                        title="Valider"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
                                        </svg>
                                    </button>
                                    <button 
                                        onClick={() => setShowSaveInput(false)} 
                                        className="text-destructive hover:text-red-700 hover:bg-destructive/20 h-8 w-8 flex items-center justify-center rounded transition-colors"
                                        title="Annuler"
                                        type="button"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                    </button>
                                </div>
                            ) : (
                                <>
                                    {charts.length > 0 && (
                                        <button 
                                            onClick={prepareSave}
                                            data-testid="dock-save-init-btn"
                                            title={t('dock_save_tooltip') || "Sauvegarder la configuration"}
                                            className="text-muted-foreground hover:text-primary p-1.5 rounded-full hover:bg-secondary transition-colors"
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                                            </svg>
                                        </button>
                                    )}
                                    <button
                                        onClick={() => {
                                            if(!isOpen) onToggleOpen();
                                            setViewMode(viewMode === 'saves' ? 'charts' : 'saves');
                                        }}
                                        data-testid="dock-load-mode-btn"
                                        title={t('dock_load_tooltip') || "Charger une configuration"}
                                        className={`p-1.5 rounded-full transition-all duration-200
                                            ${viewMode === 'saves' ? 'text-primary bg-primary/10 shadow-sm' : 'text-muted-foreground hover:text-foreground hover:bg-secondary'}
                                        `}
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
                                        </svg>
                                    </button>
                                </>
                            )}
                        </>
                    )}
                 </div>
             </div>
        )}

         {/* Content Container */}
         
         {/* VIEW: SCRATCHPAD */}
         {viewMode === 'scratchpad' && isOpen ? (
             <div className="h-full w-full overflow-hidden flex flex-col bg-popover relative">
                 <div className="flex items-center justify-between px-4 py-2 border-b border-border bg-muted/30 flex-shrink-0 z-20 relative">
                     <h3 className="font-bold text-sm flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                        {t('dock_scratchpad_title') || "Mémo / Scratchpad"}
                     </h3>
                     <div className="flex gap-2">
                        <span className="text-xs text-muted-foreground italic self-center">
                            {(localNote !== scratchpadContent) ? '...' : (t('saved') || 'Saved')}
                        </span>
                        <button onClick={() => setViewMode('charts')} className="text-xs text-primary hover:underline">
                            {t('close_viewer')}
                        </button>
                     </div>
                 </div>
                 <div className="flex-1 p-0 flex flex-col overflow-hidden">
                     <RichTextEditor 
                        value={localNote}
                        onChange={handleNoteChange}
                        placeholder={t('dock_scratchpad_placeholder') || "Écrivez vos notes ici (fréquences, clairances, mémos)..."}
                        className="bg-background text-foreground"
                     />
                 </div>
             </div>
         ) : viewMode === 'notices' && isOpen ? (
             <div className="h-full w-full overflow-hidden flex flex-col bg-popover relative">
                 {/* Main Wrapper needs to handle menu positioning context */}
                 
                 <div className="flex items-center justify-between px-4 py-2 border-b border-border bg-muted/30 flex-shrink-0 z-20 relative">
                     <h3 className="font-bold text-sm flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                        </svg>
                        {t('dock_notices_title')} ({notices.length})
                     </h3>
                     <button onClick={() => setViewMode('charts')} className="text-xs text-primary hover:underline">
                         {t('close_viewer')}
                     </button>
                 </div>

                 <div ref={scrollRef} className="flex-1 overflow-y-auto pb-4 pt-0 space-y-6 relative z-10">
                     {Object.entries(groupedNotices).length === 0 ? (
                         <div className="text-center text-muted-foreground p-8">{t('dock_notices_empty')}</div>
                     ) : (
                         Object.entries(groupedNotices).map(([cat, catsNotices]) => (
                             <div 
                                key={cat} 
                                ref={el => { if(el) sectionRefs.current[cat] = el }}
                                className="space-y-2 scroll-mt-2 relative"
                             >
                                 <h4 
                                     onClick={() => setActiveMenu(activeMenu === cat ? null : cat)}
                                     title={t('dock_notices_jump_title') || 'Menu'}
                                     className="text-xs font-bold uppercase tracking-wider text-muted-foreground border-b border-border text-center py-2 px-4 shadow-sm sticky top-0 bg-popover/95 z-20 backdrop-blur-sm cursor-pointer hover:text-primary hover:bg-secondary/50 transition-colors select-none flex items-center justify-center gap-1 group"
                                     data-active={activeMenu === cat}
                                 >
                                     {getNoticeLabel(cat)}
                                     <svg xmlns="http://www.w3.org/2000/svg" className={`h-3 w-3 opacity-50 group-hover:opacity-100 transition-all duration-200 ${activeMenu === cat ? 'rotate-180 text-primary opacity-100' : ''}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                         <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                                     </svg>
                                 </h4>

                                 {activeMenu === cat && (
                                     <>
                                        <div className="fixed inset-0 z-30" onClick={() => setActiveMenu(null)}></div>
                                        <div className="absolute top-[34px] left-1/2 -translate-x-1/2 w-56 max-h-60 z-40 bg-popover border border-border shadow-xl rounded-md flex flex-col animate-in fade-in zoom-in-95 duration-150 overflow-hidden ring-1 ring-border">
                                            <div className="overflow-y-auto py-1 dark:bg-popover">
                                                {Object.keys(groupedNotices).map(targetCat => (
                                                    <button 
                                                        key={targetCat}
                                                        type="button"
                                                        className={`w-full text-left px-3 py-2 text-xs flex items-center justify-between hover:bg-secondary transition-colors
                                                            ${cat === targetCat ? 'bg-secondary/60 font-semibold text-foreground' : 'text-muted-foreground'}
                                                        `}
                                                        onClick={() => {
                                                            const el = sectionRefs.current[targetCat];
                                                            if (el) {
                                                                // Must verify if we need to adjust offset for sticky header
                                                                const yOffset = -40; 
                                                                const y = el.getBoundingClientRect().top + window.pageYOffset + yOffset;
                                                                // Element scrollIntoView is safer for Dock
                                                                el.scrollIntoView({ behavior: 'smooth' });
                                                            }
                                                            setActiveMenu(null);
                                                        }}
                                                    >
                                                        <span className="truncate pr-2">{getNoticeLabel(targetCat)}</span>
                                                        <span className="text-[10px] text-muted-foreground px-1.5 py-0.5 rounded-full bg-secondary">
                                                            {groupedNotices[targetCat].length}
                                                        </span>
                                                    </button>
                                                ))}
                                            </div>
                                        </div>
                                     </>
                                 )}
                                 <div className={`px-4 grid gap-3 ${side === 'bottom' ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3' : 'grid-cols-1'}`}>
                                     {catsNotices.map((notice) => (
                                         <div key={notice.id} className="bg-card border border-border rounded p-3 text-sm shadow-sm hover:shadow-md transition-shadow">
                                             <div className="flex justify-between items-start mb-1">
                                                 <span className="font-mono font-bold text-primary">{notice.identifier}</span>
                                                 <span className={`text-[10px] px-1.5 py-0.5 rounded ${notice.type === 'N' ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-100' : notice.type === 'R' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100' : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100'}`}>
                                                     {notice.type}
                                                 </span>
                                             </div>
                                             <div className="text-xs text-muted-foreground mb-2">
                                                 {new Date(notice.validFrom).toLocaleDateString()} - {notice.validTo === 'PERM' ? 'PERM' : new Date(notice.validTo).toLocaleDateString()}
                                             </div>
                                             <p className="whitespace-pre-line text-card-foreground text-xs leading-relaxed font-mono">
                                                 {notice.content}
                                             </p>
                                         </div>
                                     ))}
                                 </div>
                             </div>
                         ))
                     )}
                 </div>
             </div>
         ) : viewMode === 'saves' && isOpen ? (
             <div className="h-full w-full overflow-hidden flex flex-col bg-popover relative">
                 <div className="flex items-center justify-between px-4 py-2 border-b border-border bg-muted/30 flex-shrink-0 z-20 relative">
                     <h3 className="font-bold text-sm flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
                        </svg>
                        {t('dock_saves_title')} ({savedDocks.length})
                     </h3>
                     <button onClick={() => setViewMode('charts')} className="text-xs text-primary hover:underline">
                         {t('close_viewer')}
                     </button>
                 </div>
                 <div className="flex-1 overflow-y-auto p-4 space-y-3">
                     {savedDocks.length === 0 ? (
                         <div className="text-center text-muted-foreground p-8 text-sm">{t('dock_saves_empty')}</div>
                     ) : (
                         savedDocks.map(dock => (
                             <div key={dock.id} className="bg-card border border-border rounded-lg p-3 flex justify-between items-center hover:bg-secondary cursor-pointer group shadow-sm transition-all" onClick={() => restoreSave(dock)} data-testid="dock-saved-item">
                                 <div className="flex flex-col gap-1">
                                     <h4 className="font-bold text-sm group-hover:text-primary transition-colors">{dock.name}</h4>
                                     <p className="text-[10px] text-muted-foreground font-mono">
                                        {new Date(dock.timestamp).toLocaleDateString()} {new Date(dock.timestamp).toLocaleTimeString()} • {dock.charts.length} cartes
                                        {dock.notes && <span className="ml-2 font-bold opacity-50" title="Contient des notes">📝</span>}
                                     </p>
                                 </div>
                                 <button 
                                    onClick={(e) => deleteSave(dock.id, e)} 
                                    className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 p-2 rounded-full transition-colors opacity-0 group-hover:opacity-100"
                                    title="Supprimer"
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                    </svg>
                                 </button>
                             </div>
                         ))
                     )}
                 </div>
             </div>
         ) : (
         /* VIEW: CHARTS (EXISTING) */
         charts.length === 0 ? (
             <div className="h-full flex flex-col items-center justify-center text-muted-foreground gap-2">
                 <svg xmlns="http://www.w3.org/2000/svg" className="h-8 w-8 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                 </svg>
                 <span className="text-sm">{t('dock_empty')}</span>
             </div>
         ) : (
             <div className={`h-full flex ${side === 'bottom' ? 'items-center px-4 gap-4 overflow-x-auto' : 'flex-col py-4 gap-4 overflow-y-auto w-full items-center'}`}>
                <div className={`flex-shrink-0 flex items-center justify-center gap-2
                    ${side === 'bottom' ? 'border-r border-border pr-4 flex-col' : 'border-b border-border pb-4 w-full flex-col'}
                `}>
                    <span className="text-xs font-bold text-muted-foreground uppercase tracking-wider whitespace-nowrap" data-testid="dock-header-title">
                        {side === 'bottom' ? t('dock_title') : 'DOCK'}
                    </span>
                    <button 
                       onClick={onClear}
                       data-testid="dock-clear-btn"
                       className="text-[10px] text-destructive hover:text-destructive/80 hover:underline whitespace-nowrap"
                    >
                        {t('clear_dock')}
                    </button>
                </div>
                
                <div className={`flex ${side === 'bottom' ? 'flex-row gap-6 px-2 pb-2 h-full items-center select-none' : 'flex-col w-full px-2 gap-3'}`}>
                    {Object.entries(groupedCharts).map(([airportKey, categories]) => (
                        <div key={airportKey} className={`flex ${side === 'bottom' ? 'flex-row gap-4 h-full' : 'flex-col gap-2 w-full'}`}>
                            {/* Airport Header if multiple airports */}
                            {hasMultipleAirports && (
                                <div className={`flex items-center justify-center ${side === 'bottom' ? 'bg-secondary rounded-lg px-2 border border-border flex-col py-1 h-full' : 'w-full border-b border-border pb-1 mb-1'}`}>
                                    <span className="text-sm font-bold text-primary">{airportKey}</span>
                                </div>
                            )}

                            {Object.entries(categories).map(([category, charts]) => (
                                <div key={`${airportKey}-${category}`} className={`flex flex-col gap-1 flex-shrink-0 ${side === 'bottom' ? 'h-full justify-center border-r border-border pr-4 last:border-0' : 'w-full border-b border-border pb-2 last:border-0'}`}>
                                    <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider px-1 truncate max-w-[12rem]">
                                        {getCategoryLabel(category)}
                                    </span>
                                    
                                    <div className={`flex gap-2 ${side === 'bottom' ? 'flex-row' : 'flex-col w-full'}`}>
                                        {charts.map((chart, idx) => (
                                            <div 
                                                key={`${chart.url}-${idx}`} 
                                                data-testid="dock-item"
                                                className={`relative flex-shrink-0 bg-card hover:bg-secondary border border-border rounded-lg p-2 cursor-pointer group transition-all box-border
                                                    ${viewingChart?.url === chart.url ? 'ring-2 ring-primary bg-secondary' : ''}
                                                    ${side === 'bottom' ? 'w-48' : 'w-full'}
                                                `}
                                                onClick={() => onViewChart(chart)}
                                            >
                                                <div className="flex justify-between items-start gap-1">
                                                    {editingChartUrl === chart.url ? (
                                                        <input 
                                                            ref={editInputRef}
                                                            type="text"
                                                            className="w-full text-xs bg-background border border-primary/50 rounded px-1 py-0.5 outline-none text-foreground z-20"
                                                            value={editValue}
                                                            onChange={(e) => setEditValue(e.target.value)}
                                                            onKeyDown={(e) => {
                                                                if(e.key === 'Enter') saveEditing(chart);
                                                                if(e.key === 'Escape') cancelEditing();
                                                            }}
                                                            onBlur={() => saveEditing(chart)}
                                                            onClick={(e) => e.stopPropagation()}
                                                        />
                                                    ) : (
                                                        <h5 
                                                            className="font-semibold text-xs text-card-foreground truncate leading-tight w-full group-hover/text:text-primary transition-colors select-none" 
                                                            title={chart.customTitle || (chart.category === 'Instrument Approach' ? (chart.filename.replace('.pdf','')) : (chart.subtitle || chart.category))}
                                                        >
                                                            {chart.customTitle || (chart.category === 'Instrument Approach' ? (chart.filename.replace('.pdf','')) : (chart.subtitle || chart.category))}
                                                        </h5>
                                                    )}
                                                    <button
                                                        onClick={(e) => startEditing(chart, e)}
                                                        className="text-muted-foreground hover:text-primary transition-colors opacity-0 group-hover:opacity-100 absolute top-1 right-7 bg-secondary rounded-full p-0.5"
                                                        title="Renommer"
                                                    >
                                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                                                            <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        onClick={(e) => {
                                                          e.stopPropagation();
                                                          onRemoveChart(chart);
                                                        }}
                                                        className="text-muted-foreground hover:text-destructive transition-colors opacity-0 group-hover:opacity-100 absolute top-1 right-1 bg-secondary rounded-full p-0.5"
                                                        title={t('unpin_tooltip')}
                                                    >
                                                        <svg xmlns="http://www.w3.org/2000/svg" className="h-3 w-3" viewBox="0 0 20 20" fill="currentColor">
                                                            <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                                                        </svg>
                                                    </button>
                                                </div>
                                                <div className="mt-1 flex items-center justify-between gap-1">
                                                    <span className={`text-[9px] bg-primary/20 text-primary border border-primary/20 px-1 py-0.5 rounded truncate ${side === 'bottom' ? 'max-w-[70%]' : 'max-w-full'}`}>
                                                        {chart.page || 'PDF'}
                                                    </span>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ))}
                        </div>
                    ))}
                </div>
             </div>
         )
         )}
      </div>
  );
}
