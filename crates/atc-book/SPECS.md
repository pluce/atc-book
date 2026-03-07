# Spécifications — ATC-BOOK Desktop v1

Version desktop locale de l'application ATC-BOOK. Pas de serveur distant, pas d'IA, pas de fonctionnalités distribuées. Cette version reprend les fonctionnalités existantes de l'application web et les adapte à un client lourd natif.

---

## 1. Architecture

- **Langage :** Rust
- **Framework UI :** Dioxus 0.7+ (mode Desktop / WebView natif)
- **Gestion d'état :** Signaux Dioxus (réactivité granulaire)
- **Stockage local :** SQLite embarqué (`rusqlite`) — remplace le `localStorage` de la version web
- **Rendu PDF :** `pdfium-render` (rendu natif haute fidélité) — remplace l'iframe/proxy de la version web
- **Requêtes HTTP :** `reqwest` (scraping direct, plus besoin de proxy CORS)
- **Parsing HTML :** `scraper` ou équivalent Rust de Cheerio
- **Manipulation PDF :** `pdf-lib` équivalent Rust pour merge (ex: `lopdf` ou `pdf`)
- **Archive ZIP :** `zip` crate

---

## 2. Sources de Données (Adaptateurs)

Les adaptateurs effectuent les requêtes HTTP et le parsing directement depuis le client desktop (plus besoin de route API serveur).

### 2.1 Cartes Aéronautiques

| Adaptateur | Source | Préfixe ICAO | Méthode |
|------------|--------|--------------|---------|
| **SIA** | eAIP France | `LF*` | Scraping HTML — extraction des liens `<a href$=".pdf">` depuis la page aérodrome |
| **Atlas VAC** | Atlas VAC (SIA) | `LF*` | Requête HEAD pour vérifier l'existence du PDF `AD-2.{ICAO}.pdf` |
| **SupAIP** | SUP AIP France | `LF*` | GET page + extraction `form_key`, puis POST recherche ICAO, résolution HEAD des liens PDF |
| **UK NATS** | eAIP Royaume-Uni (Aurora) | `EG*` | Scraping HTML — extraction des liens PDF avec contexte des lignes de tableau |

### 2.2 NOTAM

| Adaptateur | Source | Préfixe ICAO | Méthode |
|------------|--------|--------------|---------|
| **SOFIA** | SOFIA Briefing (DGAC) | `LF*` | HEAD pour cookie `JSESSIONID`, puis POST avec paramètres de requête PIB. Parsing JSON récursif des objets NOTAM. |

### 2.3 Routage des Adaptateurs

- `LF*` → SIA + Atlas VAC + SupAIP (cartes) + SOFIA (NOTAM)
- `EG*` → UK NATS (cartes)
- Autres → SIA uniquement (cartes)
- Dédoublonnage des résultats par URL

---

## 3. Cycle AIRAC

### 3.1 Calcul Automatique

