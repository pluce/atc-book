# VaccFR Book

Outil simple pour récupérer les cartes aéronautiques (SIA France) pour la simulation aérienne (VATSIM/IVAO).

## Fonctionnalités

- Recherche par code OACI (ex: LFPG, LFPO).
- Récupération automatique des liens PDF depuis le site du SIA (eAIP AIRAC courant).
- Interface moderne et sombre (Dark Mode).
- Liens directs vers les PDFs officiels.

## Installation

1.  Cloner le dépôt.
2.  Installer les dépendances :
    ```bash
    npm install
    ```
3.  Lancer le serveur de développement :
    ```bash
    npm run dev
    ```
4.  Ouvrir [http://localhost:3000](http://localhost:3000).

## Déploiement sur Vercel

Ce projet est optimisé pour Vercel.

1.  Poussez le code sur GitHub/GitLab/Bitbucket.
2.  Importez le projet dans Vercel.
3.  Le déploiement est automatique.

## Technologies

- Next.js 14 (App Router)
- Tailwind CSS
- Cheerio (Parsing HTML)
- TypeScript
