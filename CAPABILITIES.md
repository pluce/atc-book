# Fonctionnalités — ATC-BOOK Desktop

Application desktop native (Rust + Dioxus) pour la consultation de cartes aéronautiques et NOTAMs.

---

## 1. Recherche et Navigation
- **Recherche OACI** : Saisie d'un code OACI (4 lettres) avec recherche via Entrée ou bouton.
- **Multi-sources** : Interrogation parallèle de 5 adaptateurs selon le préfixe OACI :
  - `LF*` → SIA + Atlas VAC + SupAIP (cartes) + SOFIA (NOTAMs)
  - `EG*` → UK NATS (cartes)
  - Autres → SIA uniquement
- **Résultats groupés** : Cartes organisées par catégorie (Aerodrome, SID, STAR, IAC, VAC, etc.).
- **Cycle AIRAC** : Calcul automatique du cycle en cours (époque 22 jan 2026, cycles de 28 jours). Affiché dans la barre latérale.
- **Cache de recherche** : Les résultats sont mis en cache localement (SQLite) pour accélérer les prochaines consultations.

## 2. Visualisation PDF
- **Rendu natif** : Rendu haute-fidélité via pdfium-render (2400×3400 px par page).
- **Zoom** : Molette ou boutons -/+, avec indicateur de pourcentage et bouton de réinitialisation.
- **Fit-to-width** : Bouton pour ajuster la largeur du document à l'espace disponible.
- **Drag-to-pan** : Déplacement du document par glisser-déposer.
- **Onglets** : Plusieurs cartes ouvertes simultanément avec une barre d'onglets. Fermeture individuelle.
- **Cache PDF** : Double cache — fichiers sur disque (persistant) + 5 derniers rendus en mémoire (LRU).

## 3. Dossiers de Travail (Workspaces)
- **Création / Suppression** : Créer un dossier nommé, supprimer avec confirmation.
- **Renommage** : Éditeur inline (bouton ✎).
- **Ajout de cartes** : Via le menu « Dossier » de la barre d'outils document. Dédoublonnage automatique.
- **Retrait de cartes** : Bouton ✕ sur chaque carte (apparaît au survol). Nettoyage automatique des aéroports sans cartes.
- **Cartes groupées** : Les cartes sont affichées par aéroport dans l'arborescence du dossier (✈ LFPG (3)).
- **Chargement / Déchargement** : Bouton ▶ pour restaurer les onglets du dossier, ⏹ pour décharger. Indicateur actif (●).
- **Ouverture groupée** : Bouton ▶▶ pour ouvrir toutes les cartes du dossier.
- **Sauvegarde automatique** : L'état des onglets (ouverture, position active) est sauvegardé en temps réel.
- **Notes de briefing** : Bloc-notes texte intégré par dossier (section ▸ 📝 NOTES, collapsible). Sauvegarde sur perte de focus.
- **Persistance SQLite** : Tous les dossiers, cartes associées, onglets et notes sont stockés localement.

## 4. Interface
- **Barre latérale** : Navigation par icônes (Aéroports / Dossiers / Réglages) avec panneau dépliant.
- **Panneau navigateur** : Arborescence des résultats ou des dossiers selon le mode sélectionné.
- **Barre d'outils document** : Menu d'ajout au dossier.
- **Design « Rétro-Technique »** : Palette ambre/ardoise, polices monospace (JetBrains Mono), esthétique instrumentation.

## 5. NOTAMs
- **Source SOFIA** : Récupération des NOTAMs pour les aérodromes français via le PIB SOFIA (DGAC).
- **Affichage** : Liste des NOTAMs avec identifiant, validité et contenu.
