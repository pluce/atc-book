export interface Notice {
  id: string; // Unique ID
  icao: string; // The ICAO code derived from itemA or context
  source: string; // 'SOFIA', 'FAA', etc.
  identifier: string; // e.g. "A1234/25"
  type: string; // N, R, C
  validFrom: string; // ISO String
  validTo: string; // ISO String or "PERM"
  content: string; // The main text (Item E)
  location?: string; // Coordinates or description
  category?: string; // Q code derived category (optional for now)
}

export interface NoticeProvider {
  name: string;
  getNotices(icao: string): Promise<Notice[]>;
}
