import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    debug: false,
    fallbackLng: 'fr',
    interpolation: {
      escapeValue: false,
    },
    resources: {
      fr: {
        translation: {
          subtitle: "Préparez vos sessions VATSIM avec de vraies cartes aéronautiques.",
          search_label: "Code ICAO (ex: LFPG)",
          search_placeholder: "LF...",
          search_button: "Rechercher",
          searching: "Recherche...",
          results_title: "Résultats pour",
          visible_charts: "{{count}} carte visible",
          visible_charts_plural: "{{count}} cartes visibles",
          selected_charts: "{{count}} sélectionnée",
          selected_charts_plural: "{{count}} sélectionnées",
          select_all: "Tout cocher",
          deselect_all: "Tout décocher",
          merge_button: "PDF Unique",
          merging: "Fusion...",
          zip_button: "ZIP",
          zipping: "Zip...",
          filter_placeholder: "Filtrer les cartes (ex: ILS 26, Parking...)",
          group_stations: "Poste",
          group_runways: "Pistes",
          group_approaches: "Procédures",
          group_phases: "Phases",
          group_others: "Autres",
          no_results: "Aucune carte trouvée pour cet aérodrome.",
          footer_credits: "Réalisé par",
          error_zip: "Erreur lors de la création du fichier ZIP.",
          error_merge: "Erreur lors de la fusion des PDF.",
          error_fetch: "Une erreur est survenue",
          
          // Categories
          cat_parking: "Stationnement",
          cat_aerodrome: "Carte d'aérodrome",
          cat_ground_movements: "Mouvements à la surface",
          cat_instrument_approach: "Approche aux instruments",
          cat_sid: "Départs (SID)",
          cat_star: "Arrivées (STAR)",

          // Specific Tags
          tag_app_final: "App. Finale",
          tag_app_initial: "App. Initiale",
          tag_night: "Nuit",
          supported_airports_hint: "Aéroports en France (SIA) et au Royaume-Uni (NATS) disponibles.",

          // Dock
          dock_title: "Porte-documents",
          dock_empty: "Aucune carte épinglée",
          dock_notices_title: "NOTAM / SupAIP",
          dock_notices_empty: "Aucun NOTAM pour cet aérodrome.",
          pin_tooltip: "Épingler au porte-documents",
          unpin_tooltip: "Retirer du porte-documents",
          pin_selection_button: "Épingler",
          clear_dock: "Tout retirer",
          close_viewer: "Fermer",
          dock_notices_jump_title: "Navigation rapide",
          dock_save_tooltip: "Sauvegarder la configuration",
          dock_load_tooltip: "Charger une configuration",
          dock_saves_title: "Sauvegardes",
          dock_saves_empty: "Aucune configuration sauvegardée",
          dock_save_placeholder: "Nom de la sauvegarde...",
          dock_scratchpad_title: "Mémo",
          dock_scratchpad_placeholder: "Écrivez vos notes ici (fréquences, clairances, mémos)...",
          saved: "Enregistré",

          // Contextual Help
          help_title: "Aide & Guide",
          help_page_title: "Comment utiliser ATC Book",
          help_intro: "ATC Book est conçu pour simplifier la vie des contrôleurs virtuels sur VATSIM en regroupant les cartes et informations essentielles.",
          help_getting_started: "Démarrage Rapide",
          help_getting_started_text: "Entrez le code ICAO d'un aéroport (ex: LFPG, EGLL) pour afficher les cartes à jour. Utilisez les filtres rapides (Pistes, Départs, etc.) pour affiner l'affichage.",
          help_dock: "Porte-documents (Dock)",
          help_dock_text: "Épinglez vos cartes favorites (icône épingle) pour les garder actives. Le dock persiste à la réouverture du site. Vous pouvez le positionner en bas, à gauche ou à droite.",
          help_scratchpad: "Mémo Intelligent",
          help_scratchpad_text: "Un bloc-notes riche est intégré au dock. Notez-y les clairances ou fréquences. Il est sauvegardé automatiquement avec votre configuration.",
          help_notam: "NOTAMs & SupAIP",
          help_notam_text: "Consultez les NOTAMs triés par catégorie (Pistes, Radio, etc.) directement depuis le dock via l'icône de cloche.",
          help_tips: "Astuces Pro",
          help_tips_text: "Double-cliquez sur le titre d'une carte dans le dock pour la renommer. Créez des sauvegardes de dock pour vos différentes positions de contrôle (ex: 'LFPG_TWR', 'LFPG_APP').",
          footer_help: "Aide",

          // NOTAM Categories
          notice_cat_A: "Généralités (Services)",
          notice_cat_F: "Installations aéroportuaires",
          notice_cat_FA: "Aérodrome",
          notice_cat_FF: "Sauvetage & Incendie",
          notice_cat_L: "Pistes",
          notice_cat_LC: "Piste fermée",
          notice_cat_LA: "Piste",
          notice_cat_M: "Aires de manœuvre",
          notice_cat_MR: "Piste (Physique)",
          notice_cat_MX: "Taxiway",
          notice_cat_P: "Aires de trafic",
          notice_cat_C: "Communications",
          notice_cat_N: "Aides Radio",
          notice_cat_NB: "NDB",
          notice_cat_I: "ILS",
          notice_cat_O: "Obstacles",
          notice_cat_OB: "Obstacle",
          notice_cat_R: "Restrictions Espace",
          notice_cat_RT: "Zone Réglementée Temp.",
          notice_cat_W: "Avertissements",
          notice_cat_WU: "Drones / UAV",
          notice_cat_WP: "Parachutisme",

          // Common words
          word_runway: "Piste"
        }
      },
      en: {
        translation: {
          subtitle: "Prepare your VATSIM sessions with real aeronautical charts.",
          search_label: "ICAO Code (e.g. LFPG)",
          search_placeholder: "LF...",
          search_button: "Search",
          searching: "Searching...",
          results_title: "Results for",
          visible_charts: "{{count}} chart visible",
          visible_charts_plural: "{{count}} charts visible",
          selected_charts: "{{count}} selected",
          selected_charts_plural: "{{count}} selected",
          select_all: "Check all",
          deselect_all: "Uncheck all",
          merge_button: "Single PDF",
          merging: "Merging...",
          zip_button: "ZIP",
          zipping: "Zipping...",
          filter_placeholder: "Filter charts (e.g. ILS 26, Parking...)",
          group_stations: "Station",
          group_runways: "Runways",
          group_approaches: "Procedures",
          group_phases: "Phases",
          group_others: "Others",
          no_results: "No charts found for this aerodrome.",
          footer_credits: "Created by",
          error_zip: "Error creating ZIP file.",
          error_merge: "Error merging PDFs.",
          error_fetch: "An error occurred",

          // Categories
          cat_parking: "Parking",
          cat_aerodrome: "Aerodrome Chart",
          cat_ground_movements: "Ground Movements",
          cat_instrument_approach: "Instrument Approach",
          cat_sid: "Departures (SID)",
          cat_star: "Arrivals (STAR)",

          // Specific Tags
          tag_app_final: "Final App.",
          tag_app_initial: "Initial App.",
          tag_night: "Night",
          supported_airports_hint: "Airports in France (SIA) and United-Kingdom (NATS) available.",

          // Dock
          dock_title: "Active Deck",
          dock_empty: "No pinned charts",
          dock_notices_title: "NOTAM / SupAIP",
          dock_notices_empty: "No NOTAM for this aerodrome.",
          pin_tooltip: "Pin to Active Deck",
          unpin_tooltip: "Unpin from Active Deck",
          pin_selection_button: "Pin Selection",
          clear_dock: "Clear all",
          close_viewer: "Close",
          dock_notices_jump_title: "Quick Jump",
          dock_save_tooltip: "Save configuration",
          dock_load_tooltip: "Load configuration",
          dock_saves_title: "Saved Decks",
          dock_saves_empty: "No saved decks",
          dock_save_placeholder: "Save name...",
          dock_scratchpad_title: "Scratchpad",
          dock_scratchpad_placeholder: "Write your notes here (frequencies, clearances, memos)...",
          saved: "Saved",

          // Contextual Help
          help_title: "Help & Guide",
          help_page_title: "How to use ATC Book",
          help_intro: "ATC Book is designed to make life easier for virtual controllers on VATSIM by gathering charts and essential info.",
          help_getting_started: "Quick Start",
          help_getting_started_text: "Enter an airport ICAO code (e.g. LFPG, EGLL) to view up-to-date charts. Use quick filters (Runways, Departures, etc.) to refine the list.",
          help_dock: "Active Deck (Dock)",
          help_dock_text: "Pin your favorite charts (pin icon) to keep them active. The deck persists between sessions. You can position it at the bottom, left, or right.",
          help_scratchpad: "Smart Scratchpad",
          help_scratchpad_text: "A rich text scratchpad is built into the dock. Use it for clearances or frequencies. It is auto-saved with your layout.",
          help_notam: "NOTAMs & SupAIP",
          help_notam_text: "View categorized NOTAMs (Runways, Radio, etc.) directly from the dock via the bell icon.",
          help_tips: "Pro Tips",
          help_tips_text: "Double-click a chart title in the deck to rename it. Create deck saves for your different controller positions (e.g., 'LFPG_TWR', 'LFPG_APP').",
          footer_help: "Help",

          // NOTAM Categories
          notice_cat_A: "General (Services)",
          notice_cat_F: "Facilities",
          notice_cat_FA: "Aerodrome",
          notice_cat_FF: "Rescue & Fire Fighting",
          notice_cat_L: "Runways",
          notice_cat_LC: "Runway Closed",
          notice_cat_LA: "Runway",
          notice_cat_M: "Manoeuvring Area",
          notice_cat_MR: "Runway (Physical)",
          notice_cat_MX: "Taxiway",
          notice_cat_P: "Aprons",
          notice_cat_C: "Communications",
          notice_cat_N: "Radio Aids",
          notice_cat_NB: "NDB",
          notice_cat_I: "ILS",
          notice_cat_O: "Obstacles",
          notice_cat_OB: "Obstacle",
          notice_cat_R: "Airspace Restrictions",
          notice_cat_RT: "Temp. Restricted Area",
          notice_cat_W: "Warnings",
          notice_cat_WU: "Drones / UAV",
          notice_cat_WP: "Parachuting",

          // Common words
          word_runway: "Runway"
        }
      }
    }
  });

export default i18n;
