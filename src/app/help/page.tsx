'use client';

import { useTranslation } from 'react-i18next';
import Link from 'next/link';
import { ThemeToggle } from '@/components/ThemeToggle';
import '../../lib/i18n'; // Ensure i18n init

export default function HelpPage() {
  const { t } = useTranslation();

  return (
    <main className="min-h-screen bg-background text-foreground font-sans flex flex-col items-center py-12 px-4 relative">
       <div className="absolute top-4 left-4 z-40 flex gap-2 items-center">
           <Link href="/" data-testid="help-back-link" className="flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors pr-4 border-r border-border">
                <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                </svg>
                <span className="font-semibold">Retour</span>
           </Link>
           <ThemeToggle />
       </div>

       <div className="max-w-3xl w-full bg-card border border-border rounded-2xl p-8 shadow-xl mt-8">
            <header className="text-center mb-10 space-y-2">
                <h1 className="text-4xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 dark:from-blue-400 dark:to-indigo-400 bg-clip-text text-transparent">
                    {t('help_title')}
                </h1>
                <p className="text-muted-foreground text-lg">
                    {t('help_intro')}
                </p>
            </header>

            <div className="space-y-8">
                {/* Getting Started */}
                <section className="flex gap-4">
                    <div className="flex-shrink-0 mt-1">
                        <div className="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary">
                            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                            </svg>
                        </div>
                    </div>
                    <div>
                        <h2 className="text-xl font-bold mb-2">{t('help_getting_started')}</h2>
                        <p className="text-muted-foreground leading-relaxed">{t('help_getting_started_text')}</p>
                    </div>
                </section>

                <div className="h-px bg-border/50"></div>

                {/* Dock */}
                <section className="flex gap-4">
                    <div className="flex-shrink-0 mt-1">
                        <div className="w-10 h-10 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-500">
                             <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                            </svg>
                        </div>
                    </div>
                    <div>
                        <h2 className="text-xl font-bold mb-2">{t('help_dock')}</h2>
                        <p className="text-muted-foreground leading-relaxed">{t('help_dock_text')}</p>
                    </div>
                </section>

                <div className="h-px bg-border/50"></div>

                 {/* Scratchpad */}
                 <section className="flex gap-4">
                    <div className="flex-shrink-0 mt-1">
                        <div className="w-10 h-10 rounded-full bg-green-500/10 flex items-center justify-center text-green-500">
                             <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                 <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                             </svg>
                        </div>
                    </div>
                    <div>
                        <h2 className="text-xl font-bold mb-2">{t('help_scratchpad')}</h2>
                        <p className="text-muted-foreground leading-relaxed">{t('help_scratchpad_text')}</p>
                    </div>
                </section>

                <div className="h-px bg-border/50"></div>

                 {/* Tips */}
                 <section className="flex gap-4">
                    <div className="flex-shrink-0 mt-1">
                         <div className="w-10 h-10 rounded-full bg-amber-500/10 flex items-center justify-center text-amber-500">
                            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                        </div>
                    </div>
                    <div>
                        <h2 className="text-xl font-bold mb-2">{t('help_tips')}</h2>
                        <p className="text-muted-foreground leading-relaxed">{t('help_tips_text')}</p>
                    </div>
                </section>
            </div>

            <div className="mt-12 text-center">
                 <Link href="/" className="inline-flex items-center justify-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-primary hover:bg-primary/90 transition-colors shadow-lg">
                    {t('help_getting_started')}
                 </Link>
            </div>
       </div>
    </main>
  );
}
