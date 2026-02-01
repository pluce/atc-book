import { Chart, ChartCategory, ChartProvider } from '../types';
import * as cheerio from 'cheerio';

export class UKAdapter implements ChartProvider {
  name = 'UK';
  
  private baseUrl: string;
  private pageUrl: string = '';

  constructor() {
    // Defines the base URL for the AIRAC cycle
    // We use a dedicated ENV variable for NATS as format differs or cycle might not align with SIA
    const airacPart = process.env.NEXT_PUBLIC_NATS_AIRAC_URL_PART || '2026-01-22-AIRAC'; 
    this.baseUrl = `https://www.aurora.nats.co.uk/htmlAIP/Publications/${airacPart}/html/eAIP/`;
  }

  async getCharts(icao: string): Promise<Chart[]> {
    // UK eAIP format: EG-AD-2.EGLL-en-GB.html
    this.pageUrl = `${this.baseUrl}EG-AD-2.${icao}-en-GB.html`;
    
    try {
      const response = await fetch(this.pageUrl);
      if (!response.ok) {
        if (response.status === 404) return [];
        throw new Error(`UK AIP fetch failed: ${response.statusText}`);
      }

      const html = await response.text();
      return this.parseHtml(html);
    } catch (error) {
       console.error('UK Adapter Error:', error);
       throw error;
    }
  }

  private parseHtml(html: string): Chart[] {
    const $ = cheerio.load(html);
    const charts: Chart[] = [];

    // UK AIP usually lists charts in tables (AD 2.24)
    // We look for all PDF links in the document to coverage embedded charts too
    $('a[href$=".pdf"]').each((_, element) => {
      const $el = $(element);
      const href = $el.attr('href');
      if (!href) return;

      const title = $el.text().trim() || $el.attr('title') || '';
      
      // Attempt to get more context if the link text is short (e.g., just "PDF")
      let fullTitle = title;
      const row = $el.closest('tr');
      const prevRow = row.prev();

      // NATS layout often puts the title in the previous row
      // Row 1: <p>AERODROME CHART - ICAO</p>
      // Row 2: <a href...>AD 2.EGLL...</a>
      if (prevRow.length > 0) {
          const prevText = prevRow.text().replace(/\s+/g, ' ').trim();
          // If previous text exists and seems descriptive (longer than 3 chars), prefer it or combine it
          if (prevText.length > 3) {
              fullTitle = prevText;
          }
      } else if (row.length > 0) {
          // Fallback: If inside a table but no prev row (unlikely for this layout), check current row text
          // But exclude the link text itself to avoid duplication
          const rowText = row.text().replace(title, '').replace(/\s+/g, ' ').trim();
          if (rowText.length > 3) fullTitle = rowText;
      }
      
      const decodedFilename = decodeURIComponent(href.split('/').pop() || '');
      const typeInfo = this.identifyType(fullTitle, decodedFilename);
      
      // If we can't identify the type, it might not be a chart (e.g. textual annex)
      // UK AIP charts usually have clear names "Instrument Approach Chart", "Aerodrome Chart"
      if (!typeInfo) return;

      const absoluteUrl = new URL(href, this.pageUrl).toString();
      const tags = this.generateTags(fullTitle);
      const { subtitle, page } = this.extractSubtitle(fullTitle, typeInfo.category);

      charts.push({
        id: absoluteUrl,
        source: 'UK',
        category: typeInfo.category,
        subtitle: subtitle || title, // Fallback to link text
        filename: decodedFilename,
        url: absoluteUrl,
        page,
        tags
      });
    });

    return this.postProcess(charts);
  }