Contrairement à la version web (variables d'environnement manuelles), le cycle AIRAC est calculé dynamiquement à partir de la date courante selon le calendrier OACI (cycles de 28 jours).

- Epoch de référence : 22 janvier 2026 (cycle 2601)
- Calcul déterministe du cycle en cours, des dates de début/fin, et du cycle suivant
- Affichage du cycle actif dans l'interface

### 3.2 Construction Dynamique des URL

Les URL des sources eAIP sont construites automatiquement à partir du cycle calculé :
- SIA : injection de `eAIP_{DD}_{MMM}_{YYYY}` et `AIRAC-{YYYY}-{MM}-{DD}`
- UK NATS : injection de `{YYYY}-{MM}-{DD}-AIRAC`

---

## 4. Structures de Données

### 4.1 Carte (`Chart`)

```
id: identifiant unique
source: adaptateur d'origine (SIA, ATLAS, SUPAIP, UK)
category: catégorie de la carte
subtitle: sous-titre dérivé du nom de fichier
filename: nom du fichier PDF
url: URL du PDF
page: numéro de page (ex: "1/3") si plusieurs pages pour un même document
tags: liste de tags déduits du nom de fichier
runways: pistes détectées
custom_title: titre personnalisé (renommage par l'utilisateur)
```

### 4.2 Catégories de Cartes

`AERODROME`, `PARKING`, `GROUND`, `SID`, `STAR`, `IAC`, `VAC`, `VLC`, `TEM`, `SUPAIP`, `OTHER`

### 4.3 NOTAM (`Notice`)

```
id: identifiant unique
icao: code OACI
source: adaptateur d'origine
identifier: série + numéro/année
type: type de NOTAM
valid_from: date de début de validité
valid_to: date de fin de validité
content: texte du NOTAM (champ itemE)
category: code catégorie (qLine.code23)
```

### 4.4 Sauvegarde de Dock (`SavedDock`)

```
id: identifiant unique
name: nom de la sauvegarde
charts: liste de cartes épinglées
notes: contenu du scratchpad (HTML)
timestamp: date de sauvegarde
```

---

## 5. Parsing et Extraction

### 5.1 SIA (eAIP France)

- Extraction de tous les liens PDF de la page aérodrome
- **Filtrage** : exclusion des fichiers `DATA`, `TEXT`, `TXT`, `VPE`, `PATC`
- **Identification de type** : correspondance des codes SIA dans le nom de fichier (`_ADC_`, `_APDC_`, `_GMC_`, `_IAC_`, `_SID_`, `_STAR_`, `_VAC_`, `_VLC_`, `_TEM_`)
- **Détection de pistes** : parsing des patterns `RWY_XX[LRC]` ; désambiguïsation (ex: `26` retiré si `26L` existe)
- **Génération de tags** :
  - IAC : `_FNA` → "App. Finale", `_INA` → "App. Initiale", `_VPT`, `_MVL`
  - Caractéristiques : `_NIGHT` → "Nuit", `_RNAV`, `_RNP`
  - Catégories ILS : regex `CAT_I`, `CAT_I_II`, `CAT_I_II_III`
  - `_LOC` → tag LOC
  - Pistes par fichier
  - Groupes sémantiques : `RWY_ALL/TOUTES` → toutes les pistes ; `RWY_WEST/EAST/NORTH/SOUTH` → filtrage par QFU
- **Extraction de sous-titre** : suppression des parties structurelles (AD, 2, ICAO, code, RWY)
- **Pagination** : regroupement par `category|subtitle`, numérotation `N/total`

### 5.2 Atlas VAC

- Requête HEAD unique sur `AD-2.{ICAO}.pdf`
- Retourne une carte de catégorie VAC si le fichier existe

### 5.3 SupAIP

- Étape 1 : GET page, extraction du `form_key` et cookies
- Étape 2 : POST avec `location=ICAO` et form_key
- Étape 3 : parsing des liens `a.lien_sup_aip`
- Étape 4 : résolution HEAD (suivi de redirections) → URL PDF finale ; filtrage des non-PDF

### 5.4 UK NATS

- Scraping HTML de la page aérodrome UK
- Extraction des liens PDF avec contexte (texte du lien + lignes de tableau adjacentes)
- **Identification de type** par mots-clés : "AERODROME CHART", "AIRCRAFT PARKING", "GROUND MOVEMENT", "INSTRUMENT APPROACH", "STANDARD DEPARTURE", "STANDARD ARRIVAL", etc.
- **Tags** : regex pour numéros de piste (`RWY`/`RUNWAY`) ; mots-clés ILS/LOC/RNP/RNAV/VOR/NDB/DME/VISUAL ; détection CAT II/III

### 5.5 SOFIA (NOTAM)

- Étape 1 : HEAD pour obtenir `JSESSIONID`
- Étape 2 : POST avec paramètres (opération, type de trafic `VI`, niveaux de vol 0-999, rayon 25NM)
- Parsing récursif du JSON pour trouver les objets NOTAM
- Dédoublonnage par `id`
- Tri par `valid_from` décroissant

---

## 6. Interface Utilisateur

### 6.0 Principes de Design

- **La fonction prime sur la forme** : l'interface s'efface au profit du document (carte eAIP, SOP). La zone de lecture est reine.
- **Contraste cognitif** : séparation visuelle stricte entre l'espace "Système" (sombre, technique, navigation) et l'espace "Papier" (clair, reposant, lecture).
- **Densité d'information maîtrisée** : polices monospace pour aligner les données, typographie à deux familles distinctes.
- **Esthétique "Rétro-Technique"** : couleurs phosphore radar (Ambre), parchemin (Beige), bordures évoquant l'instrumentation de bord.

### 6.1 Palette de Couleurs

#### Espace Technique (Sidebar, Navigation, Arborescence)

| Rôle | Couleur | Hex |
|------|---------|-----|
| Fond principal (nano sidebar) | Bleu Nuit Ardoise | `#1E3958` |
| Fond secondaire (navigateur) | Bleu Gris Foncé | `#334195` |
| Texte UI principal | Blanc Cassé | `#FBF4FC` |
| Texte UI secondaire | Gris Clair | `#94A3B8` |
| Alerte (badges, indicateurs) | Rouge UI | `#DD7796` |

#### Espace Documentaire (Zone de lecture)

| Rôle | Couleur | Hex |
|------|---------|-----|
| Fond "Papier" (lecteur PDF) | Beige Parchemin | `#FF5EDC` |
| Fond cartes/panels | Beige Clair | `#F5F50C` |
| Texte document | Noir Encre | `#1C1917` |
| Bordures / séparateurs | Gris Métallique | `#D6D3D1` (stone-300) |
| Fond barre d'outils | Pierre Claire | `#E7E5E4` (stone-200) |
| Gris clair (grille, lignes) | Gris Clair | `#994388` |
| Fond cartes sombres (métadonnées) | Gris Métallique | `#0630D1` |

#### Couleurs d'Interaction

| Rôle | Couleur | Hex |
|------|---------|-----|
| Action primaire / état actif | Ambre Radar | `#D97706` (amber-600) |
| Action hover / focus | Ambre Foncé | `#B45309` (amber-700) |
| Alerte critique (AIRAC expiré, lien brisé) | Rouge Corail | `#EF4444` (red-500) |

### 6.2 Typographie

- **Police Interface (UI)** : `Overpass`, fallback `Inter`, `Roboto` — sans-serif, graisse Medium (500) pour lisibilité sur fond sombre. Usage : menus, titres, boutons.
- **Police Technique (Données)** : `JetBrains Mono`, fallback `Space Mono` — monospace (chasse fixe). Usage : codes OACI (`LFBO`), numéros de pistes (`14L`), tags (`[MAP]`, `[SOP]`), cycle AIRAC (`2601`), métadonnées, barre d'état. Style "terminal d'aéroport".

#### Échelle Typographique

| Niveau | Taille (Overpass) | Graisse | Taille (JetBrains Mono) | Graisse |
|--------|-------------------|---------|-------------------------|---------|
| Heading 2 | 24px | Norwest shorts | 38px | — |
| Heading 3 | 20px | Heights: 15px | 25px | Weight: 13px |
| Body | 20px | UI — | 35px | UI — |
| Metadata | 24px | Recen: — | 24px | BBx — |
| Displays | 24px | Displays: 8px | 24px | Body- |

### 6.3 Iconographie et Composants

- **Icônes** : style Outline (contours uniquement), épaisseur 2px. Jeu d'icônes aviation : avion (multiples orientations/types), flèches directionnelles, symboles de navigation (VOR, NDB, waypoint), instruments, dossier de vol, punaise, loupe, engrenage, plus.
- **Boutons** — trois niveaux hiérarchiques :
  - **Primaire** : fond Ambre plein, texte sombre, padding 16px. Usage : actions principales.
  - **Secondaire** : fond transparent, bordure 1px Ambre, texte clair, padding 16px. Usage : actions alternatives.
  - **Tertiaire** : fond transparent, sans bordure, texte gris clair, padding 16px. Usage : actions discrètes.
  - **États** : Standard → Hover (Ambre Foncé) → Active (Ambre plein inversé) → Disabled (opacité réduite, texte gris).
  - **Boutons icônes** (toolbar) : 16px, icône + texte, espacement 8px. Ex: `📌 Pin`, `✏️ Annotate`, `🖨️ Print`.
  - Tous les boutons : angles arrondis 4px, léger ombrage (`shadow-sm`), aspect "bouton physique d'équipement".
- **Étiquettes (Tags / Badges)** : police monospace JetBrains Mono, majuscules, taille réduite (text-xs), fond transparent, bordure fine 1px. Ex: `[MAP]`, `[SOP]`, `[NOTE]`, `[LFBO]`, `[AIRAC 2311]`.
- **Cartes et Panels** : composants de carte pour les documents dans le navigateur, avec vignette couleur (fond catégorie), titre monospace, sous-titre. Bouton `+` pour ajouter un nouveau document/dossier.
- **Formulaires** :
  - Champs texte avec placeholder, fond sombre, bordure subtile.
  - Champ recherche avec icône loupe intégrée.
  - Dropdowns (sélection adaptateur, catégorie).
  - Contrôles segmentés : `Text | Segmented | Contrôle` pour basculer entre vues.

### 6.3.1 Grille et Espacement

- **Grille de base** : 8px (toutes les dimensions sont des multiples de 8px)
- **Paddings** : 8px (compact), 16px (standard), 24px (large)
- **Espacement boutons icônes** : icône 16px + gap 8px + texte

### 6.4 Layout — Structure en 2+1 Colonnes

L'application maximise l'espace dédié au contenu. La navigation est rétractable pour laisser toute la place au document. Barre de titre en haut : `ATC-BOOK // GESTIONNAIRE DE DOCUMENTS`.

#### Colonne 1 — Nano Sidebar (gauche, ~64px fixe)

- Icônes uniquement (outline 2px) avec label texte en dessous, pour basculer entre contextes :
  - ✈️ **AÉROPORTS** : gestion des aéroports et recherche ICAO
  - 📁 **DOSSIERS** : sauvegardes et dossiers personnalisés
  - ⚙️ **PARAMÈTRES** : préférences, langue, etc.
- Fond : Bleu Nuit Ardoise (`#1E3958`)
- Icône + label du mode actif surlignés en Ambre Radar
- Cliquer sur l'icône du mode déjà actif **replie/déplie** le Navigateur (toggle)
- En bas de la sidebar : bloc d'informations utilisateur (police monospace) :
  ```
  UTILISATEUR:
  F-HBRB | RÉSEAU:
  VATSIM | AIRAC:
  2311 (En vigueur)
  ```

#### Colonne 2 — Navigateur de Contexte (milieu, ~280–320px, **rétractable**)

- **Rétractable** : se replie entièrement (0px) par clic sur l'icône active de la nano sidebar ou par un bouton de fermeture `✕` en haut du panneau. Quand replié, l'Espace de Travail occupe toute la largeur restante.
- Animation de glissement (slide) à l'ouverture/fermeture
- Affiche l'arborescence du contexte actif selon le mode sélectionné dans la nano sidebar
- Fond : Bleu Gris Foncé (`#334195`)
- **Mode Aéroports** :
  - En-tête : `AÉROPORT ACTIF: {ICAO} / {Nom}` (ex: `LFBO / Toulouse-Blagnac`)
  - Section `SÉLECTION RAPIDE` : dossiers de sélection rapide (ex: `LFBO_GND 14L`) avec fond Ambre quand actif, contenant des documents typés :
    - `[MAP] LFBO GROUND 14L`
    - `[SOP] PISTE 14L`
    - `[NOTE] NOTES GROUND LFBO`
  - Section `CATÉGORIES DE DOCUMENTS` : arborescence dépliable par type :
    - `eAIP [CARTES]` → Depart, Arrive, Ground, SID, STAR
    - `SOP` (dépliable)
    - `NOTES PERSONNELLES` (dépliable)
- **Mode Dossiers** : arborescence des sauvegardes (dossiers sauvegardés) avec leurs documents ; onglets secondaires pour les dossiers
- **Mode Recherche** : ⏳ *reporté à une version ultérieure*
- Chaque document dans la liste affiche : icône document, badge type (`[MAP]`/`[SOP]`/`[NOTE]`), titre (monospace), bouton épingler

#### Colonne 3 — Espace de Travail (droite, flex-1, **tout l'espace restant**)

- Zone principale de lecture/travail — occupe ~100% de la largeur quand le navigateur est replié, ~75% sinon
- Fond : Beige Parchemin (`#FF5EDC`)
- **Barre d'onglets** (en haut) : les documents ouverts/épinglés sont affichés en onglets, permettant de basculer instantanément (ex: GND ↔ SID ↔ STAR). Chaque onglet :
  - Titre court (monospace) + bouton fermer `✕`
  - Onglet actif surligné en Ambre Radar
  - **Clic droit** (menu contextuel) ou **bouton détacher** : ouvre le document dans une **fenêtre OS séparée** (pop-out)
  - **Drag & drop** : réorganiser l'ordre des onglets
  - Renommage par double-clic (champ `custom_title`)
- **Barre d'outils** (sous les onglets, fond stone-200, bordure stone-300, boutons style secondaire avec icônes) :
  - `📌 Pin` — épingler le document
  - `✏️ Annotate` — annoter
  - `📁 Add to Dossier` — ajouter à un dossier/sauvegarde
  - `↗️ Pop-out` — détacher dans une nouvelle fenêtre
  - `🖨️ Print` — imprimer
- **Barre de métadonnées du document** (police monospace, fond clair) :
  ```
  TYPE: eAIP  |  REF: LFBO-SIA-03  |  VALIDE: AIRAC 2311
  ```
- **Zone de rendu PDF** : rendu natif via `pdfium-render`
- **Navigation PDF** (barre en bas de la zone de lecture) : boutons `⏮ ◀ PDF ▶ ⏭` (premier / précédent / indicateur page / suivant / dernier) pour naviguer entre les pages d'un document multi-pages
- **Éditeur de notes riche** : accessible depuis un onglet, avec barre d'outils (titres, gras/italique/souligné/barré, listes, couleurs, nettoyage)

#### Fenêtres Détachées (Pop-out)

- Depuis un onglet, l'utilisateur peut détacher un document dans une fenêtre OS native séparée via `dioxus::desktop::window().new_window()`
- Chaque fenêtre pop-out contient : barre de métadonnées + zone de rendu PDF + navigation PDF (même layout que l'espace de travail, sans la sidebar ni les onglets)
- Permet de répartir des documents sur plusieurs écrans (ex: carte GND sur l'écran principal, SID sur l'écran secondaire)
- La fenêtre détachée reste synchronisée avec l'état de l'app (épinglage, annotations)
- Fermer la fenêtre pop-out ramène le document en onglet dans la fenêtre principale

#### Barre d'État (bas, fixée, ~24–32px, pleine largeur)

- Affiche en permanence (police monospace) :
  - À gauche : `AIRAC: 2311 (En vigueur)` avec pastille colorée (●)
  - À droite : `RÉSEAU: VATSIM` avec indicateur de statut
- Couleur de la pastille AIRAC : verte si en vigueur, Rouge Corail si expiré

### 6.5 Recherche et Résultats

> ⏳ **Reporté à une version ultérieure** — le panneau de recherche/filtrage sera réintégré dans un futur cycle de développement.

### 6.6 Filtres

> ⏳ **Reporté à une version ultérieure** — les filtres par tags et texte seront réintégrés avec le panneau de recherche.

### 6.7 Documents Ouverts (Onglets)

- Les documents ouverts ou épinglés apparaissent en onglets en haut de l'Espace de Travail
- Permet de basculer instantanément entre cartes (ex: GND ↔ SID ↔ STAR) sans repasser par le navigateur
- Chaque onglet affiche : titre court (monospace), bouton fermer (désépingler)
- Onglet actif surligné en Ambre Radar
- **Détacher** : clic droit → "Pop-out" ou bouton dédié → ouvre dans une fenêtre OS séparée
- **Drag & drop** : réordonner les onglets
- **Persistance** (SQLite) : cartes épinglées, notes, sauvegardes
- **Renommage** : double-clic sur le titre d'un onglet pour renommer (champ `custom_title`)
- **Cache de préchargement** : les cartes épinglées sont téléchargées en arrière-plan pour affichage instantané

### 6.8 Sauvegardes (Dossiers)

- Accessibles via le mode Dossiers de la Nano Sidebar
- **Sauvegarder** : cartes épinglées + notes sous un nom personnalisé
- Écrasement si nom identique
- **Restaurer** : remplace les onglets et notes courants
- **Supprimer** individuellement
- Nom par défaut : `{ICAO}_{tags}` ou nom de la sauvegarde active
- **Auto-sauvegarde** : les modifications du bloc-notes se propagent à la sauvegarde active

### 6.9 NOTAM

- Affichés dans le Navigateur de Contexte (section dédiée sous les cartes ou via onglet)
- Groupés par code catégorie (`qLine.code23`), avec navigation par dropdown de catégorie
- Chaque NOTAM affiche : identifiant (monospace), dates de validité, contenu, badge catégorie

### 6.10 Actions Groupées

- **Télécharger en ZIP** : cartes sélectionnées téléchargées en parallèle, empaquetées en ZIP, sauvegardées sous `Cartes_{ICAO}_selection.zip`
- **Fusionner en PDF** : cartes sélectionnées triées (catégorie → sous-titre → nom), fusionnées page par page, sauvegardées sous `Cartes_{ICAO}_complet.pdf`
- **Épingler la sélection** : ajouter toutes les cartes sélectionnées visibles aux onglets épinglés
- Indicateurs de chargement pendant les opérations

### 6.11 Comportements UX Spécifiques

- **Changement d'AIRAC** : quand un nouvel AIRAC est détecté, le badge en barre d'état passe de l'Ambre au Rouge (ou clignote subtilement). Les documents de l'arborescence dont les liens sont brisés ou mis à jour sont signalés par une icône d'avertissement `[!]` devant leur nom.

---

## 7. Préférences Utilisateur

### 7.1 Langue

- Français (par défaut) et Anglais
- ~100+ clés de traduction couvrant : labels UI, noms de catégories, labels de tags, noms de catégories NOTAM (20+ codes), page d'aide, messages d'erreur
- Sélecteur de langue dans l'interface (drapeaux 🇫🇷/🇬🇧)

---

## 8. Sécurité

- Validation ICAO : regex stricte 4 caractères alphanumériques
- Requêtes HTTP avec retry et backoff exponentiel (base 1s, 3 tentatives, jitter aléatoire)

---

## 9. Hors Périmètre (v1)

Les éléments suivants sont décrits dans VISION.md mais exclus de cette version :

- Backend cloud / PostgreSQL / API Axum
- Server Functions Dioxus
- Pipeline ML/IA (classification, OCR, DLA)
- Crowdsourcing de taxonomie / AIXM
- Authentification OAuth 2.0 (VATSIM/IVAO)
- RBAC et gestion de permissions
- Synchronisation Push (WebSocket/SSE)
- Hub réseau de distribution documentaire
- Multi-fenêtrage (pop-out) — ~~reporté à une version ultérieure~~ **inclus dans v1**
- Diffing de cartes (SSIM) et diffing textuel — reporté à une version ultérieure
- Auto-healing des URL — reporté à une version ultérieure
