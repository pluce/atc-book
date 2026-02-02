export const STATION_TAGS = ['DEL', 'GND', 'TWR', 'APP', 'DEP'];

export const STATION_RULES: Record<string, string[]> = {
  'DEL': ["PARKING", "AERODROME", "SID", "VAC", "SupAIP"],
  'GND': ["PARKING", "AERODROME", "GROUND", "VAC", "SupAIP"],
  'TWR': ["GROUND", "AERODROME", "IAC", "SID", "VAC", "VLC", "SupAIP"],
  'APP': ["STAR", "IAC", "VAC", "SupAIP"],
  'DEP': ["SID", "VAC", "SupAIP"]
};

export const CATEGORY_MAP: Record<string, string> = {
  "PARKING": "cat_parking",
  "AERODROME": "cat_aerodrome",
  "GROUND": "cat_ground_movements",
  "IAC": "cat_instrument_approach",
  "SID": "cat_sid",
  "STAR": "cat_star",
  "VAC": "VAC",
  "VLC": "VLC",
  "TEM": "TEM",
  "SupAIP": "SupAIP"
};
