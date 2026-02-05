export type Chart = {
  category: string;
  subtitle: string;
  filename: string;
  url: string;
  page?: string;
  tags?: string[];
  icao?: string;
  customTitle?: string;
};

export interface SavedDock {
  id: string;
  name: string;
  charts: Chart[];
  notes?: string;
  timestamp: number;
}
