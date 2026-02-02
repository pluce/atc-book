import { useTranslation } from 'react-i18next';
import { Chart } from '../types';
import { CATEGORY_MAP } from '../lib/constants';

interface DockProps {
  charts: Chart[];
  onRemoveChart: (chart: Chart) => void;
  onClear: () => void;
  isOpen: boolean;
  onToggleOpen: () => void;
  side: 'bottom' | 'left' | 'right';
  onCycleSide: () => void;
  viewingChart: Chart | null;
  onViewChart: (chart: Chart) => void;
}

export function Dock({ 
  charts, 
  onRemoveChart, 
  onClear, 
  isOpen, 
  onToggleOpen, 
  side, 
  onCycleSide,
  viewingChart,
  onViewChart 
}: DockProps) {
  const { t } = useTranslation();

  const getCategoryLabel = (category: string) => {
    return CATEGORY_MAP[category] ? t(CATEGORY_MAP[category]) : category;
  };

  const dockVisible = charts.length > 0;
  
  // Grouping logic
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

  return (
      <div 
        data-testid="dock-container"
        className={`fixed z-[60] transition-all duration-300 ease-in-out bg-popover/95 backdrop-blur-md border-border shadow-2xl
            ${!dockVisible ? 'translate-y-[200%] opacity-0' : 'translate-y-0 opacity-100'}
            ${side === 'bottom' 
                ? 'bottom-0 left-0 right-0 h-32 border-t' 
                : side === 'left'
                    ? 'top-0 bottom-0 left-0 w-32 border-r'
                    : 'top-0 bottom-0 right-0 w-32 border-l'
            }
            ${!isOpen && side === 'bottom' ? 'translate-y-[calc(100%-2.5rem)]' : ''}
            ${!isOpen && side === 'left' ? '-translate-x-[calc(100%-2.5rem)]' : ''}
            ${!isOpen && side === 'right' ? 'translate-x-[calc(100%-2.5rem)]' : ''}
        `}
      >
        {/* Toggle Handle & Controls */}
        {dockVisible && (
             <div className={`absolute flex items-center justify-center
                 ${side === 'bottom' 
                    ? 'top-0 left-1/2 -translate-x-1/2 -translate-y-full w-auto' 
                    : side === 'left'
                        ? 'right-0 top-1/2 -translate-y-1/2 translate-x-full'
                        : 'left-0 top-1/2 -translate-y-1/2 -translate-x-full'
                 }
             `}>
                 <div className={`bg-popover border-border flex items-center shadow-xl overflow-hidden
                    ${side === 'bottom' 
                        ? 'rounded-t-xl border-t border-x px-4 py-1 flex-row gap-3' 
                        : side === 'left'
                            ? 'rounded-r-xl border-y border-r py-3 px-1.5 flex-col gap-2'
                            : 'rounded-l-xl border-y border-l py-3 px-1.5 flex-col gap-2'
                    }
                 `}>
                    <button 
                        onClick={onToggleOpen}
                        className={`text-muted-foreground hover:text-foreground flex items-center gap-2 text-xs font-semibold uppercase tracking-wider p-1 transition-colors
                            ${side !== 'bottom' ? 'flex-col-reverse' : 'flex-row'}
                        `}
                        title={isOpen ? "Réduire" : "Agrandir"}
                    >
                        {side === 'bottom' && <span>{t('dock_title')} ({charts.length})</span>}
                        
                        <svg xmlns="http://www.w3.org/2000/svg" className={`h-4 w-4 transition-transform duration-300 transform
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
                    
                    {/* Divider */}
                    <div className={`${side === 'bottom' ? 'w-px h-4' : 'h-px w-4'} bg-border`}></div>

                    {/* Rotate Button */}
                    <button 
                        onClick={onCycleSide}
                        className="text-muted-foreground hover:text-primary p-1.5 rounded-lg hover:bg-secondary transition-colors"
                        title="Changer la position (Bas / Gauche / Droite)"
                    >
                         <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                         </svg>
                    </button>
                 </div>
             </div>
        )}

         {/* Content Container */}
         {charts.length === 0 ? (
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
                                                    <h5 className="font-semibold text-xs text-card-foreground truncate leading-tight w-full" title={chart.subtitle || chart.category}>
                                                        {chart.category === 'Instrument Approach' ? (chart.filename.replace('.pdf','')) : (chart.subtitle || chart.category)}
                                                    </h5>
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
         )}
      </div>
  );
}
