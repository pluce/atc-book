import { Chart, ChartCategory, ChartProvider } from '../types';

export class AtlasVACAdapter implements ChartProvider {
  name = 'Atlas VAC';
  
  constructor() {}

  async getCharts(icao: string): Promise<Chart[]> {
    const cycleName = process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'eAIP_22_JAN_2026';
    // URL pattern: .../Atlas-VAC/PDF_AIPparSSection/VAC/AD/AD-2.<ICAO>.pdf
    const url = `https://www.sia.aviation-civile.gouv.fr/media/dvd/${cycleName}/Atlas-VAC/PDF_AIPparSSection/VAC/AD/AD-2.${icao}.pdf`;

    try {
      // Check for existence using HEAD request to avoid downloading the full PDF
      const response = await fetch(url, { method: 'HEAD' });
      
      if (!response.ok) {
        if (response.status === 404) return [];
        // For other errors, we might want to log or ignore. 
        // If the service is down, maybe return empty?
        console.warn(`Atlas VAC check failed for ${icao}: ${response.status}`);
        return [];
      }

      // If exists, return the chart
      return [{
        id: url,
        source: 'ATLAS',
        category: ChartCategory.VAC,
        subtitle: 'Carte VAC',
        filename: `AD-2.${icao}.pdf`,
        url: url,
        tags: ['VAC']
      }];

    } catch (error) {
      console.error('Atlas VAC Adapter Error:', error);
      return [];
    }
  }
}
