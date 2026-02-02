
import { Chart, ChartCategory, ChartProvider } from '../types';
import * as cheerio from 'cheerio';

const SIA_SUPAIP_URL = 'https://www.sia.aviation-civile.gouv.fr/documents/supaip/aip/id/6';

export class SupAIPAdapter implements ChartProvider {
  name = 'SUPAIP';

  async getCharts(icao: string): Promise<Chart[]> {
    try {
      // Step 1: Get the Session & Form Key (HEAD/GET)
      const initialResp = await fetch(SIA_SUPAIP_URL, {
        headers: {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        }
      });
      
      const setCookie = initialResp.headers.get('set-cookie');
      const htmlBody = await initialResp.text();
      
      // Extract form_key from HTML
      const $init = cheerio.load(htmlBody);
      const formKeyInput = $init('input[name="form_key"]');
      const formKey = formKeyInput.val() as string;

      if (!formKey) {
          console.error('[SUPAIP] Cannot parse form_key. Body length:', htmlBody.length);
          return [];
      }
      
      console.log(`[SUPAIP] Found form_key: ${formKey} for ${icao}`);

      // Step 2: POST Search
      // Cookies need to be passed forward
      
      let cookieHeader = '';
      // @ts-ignore - getSetCookie might be available in newer Node envs
      if (typeof initialResp.headers.getSetCookie === 'function') {
          // @ts-ignore
          const cookies = initialResp.headers.getSetCookie();
          // @ts-ignore
          cookieHeader = cookies.map(c => c.split(';')[0]).join('; ');
      } else {
          const setCookie = initialResp.headers.get('set-cookie');
          cookieHeader = setCookie ? setCookie.split(',').map(c => c.split(';')[0]).join('; ') : '';
      }

      // Ensure form_key is in cookies if not present (Magento often requires it)
      if (!cookieHeader.includes('form_key=')) {
          cookieHeader += (cookieHeader ? '; ' : '') + `form_key=${formKey}`;
      }
      
      console.log(`[SUPAIP] Cookies for POST: ${cookieHeader}`);

      const searchParams = new URLSearchParams();
      // title=&location=LFBO&form_key=eJT82DD7g1kfQsLz
      searchParams.append('title', '');
      searchParams.append('location', icao);
      searchParams.append('form_key', formKey);

      const searchResp = await fetch(SIA_SUPAIP_URL, {
        method: 'POST',
        headers: {
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Content-Type': 'application/x-www-form-urlencoded',
            'Cookie': cookieHeader,
            'Origin': 'https://www.sia.aviation-civile.gouv.fr',
            'Referer': SIA_SUPAIP_URL
        },
        body: searchParams.toString()
      });

      if (!searchResp.ok) {
          throw new Error(`SIA SupAIP responded with ${searchResp.status}`);
      }

      const responseHtml = await searchResp.text();
      const $ = cheerio.load(responseHtml);

      const foundLinks = $('a.lien_sup_aip').length;
      console.log(`[SUPAIP] Search response status: ${searchResp.status}, Found links: ${foundLinks}`);

      // Collect raw links first
      const candidates: { url: string; text: string }[] = [];
      
      $('a.lien_sup_aip').each((_, el) => {
        const link = $(el);
        let url = link.attr('href');
        const text = link.text().trim(); 

        if (url) {
             // Handle relative URLs
             if (!url.startsWith('http')) {
                 if (url.startsWith('/')) {
                     url = `https://www.sia.aviation-civile.gouv.fr${url}`;
                 } else {
                     url = `https://www.sia.aviation-civile.gouv.fr/documents/supaip/aip/id/6/${url}`;
                 }
             }
             candidates.push({ url, text });
        }
      });

      // Resolve links in parallel (HEAD request to follow 302 and get real PDF url)
      const resolvedCharts = await Promise.all(candidates.map(async ({ url, text }) => {
          try {
              console.log(`[SUPAIP] Resolving: ${url}`);
              const headResp = await fetch(url, {
                  method: 'HEAD',
                  redirect: 'follow', // Follow redirects to get final URL
                  headers: {
                      'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
                      'Cookie': cookieHeader
                  }
              });

              const finalUrl = headResp.url;
              const contentType = headResp.headers.get('content-type');
              
              // Accept if extension is PDF OR Content-Type is PDF
              // Often HEAD request to download link returns the final URL ending in .pdf
              if (finalUrl.toLowerCase().endsWith('.pdf') || contentType?.includes('application/pdf')) {
                  const parts = text.split(' - ');
                  const identifier = parts[0] || 'Unknown';
                  const title = parts.slice(1).join(' - ') || identifier;
                  
                  return {
                    id: `supaip-${icao}-${identifier.replace(/\//g,'-')}`,
                    source: 'SIA',
                    category: ChartCategory.SUPAIP, 
                    subtitle: `SUPAIP ${identifier}`,
                    filename: title,
                    url: finalUrl,
                    tags: ['SUPAIP'] 
                  } as Chart;
              } else {
                  console.log(`[SUPAIP] Skipped, not a PDF after resolve: ${finalUrl} (${contentType})`);
                  return null;
              }
          } catch (e) {
              console.error(`[SUPAIP] Failed to resolve ${url}:`, e);
              return null;
          }
      }));

      return resolvedCharts.filter((c): c is Chart => c !== null);

    } catch (error) {
      console.error(`[SUPAIP] Error fetching for ${icao}:`, error);
      return [];
    }
  }
}
