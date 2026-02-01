# ATC BOOK

Outil pour les contrôleurs et pilotes (VATSIM/IVAO) permettant de rechercher, filtrer et télécharger instantanément les cartes aéronautiques du SIA (Service de l'Information Aéronautique).

![ATC BOOK Interface](https://via.placeholder.com/800x400?text=Interface+ATC+BOOK)

## Fonctionnalités

- 🔍 **Recherche OACI** : Récupération instantanée des cartes (ex: LFPG, LFPO).
- 🏷️ **Filtres Intelligents** : Filtrage par tags (ILS, Pistes, Parking, SID/STAR...).
- 📦 **Téléchargement Groupé** :
  - **ZIP** : Télécharger une sélection de cartes en une archive.
  - **PDF Unique** : Fusionner plusieurs cartes en un seul document PDF.
- 🌑 **Interface Moderne** : Thème sombre et responsive (mobile/desktop).
- 🛡️ **Proxy Sécurisé** : Contournement des restrictions CORS du SIA avec validation de sécurité.

## Configuration (Variables d'environnement)

Pour fonctionner correctement (et cibler le bon cycle AIRAC), l'application nécessite les variables d'environnement suivantes. 

Créez un fichier `.env.local` à la racine pour le développement, ou configurez ces variables dans votre hébergeur (Vercel, Netlify...) :

```bash
# Exemple pour le cycle de Janvier 2026
NEXT_PUBLIC_AIRAC_CYCLE_NAME=eAIP_22_JAN_2026
NEXT_PUBLIC_AIRAC_DATE=AIRAC-2026-01-22
```

> **Note :** Ces valeurs doivent être mises à jour à chaque nouveau cycle AIRAC (tous les 28 jours) pour continuer à récupérer les cartes valides depuis le site du SIA.

## Installation Locale

1.  Cloner le dépôt :
    ```bash
    git clone https://github.com/votre-user/atc-book.git
    cd atc-book
    ```
2.  Installer les dépendances :
    ```bash
    npm install
    ```
3.  Lancer le serveur de développement :
    ```bash
    npm run dev
    ```
4.  Ouvrir [http://localhost:3000](http://localhost:3000).

## Déploiement

Ce projet est conçu pour être déployé facilement sur **Vercel**.

1.  Poussez votre code sur un dépôt Git.
2.  Importez le projet sur Vercel.
3.  **IMPORTANT** : Ajoutez les variables d'environnement (`NEXT_PUBLIC_AIRAC_CYCLE_NAME`, `NEXT_PUBLIC_AIRAC_DATE`) dans les paramètres du projet Vercel.

## Technologies

- **Next.js 16** (App Router)
- **TypeScript**
- **Tailwind CSS**
- **Cheerio** (Scraping)
- **PDF-Lib** (Fusion PDF)
- **JSZip** (Création d'archives)

## Crédits

Réalisé par **Stardust Citizen**.
*YouTube : [Stardust Citizen](https://youtube.com/channel/UCoeiQSBuqp3oFpK16nQT1_Q/)*

---
*Cet outil est destiné à la simulation de vol uniquement. Ne pas utiliser pour la navigation réelle.*
