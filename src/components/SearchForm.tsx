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
    <section className="bg-card p-6 rounded-2xl shadow-xl border border-border max-w-2xl mx-auto transition-colors duration-300">
      <form onSubmit={onSubmit} className="flex gap-4 items-end sm:items-stretch flex-col sm:flex-row">
        <div className="flex-1 space-y-2 w-full">
          <label htmlFor="icao" className="block text-sm font-medium text-muted-foreground">
            {t('search_label')}
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              id="icao"
              data-testid="search-input"
              value={icao}
              onChange={(e) => setIcao(e.target.value.toUpperCase())}
              className="flex-1 w-full bg-background border border-input rounded-lg px-4 py-3 text-lg focus:ring-2 focus:ring-ring focus:outline-none transition-all placeholder:text-muted-foreground text-foreground"
              placeholder={t('search_placeholder')}
              maxLength={4}
              required
            />
          </div>
        </div>
        <button
          type="submit"
          data-testid="search-submit"
          disabled={loading || icao.length < 4}
          className="bg-primary hover:bg-primary/90 text-primary-foreground font-semibold py-3 px-8 rounded-lg shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center min-w-[150px] w-full sm:w-auto"
        >
          {loading ? (
            <span className="animate-pulse">{t('searching')}</span>
          ) : (
            t('search_button')
          )}
        </button>
      </form>

      <p className="mt-4 text-xs text-muted-foreground text-center">
        {t('supported_airports_hint')}
      </p>

      {error && (
        <div className="mt-6 p-4 bg-destructive/10 border border-destructive/30 text-destructive rounded-lg animate-fade-in text-center font-medium">
          ⚠️ {error}
        </div>
      )}
    </section>
  );
}
