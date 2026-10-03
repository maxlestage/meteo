//! Les formats de fil de Klima : construire les adresses, lire les réponses.
//!
//! Troisième crate de l'espace de travail, et elle existe pour une seule
//! raison : `klima-core` n'a aucune dépendance et doit le rester. Lire du JSON
//! en demande une. Plutôt que d'ouvrir le cœur à `serde`, les règles restent
//! pures d'un côté, et le décodage du fil vit ici. Le relais et l'interface
//! en dépendent tous les deux : une réponse d'Open-Meteo se lit de la même
//! façon qu'on soit serveur ou navigateur.
//!
//! En TypeScript, le client HTTP et le décodage habitent le même fichier
//! (`core/src/openMeteo.ts`). Ici le décodage est séparé de l'appel : il est
//! pur, donc testable sur une réponse enregistrée, sans réseau ni horloge.

pub mod open_meteo;
pub mod parcelle_url;
pub mod readings;
pub mod today;
