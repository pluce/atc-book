Voici le cahier des spécifications techniques et fonctionnelles détaillées, conçu pour servir de document de référence à une équipe de développement logiciel.

# Spécifications Techniques Détaillées - Electronic Controller Bag (ECB)

## 1. Architecture Globale et Choix Technologiques

L'application repose sur un écosystème entièrement développé en Rust, garantissant une sécurité mémoire absolue et des performances optimales adaptées aux environnements de simulation nécessitant de faibles latences.

### 1.1. Client Multiplateforme (Desktop, Tablette, Web)

* **Framework UI :** Dioxus (version 0.6+). Le choix de Dioxus permet une base de code unifiée. Pour la version Desktop, l'application s'appuie sur `dioxus-webview` qui utilise les composants natifs du système d'exploitation hôte (EdgeHTML/WebKit), éliminant ainsi la surcharge de mémoire propre à Electron.
* **Gestion d'état :** Utilisation du système de signaux (Signals) natif de Dioxus pour une réactivité granulaire de l'interface sans re-rendus inutiles.


* **Stockage Local (Local-First) :** Base de données SQLite embarquée via la crate `rusqlite`. La connexion est maintenue persistante pour l'application Desktop via la macro `thread_local!` pour garantir un accès non-bloquant depuis n'importe quel composant. Pour la version Web, le stockage s'opère via l'API `IndexedDB` en utilisant les bindings `web-sys`.



### 1.2. Infrastructure Cloud (Backend)

* **Serveur API :** Axum (framework web Rust) interfacé avec les "Server Functions" de Dioxus pour une communication RPC typée de bout en bout.


* **Base de Données Centrale :** PostgreSQL pour stocker les métadonnées taxonomiques crowdsourcées, les profils utilisateurs et les arborescences documentaires des divisions (VATSIM/IVAO).

---

## 2. Spécifications du Client Lourd (Interface et Métier)

### 2.1. Manipulation et Rendu Documentaire (PDF)

* **Moteur de Rendu Visuel :** Intégration de `pdfium-render` (basé sur Google PDFium) pour un affichage natif, fluide et de haute fidélité sur Desktop.


* **Extraction et Indexation :** Utilisation de la crate `pdf_oxide` (jusqu'à 47 fois plus rapide que les alternatives classiques) pour l'extraction textuelle brute en arrière-plan, permettant la constitution d'un index de recherche plein-texte. La crate `pdf-text-extract`, compatible WebAssembly, assurera la parité des fonctionnalités sur la version navigateur.


* **Multi-fenêtrage (Pop-out) :** L'interface doit permettre de détacher des cartes ou des notes. Ceci est implémenté via la fonction `dioxus::desktop::window().new_window()` pour instancier un nouveau VirtualDOM dans une fenêtre OS native séparée, permettant la répartition sur plusieurs écrans.

### 2.2. Éditeur de Notes Enrichies (Rich Text)

* L'éditeur de consignes et de mémos (WYSIWYG) s'appuiera sur la crate `tiptap-rs`, qui fournit des liaisons WebAssembly (WASM) directes vers le framework headless Tiptap (basé sur ProseMirror).


* L'éditeur doit supporter les tableaux de coordination, la mise en surbrillance, et la création d'hyperliens internes pointant vers des identifiants (ID) de cartes spécifiques dans la base de données SQLite.

### 2.3. Moteur de Détection des Modifications (Diffing)

* **Analyse Matricielle (Images/Cartes) :** Implémentation de la crate Rust `image-compare`. L'algorithme principal utilisé sera le SSIM (Structural Similarity Index Measure) pour comparer les pixels structurels entre deux versions de cartes (ex: déplacement d'un axe de piste). Le système clamp l'alpha minimal à 0.1 pour générer un calque visuel isolant les seules différences géométriques entre deux cycles.


* **Analyse Textuelle (AIP/SOP) :** Extraction du texte via `pdf-text-extract` , suivi d'une analyse différentielle standard pour surligner les changements de procédures ou de fréquences.



---

## 3. Gestion Algorithmique du Cycle AIRAC

Le cœur de la résilience du système face au vieillissement des liens web repose sur son moteur temporel automatisé.

### 3.1. Calcul Déterministe des Dates

