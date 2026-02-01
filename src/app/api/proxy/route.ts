import { NextResponse } from 'next/server';

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const icao = searchParams.get('icao');
  const filename = searchParams.get('filename');

  if (!icao || !filename) {
    return new NextResponse('Missing ICAO or Filename', { status: 400 });
  }

  const cycleName = process.env.NEXT_PUBLIC_AIRAC_CYCLE_NAME || 'eAIP_22_JAN_2026';
  const airacDate = process.env.NEXT_PUBLIC_AIRAC_DATE || 'AIRAC-2026-01-22';
  
  // Reconstruct URL
  // https://www.sia.aviation-civile.gouv.fr/media/dvd/eAIP_22_JAN_2026/FRANCE/AIRAC-2026-01-22/html/eAIP/Cartes/LFPO/AD_2_LFPO_ADC_02.pdf
  const targetUrl = `https://www.sia.aviation-civile.gouv.fr/media/dvd/${cycleName}/FRANCE/${airacDate}/html/eAIP/Cartes/${icao.toUpperCase()}/${filename}`;

  try {
    const response = await fetch(targetUrl);
    
    if (!response.ok) {
        return new NextResponse(`Upstream error: ${response.status}`, { status: response.status });
    }

    const data = await response.arrayBuffer();

    // On transfère les headers pertinents
    const headers = new Headers();
    headers.set('Content-Type', response.headers.get('Content-Type') || 'application/pdf');
    if (response.headers.get('Content-Length')) {
        headers.set('Content-Length', response.headers.get('Content-Length')!);
    }
    // Cache control pour éviter de re-fetcher constamment
    headers.set('Cache-Control', 'public, max-age=3600');

    return new NextResponse(data, {
        status: 200,
        headers
    });

  } catch (err) {
      console.error('Proxy Error:', err);
      return new NextResponse('Internal Server Error', { status: 500 });
  }
}