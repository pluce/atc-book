# Fonctionnalités de l'application (Capabilities)

Ce document liste les fonctionnalités clés de l'application ATC BOOK, servant de référence pour les tests End-to-End (E2E).

## 1. Recherche et Navigation
- **Recherche de terrain** : L'utilisateur peut entrer un code OACI (ex: LFPG) et lancer une recherche.
- **Affichage des résultats** : La liste des cartes s'affiche, groupée par catégories (Approche, Atterrissage, etc.).
- **Feedback visuel** : Indicateurs de chargement lors de la récupération des données.
- **Persistance URL** : Les paramètres de recherche (icao, filtres) sont reflétés dans l'URL.

## 2. Filtrage et Tri
- **Filtre textuel** : Barre de recherche pour filtrer les cartes par nom ou catégorie.
- **Filtres par Tags** : Boutons pour filtrer par tags spécifiques (ex: "App. Finale", "Nuit").
- **Tags groupés** : Les tags sont organisés logiquement (Pistes, Approches, Phases, Autres).

## 3. Gestion de la Sélection
- **Sélection multiple** : Cases à cocher pour sélectionner des cartes individuelles ou des groupes entiers.
- **Sélection rapide** : Boutons "Tout sélectionner" / "Tout désélectionner".
- **Compteurs** : Affichage du nombre de cartes visibles et sélectionnées.

## 4. Actions sur les cartes
- **Épingler (Pin)** : Ajouter les cartes sélectionnées au Dock pour un accès rapide.
- **Fusionner (Merge)** : Combiner les cartes sélectionnées en un seul fichier PDF.
- **Télécharger (Zip)** : Télécharger les cartes sélectionnées dans une archive ZIP.
- **Indicateurs d'état** : Spinners lors des opérations de téléchargement ou fusion.

## 5. Dock (Barre d'outils)
- **Persistance** : Le contenu du Dock est sauvegardé (localStorage).
- **Positionnement** : Le Dock peut être placé en bas, à gauche ou à droite.
- **Gestion** :
  - Supprimer une carte spécifique.
  - Vider tout le Dock.
  - Ouvrir une carte depuis le Dock.

## 6. Visualisation (Viewer)
- **Modale** : Affichage des cartes PDF dans une modale superposée.
- **Navigation** : Bouton de fermeture, touche Echap pour fermer.

## 7. Préférences Utilisateur
- **Thème** : Bascule entre Mode Clair (Light) et Mode Sombre (Dark).
- **Langue** : Bascule entre Français (FR) et Anglais (EN).
