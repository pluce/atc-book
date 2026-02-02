export type Chart = {
  category: string;
  subtitle: string;
  filename: string;
  url: string;
  page?: string;
  tags?: string[];
  icao?: string;
};

export interface SavedDock {
  id: string;
  name: string;
  charts: Chart[];
  timestamp: number;
}
