# ATC-BOOK — Desktop

Application desktop native pour la consultation de cartes aéronautiques (eAIP) et NOTAMs, destinée aux contrôleurs et pilotes sur réseau (VATSIM/IVAO).

## Fonctionnalités

- 🔍 **Recherche multi-sources** : SIA, Atlas VAC, SupAIP, UK NATS, SOFIA (NOTAMs)
- 📄 **Rendu PDF natif** : Haute fidélité via pdfium-render, avec zoom, fit-to-width et drag-to-pan
- 📁 **Dossiers de travail** : Regrouper des cartes par position/secteur, avec restauration d'onglets et notes de briefing
- 📅 **Cycle AIRAC automatique** : Calcul dynamique à partir de l'époque OACI
- 💾 **Persistance locale** : SQLite embarqué (cache de recherche, cache PDF, dossiers)

## Prérequis

- [Rust](https://rustup.rs/) (edition 2024)
- [Dioxus CLI](https://dioxuslabs.com/) :
  ```bash
  curl -sSL http://dioxus.dev/install.sh | sh
  ```

## Lancement

Depuis la racine du workspace :

```bash
dx serve --package atc-book
```

Ou depuis le crate :

```bash
cd crates/atc-book
dx serve
```

## Architecture

```
crates/atc-book/
├── src/
│   ├── main.rs            # Point d'entrée Dioxus
│   ├── models.rs          # Structures de données (Chart, Workspace, Notice)
│   ├── state.rs           # État global (signaux Dioxus)
│   ├── airac.rs           # Calcul du cycle AIRAC
│   ├── pdf.rs             # Pipeline de rendu PDF (pdfium → PNG → base64)
│   ├── adapters/          # Adaptateurs de données (SIA, Atlas, SupAIP, UK, SOFIA)
│   ├── persistence/       # Couche SQLite (cache, dossiers)
│   └── components/        # Composants UI (layout, navigator, workspace, sidebar)
└── assets/
    └── main.css           # Design system (palette ambre/ardoise)
```

## Technologies

- **Rust** + **Dioxus 0.7** (mode Desktop / WebView)
- **pdfium-render** (rendu PDF natif)
- **rusqlite** (SQLite embarqué, WAL)
- **reqwest** + **scraper** (HTTP / parsing HTML)
- **chrono**, **uuid**, **serde_json**

## Crédits

Réalisé par **Stardust Citizen**.
*YouTube : [Stardust Citizen](https://youtube.com/channel/UCoeiQSBuqp3oFpK16nQT1_Q/)*

---
*Cet outil est destiné à la simulation de vol uniquement. Ne pas utiliser pour la navigation réelle.*
