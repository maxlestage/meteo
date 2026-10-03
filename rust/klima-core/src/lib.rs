//! Cœur partagé de Klima, en Rust.
//!
//! Troisième écriture des mêmes règles, après TypeScript (`core/src/`) et
//! Swift (`ios/Kliima/Models/`). Trois sources pour une seule vérité, c'est
//! deux de trop — c'est la raison d'être de ce portage : le TypeScript
//! disparaîtra quand le relais et les interfaces seront passés en Rust.
//!
//! En attendant, les cas de test sont recopiés à l'identique d'un côté à
//! l'autre. C'est ce qui garantit qu'une position tombe dans la même cellule
//! quelle que soit la langue qui la calcule.

pub mod grid;
pub mod position;
