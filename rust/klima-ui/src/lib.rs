//! Ce que les deux interfaces web partagent.
//!
//! Miroir de `core/src/ui/`.
//!
//! La vitrine et l'application affichent le même signe, parlent les mêmes
//! langues, tiennent leur parcelle dans l'adresse de la même façon et
//! appellent les fournisseurs par le même chemin. Deux copies de tout cela
//! divergeraient : la première correction n'en toucherait qu'une.
//!
//! Ce qui reste chez chaque interface : ses textes, sa feuille de style, et
//! ses composants à elle.

pub mod composants;
pub mod crochets;
pub mod dates;
pub mod horloge;
pub mod i18n;
pub mod reseau;
pub mod storage;
