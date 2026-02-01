import { Chart, ChartCategory, ChartProvider } from '../types';
import * as cheerio from 'cheerio';

export class SIAAdapter implements ChartProvider {
  name = 'SIA';
  
  private baseUrl: string;
  private pageUrl: string = '';

  constructor() {
    const cycleName = process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'eAIP_22_JAN_2026';
    const airacDate = process.env.NEXT_PUBLIC_AIRAC_DATE || 'AIRAC-2026-01-22';
    this.baseUrl = `https://www.sia.aviation-civile.gouv.fr/media/dvd/${cycleName}/FRANCE/${airacDate}/html/eAIP/`;
  }

  async getCharts(icao: string): Promise<Chart[]> {
    this.pageUrl = `${this.baseUrl}FR-AD-2.${icao}-fr-FR.html`;
    
    try {
      const response = await fetch(this.pageUrl);
      if (!response.ok) {
        if (response.status === 404) return [];
        throw new Error(`SIA fetch failed: ${response.statusText}`);
      }

      const html = await response.text();
      return this.parseHtml(html, icao);
    } catch (error) {
      console.error('SIA Adapter Error:', error);
      throw error;
    }
  }

  private parseHtml(html: string, icao: string): Chart[] {
    const $ = cheerio.load(html);
    const charts: Chart[] = [];

    // Store all files for global analysis
    const rawFiles: string[] = [];
    $('a[href$=".pdf"]').each((_, element) => {
      const h = $(element).attr('href');
      if (h) rawFiles.push(h);
    });

    const airportRunways = this.detectRunways(rawFiles);
    const sortedRunways = Array.from(airportRunways).sort();

    rawFiles.forEach((href) => {
      const filename = href.split('/').pop() || '';
      const decodedFilename = decodeURIComponent(filename);

      // Filters
      if (['DATA', 'TEXT', 'TXT', 'VPE', 'PATC'].some(k => decodedFilename.includes(`_${k}_`))) {
         return;
      }

      const typeInfo = this.identifyType(decodedFilename);
      if (!typeInfo) return;

      const absoluteUrl = new URL(href, this.pageUrl).toString();
      const tags = this.generateTags(decodedFilename, typeInfo.code, sortedRunways);
      const { subtitle, page } = this.extractSubtitle(decodedFilename, icao, typeInfo.code);

      charts.push({
        id: absoluteUrl,
        source: 'SIA',
        category: typeInfo.category,
        subtitle,
        filename, // Keep original filename
        url: absoluteUrl,
        page,
        tags
      });
    });

    return this.postProcess(charts);
  }

  private detectRunways(files: string[]): Set<string> {
    const runways = new Set<string>();
    files.forEach(f => {
       const u = decodeURIComponent(f);
       this.extractRunwaysFromFilename(u).forEach(r => runways.add(r));
    });
    
    // Cleanup generic (26 vs 26L)
    const unique = Array.from(runways);
    const cleaned = unique.filter(r => {
      if (r.length === 2) {
        return !unique.some(other => other !== r && other.startsWith(r));
      }
      return true;
    });
    
    return new Set(cleaned);
  }

  private extractRunwaysFromFilename(filename: string): string[] {
    const match = filename.match(/(?:^|_)RWY[_ -]?([A-Z0-9\-\/]+)(?:_|\.)/i);
    if (match && match[1]) {
        return match[1].split(/[-_\/]/).filter(p => /^\d{2}[LRC]?$/.test(p));
    }
    return [];
  }

  private identifyType(filename: string): { code: string, category: ChartCategory } | null {
    // Map SIA codes to Generic Types
    const MAPPING: Record<string, ChartCategory> = {
      'ADC': ChartCategory.AERODROME,
      'APDC': ChartCategory.PARKING,
      'GMC': ChartCategory.GROUND,
      'IAC': ChartCategory.IAC,
      'SID': ChartCategory.SID,
      'STAR': ChartCategory.STAR,
      'VAC': ChartCategory.VAC,
      'VLC': ChartCategory.VLC,
      'TEM': ChartCategory.TEM
    };

    for (const [code, category] of Object.entries(MAPPING)) {
       if (filename.includes(`_${code}_`) || filename.includes(`_${code}.`)) {
         return { code, category };
       }
    }
    return null;
  }

