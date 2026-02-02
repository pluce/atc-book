import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Chart } from '../types';
import { CATEGORY_MAP } from '../lib/constants';

interface ChartViewerProps {
  chart: Chart;
  onClose: () => void;
  blobUrl?: string;
  style?: React.CSSProperties;
}

export function ChartViewer({ chart, onClose, blobUrl, style }: ChartViewerProps) {
  const { t } = useTranslation();

  const getCategoryLabel = (category: string) => {
    return CATEGORY_MAP[category] ? t(CATEGORY_MAP[category]) : category;
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const src = blobUrl || `/api/proxy?url=${encodeURIComponent(chart.url)}`;

  return (
    <div 
        className="fixed z-[50] bg-background/95 backdrop-blur-sm flex flex-col animate-fade-in transition-all duration-300"
        style={style}
    >
      <div className="flex items-center justify-between px-4 py-2 bg-card border-b border-border">
        <h3 className="text-card-foreground font-semibold truncate flex items-center gap-2">
          {getCategoryLabel(chart.category)} - {chart.subtitle || chart.filename}
        </h3>
        <div className="flex items-center gap-2">
            <span className="hidden md:inline text-xs text-muted-foreground mr-2">
                <kbd className="bg-secondary px-1.5 py-0.5 rounded border border-border text-xs">ESC</kbd> {t('close_viewer')}
            </span>
            <button 
            onClick={onClose}
            className="p-2 hover:bg-secondary rounded-full text-muted-foreground hover:text-foreground transition-colors"
            title={t('close_viewer')}
            >
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
            </button>
        </div>
      </div>
      <div className="flex-1 w-full bg-secondary relative">
         <iframe 
            src={src} 
            className="w-full h-full border-none"
            title={chart.filename}
         />
      </div>
    </div>
  );
}
