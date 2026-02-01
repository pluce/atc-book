import { ChartProvider, Chart } from './types';
import { SIAAdapter } from './adapters/sia';
import { UKAdapter } from './adapters/uk';

const providers: Record<string, ChartProvider> = {
  'SIA': new SIAAdapter(),
  'UK': new UKAdapter()
};

export async function getCharts(source: string, icao: string): Promise<Chart[]> {
  const provider = providers[source];
  if (!provider) throw new Error(`Provider ${source} not found`);
  
  return provider.getCharts(icao);
}

export const AVAILABLE_SOURCES = Object.keys(providers);