* L'application utilise une crate de calcul temporel (par exemple, `airac`) pour déduire le cycle en cours selon le standard de l'OACI.


* **Vecteurs de test 2026/2027 :** L'algorithme doit valider les entrées en vigueur sur des cycles de 28 jours. Par exemple :


* Cycle 2601 : 22 Janvier 2026.


* Cycle 2602 : 19 Février 2026.


* Cycle 2701 : 21 Janvier 2027.





### 3.2. Auto-Healing (Cicatrisation) des URL

* **Logique de construction :** L'outil doit appliquer des règles d'expression régulière (RegEx) pour substituer dynamiquement la variable de date dans les signets de l'utilisateur.
* **Exemple d'implémentation SIA France :** L'URL `https://www.sia.aviation-civile.gouv.fr/media/dvd/eAIP_19_FEB_2026/FRANCE/AIRAC-2026-02-19/html/eAIP/FR-AD-2.LFMN-fr-FR.html` sera analysée. À la bascule vers le cycle 2603, le moteur asynchrone (via `reqwest` ) reconstruira l'URL en injectant `19_MAR_2026` et `2026-03-19`, puis effectuera une requête `HTTP HEAD` pour valider la disponibilité du nouveau fichier PDF.



---

## 4. Pipeline Machine Learning et Taxonomie (Backend Cloud)

Le classement par type de document (Sol, Approche, Départ) sur de grands volumes de cartes PDF importées s'effectue automatiquement via l'intelligence artificielle.

### 4.1. Analyse et Extraction de Structure (DLA)

* **Document Layout Analysis :** Utilisation d'un modèle de type Transformateur/CNN pour isoler les "bounding boxes" (zones de délimitation) correspondant aux en-têtes et aux blocs de données des cartes.


* **Pipeline OCR Ciblée :** Une fois la zone de titre isolée, une reconnaissance optique de caractères identifie les mots-clés ("STANDARD ARRIVAL", "RNAV", "RWY") pour peupler les métadonnées de la carte.

### 4.2. Classification de Topologie Visuelle

* Pour les cartes non standardisées, un réseau de neurones convolutif (CNN) tel que ResNet ou VGG-16, pré-entraîné sur un jeu de données de cartes aéronautiques (SID, STAR, VAC, GND), analysera l'image. Par exemple, la détection de longues lignes convergentes classera le document en procédure d'arrivée (STAR).



### 4.3. Standardisation et Crowdsourcing

* La taxonomie résultante est standardisée selon le modèle de métadonnées AIXM 5.1 (norme ISO 19115) assurant l'interopérabilité.
* En cas de classification erronée de l'IA, la modification manuelle d'un contrôleur dans son client lourd envoie un payload JSON au serveur Cloud. Après confirmation, cette correction est poussée à tous les utilisateurs téléchargeant la même ressource (identifiée par son hash).

---

## 5. Hub Réseau et Distribution Documentaire ATC

L'outil se connectera directement aux bases de données des réseaux de simulation pour une gestion documentaire institutionnelle de niveau "Division" ou "vACC".

### 5.1. Authentification et RBAC (Role-Based Access Control)

* **OAuth 2.0 :** Intégration avec la "VATSIM Core API v2" pour l'authentification réseau. L'API retournera le statut du contrôleur (S1, S2, C1, Instructeur) et ses affectations de subdivision (ex: San Juan CERAP, FIR Bordeaux).


* **Gestion des permissions :** Le backend Cloud restreindra la visibilité des Manuels d'Exploitation (SOP) internes ou des grilles d'examens aux seuls membres disposant d'un rôle "Staff" ou "Mentor", empêchant les fuites documentaires.



### 5.2. Synchronisation "Push" des Lettres d'Accord (LOA)

* Les administrateurs (ex: Air Traffic Manager) uploadent les SOP et LOA de leur division sur le portail Web de l'ECB.


* Le client Desktop des membres de cette division interroge l'API via des WebSockets ou Server-Sent Events (SSE) (natifs dans Dioxus via Axum ) pour télécharger automatiquement en arrière-plan (au format SQLite binaire) toute nouvelle révision d'un manuel opérationnel avant la prise de service du contrôleur sur le réseau.