  private generateTags(filename: string, code: string, airportRunways: string[]): string[] {
    const tags: string[] = [];

    // IAC specific
    if (code === 'IAC') {
        if (filename.includes('_FNA')) tags.push("App. Finale");
        if (filename.includes('_INA')) tags.push("App. Initiale");
        if (filename.includes('_VPT')) tags.push("VPT");
        if (filename.includes('_MVL')) tags.push("MVL");
    }

    // Features
    if (filename.includes('_NIGHT')) tags.push("Nuit");
    if (filename.includes('_RNAV')) tags.push("RNAV");
    if (filename.includes('_RNP')) tags.push("RNP");

    // ILS
    const catFullRegex = /(?:_| |^)CAT[\W_]*(?:1(?:_| |)?2(?:_| |)?3|I(?:_| |)?II(?:_| |)?III|123)(?:_| |\.|$)/i;
    const catTwoRegex = /(?:_| |^)CAT[\W_]*(?:1(?:_| |)?2|I(?:_| |)?II|12)(?:_| |\.|$)/i;
    const catOneRegex = /(?:_| |^)CAT[\W_]*(?:1|I)(?:_| |\.|$)/i;

    if (catFullRegex.test(filename)) tags.push("ILS I/II/III"); 
    else if (catTwoRegex.test(filename)) tags.push("ILS I/II");
    else if (catOneRegex.test(filename)) tags.push("ILS I");
    else if (filename.includes('_ILS')) tags.push("ILS");
    
    if (filename.includes('_LOC')) tags.push("LOC");

    // Runways
    this.extractRunwaysFromFilename(filename).forEach(r => {
        if (!tags.includes(r)) tags.push(r);
    });

    // Semantic Groups
    if (/RWY[_ -]?(ALL|TOUTES)/i.test(filename)) tags.push(...airportRunways);
    
    // QFU Logic
    airportRunways.forEach(r => {
       const qfu = parseInt(r.substring(0, 2)) * 10;
       if (/RWY[_ -]?WEST/i.test(filename) && qfu > 0 && qfu <= 180) tags.push(r);
       if (/RWY[_ -]?EAST/i.test(filename) && qfu > 180 && qfu <= 360) tags.push(r);
       if (/RWY[_ -]?NORTH/i.test(filename) && (qfu >= 270 || qfu <= 90)) tags.push(r);
       if (/RWY[_ -]?SOUTH/i.test(filename) && (qfu >= 90 && qfu <= 270)) tags.push(r);
    });

    return Array.from(new Set(tags));
  }

  private extractSubtitle(filename: string, icao: string, code: string): { subtitle: string, page?: string } {
      let cleanName = filename.replace('.pdf', '');
      let parts = cleanName.split('_');
      let pageInfo = '';

      // Ignore structural parts
      const ignored = ['AD', '2', icao, code];
      
      const filteredParts = parts.filter(part => {
          if (ignored.includes(part.toUpperCase())) return false;
          if (part.startsWith('RWY')) return false; // Usually redundancy in subtitle
          if (/^\d{1,2}$/.test(part)) {
              pageInfo = part;
              return false;
          }
          return true;
      });

      // Special handling: if SID/STAR/IAC, maybe add runway info solely if not parsed?
      // For now we trust the filter. 
      // Current logic in old file matches Categories + Suffix. 
      // But here `category` is a generic Enum. 
      // We should append the specific Descriptor to the subtitle if relevant.
      
      return { 
          subtitle: filteredParts.join(' '), 
          page: pageInfo 
      };
  }

  private postProcess(charts: Chart[]): Chart[] {
      // Pagination logic
      const groups = new Map<string, Chart[]>();
      charts.forEach(c => {
          const key = `${c.category}|${c.subtitle}`;
          if (!groups.has(key)) groups.set(key, []);
          groups.get(key)?.push(c);
      });

      groups.forEach(group => {
          if (group.length > 1) {
              const max = Math.max(...group.map(c => parseInt(c.page || '0')));
              if (max > 0) {
                  group.forEach(c => c.page = `${parseInt(c.page || '0')}/${group.length}`);
              }
          } else {
              if (group[0]) group[0].page = undefined;
          }
      });

      // Dedupe by URL
      return Array.from(new Map(charts.map(c => [c.url, c])).values());
  }
}
