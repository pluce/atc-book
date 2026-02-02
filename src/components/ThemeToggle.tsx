"use client";

import { useTheme } from "./ThemeProvider";
import { useEffect, useState } from "react";

export function ThemeToggle() {
  const { setTheme, theme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const toggleTheme = () => {
    if (theme === 'dark') {
      setTheme('light');
    } else if (theme === 'light') {
      setTheme('dark');
    } else {
      // If system, switch to the opposite of what system currently is
      const isSystemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      setTheme(isSystemDark ? "light" : "dark");
    }
  };

  if (!mounted) {
    return (
        <div className="w-9 h-9"></div> // Placeholder
    );
  }

  return (
    <button
      onClick={toggleTheme}
      data-testid="theme-toggle"
      className="p-2 rounded-lg bg-secondary text-secondary-foreground hover:bg-secondary/80 border border-border transition-colors animate-fade-in"
      title={theme === "light" || (theme === "system" && !window.matchMedia("(prefers-color-scheme: dark)").matches) ? "Passer en mode sombre" : "Passer en mode clair"}
    >
      {theme === "light" || (theme === "system" && !window.matchMedia("(prefers-color-scheme: dark)").matches) ? (
         <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-foreground">
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
         </svg>
      ) : (
         <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-amber-400">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2" />
            <path d="M12 20v2" />
            <path d="m4.93 4.93 1.41 1.41" />
            <path d="m17.66 17.66 1.41 1.41" />
            <path d="M2 12h2" />
            <path d="M20 12h2" />
            <path d="m6.34 17.66-1.41 1.41" />
            <path d="m19.07 4.93-1.41 1.41" />
         </svg>
      )}
    </button>
  );
}
