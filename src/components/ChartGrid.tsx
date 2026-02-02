import { Chart } from '../types';
import { useTranslation } from 'react-i18next';
import { CATEGORY_MAP } from '../lib/constants';

interface ChartGridProps {
  groupedCharts: Record<string, Chart[]>;
  selectedUrls: Set<string>;
  toggleChart: (url: string) => void;
  toggleGroup: (charts: Chart[]) => void;
  isPinned: (chart: Chart) => boolean;
  togglePin: (chart: Chart, e?: React.MouseEvent) => void;
  openViewer: (chart: Chart) => void;
}

export function ChartGrid({
  groupedCharts,
  selectedUrls,
  toggleChart,
  toggleGroup,
  isPinned,
  togglePin,
  openViewer
}: ChartGridProps) {
  const { t } = useTranslation();

  const getCategoryLabel = (category: string) => {
    return CATEGORY_MAP[category] ? t(CATEGORY_MAP[category]) : category;
  };

  const getTagLabel = (tag: string) => {
    if (tag === 'App. Finale') return t('tag_app_final');
    if (tag === 'App. Initiale') return t('tag_app_initial');
    if (tag === 'Nuit') return t('tag_night');
    return tag;
  };

  if (Object.keys(groupedCharts).length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground bg-secondary/30 rounded-xl border-2 border-dashed border-border">
        <p>{t('no_results')}</p>
      </div>
    );
  }

  return (
    <div className="space-y-10">
      {Object.entries(groupedCharts).map(([category, groupCharts]) => (
        <div key={category} className="space-y-4">
          <div className="flex items-center gap-3 border-b border-border pb-2">
            <input
              type="checkbox"
              checked={groupCharts.every(c => selectedUrls.has(c.url))}
              onChange={() => toggleGroup(groupCharts)}
              className="w-5 h-5 rounded border-input text-primary focus:ring-ring bg-background cursor-pointer"
            />
            <h3 className="text-xl font-semibold text-primary">{getCategoryLabel(category)}</h3>
            <span className="text-sm text-muted-foreground bg-secondary px-2 py-0.5 rounded-full">
              {groupCharts.length}
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {groupCharts.map((chart, index) => {
              const isSelected = selectedUrls.has(chart.url);
              const pinned = isPinned(chart);
              return (
                <div
                  key={index}
                  onClick={(e) => {
                    // Toggle selection if clicking card (unless clicking link or buttons)
                    if (!(e.target as HTMLElement).closest('a') && !(e.target as HTMLElement).closest('button') && !(e.target as HTMLElement).closest('input')) {
                      toggleChart(chart.url);
                    }
                  }}
                  className={`group border p-4 rounded-xl transition-all duration-200 flex flex-col justify-between h-full hover:shadow-lg cursor-pointer select-none relative
                    ${isSelected
                      ? 'bg-secondary border-primary/50 shadow-primary/10'
                      : 'bg-card border-border hover:bg-secondary/50 hover:border-border/80'} // Changed simplified hover
                  `}
                >
                  <div className="flex items-start gap-3 mb-2">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleChart(chart.url)}
                      className="mt-1 w-4 h-4 rounded border-input text-primary focus:ring-ring bg-background/50 cursor-pointer"
                    />
                    <div className="flex-1 min-w-0 pr-8">
                      <div className="flex justify-between items-start gap-2">
                        <h4 data-testid="chart-title" className={`font-semibold leading-snug truncate ${isSelected ? 'text-primary' : 'text-card-foreground'} group-hover:text-primary transition-colors`}>
                          {(!chart.subtitle || chart.subtitle.toLowerCase().trim() === chart.category.toLowerCase().trim())
                            ? getCategoryLabel(chart.category)
                            : chart.subtitle}
                        </h4>
                      </div>
                      <div className="mt-1">
                        {chart.page && (
                          <span className="text-[10px] bg-secondary text-secondary-foreground px-1.5 py-0.5 rounded font-mono border border-border whitespace-nowrap mr-2">
                            {chart.page}
                          </span>
                        )}
                        {chart.tags && chart.tags.length > 0 && (
                          <span className="inline-flex flex-wrap gap-1">
                            {chart.tags.map(tag => (
                              <span key={tag} className="text-[9px] uppercase tracking-wide font-semibold bg-muted text-muted-foreground px-1.5 py-0.5 rounded border border-border">
                                {getTagLabel(tag)}
                              </span>
                            ))}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Pin Button - Absolute positioned top right */}
                  <button
                    data-testid="btn-pin-card"
                    onClick={(e) => togglePin(chart, e)}
                    className={`absolute top-3 right-3 p-1.5 rounded-full transition-all duration-200 
                      ${pinned
                        ? 'bg-amber-500/20 text-amber-500 hover:bg-amber-500/30'
                        : 'text-muted-foreground hover:bg-muted hover:text-foreground'}
                    `}
                    title={pinned ? t('unpin_tooltip') : t('pin_tooltip')}
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                      <path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z" />
                    </svg>
                  </button>

                  <div className="mt-2 pt-2 border-t border-border flex items-center justify-between gap-2">
                    <div className="flex-1 flex items-center gap-2 min-w-0">
                      <a
                        href={chart.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-[10px] font-mono text-muted-foreground truncate hover:text-primary hover:underline transition-colors"
                        title={chart.filename}
                        onClick={(e) => e.stopPropagation()}
                      >
                        {chart.filename}
                      </a>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        openViewer(chart);
                      }}                      data-testid="btn-viewer-open"                      className="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors"
                      title={t('merge_button')} // Using existing translation for "Open" context or generic view
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                      </svg>
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
