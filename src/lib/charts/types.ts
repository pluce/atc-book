export enum ChartCategory {
  AERODROME = 'AERODROME',
  PARKING = 'PARKING',
  GROUND = 'GROUND',
  SID = 'SID',
  STAR = 'STAR',
  IAC = 'IAC',
  VAC = 'VAC',
  VLC = 'VLC', // Landing visual
  TEM = 'TEM', // Temps/Mouvements
  SUPAIP = 'SupAIP',
  OTHER = 'OTHER'
}

export interface Chart {
  id: string; // Unique ID (url/filename)
  source: string; // 'SIA', 'JEPP', etc.
  category: ChartCategory;
  subtitle: string;
  filename: string;
  url: string;
  page?: string; // "1/2"
  tags: string[]; // ["ILS", "RWY09", "Nuit"]
  runways?: string[]; // ["09", "27"] for easier filtering
}

export interface ChartProvider {
  name: string;
  getCharts(icao: string): Promise<Chart[]>;
}
