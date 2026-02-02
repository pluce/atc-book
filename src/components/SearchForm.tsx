import { useTranslation } from 'react-i18next';

interface SearchFormProps {
  icao: string;
  setIcao: (value: string) => void;
  onSubmit: (e: React.FormEvent) => void;
  loading: boolean;
  error: string | null;
}

export function SearchForm({ icao, setIcao, onSubmit, loading, error }: SearchFormProps) {
  const { t } = useTranslation();

  return (
    <section className="bg-slate-800 p-6 rounded-2xl shadow-xl border border-slate-700 max-w-2xl mx-auto">
      <form onSubmit={onSubmit} className="flex gap-4 items-end sm:items-stretch flex-col sm:flex-row">
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

      <p className="mt-4 text-xs text-slate-500 text-center">
        {t('supported_airports_hint')}
      </p>

      {error && (
        <div className="mt-6 p-4 bg-red-900/50 border border-red-700 text-red-200 rounded-lg animate-fade-in text-center">
          ⚠️ {error}
        </div>
      )}
    </section>
  );
}
