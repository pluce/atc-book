export const STATION_TAGS = ['DEL', 'GND', 'TWR', 'APP', 'DEP'];

export const STATION_RULES: Record<string, string[]> = {
  'DEL': ["PARKING", "AERODROME", "SID"],
  'GND': ["PARKING", "AERODROME", "GROUND"],
  'TWR': ["GROUND", "AERODROME", "IAC", "SID", "VAC", "VLC"],
  'APP': ["STAR", "IAC"],
  'DEP': ["SID"]
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
  "TEM": "TEM"
};
