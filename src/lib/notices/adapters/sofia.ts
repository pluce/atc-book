import { Notice, NoticeProvider } from '../types';

export class SofiaAdapter implements NoticeProvider {
  name = 'SOFIA';
  
  private async getSessionCookie(): Promise<string> {
    try {
      const res = await fetch('https://sofia-briefing.aviation-civile.gouv.fr/sofia/pages/notamroute.html', {
        method: 'HEAD',
      });
      const setCookie = res.headers.get('set-cookie');
      if (!setCookie) return '';
      // Extract JSESSIONID
      const match = setCookie.match(/JSESSIONID=([^;]+)/);
      return match ? match[1] : '';
    } catch (e) {
      console.error('Error fetching SOFIA cookie:', e);
      return '';
    }
  }

  async getNotices(icao: string): Promise<Notice[]> {
    const now = new Date();
    const sessionId = await this.getSessionCookie();
    
    if (!sessionId) {
      console.error('Could not retrieve JSESSIONID from SOFIA');
      return [];
    }
    
    // Format dates
    const pad = (n: number) => n.toString().padStart(2, '0');
    // valid_from seems to require ISO-ish format: YYYY-MM-DDTHH:mm:ssZ
    const validFrom = now.toISOString().split('.')[0] + 'Z';
    
    // departure_date: dd-mm-yyyy
    const depDate = `${pad(now.getDate())}-${pad(now.getMonth() + 1)}-${now.getFullYear()}`;
    // departure_time: HHMM
    const depTime = `${pad(now.getHours())}${pad(now.getMinutes())}`;

    const params = new URLSearchParams();
    params.append(':operation', 'postAreaAeroPibRequest');
    params.append('isFromSofia', 'true');
    params.append('valid_from', validFrom);
    params.append('duration', '1200'); 
    params.append('traffic', 'VI');
    params.append('fl_lower', '0');
    params.append('fl_upper', '999');
    params.append('radius', '25');
    params.append('adep', icao);
    params.append('width', '15');
    params.append('aero[]', icao);
    params.append('operation', 'postAreaAeroPibRequest');
    params.append('target', '#aside-target');
    params.append('href', '/sofia/pages/notamroute.html');
    params.append('typeVol', 'L');
    params.append('departure_date', depDate);
    params.append('departure_time', depTime);
    params.append('lang', 'fr');
    params.append('routeVal', 'false');

    try {
      const res = await fetch('https://sofia-briefing.aviation-civile.gouv.fr/sofia', {
        method: 'POST',
        headers: {
          'Accept': 'application/json, text/javascript, */*; q=0.01',
          'Cookie': `JSESSIONID=${sessionId}`,
          'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'
        },
        body: params
      });

      if (!res.ok) {
          console.error('SOFIA Error', res.status, res.statusText);
          return [];
      }

      const json = await res.json();
      if (!json['status.message']) return [];

      let messageData;
      try {
          messageData = JSON.parse(json['status.message']);
      } catch (e) {
          console.error('Failed to parse inner status.message', e);
          return [];
      }

      const notices: Notice[] = [];
      const processedIds = new Set<string>();
      
      // Parsing logic
      const extractNotams = (obj: any) => {
          if (!obj) return;
          if (Array.isArray(obj)) {
              obj.forEach(item => extractNotams(item));
          } else if (typeof obj === 'object') {
              // Check if it's a NOTAM object
              if (obj.id && obj.series && obj.number && obj.itemE && !processedIds.has(obj.id)) {
                  processedIds.add(obj.id);
                  notices.push(this.mapToNotice(obj, icao));
              } 
              // Always dig deeper, even if we found a notam (though usually leaves are notams)
              // But notams don't contain other notams, so maybe else is fine.
              // However, the structure has arrays of objects.
              
              if (!obj.id) { // Optimize: dont search inside a NOTAM object
                Object.values(obj).forEach(val => extractNotams(val));
              }
          }
      };

      if (messageData.listnotams) {
          extractNotams(messageData.listnotams);
      }

      return notices.sort((a, b) => {
        // Sort by startValidity descending (newest first) or identifier
        return b.validFrom.localeCompare(a.validFrom);
      });

    } catch (error) {
      console.error('Error in SofiaAdapter:', error);
      return [];
    }
  }
  
  private mapToNotice(raw: any, defaultIcao: string): Notice {
      const series = raw.series || '';
      const number = raw.number ? raw.number.toString().padStart(4, '0') : '0000';
      const year = raw.year ? raw.year.toString().padStart(2, '0') : '00';
      const identifier = `${series}${number}/${year}`;
      
      const content = raw.multiLanguage?.itemE || raw.itemE || '';
      
      return {
          id: raw.id,
          icao: raw.itemA ? raw.itemA.split(' ')[0] : defaultIcao, 
          source: 'SOFIA',
          identifier,
          type: raw.type,
          validFrom: raw.startValidity,
          validTo: raw.endValidity,
          content: content,
          location: raw.coordinates,
          category: raw.qLine?.code23, 
      };
  }
}
