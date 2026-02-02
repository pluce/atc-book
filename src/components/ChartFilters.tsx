import { useState } from 'react';
import { useTranslation } from 'react-i18next';

interface ChartFiltersProps {
  searchedIcao: string;
  visibleCount: number;
  selectedCount: number;
  filterText: string;
  setFilterText: (text: string) => void;
  availableTags: string[];
  groupedTags: Record<string, string[]>;
  selectedTags: Set<string>;
  toggleTag: (tag: string) => void;
  onSelectVisible: (select: boolean) => void;
  onPinSelected: () => void;
  onMergeSelected: () => void;
  onDownloadSelected: () => void;
  merging: boolean;
  downloading: boolean;
}

export function ChartFilters({
  searchedIcao,
  visibleCount,
  selectedCount,
  filterText,
  setFilterText,
  availableTags,
  groupedTags,
  selectedTags,
  toggleTag,
  onSelectVisible,
  onPinSelected,
  onMergeSelected,
  onDownloadSelected,
  merging,
  downloading
}: ChartFiltersProps) {
  const { t } = useTranslation();
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const getTagLabel = (tag: string) => {
    if (tag === 'App. Finale') return t('tag_app_final');
    if (tag === 'App. Initiale') return t('tag_app_initial');
    if (tag === 'Nuit') return t('tag_night');
    return tag;
  };

  return (
    <div className="sticky top-4 z-10 flex flex-col gap-2 md:gap-4">
      <div className="bg-slate-800/90 backdrop-blur-md border border-slate-700/50 rounded-xl shadow-2xl overflow-hidden transition-all duration-300">
        <div className="p-4 flex items-center justify-between gap-4">
          <div className="flex-1 min-w-0">
            <h2 className="text-xl md:text-2xl font-bold text-white truncate">
              {t('results_title')} <span className="text-blue-400">{searchedIcao}</span>
            </h2>
            <div className="flex items-center gap-2 text-xs md:text-sm text-slate-400 mt-1">
              <span>
                {t('visible_charts_plural', { count: visibleCount })}
              </span>
              <span className="hidden md:inline">|</span>
              <span className="text-blue-300 font-medium">
                {t('selected_charts_plural', { count: selectedCount })}
              </span>
            </div>
          </div>

          <div className="hidden md:flex items-center gap-3">
            <div className="flex items-center gap-2">
              <button
                onClick={() => onSelectVisible(true)}
                className="px-3 py-1.5 text-xs font-medium text-blue-300 bg-blue-900/30 hover:bg-blue-900/50 border border-blue-800/50 rounded-lg transition-colors whitespace-nowrap"
              >
                {t('select_all')}
              </button>
              <button
                onClick={() => onSelectVisible(false)}
                className="px-3 py-1.5 text-xs font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition-colors whitespace-nowrap"
              >
                {t('deselect_all')}
              </button>
            </div>

            <div className="h-6 w-px bg-slate-700 mx-1"></div>

            <div className="flex gap-2">
              <button
                onClick={onPinSelected}
                disabled={selectedCount === 0}
                className="flex items-center gap-2 bg-amber-600 hover:bg-amber-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z" />
                </svg>
                <span>{t('pin_selection_button')}</span>
              </button>

              <button
                onClick={onMergeSelected}
                disabled={merging || selectedCount === 0}
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
                onClick={onDownloadSelected}
                disabled={downloading || selectedCount === 0}
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

            {/* Action Buttons (Mobile) */}
            <div className="flex flex-wrap items-center gap-3 justify-end border-b md:border-none border-slate-700/50 pb-4 md:pb-0 md:hidden">
              <div className="flex items-center gap-2">
                <button
                  onClick={() => onSelectVisible(true)}
                  className="px-3 py-1.5 text-xs font-medium text-blue-300 bg-blue-900/30 hover:bg-blue-900/50 border border-blue-800/50 rounded-lg transition-colors whitespace-nowrap"
                >
                  {t('select_all')}
                </button>
                <button
                  onClick={() => onSelectVisible(false)}
                  className="px-3 py-1.5 text-xs font-medium text-slate-400 bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg transition-colors whitespace-nowrap"
                >
                  {t('deselect_all')}
                </button>
              </div>

              <div className="h-6 w-px bg-slate-700 mx-1 hidden md:block"></div>

              <div className="flex gap-2 w-full md:w-auto">
                <button
                  onClick={onPinSelected}
                  disabled={selectedCount === 0}
                  className="flex-1 md:flex-none flex items-center justify-center gap-2 bg-amber-600 hover:bg-amber-500 text-white px-5 py-2.5 rounded-lg shadow-lg transition-all font-medium disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap text-sm"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z" />
                  </svg>
                  <span>{t('pin_selection_button')}</span>
                </button>

                <button
                  onClick={onMergeSelected}
                  disabled={merging || selectedCount === 0}
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
                  onClick={onDownloadSelected}
                  disabled={downloading || selectedCount === 0}
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
                        {idx < arr.length - 1 && groupedTags[arr[idx + 1]]?.length > 0 && (
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
  );
}
