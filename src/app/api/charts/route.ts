import { NextResponse } from 'next/server';
import { getCharts, AVAILABLE_SOURCES } from '@/lib/charts';

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const icao = searchParams.get('icao')?.toUpperCase();
  // Default to SIA for now as it's the only one implemented
  const source = searchParams.get('source')?.toUpperCase() || 'SIA';

  // SECURITY: Ensure ICAO is strictly 4 alphanumeric chars
  if (!icao || !/^[A-Z0-9]{4}$/.test(icao)) {
    return NextResponse.json(
      { error: 'Code ICAO invalide. Il doit faire 4 caractères alphanumériques.' },
      { status: 400 }
    );
  }

  // Provider Check
  if (!AVAILABLE_SOURCES.includes(source)) {
      return NextResponse.json(
        { error: `Source inconnue: ${source}. Sources valides: ${AVAILABLE_SOURCES.join(', ')}` },
        { status: 400 }
      );
  }

  try {
    const charts = await getCharts(source, icao);

    if (charts.length === 0) {
      // If result is empty, it might be 404
      // Adapter returns [] on 404 to avoid throwing control flow exceptions
      return NextResponse.json(
        { error: `Aérodrome ${icao} introuvable ou aucune carte disponible.` },
        { status: 404 }
      );
    }
    
    return NextResponse.json({ 
      icao, 
      count: charts.length, 
      charts 
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
