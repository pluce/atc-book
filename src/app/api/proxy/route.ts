import { NextResponse } from 'next/server';

const ALLOWED_DOMAINS = [
  'www.sia.aviation-civile.gouv.fr',
  'www.aurora.nats.co.uk'
];

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const targetUrl = searchParams.get('url');

  if (!targetUrl) {
    return new NextResponse('Missing URL parameter', { status: 400 });
  }

  let parsedUrl: URL;
  try {
      parsedUrl = new URL(targetUrl);
  } catch {
      return new NextResponse('Invalid URL', { status: 400 });
  }

  // SECURITY: Whitelist Domain Check
  if (!ALLOWED_DOMAINS.includes(parsedUrl.hostname)) {
      return new NextResponse(`Domain not allowed: ${parsedUrl.hostname}`, { status: 403 });
  }

  // SECURITY: Filename Check (Must be PDF)
  if (!parsedUrl.pathname.toLowerCase().endsWith('.pdf')) {
      return new NextResponse('Target must be a PDF', { status: 400 });
  }

  try {
    const response = await fetch(targetUrl);
    
    if (!response.ok) {
        return new NextResponse(`Upstream error: ${response.status}`, { status: response.status });
    }

    const data = await response.arrayBuffer();
    const headers = new Headers();
    headers.set('Content-Type', 'application/pdf');
    headers.set('Cache-Control', 'public, max-age=3600');
    // Forward Content-Length if available
    const len = response.headers.get('Content-Length');
    if (len) headers.set('Content-Length', len);

    return new NextResponse(data, {
        status: 200,
        headers
    });

  } catch (err) {
      console.error('Proxy Error:', err);
      return new NextResponse('Internal Server Error', { status: 500 });
  }
}