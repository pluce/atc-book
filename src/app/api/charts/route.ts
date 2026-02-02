import { NextResponse } from 'next/server';
import { getCharts, AVAILABLE_SOURCES } from '@/lib/charts';

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const icao = searchParams.get('icao')?.toUpperCase();

  // SECURITY: Ensure ICAO is strictly 4 alphanumeric chars
  if (!icao || !/^[A-Z0-9]{4}$/.test(icao)) {
    return NextResponse.json(
      { error: 'Code ICAO invalide. Il doit faire 4 caractères alphanumériques.' },
      { status: 400 }
    );
  }

  // Determine sources based on ICAO prefix
  let targetSources: string[] = ['SIA']; // Default

  if (icao.startsWith('LF')) {
    targetSources = ['SIA', 'ATLAS'];
  } else if (icao.startsWith('EG')) {
    targetSources = ['UK'];
  }

  try {
    // Run all provider queries in parallel
    const promises = targetSources.map(async (source) => {
        try {
            return await getCharts(source, icao);
        } catch (err) {
            console.error(`Error fetching from ${source} for ${icao}:`, err);
            return []; // Fail gracefully for individual providers
        }
    });

    const results = await Promise.all(promises);
    const charts = results.flat();

    // Deduplicate by URL (just in case)
    const uniqueCharts = Array.from(new Map(charts.map(c => [c.url, c])).values());

    if (uniqueCharts.length === 0) {
      // If result is empty, it might be 404
      // Adapter returns [] on 404 to avoid throwing control flow exceptions
      return NextResponse.json(
        { error: `Aérodrome ${icao} introuvable ou aucune carte disponible.` },
        { status: 404 }
      );
    }
    
    return NextResponse.json({ 
      icao, 
      count: uniqueCharts.length, 
      charts: uniqueCharts
    });

  } catch (error: unknown) {
    console.error('API Error:', error);
    const errorMessage = error instanceof Error ? error.message : 'Erreur serveur.';
    return NextResponse.json(
      { error: errorMessage },
      { status: 500 }
    );
  }
}
