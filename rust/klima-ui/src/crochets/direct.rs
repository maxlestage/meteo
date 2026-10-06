//! Le direct : la météo que le relais pousse par WebSocket.
//!
//! Une connexion par page, ouverte dès qu'il y a un relais
//! (`Endpoints::direct_url`). Elle s'abonne à la ville affichée, puis reçoit
//! huit sujets — `base`, `ensemble`, `quarts`, `met`, `station`, `air`, `ciel`, `radar` —, chacun
//! dans la forme exacte que le fournisseur renvoie : les décodeurs de
//! `klima-api` les lisent comme s'ils venaient d'une requête. Le relais ne
//! pousse que ce qui a changé ; ce crochet garde le dernier corps de chaque
//! sujet, pour la ville en cours.
//!
//! Le direct s'ajoute aux requêtes, il ne les remplace pas : la première
//! prévision arrive toujours par HTTP, et si la connexion tombe, tout continue
//! comme avant pendant qu'elle se rouvre — deux secondes, puis quatre,
//! huit… jusqu'à une minute. Sans relais, rien ne s'ouvre.

use std::cell::RefCell;
use std::rc::Rc;

use klima_core::endpoints::Endpoints;
use klima_core::position::Parcelle;
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{MessageEvent, WebSocket};
use yew::prelude::*;

/// Le dernier corps reçu de chaque sujet, pour une ville.
#[derive(Clone, Default, PartialEq)]
pub struct Corps {
    /// Le point de l'abonnement : les corps qui ne le portent pas sont écartés.
    pub point: Option<(f64, f64)>,
    pub base: Option<Rc<String>>,
    pub ensemble: Option<Rc<String>>,
    pub quarts: Option<Rc<String>>,
    pub met: Option<Rc<String>>,
    pub station: Option<Rc<String>>,
    pub air: Option<Rc<String>>,
    /// Les bulletins des aéroports proches.
    pub ciel: Option<Rc<String>>,
    /// Ce que le radar voit, et prévoit pour deux heures.
    pub radar: Option<Rc<String>>,
    /// Vrai quand la connexion est ouverte.
    pub connecte: bool,
}

pub enum Evenement {
    Ville((f64, f64)),
    Recu { sujet: String, point: (f64, f64), corps: String },
    Connexion(bool),
}

impl Reducible for Corps {
    type Action = Evenement;

    fn reduce(self: Rc<Self>, action: Evenement) -> Rc<Self> {
        let mut suivant = (*self).clone();
        match action {
            Evenement::Ville(point) => {
                // Une autre ville : on oublie tout ce qu'on avait.
                suivant = Corps { point: Some(point), connecte: self.connecte, ..Corps::default() };
            }
            Evenement::Connexion(ouverte) => suivant.connecte = ouverte,
            Evenement::Recu { sujet, point, corps } => {
                if self.point != Some(point) {
                    return self;
                }
                let corps = Some(Rc::new(corps));
                match sujet.as_str() {
                    "base" => suivant.base = corps,
                    "ensemble" => suivant.ensemble = corps,
                    "quarts" => suivant.quarts = corps,
                    "met" => suivant.met = corps,
                    "station" => suivant.station = corps,
                    "air" => suivant.air = corps,
                    "ciel" => suivant.ciel = corps,
                    "radar" => suivant.radar = corps,
                    _ => return self,
                }
            }
        }
        Rc::new(suivant)
    }
}

/// Lit un message du relais : son sujet, son point, et son corps tel quel.
pub fn lire_message(texte: &str) -> Option<(String, (f64, f64), String)> {
    let v: Value = serde_json::from_str(texte).ok()?;
    Some((
        v["sujet"].as_str()?.to_owned(),
        (v["latitude"].as_f64()?, v["longitude"].as_f64()?),
        v.get("corps")?.to_string(),
    ))
}

/// L'abonnement, tel que le relais l'attend.
pub fn abonnement(parcelle: &Parcelle, jours: u32) -> String {
    format!(
        r#"{{"latitude":{},"longitude":{},"jours":{jours}}}"#,
        parcelle.latitude, parcelle.longitude
    )
}

/// L'attente avant la `n`-ième reconnexion : deux secondes, puis le double,
/// jusqu'à une minute.
pub fn attente_ms(essai: u32) -> i32 {
    (1000_i64 << essai.min(6)).min(60_000) as i32
}

struct Liaison {
    socket: Option<WebSocket>,
    abonnement: String,
    essais: u32,
    ferme: bool,
}

#[hook]
pub fn use_direct(parcelle: Parcelle, endpoints: Endpoints, jours: u32) -> Corps {
    let corps = use_reducer(Corps::default);
    let liaison = use_mut_ref(|| Liaison { socket: None, abonnement: String::new(), essais: 0, ferme: false });

    // La ville change : on le dit au relais, et on oublie l'ancienne.
    {
        let corps = corps.clone();
        let liaison = liaison.clone();
        use_effect_with(parcelle, move |parcelle| {
            corps.dispatch(Evenement::Ville((parcelle.latitude, parcelle.longitude)));
            let message = abonnement(parcelle, jours);
            let mut l = liaison.borrow_mut();
            l.abonnement = message.clone();
            if let Some(socket) = &l.socket {
                if socket.ready_state() == WebSocket::OPEN {
                    let _ = socket.send_with_str(&message);
                }
            }
        });
    }

    // La connexion : ouverte une fois, rouverte quand elle tombe.
    {
        let corps = corps.clone();
        let liaison = liaison.clone();
        use_effect_with(endpoints.direct_url(), move |adresse| {
            if let Some(adresse) = adresse.clone() {
                ouvrir(adresse, liaison.clone(), corps.dispatcher());
            }
            move || {
                let mut l = liaison.borrow_mut();
                l.ferme = true;
                if let Some(socket) = l.socket.take() {
                    let _ = socket.close();
                }
            }
        });
    }

    (*corps).clone()
}

