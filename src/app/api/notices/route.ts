import { NextRequest, NextResponse } from 'next/server';
import { getNotices } from '@/lib/notices';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const icao = searchParams.get('icao');

  if (!icao) {
    return NextResponse.json(
      { error: 'ICAO code is required' },
      { status: 400 }
    );
  }

  try {
    const notices = await getNotices(icao.toUpperCase());
    return NextResponse.json({ notices });
  } catch (error) {
    console.error('Error in notices API:', error);
    return NextResponse.json(
      { error: 'Failed to fetch notices' },
      { status: 500 }
    );
  }
}
