//! La ronde : chaque minute, ce que chaque île devrait montrer, et l'envoi de
//! ce qui a changé.
//!
//! Une prévision par maille, pas par île : toutes les îles d'une même maille
//! partagent la même lecture, et cette lecture passe par le cache du relais —
//! celui-là même qui sert les applications. Suivre une île ne coûte donc
//! aucune interrogation de plus qu'ouvrir l'application à la même heure.
//!
//! Rien ne part tant que rien ne change. L'heure qui commence part en priorité
//! haute — c'est le moment que l'île doit refléter à la minute ; une prévision
//! renouvelée en cours d'heure part en priorité basse, qu'Apple ne compte pas
//! sur le budget de l'application.

use std::collections::HashMap;

use klima_api::open_meteo::{decode_forecast, forecast_url, Forecast};
use klima_core::endpoints::Endpoints;
use klima_core::grid::{cell_for, cell_key};
use klima_core::position::Parcelle;
use reqwest::Url;
use tokio::task::JoinSet;

use crate::iles::{self, AExaminer};
use crate::routes::{Etat, empreinte};
use crate::upstream::{self, Params};

/// Deux jours : de quoi avoir toujours une heure suivante, même à 23 h.
const JOURS: u32 = 2;

/// Ce qu'une ronde a fait.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Bilan {
    pub poussees: usize,
    pub retirees: usize,
    pub echecs: usize,
}

/// Une ronde complète. Sans clé APNs, elle ne fait rien.
pub async fn ronde(etat: &Etat) -> Bilan {
    let Some(apns) = etat.apns.clone() else { return Bilan::default() };
    let maintenant = (etat.now)();

    let mut par_maille: HashMap<String, Vec<AExaminer>> = HashMap::new();
    for ile in etat.iles.a_examiner(maintenant) {
        par_maille.entry(format!("{:.4},{:.4}", ile.latitude, ile.longitude)).or_default().push(ile);
    }

    let mut envois = JoinSet::new();
    for groupe in par_maille.into_values() {
        let Some(prevision) = prevision(etat, groupe[0].latitude, groupe[0].longitude).await else {
            continue;
        };
        let Some(contenu) = iles::contenu(&prevision, maintenant) else { continue };

        for ile in groupe {
            let urgent = match &ile.poussee {
                Some((deja, _)) if *deja == contenu.empreinte => continue,
                Some((_, heure)) => *heure != contenu.heure,
                // Première ronde après l'inscription : l'iPhone vient de poser
                // son propre état, rien ne presse.
                None => false,
            };
            let corps = iles::corps(&contenu, maintenant);
            let Some(envoi) = apns.envoi(&ile.jeton, corps, urgent, maintenant / 1000) else { continue };
            let apns = apns.clone();
            let (empreinte, heure) = (contenu.empreinte.clone(), contenu.heure);
            envois.spawn(async move { (ile.jeton, empreinte, heure, apns.pousser(envoi).await) });
        }
    }

    let mut bilan = Bilan::default();
    while let Some(fini) = envois.join_next().await {
        let Ok((jeton, empreinte, heure, reponse)) = fini else { continue };
        if reponse.status == Some(200) {
            etat.iles.noter(&jeton, empreinte, heure);
            bilan.poussees += 1;
        } else if reponse.jeton_mort() {
            etat.iles.retirer(&jeton);
            bilan.retirees += 1;
        } else {
            // Le motif, jamais le jeton : il désigne un appareil.
            eprintln!("poussée refusée ({:?}, {:?})", reponse.status, reponse.raison);
            bilan.echecs += 1;
        }
    }
    bilan
}

/// La prévision d'une maille, par le cache du relais.
async fn prevision(etat: &Etat, latitude: f64, longitude: f64) -> Option<Forecast> {
    let parcelle = Parcelle::new("", latitude, longitude);
    let adresse = Url::parse(&forecast_url(&Endpoints::direct(), &parcelle, JOURS)).ok()?;
    let params: Params = adresse.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();

    let maille = cell_for(latitude, longitude);
    let cle = cell_key(&format!("om:{}", empreinte(&params)), &maille);
    let appel =
        upstream::open_meteo_forecast(etat.open_meteo_key.as_deref(), &params, maille.latitude, maille.longitude);
    let lu = etat.forecasts.serve(&cle, || (etat.fetch)(appel)).await.ok()?;
    decode_forecast(parcelle, &lu.value, (etat.now)()).ok()
}

