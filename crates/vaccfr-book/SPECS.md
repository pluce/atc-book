# Archive: documentation déplacée

Cette copie historique est conservée pour compatibilite avec les anciens chemins.

La specification active se trouve ici:

- `crates/atc-book/SPECS.md`

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
