import { NextResponse } from 'next/server';
import * as cheerio from 'cheerio';

// Helper to parse runways from filename
// e.g. "AD_2_LFPO_SID_RWY06-07_RNAV..." -> ["06", "07"]
function extractRunways(filename: string): string[] {
    const match = filename.match(/(?:^|_)RWY[_ -]?([A-Z0-9\-\/]+)(?:_|\.)/i);
    if (match && match[1]) {
        // Split by typical separators found IN the segment (-, _, /)
        return match[1].split(/[-_\/]/).filter(p => /^\d{2}[LRC]?$/.test(p));
    }
    return [];
}

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const icao = searchParams.get('icao')?.toUpperCase();

  if (!icao || icao.length !== 4) {
    return NextResponse.json(
      { error: 'Code ICAO invalide. Il doit faire 4 caractères.' },
      { status: 400 }
    );
  }

  // URL structure found:
  // https://www.sia.aviation-civile.gouv.fr/media/dvd/eAIP_22_JAN_2026/FRANCE/AIRAC-2026-01-22/html/eAIP/FR-AD-2.LFPG-fr-FR.html
  const cycleName = process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'eAIP_22_JAN_2026';
  const airacDate = process.env.NEXT_PUBLIC_AIRAC_DATE || 'AIRAC-2026-01-22';
  const baseUrl = `https://www.sia.aviation-civile.gouv.fr/media/dvd/${cycleName}/FRANCE/${airacDate}/html/eAIP/`;
  const pageUrl = `${baseUrl}FR-AD-2.${icao}-fr-FR.html`;

  try {
    const response = await fetch(pageUrl);

    if (!response.ok) {
      if (response.status === 404) {
        return NextResponse.json(
          { error: `Aérodrome ${icao} introuvable sur le site du SIA.` },
          { status: 404 }
        );
      }
      throw new Error(`Erreur lors de la récupération de la page: ${response.statusText}`);
    }

    const html = await response.text();
    const $ = cheerio.load(html);

    interface ChartResult {
        category: string;
        subtitle: string;
        filename: string;
        url: string;
        page?: string;
        tags: string[];
    }

    const charts: ChartResult[] = [];

    // Mots-clés pour identifier les types de cartes et leur donner un titre lisible
    const CHART_TYPES: Record<string, string> = {
      'ADC': "Carte d'aérodrome",
      'APDC': "Stationnement",
      'GMC': "Mouvements à la surface",
      'IAC': "Approche aux instruments",
      'SID': "Départs (SID)",
      'STAR': "Arrivées (STAR)",
      'VAC': "Approche à vue",
      'VLC': "Atterrissage à vue",
      'TEM': "Carte de mouvements (Temps)"
    };

    // Store all files first to perform global analysis (Runway detection)
    const rawFiles: string[] = [];
    $('a[href$=".pdf"]').each((_, element) => {
        const h = $(element).attr('href');
        if (h) rawFiles.push(h);
    });

    // 1. Identify all explicit runways mentioned in filenames for this airport
    const airportRunways = new Set<string>();
    rawFiles.forEach(href => {
        const filename = href.split('/').pop() || '';
        extractRunways(filename).forEach(r => airportRunways.add(r));
    });

    // Clean up runways: remove generic number if specific L/R/C exists
    // ex: If we have 26L and 26R, we remove 26 if it exists.
    // If we only have 26, we keep it.
    const uniqueRunways = Array.from(airportRunways);
    const cleanedRunways = uniqueRunways.filter(r => {
        // If length is 2 (e.g. 26), check if specific versions exist (26L, 26R, 26C)
        if (r.length === 2) {
             const hasSpecific = uniqueRunways.some(other => other !== r && other.startsWith(r));
             return !hasSpecific;
        }
        return true;
    });

    const sortedRunways = cleanedRunways.sort();

    rawFiles.forEach((href) => {
      if (href) {
        const filename = href.split('/').pop() || '';

        // Exclusions explicites (Données textuelles, tableaux, etc.) + Demande utilisateur (VPE, PATC)
        if (filename.includes('_DATA_') || filename.includes('_TEXT_') || filename.includes('_TXT_') || filename.includes('_VPE_') || filename.includes('_PATC_')) {
          return; // On passe au suivant
        }
        
        let matchedCode = null;
        let matchedCategory = null;

        // Identification du type
        for (const [code, label] of Object.entries(CHART_TYPES)) {
          // On cherche _CODE_ ou _CODE.pdf ou similaire. 
          // Ex: AD_2_LFPG_SID_...
          if (filename.includes(`_${code}_`) || filename.includes(`_${code}.`)) {
            matchedCode = code;
            matchedCategory = label;
            break; 
          }
        }

        if (matchedCode && matchedCategory) {
          const absoluteUrl = new URL(href, pageUrl).toString();

          // TAG GENERATION
          const tags: string[] = [];
          
          // Specific IAC tags
          if (matchedCode === 'IAC') {
              if (filename.includes('_FNA')) tags.push("App. Finale");
              if (filename.includes('_INA')) tags.push("App. Initiale");
              if (filename.includes('_VPT')) tags.push("VPT");
              if (filename.includes('_MVL')) tags.push("MVL");
          }
          
          // General tags
          if (filename.includes('_NIGHT')) tags.push("Nuit");
          if (filename.includes('_RNAV')) tags.push("RNAV");
          if (filename.includes('_RNP')) tags.push("RNP");
          
          if (filename.includes('ILS_CAT_I_II_III')) tags.push("ILS I/II/III"); 
          else if (filename.includes('ILS_CAT_I_II')) tags.push("ILS I/II");
          else if (filename.includes('ILS_CAT_I')) tags.push("ILS I");
          else if (filename.includes('_ILS_CAT123')) tags.push("ILS I/II/III"); // Alternative naming if exists
          else if (filename.includes('_ILS_CAT12')) tags.push("ILS I/II");
          else if (filename.includes('_ILS_CAT1')) tags.push("ILS I");
          else if (filename.includes('_ILS')) tags.push("ILS");
          
          if (filename.includes('_LOC')) tags.push("LOC");

          // Runways Tags
          
          // 1. Explicit runways matching known airport runways
          extractRunways(filename).forEach(r => {
             if (!tags.includes(r)) tags.push(r);
          });

          // 2. Groups (ALL, WEST, EAST...)
          if (/RWY[_ -]?(ALL|TOUTES)/i.test(filename)) {
              sortedRunways.forEach(r => {
                  if (!tags.includes(r)) tags.push(r);
              });
          }
          
          // Rule: WEST -> QFU 0-180
          if (/RWY[_ -]?WEST/i.test(filename)) {
              sortedRunways.forEach(r => {
                  const qfu = parseInt(r.substring(0, 2)) * 10;
                  if (qfu > 0 && qfu <= 180) {
                       if (!tags.includes(r)) tags.push(r);
                  }
              });
          }
          
          // Rule: EAST -> QFU 180-360
          if (/RWY[_ -]?EAST/i.test(filename)) {
              sortedRunways.forEach(r => {
                  const qfu = parseInt(r.substring(0, 2)) * 10;
                  if (qfu > 180 && qfu <= 360) {
                      if (!tags.includes(r)) tags.push(r);
                  }
              });
          }
          
           // Rule: NORTH -> QFU 270-090
          if (/RWY[_ -]?NORTH/i.test(filename)) {
              sortedRunways.forEach(r => {
                  const qfu = parseInt(r.substring(0, 2)) * 10;
                  if (qfu >= 270 || qfu <= 90) {
                      if (!tags.includes(r)) tags.push(r);
                  }
              });
          }

          // Rule: SOUTH -> QFU 90-270
          if (/RWY[_ -]?SOUTH/i.test(filename)) {
              sortedRunways.forEach(r => {
                  const qfu = parseInt(r.substring(0, 2)) * 10;
                  if (qfu >= 90 && qfu <= 270) {
                      if (!tags.includes(r)) tags.push(r);
                  }
              });
          }

          // Extraction du sous-titre
          // Ex: AD_2_LFBO_SID_RWY32L-32R_RNAV_INSTR_02.pdf
          // On veut: RWY32L-32R RNAV INSTR
          
          let cleanName = filename.replace('.pdf', '');
          let parts = cleanName.split('_');

          let runwaySuffix = '';
          const runwayPartsToRemove: string[] = [];

          if (['SID', 'STAR', 'IAC'].includes(matchedCode)) {
              for (let i = 0; i < parts.length; i++) {
                  const part = parts[i];
                  if (part.startsWith('RWY')) {
                      runwaySuffix = part; 
                      runwayPartsToRemove.push(part);
                      
                      // Cas RWY_ALL, RWY_WEST etc où RWY est séparé par un underscore
                      if (part === 'RWY' && i + 1 < parts.length) {
                           const nextPart = parts[i+1];
                           runwaySuffix += ' ' + nextPart;
                           runwayPartsToRemove.push(nextPart);
                      }
                      break; 
                  }
              }
              
              if (runwaySuffix) {
                  matchedCategory = `${matchedCategory} ${runwaySuffix}`;
              }
          }
          
          // On filtre les parties "techniques" connues
          // ICAO, AD, 2, et le CODE détecté (SID, IAC...)
          const keywordsToIgnore = ['AD', '2', icao, matchedCode, ...runwayPartsToRemove];
          
          let pageInfo = '';
          
          let subtitleParts = parts.filter(part => {
             const upperPart = part.toUpperCase();
             // Si c'est un mot clé structurel, on vire
             if (keywordsToIgnore.includes(upperPart)) return false;
             
             // Si c'est juste de la pagination (chiffres seuls, souvent 2 digits à la fin)
             // On le détecte pour l'affichage mais on l'enlève du titre
             if (/^\d{1,2}$/.test(part)) {
                pageInfo = part;
                return false;
             }
             return true;
          });
          
          const subtitle = subtitleParts.join(' ');

          charts.push({
            category: matchedCategory,
            subtitle: subtitle,
            filename: filename, // On garde le filename pour ref technique
            url: absoluteUrl,
            page: pageInfo,
            tags: tags
          });
        }
      }
    });

    // Detect pagination totals
    // Group by (category + subtitle) to find sequences like ADC 01, ADC 02
    const groups = new Map<string, typeof charts>();
    charts.forEach(c => {
        const key = `${c.category}|${c.subtitle}`;
        if (!groups.has(key)) groups.set(key, []);
        groups.get(key)?.push(c);
    });

    groups.forEach(group => {
        if (group.length > 1) {
            // Assign totals if pages are detected
            const maxPage = Math.max(...group.map(c => parseInt(c.page || '0')));
            if (maxPage > 0) {
                 group.forEach(c => {
                     if (c.page) {
                         c.page = `${parseInt(c.page)}/${group.length}`;
                     }
                 });
            }
        } else {
             // Single page, clear if it was just "01"
             group[0].page = ''; 
        }
    });

    // Remove duplicates based on URL
    const uniqueCharts = Array.from(new Map(charts.map(item => [item.url, item])).values());

    return NextResponse.json({ 
      icao, 
      count: uniqueCharts.length, 
      charts: uniqueCharts 
    });

  } catch (error) {
    console.error('Error fetching data:', error);
    return NextResponse.json(
      { error: 'Erreur serveur lors de la récupération des données.' },
      { status: 500 }
    );
  }
}
