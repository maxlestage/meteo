//! Cœur partagé de Klima, en Rust.
//!
//! Il y avait trois écritures des mêmes règles — TypeScript, Swift, Rust —
//! et trois sources pour une seule vérité, c'est deux de trop. Le TypeScript
//! est parti ; restent celle-ci et son miroir Swift (`ios/Kliima/Models/`).
//!
//! Les cas de test sont recopiés à l'identique d'un côté à l'autre. C'est ce
//! qui garantit qu'une position tombe dans la même cellule quelle que soit la
//! langue qui la calcule.

pub mod alerts;
pub mod calendar;
pub mod cumuls;
pub mod endpoints;
pub mod format;
pub mod grid;
pub mod horizon;
pub mod i18n;
pub mod agro;
pub mod consensus;
pub mod messages;
pub mod plan;
pub mod position;
pub mod providers;
pub mod register;
pub mod weather;
