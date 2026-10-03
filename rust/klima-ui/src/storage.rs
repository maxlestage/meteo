//! Le stockage local, et la permission de ne pas y arriver.
//!
//! En navigation privée, avec les cookies bloqués ou le disque plein, lire et
//! écrire échouent. Aucun des usages de Klima ne le mérite : la langue tient
//! le temps de la visite, la dernière parcelle se retrouve dans l'adresse.
//! Toutes les erreurs sont donc avalées — mais une seule fois, et ici.

pub fn get(key: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
}

pub fn set(key: &str, value: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.set_item(key, value);
    }
}