  private identifyType(text: string, filename: string): { category: ChartCategory } | null {
    const t = text.toUpperCase();
    const f = filename.toUpperCase();

    if (t.includes('AERODROME CHART') || t.includes('AERODROME OBSTACLE')) return { category: ChartCategory.AERODROME };
    if (t.includes('AIRCRAFT PARKING') || t.includes('DOCKING')) return { category: ChartCategory.PARKING };
    if (t.includes('GROUND MOVEMENT') || t.includes('TAXI')) return { category: ChartCategory.GROUND };
    
    // Approaches
    if (t.includes('INSTRUMENT APPROACH') || t.includes('IAC') || f.includes('IAC')) return { category: ChartCategory.IAC };
    if (t.includes('VISUAL APPROACH') || t.includes('VAC')) return { category: ChartCategory.VAC };
    
    // Departments / Arrivals
    if (t.includes('STANDARD DEPARTURE') || t.includes('SID')) return { category: ChartCategory.SID };
    if (t.includes('STANDARD ARRIVAL') || t.includes('STAR')) return { category: ChartCategory.STAR };
    
    // UK specific "Initial Approach Procedures"
    if (t.includes('INITIAL APPROACH')) return { category: ChartCategory.IAC }; 

    if (t.includes('ATC SURVEILLANCE') || t.includes('MINIMUM ALTITUDE')) return { category: ChartCategory.IAC };

    return null;
  }

  private generateTags(text: string): string[] {
    const tags: string[] = [];
    const t = text.toUpperCase();

    // Runways
    // Look for RWY 09L, RWY 27, Runway 27R etc.
    const rwyMatch = t.match(/(?:RWY|RUNWAY)[\s]*(\d{2}[LRC]?)/g);
    if (rwyMatch) {
       rwyMatch.forEach(m => {
           const r = m.replace(/(RWY|RUNWAY)[\s]*/, '').trim();
           tags.push(r);
       });
    }

    // Procedures
    if (t.includes('ILS')) tags.push('ILS');
    if (t.includes('LOC')) tags.push('LOC');
    if (t.includes('RNP')) tags.push('RNP');
    if (t.includes('RNAV')) tags.push('RNAV');
    if (t.includes('VOR')) tags.push('VOR');
    if (t.includes('NDB')) tags.push('NDB');
    if (t.includes('DME')) tags.push('DME');
    if (t.includes('VISUAL')) tags.push('VISUAL');

    // CAT categories (UK often lists CAT II/III in title)
    if (t.includes('CAT II') || t.includes('CAT III')) tags.push('ILS I/II/III');

    return Array.from(new Set(tags));
  }

  private extractSubtitle(fullTitle: string, category: ChartCategory): { subtitle: string, page?: string } {
    // UK Titles are often verbose: "AD 2.24-1 Aerodrome Chart - ICAO - 1"
    
    // Clean up common prefixes/suffixes
    let subtitle = fullTitle
        .replace(/^[A-Z]{2}\s+AD\s+2\.[^ ]+\s+/, '') // Remove "EG AD 2.EGLL..."
        .replace(/AD 2\.[^ ]+\s+/, '')
        // Specific UK cleanups requested by user
        .replace(/^INSTRUMENT APPROACH CHART\s*/i, '')
        .replace(/INITIAL APPROACH PROCEDURES?\s*/i, 'INITIAL ')
        .replace(/STANDARD ARRIVAL CHART\s*(?:-\s*INSTRUMENT\s*)?(?:\(\s*\))?\s*/i, '')
        .replace(/STANDARD DEPARTURE CHART\s*(?:-\s*INSTRUMENT\s*)?(?:\(?SID\)?\s*)?(?:\(\s*\))?\s*/i, '')
        // General cleanup
        .replace(category.toString(), '') // Remove category name if present verbatim
        .replace('Chart', '')
        .replace(' - ICAO', '')
        .replace(/\s+/g, ' ')
        .trim();

    // Remove file extensions or parens
    subtitle = subtitle.replace(/\.pdf$/i, '');

    return { subtitle };
  }

  private postProcess(charts: Chart[]): Chart[] {
      // Deduplicate by URL
      const unique = new Map<string, Chart>();
      charts.forEach(c => unique.set(c.url, c));
      return Array.from(unique.values());
  }
}