/// Lance la ronde chaque minute, si la poussée est configurée.
pub fn tourner(etat: Etat) {
    if etat.apns.is_none() {
        return;
    }
    tokio::spawn(async move {
        let mut horloge = tokio::time::interval(std::time::Duration::from_secs(60));
        // Une ronde lente ne s'empile pas sur la suivante.
        horloge.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            horloge.tick().await;
            ronde(&etat).await;
        }
    });
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apns::{Apns, Envoi, Reponse};
    use crate::iles::tests::{a, reponse};
    use crate::upstream::UpstreamError;
    use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    const JETON: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";
    const AUTRE: &str = "0f0e0d0c0b0a09080706050403020100";

    struct Banc {
        etat: Etat,
        horloge: Arc<AtomicI64>,
        envois: Arc<Mutex<Vec<Envoi>>>,
        interrogations: Arc<AtomicUsize>,
    }

    fn banc(repondre: fn(&Envoi) -> Reponse) -> Banc {
        let horloge = Arc::new(AtomicI64::new(a(13, 50)));
        let interrogations = Arc::new(AtomicUsize::new(0));
        let compte = interrogations.clone();
        let fetch: crate::upstream::Fetch = Arc::new(move |_| {
            compte.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok::<_, UpstreamError>(reponse()) })
        });
        let lue = horloge.clone();
        let mut etat = crate::routes::etat(fetch, Arc::new(move || lue.load(Ordering::SeqCst)));

        let envois = Arc::new(Mutex::new(Vec::new()));
        let notes = envois.clone();
        let envoyer: crate::apns::Envoyer = Arc::new(move |envoi: Envoi| {
            let reponse = repondre(&envoi);
            notes.lock().unwrap().push(envoi);
            Box::pin(async move { reponse })
        });
        let (pem, _) = crate::apns::tests::cle_p8();
        etat.apns = Some(Apns::new(&pem, "ABC123DEFG", "TEAM123456", envoyer).unwrap());
        Banc { etat, horloge, envois, interrogations }
    }

    fn accepte(_: &Envoi) -> Reponse {
        Reponse { status: Some(200), raison: None }
    }

    fn priorite(envoi: &Envoi) -> String {
        envoi.entetes.iter().find(|(n, _)| *n == "apns-priority").unwrap().1.clone()
    }

    #[tokio::test]
    async fn une_ile_recoit_lheure_suivante_puis_rien_tant_que_rien_ne_change() {
        let b = banc(accepte);
        b.etat.iles.inscrire(JETON, 48.45, 1.49, a(13, 50)).unwrap();

        assert_eq!(ronde(&b.etat).await.poussees, 1);
        assert_eq!(priorite(&b.envois.lock().unwrap()[0]), "5");
        let corps: serde_json::Value = serde_json::from_str(&b.envois.lock().unwrap()[0].corps).unwrap();
        assert_eq!(corps["aps"]["content-state"]["next"]["temperature"], 22.0);

        b.horloge.store(a(13, 58), Ordering::SeqCst);
        assert_eq!(ronde(&b.etat).await.poussees, 0, "même heure, même prévision : rien ne part");
    }

    #[tokio::test]
    async fn lheure_qui_commence_part_en_priorite_haute() {
        let b = banc(accepte);
        b.etat.iles.inscrire(JETON, 48.45, 1.49, a(13, 50)).unwrap();
        ronde(&b.etat).await;

        b.horloge.store(a(14, 0) + 30_000, Ordering::SeqCst);
        assert_eq!(ronde(&b.etat).await.poussees, 1);
        let envois = b.envois.lock().unwrap();
        assert_eq!(priorite(&envois[1]), "10");
        let corps: serde_json::Value = serde_json::from_str(&envois[1].corps).unwrap();
        assert_eq!(corps["aps"]["content-state"]["temperature"], 22.0);
        assert_eq!(corps["aps"]["content-state"]["next"]["weatherCode"], 61);
        assert_eq!(corps["aps"]["stale-date"], a(15, 0) / 1000);
    }

    #[tokio::test]
    async fn deux_iles_de_la_meme_maille_ne_font_quune_interrogation() {
        let b = banc(accepte);
        b.etat.iles.inscrire(JETON, 48.4501, 1.4901, a(13, 50)).unwrap();
        b.etat.iles.inscrire(AUTRE, 48.4502, 1.4902, a(13, 50)).unwrap();
        assert_eq!(ronde(&b.etat).await.poussees, 2);
        assert_eq!(b.interrogations.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn un_jeton_quapple_ne_connait_plus_est_oublie() {
        let b = banc(|_| Reponse { status: Some(410), raison: Some("Unregistered".to_owned()) });
        b.etat.iles.inscrire(JETON, 48.45, 1.49, a(13, 50)).unwrap();
        assert_eq!(ronde(&b.etat).await.retirees, 1);
        assert_eq!(b.etat.iles.taille(), 0);
    }

    #[tokio::test]
    async fn un_refus_passager_se_retente_a_la_ronde_suivante() {
        let b = banc(|_| Reponse { status: Some(429), raison: Some("TooManyRequests".to_owned()) });
        b.etat.iles.inscrire(JETON, 48.45, 1.49, a(13, 50)).unwrap();
        assert_eq!(ronde(&b.etat).await.echecs, 1);
        assert_eq!(ronde(&b.etat).await.echecs, 1);
        assert_eq!(b.etat.iles.taille(), 1);
    }

    #[tokio::test]
    async fn sans_cle_apns_la_ronde_ne_fait_rien() {
        let mut b = banc(accepte);
        b.etat.apns = None;
        b.etat.iles.inscrire(JETON, 48.45, 1.49, a(13, 50)).unwrap();
        assert_eq!(ronde(&b.etat).await, Bilan::default());
        assert_eq!(b.interrogations.load(Ordering::SeqCst), 0);
    }
}
