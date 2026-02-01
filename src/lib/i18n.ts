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
          subtitle: "Récupérez instantanément les cartes du SIA pour vos sessions VATSIM.",
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

          // Common words
          word_runway: "Piste"
        }
      },
      en: {
        translation: {
          subtitle: "Instantly retrieve SIA charts for your VATSIM sessions.",
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

          // Common words
          word_runway: "Runway"
        }
      }
    }
  });

export default i18n;
