//! L'heure, et ce qu'elle devient à la parcelle.
//!
//! Les séries de Klima sont en heure locale du champ. Pour les comparer à
//! « maintenant », il faut la même horloge : l'instant du navigateur, décalé
//! du fuseau de la parcelle. Avant qu'une prévision soit arrivée, on ne connaît
//! pas ce décalage — on prend celui du lecteur, qui est le bon dans l'immense
//! majorité des cas et n'est de toute façon utilisé que pour choisir l'heure
//! courante d'une série.

use js_sys::Date;

/// Millisecondes depuis l'époque, telles que le navigateur les donne.
pub fn maintenant_utc() -> i64 {
    Date::now() as i64
}

/// Maintenant, dans le temps du lecteur — le décalage de sa machine inclus.
pub fn maintenant_local() -> i64 {
    let date = Date::new_0();
    // `getTimezoneOffset` est en minutes et de signe inversé.
    maintenant_utc() - (date.get_timezone_offset() as i64) * 60_000
}

/// Maintenant, dans le temps de la parcelle.
pub fn maintenant_a_la_parcelle(utc_offset_seconds: i64) -> i64 {
    maintenant_utc() + utc_offset_seconds * 1000
}