fn ouvrir(adresse: String, liaison: Rc<RefCell<Liaison>>, envoyer: UseReducerDispatcher<Corps>) {
    if liaison.borrow().ferme {
        return;
    }
    let Ok(socket) = WebSocket::new(&adresse) else { return };

    let a_l_ouverture = {
        let liaison = liaison.clone();
        let envoyer = envoyer.clone();
        let socket = socket.clone();
        Closure::<dyn FnMut()>::new(move || {
            let message = {
                let mut l = liaison.borrow_mut();
                l.essais = 0;
                l.abonnement.clone()
            };
            if !message.is_empty() {
                let _ = socket.send_with_str(&message);
            }
            envoyer.dispatch(Evenement::Connexion(true));
        })
    };
    let au_message = {
        let envoyer = envoyer.clone();
        Closure::<dyn FnMut(MessageEvent)>::new(move |evenement: MessageEvent| {
            if let Some((sujet, point, corps)) =
                evenement.data().as_string().as_deref().and_then(lire_message)
            {
                envoyer.dispatch(Evenement::Recu { sujet, point, corps });
            }
        })
    };
    let a_la_fermeture = {
        let liaison = liaison.clone();
        let envoyer = envoyer.clone();
        let adresse = adresse.clone();
        Closure::<dyn FnMut()>::new(move || {
            envoyer.dispatch(Evenement::Connexion(false));
            let attente = {
                let mut l = liaison.borrow_mut();
                l.socket = None;
                if l.ferme {
                    return;
                }
                l.essais += 1;
                attente_ms(l.essais)
            };
            let liaison = liaison.clone();
            let envoyer = envoyer.clone();
            let adresse = adresse.clone();
            let rouvrir = Closure::once_into_js(move || ouvrir(adresse, liaison, envoyer));
            if let Some(fenetre) = web_sys::window() {
                let _ = fenetre.set_timeout_with_callback_and_timeout_and_arguments_0(
                    rouvrir.unchecked_ref(),
                    attente,
                );
            }
        })
    };

    socket.set_onopen(Some(a_l_ouverture.as_ref().unchecked_ref()));
    socket.set_onmessage(Some(au_message.as_ref().unchecked_ref()));
    socket.set_onclose(Some(a_la_fermeture.as_ref().unchecked_ref()));
    // Les rappels vivent aussi longtemps que la connexion : le navigateur les
    // garde, et une page a une seule connexion à la fois.
    a_l_ouverture.forget();
    au_message.forget();
    a_la_fermeture.forget();

    liaison.borrow_mut().socket = Some(socket);
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_message_du_relais_se_lit_avec_son_point() {
        let (sujet, point, corps) = lire_message(
            r#"{"sujet":"quarts","latitude":48.8566,"longitude":2.3522,"corps":{"utc_offset_seconds":7200}}"#,
        )
        .unwrap();
        assert_eq!(sujet, "quarts");
        assert_eq!(point, (48.8566, 2.3522));
        assert_eq!(corps, r#"{"utc_offset_seconds":7200}"#);
        assert!(lire_message("pas du json").is_none());
        assert!(lire_message(r#"{"sujet":"base","corps":{}}"#).is_none(), "sans point, écarté");
    }

    #[test]
    fn un_corps_d_une_autre_ville_est_ecarte() {
        let corps = Rc::new(Corps::default()).reduce(Evenement::Ville((48.85, 2.35)));
        let recu = |point| Evenement::Recu { sujet: "base".into(), point, corps: "{}".into() };
        let ancienne = corps.clone().reduce(recu((45.76, 4.83)));
        assert!(ancienne.base.is_none());
        let bonne = corps.reduce(recu((48.85, 2.35)));
        assert_eq!(bonne.base.as_deref().map(String::as_str), Some("{}"));
        // Changer de ville oublie tout.
        let ailleurs = bonne.reduce(Evenement::Ville((45.76, 4.83)));
        assert!(ailleurs.base.is_none());
    }

    #[test]
    fn la_reconnexion_attend_de_plus_en_plus_sans_depasser_la_minute() {
        assert_eq!(attente_ms(1), 2000);
        assert_eq!(attente_ms(3), 8000);
        assert_eq!(attente_ms(10), 60_000);
    }

    #[test]
    fn l_abonnement_porte_le_point_et_les_jours() {
        let paris = Parcelle { name: "Paris".into(), latitude: 48.8566, longitude: 2.3522, admin: None, country: None };
        assert_eq!(abonnement(&paris, 7), r#"{"latitude":48.8566,"longitude":2.3522,"jours":7}"#);
    }
}
