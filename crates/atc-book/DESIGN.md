

## Document de Spécifications Design : **ATC-BOOK**

### 1. Vision et Principes Fondamentaux

L'interface d'ATC-BOOK doit refléter le sérieux et la précision de l'aviation. Elle est conçue autour de quatre principes :

* **La fonction prime sur la forme :** L'interface doit s'effacer au profit du document (carte eAIP, SOP). La zone de lecture est reine.
* **Contraste cognitif :** Séparation visuelle stricte entre l'espace "Système" (sombre, technique, navigation) et l'espace "Papier" (clair, reposant, lecture).
* **Densité d'information maîtrisée :** Afficher beaucoup d'éléments (dossiers, tags, réseaux) sans surcharger l'œil, en utilisant des polices à chasse fixe (monospace) pour aligner naturellement les données.
* **Esthétique "Rétro-Technique" :** Utilisation de couleurs rappelant le phosphore des anciens radars (Ambre) et les manuels de vol imprimés (Beige/Parchemin), couplés à des bordures subtiles évoquant l'instrumentation de bord.

---

### 2. Palette de Couleurs (Variables système)

L'application est divisée en deux zones colorimétriques distinctes. Les classes entre parenthèses sont des équivalents standards (type Tailwind) pour faciliter votre intégration.

#### Espace Technique (Navigation, Menus, Arborescence)

* **Fond principal (Sidebar icônes) :** Bleu Nuit Ardoise `##1E293B` (*slate-800*)
* **Fond secondaire (Arborescence dossiers) :** Bleu Gris Foncé `#334155` (*slate-700*)
* **Texte UI principal :** Blanc Cassé `#F8FAFC` (*slate-50*)
* **Texte UI secondaire (Détails, données) :** Gris Clair `#94A3B8` (*slate-400*)

#### Espace Documentaire (Zone de lecture)

* **Fond "Papier" (Lecteur eAIP/SOP) :** Beige Parchemin `#F5F5DC` ou `#FAF8F5` (*stone-50*)
* **Texte Document (si éditeur de texte riche) :** Noir Encre `#1C1917` (*stone-900*)
* **Bordures et Séparateurs (Barre d'outils) :** Gris Métallique `#D6D3D1` (*stone-300*) et fond de barre `#E7E5E4` (*stone-200*)

#### Couleurs d'Interaction et d'Accentuation

* **Action Primaire / État Actif :** Ambre Radar `#D97706` (*amber-600*). Utilisé pour le dossier sélectionné, l'icône active, ou un statut AIRAC valide.
* **Action Hover / Focus (Le "Marron") :** Ambre Foncé `#B45309` (*amber-700*). Utilisé au survol des boutons ou pour les alertes modérées.
* **Alerte Critique :** Rouge Corail `#EF4444` (*red-500*). Exclusivement réservé pour indiquer qu'un document est expiré (AIRAC précédent) ou qu'un lien est brisé.

---

### 3. Typographie

Pour marquer le côté professionnel et technique, le système utilise deux familles de polices distinctes :

* **Police de l'Interface (UI) : `Overpass`, `Inter` ou `Roboto**`
* **Usage :** Noms des menus, titres des fenêtres, boutons d'action.
* **Style :** Sans-serif. Privilégier des graisses moyennes (Medium 500) pour une bonne lisibilité sur fond sombre.


* **Police Technique (Données) : `JetBrains Mono` ou `Space Mono**`
* **Usage :** Codes OACI (LFBO), numéros de pistes (14L), tags catégoriels (`[MAP]`, `[SOP]`), cycle AIRAC (2311), et métadonnées.
* **Style :** Monospace (chasse fixe). Cela donne un look "terminal d'aéroport" très immersif et aligne parfaitement les listes de documents.



---

### 4. Iconographie et Composants (UI)

* **Icônes :** Style "Outline" (contours uniquement), épaisseur de 2px. Elles doivent être universelles (avion, dossier de vol, punaise, loupe, engrenage).
* **Boutons d'action (Zone de lecture) :**
* Pas de style "flat" moderne. Ils doivent avoir une bordure d'un pixel plus sombre que leur fond et un très léger ombrage (shadow-sm) pour paraître "cliquables", comme des boutons physiques d'équipement.
* Angles légèrement arrondis (`border-radius: 4px`).


* **Étiquettes (Tags / Badges) :**
* Utilisés pour qualifier les documents : `[SOP]`, `[eAIP]`, `[NOTE]`.
* Rendu : Police Monospace en majuscules, taille de police réduite (text-xs), fond transparent, bordure fine.



---

### 5. Grille et Mise en Page (Layout)

L'application suit une structure stricte en 3 colonnes ajustables (Flexbox), conçue pour optimiser l'espace sur écran large :

1. **La "Nano" Sidebar (Gauche - env. 64px) :** Uniquement des icônes pour passer d'un contexte à l'autre (Aéroports, Dossiers, Recherche, Paramètres).
2. **Le Navigateur de Contexte (Milieu - env. 280px à 320px) :** Affiche l'arborescence du contexte actif (ex: le dossier "LFBO_GND 14L" et la liste de ses documents).
3. **L'Espace de Travail (Droite - Espace restant, `flex-1`) :** La zone principale contenant le lecteur de PDF/Images ou l'éditeur de notes riche. Elle possède sa propre barre d'outils en haut (Épingler, Annoter, Imprimer).
4. **La Barre d'État (Bas, fixée) :** Très fine (env. 24px à 32px), elle court sur toute la largeur (ou juste sous le navigateur) pour afficher en permanence : `Utilisateur: F-HBRB | Réseau: VATSIM | AIRAC: 2311 (En vigueur)`.

---

### 6. Comportements Spécifiques (UX ATC)

* **Changement d'AIRAC :** Lorsqu'un nouvel AIRAC est détecté, l'indicateur dans la barre d'état passe de l'Ambre au Rouge (ou clignote subtilement). Les documents de l'arborescence qui ont des liens brisés ou qui ont été mis à jour sont signalés par une icône d'avertissement `[!]` devant leur nom.
* **Épinglage Rapide ("Pin") :** Un document épinglé reste accessible dans une barre d'onglets au-dessus de l'Espace de Travail, permettant au contrôleur de basculer instantanément entre la carte de roulage (GND) et la carte de départ (SID) sans repasser par l'arborescence.

