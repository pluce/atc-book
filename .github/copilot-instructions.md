# Instructions du projet "vaccfr-book"

## Checklist de démarrage
- [x] Initialiser une nouvelle application Next.js
- [x] Installer Cheerio (`npm install cheerio`)
- [x] Créer la route API `/api/charts` pour le scraping du SIA
- [x] Créer l'interface utilisateur (Page d'accueil)
- [x] Ajouter le support Dark Mode (automatique via Tailwind)
- [x] Tester la récupération pour un terrain (ex: LFPG)
- [x] Rédiger le README.md

## Notes techniques
- **Source SIA** : Le script cible la structure eAIP standard. L'URL de base est hardcodée pour ce prototype (cycle AIRAC de janvier 2026 simulé pour l'exemple, ou actuel selon disponibilité), mais l'architecture permet de la rendre dynamique plus tard.
- **Proxy** : L'API Route sert de proxy pour contourner les problèmes de CORS du site SIA et pour parser le HTML côté serveur.
- **Styling** : Tailwind CSS est utilisé pour une UI rapide et responsive.

## Commandes utiles
- `npm run dev` : Lancer le serveur de développement.
- `npm run build` : Builder pour la production (Vercel).
