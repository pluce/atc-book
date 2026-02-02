// Helper for retrying requests with exponential backoff & jitter
import { STATION_TAGS } from './constants';

export async function fetchWithRetry(url: string, retries = 3, baseDelay = 1000): Promise<Response> {
  let lastError: unknown;
  
  for (let i = 0; i <= retries; i++) {
    try {
      const response = await fetch(url);
      if (response.ok) return response;
      throw new Error(`HTTP error! status: ${response.status}`);
    } catch (error) {
      lastError = error;
      if (i < retries) {
        // Backoff: base * 2^attempt + random jitter (0-1000ms)
        const delay = (baseDelay * Math.pow(2, i)) + (Math.random() * 1000);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
  throw lastError;
}

export const getTagGroupKey = (tag: string) => {
  if (STATION_TAGS.includes(tag)) return 'group_stations';
  if (/^\d{2}[LRC]?$/.test(tag)) return 'group_runways';
  if (tag.startsWith('App.')) return 'group_phases';
  if (['ILS', 'LOC', 'RNAV', 'RNP', 'VPT', 'MVL', 'Nuit', 'DME'].some(t => tag.includes(t))) return 'group_approaches';
  return 'group_others';
};

export const groupTags = (tags: string[]) => {
  const groups: Record<string, string[]> = {
    'group_stations': [...STATION_TAGS],
    'group_runways': [],
    'group_approaches': [],
    'group_phases': [],
    'group_others': []
  };
  
  tags.forEach(tag => {
    const g = getTagGroupKey(tag);
    if (groups[g]) groups[g].push(tag);
    else groups['group_others'].push(tag);
  });
  
  // Sort specific groups
  groups['group_runways'].sort(); // Keep runways sorted alphanumerically
  
  return groups;
};
