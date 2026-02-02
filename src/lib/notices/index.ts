import { Notice, NoticeProvider } from './types';
import { SofiaAdapter } from './adapters/sofia';

const providers: NoticeProvider[] = [
  new SofiaAdapter(),
];

export async function getNotices(icao: string): Promise<Notice[]> {
  // Logic to select providers based on ICAO
  // For now, only SOFIA for France, and we can default to it or check prefix.
  const applicableProviders = providers.filter(p => {
    if (p.name === 'SOFIA') return icao.startsWith('LF');
    return false;
  });

  const results = await Promise.all(
    applicableProviders.map(p => 
      p.getNotices(icao).catch(err => {
        console.error(`Error fetching notices from ${p.name}:`, err);
        return [];
      })
    )
  );

  return results.flat();
}
