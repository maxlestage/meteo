//! Les mises à jour poussées aux îles dynamiques, par le service de
//! notifications d'Apple (APNs).
//!
//! Sans elles, une activité en direct ne change que quand l'iPhone le décide :
//! l'application ouverte, ou un réveil d'arrière-plan qu'iOS place où il veut
//! — parfois des heures plus tard. Le relais, lui, connaît l'heure : il pousse
//! la nouvelle heure quand elle commence, et la suivante calculée d'avance.
//!
//! **La clé vit ici et nulle part ailleurs.** Celle qu'Apple délivre (un `.p8`)
//! signe tout envoi à n'importe quel appareil de l'équipe : glissée dans
//! l'application, elle serait publiée. Elle arrive par l'environnement, avec
//! son identifiant et celui de l'équipe ; sans elle, rien n'est poussé et
//! `/health` le dit.
//!
//! L'envoi est injectable, comme les interrogations des fournisseurs : les
//! tests n'ont pas Apple sous la main, et tout ce qui compte — les en-têtes,
//! le jeton signé, le corps — se vérifie sans.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header};

/// Le service de production. Les versions TestFlight et App Store y sont
/// inscrites ; seule une compilation de développement passerait par le bac à
/// sable (`KLIMA_APNS_HOST`).
pub const HOTE_PRODUCTION: &str = "https://api.push.apple.com";

/// Le sujet d'une activité en direct : l'identifiant de l'application suivi
/// du suffixe qu'Apple réserve à ce type d'envoi.
pub const SUJET: &str = "com.kliima.app.push-type.liveactivity";

/// Apple refuse un jeton de plus d'une heure, et un jeton renouvelé plus
/// d'une fois toutes les vingt minutes. Quarante minutes tiennent entre les
/// deux.
const JETON_DUREE_S: i64 = 40 * 60;

/// Un envoi prêt à partir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envoi {
    pub url: String,
    pub entetes: Vec<(&'static str, String)>,
    pub corps: String,
}

/// Ce qu'Apple a répondu : le code, et le motif d'un refus.
///
/// `status` vaut `None` quand Apple n'a pas répondu du tout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reponse {
    pub status: Option<u16>,
    pub raison: Option<String>,
}

impl Reponse {
    /// Le jeton ne mène plus nulle part : l'activité est finie, ou n'a jamais
    /// existé. Il ne sert à rien de réessayer.
    pub fn jeton_mort(&self) -> bool {
        matches!(self.status, Some(410))
            || matches!(
                self.raison.as_deref(),
                Some("BadDeviceToken" | "Unregistered" | "ExpiredToken" | "DeviceTokenNotForTopic")
            )
    }
}

/// Ce qui exécute l'envoi.
pub type Envoyer = Arc<dyn Fn(Envoi) -> Pin<Box<dyn Future<Output = Reponse> + Send>> + Send + Sync>;

/// Pourquoi la configuration ne tient pas. Le motif va au journal ; la valeur,
/// jamais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalide {
    /// La clé ne se lit pas, ou ne signe pas.
    Cle,
    /// L'identifiant de la clé : dix caractères, lettres et chiffres.
    IdCle,
    /// L'identifiant de l'équipe : dix caractères, lettres et chiffres.
    Equipe,
}

#[derive(Clone)]
pub struct Apns {
    cle: Arc<EncodingKey>,
    id_cle: String,
    equipe: String,
    pub hote: String,
    /// Le dernier jeton signé, et quand.
    jeton: Arc<Mutex<Option<(i64, String)>>>,
    envoyer: Envoyer,
}

impl std::fmt::Debug for Apns {
    // À la main : la clé n'a rien à faire dans un journal.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Apns").field("hote", &self.hote).finish_non_exhaustive()
    }
}

impl Apns {
    /// Monte l'envoi depuis la clé `.p8`, son identifiant et celui de l'équipe.
    ///
    /// La clé s'accepte telle qu'Apple la donne — avec ses lignes « BEGIN » et
    /// « END » et ses retours à la ligne — ou réduite à son corps sur une seule
    /// ligne, ce que laisse un champ de tableau de bord qui avale les retours.
    /// Elle est essayée tout de suite : une clé qui ne signe pas se signale au
    /// démarrage, pas à la première heure pleine.
    pub fn new(p8: &str, id_cle: &str, equipe: &str, envoyer: Envoyer) -> Result<Self, Invalide> {
        let identifiant = |v: &str| v.len() == 10 && v.chars().all(|c| c.is_ascii_alphanumeric());
        let id_cle = id_cle.trim();
        let equipe = equipe.trim();
        if !identifiant(id_cle) {
            return Err(Invalide::IdCle);
        }
        if !identifiant(equipe) {
            return Err(Invalide::Equipe);
        }

        let der = lire_p8(p8).ok_or(Invalide::Cle)?;
        let apns = Apns {
            cle: Arc::new(EncodingKey::from_ec_der(&der)),
            id_cle: id_cle.to_owned(),
            equipe: equipe.to_owned(),
            hote: HOTE_PRODUCTION.to_owned(),
            jeton: Arc::new(Mutex::new(None)),
            envoyer,
        };
        apns.signer(0).ok_or(Invalide::Cle)?;
        Ok(apns)
    }

    /// Le jeton du fournisseur, signé de la clé, renouvelé toutes les quarante
    /// minutes.
    pub fn jeton(&self, maintenant_s: i64) -> Option<String> {
        let mut garde = self.jeton.lock().ok()?;
        if let Some((quand, jeton)) = garde.as_ref() {
            if maintenant_s - quand < JETON_DUREE_S && maintenant_s >= *quand {
                return Some(jeton.clone());
            }
        }
        let jeton = self.signer(maintenant_s)?;
        *garde = Some((maintenant_s, jeton.clone()));
        Some(jeton)
    }

    fn signer(&self, maintenant_s: i64) -> Option<String> {
        let mut entete = Header::new(Algorithm::ES256);
        entete.kid = Some(self.id_cle.clone());
        let revendications = serde_json::json!({ "iss": self.equipe, "iat": maintenant_s });
        jsonwebtoken::encode(&entete, &revendications, &self.cle).ok()
    }

    /// L'envoi d'une mise à jour d'activité vers un appareil.
    ///
    /// `urgent` demande la priorité haute : Apple la compte sur un budget,
    /// on la garde pour le changement d'heure.
    pub fn envoi(&self, jeton_appareil: &str, corps: String, urgent: bool, maintenant_s: i64) -> Option<Envoi> {
        let jeton = self.jeton(maintenant_s)?;
        Some(Envoi {
            url: format!("{}/3/device/{jeton_appareil}", self.hote.trim_end_matches('/')),
            entetes: vec![
                ("authorization", format!("bearer {jeton}")),
                ("apns-push-type", "liveactivity".to_owned()),
                ("apns-topic", SUJET.to_owned()),
                ("apns-priority", if urgent { "10" } else { "5" }.to_owned()),
            ],
            corps,
        })
    }

    pub async fn pousser(&self, envoi: Envoi) -> Reponse {
        (self.envoyer)(envoi).await
    }
}

/// Le corps DER (PKCS#8) d'une clé `.p8`.
fn lire_p8(p8: &str) -> Option<Vec<u8>> {
    // Les en-têtes, puis tout ce qui n'est pas de la base 64 : retours à la
    // ligne, espaces, et les « \n » littéraux qu'un copier-coller laisse
    // parfois dans une variable d'environnement.
    let corps: String = p8
        .replace("\\n", "\n")
        .lines()
        .filter(|ligne| !ligne.trim_start().starts_with("-----"))
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = base64::engine::general_purpose::STANDARD.decode(corps).ok()?;
    (!der.is_empty()).then_some(der)
}

/// L'envoi réel, en HTTP/2 — APNs n'accepte rien d'autre.
pub fn http_envoyer(client: reqwest::Client) -> Envoyer {
    Arc::new(move |envoi: Envoi| {
        let client = client.clone();
        Box::pin(async move {
            let mut demande = client.post(&envoi.url).body(envoi.corps);
            for (nom, valeur) in &envoi.entetes {
                demande = demande.header(*nom, valeur);
            }
            match demande.send().await {
                Ok(reponse) => {
                    let status = reponse.status().as_u16();
                    let raison = if status == 200 {
                        None
                    } else {
                        reponse
                            .text()
                            .await
                            .ok()
                            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                            .and_then(|v| v.get("reason")?.as_str().map(str::to_owned))
                    };
                    Reponse { status: Some(status), raison }
                }
                Err(_) => Reponse { status: None, raison: None },
            }
        })
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, Validation};
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    use p256::pkcs8::EncodePrivateKey;

    /// Une clé fabriquée à l'exécution, au format d'Apple : aucune clé privée
    /// n'est commitée.
    pub(crate) fn cle_p8() -> (String, Vec<u8>) {
        let secrete = p256::SecretKey::random(&mut rand::rngs::OsRng);
        let pem = secrete.to_pkcs8_pem(p256::pkcs8::LineEnding::LF).expect("pem").to_string();
        let publique = secrete.public_key().to_encoded_point(false).as_bytes().to_vec();
        (pem, publique)
    }

    pub(crate) fn muet() -> Envoyer {
        Arc::new(|_| Box::pin(async { Reponse { status: Some(200), raison: None } }))
    }

    #[test]
    fn la_cle_dapple_signe_un_jeton_quapple_sait_lire() {
        let (pem, publique) = cle_p8();
        let apns = Apns::new(&pem, "ABC123DEFG", "TEAM123456", muet()).expect("clé lue");

        let jeton = apns.jeton(1_790_000_000).expect("jeton");
        let entete = jsonwebtoken::decode_header(&jeton).expect("en-tête");
        assert_eq!(entete.alg, Algorithm::ES256);
        assert_eq!(entete.kid.as_deref(), Some("ABC123DEFG"));

        let mut validation = Validation::new(Algorithm::ES256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        let lu = jsonwebtoken::decode::<serde_json::Value>(
            &jeton,
            &DecodingKey::from_ec_der(&publique),
            &validation,
        )
        .expect("signature valide");
        assert_eq!(lu.claims["iss"], "TEAM123456");
        assert_eq!(lu.claims["iat"], 1_790_000_000);
    }

    #[test]
    fn la_cle_se_lit_aussi_sur_une_seule_ligne() {
        let (pem, _) = cle_p8();
        let corps: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
        assert!(Apns::new(&corps, "ABC123DEFG", "TEAM123456", muet()).is_ok());

        let echappee = pem.replace('\n', "\\n");
        assert!(Apns::new(&echappee, "ABC123DEFG", "TEAM123456", muet()).is_ok());
    }

    #[test]
    fn une_configuration_bancale_dit_laquelle_sans_la_repeter() {
        let (pem, _) = cle_p8();
        assert_eq!(Apns::new("pas une clé", "ABC123DEFG", "TEAM123456", muet()).unwrap_err(), Invalide::Cle);
        assert_eq!(Apns::new("QUJD", "ABC123DEFG", "TEAM123456", muet()).unwrap_err(), Invalide::Cle);
        assert_eq!(Apns::new(&pem, "court", "TEAM123456", muet()).unwrap_err(), Invalide::IdCle);
        assert_eq!(Apns::new(&pem, "ABC123DEFG", "", muet()).unwrap_err(), Invalide::Equipe);
        assert!(!format!("{:?}", Apns::new(&pem, "ABC123DEFG", "TEAM123456", muet()).unwrap()).contains("PRIVATE"));
    }

    #[test]
    fn le_jeton_se_garde_quarante_minutes_puis_se_renouvelle() {
        let (pem, _) = cle_p8();
        let apns = Apns::new(&pem, "ABC123DEFG", "TEAM123456", muet()).unwrap();
        let premier = apns.jeton(1_000_000).unwrap();
        assert_eq!(apns.jeton(1_000_000 + 39 * 60).unwrap(), premier);
        assert_ne!(apns.jeton(1_000_000 + 41 * 60).unwrap(), premier);
    }

    #[test]
    fn lenvoi_porte_ce_quapple_exige_pour_une_activite() {
        let (pem, _) = cle_p8();
        let apns = Apns::new(&pem, "ABC123DEFG", "TEAM123456", muet()).unwrap();
        let envoi = apns.envoi("a1b2", "{}".to_owned(), true, 1_000_000).unwrap();
        assert_eq!(envoi.url, "https://api.push.apple.com/3/device/a1b2");
        let entete = |nom: &str| envoi.entetes.iter().find(|(n, _)| *n == nom).map(|(_, v)| v.clone());
        assert_eq!(entete("apns-push-type").as_deref(), Some("liveactivity"));
        assert_eq!(entete("apns-topic").as_deref(), Some("com.kliima.app.push-type.liveactivity"));
        assert_eq!(entete("apns-priority").as_deref(), Some("10"));
        assert!(entete("authorization").unwrap().starts_with("bearer "));

        let calme = apns.envoi("a1b2", "{}".to_owned(), false, 1_000_000).unwrap();
        assert!(calme.entetes.contains(&("apns-priority", "5".to_owned())));
    }

    #[test]
    fn un_jeton_mort_se_reconnait_a_la_reponse_dapple() {
        let r = |status, raison: Option<&str>| Reponse { status, raison: raison.map(str::to_owned) };
        assert!(r(Some(410), Some("Unregistered")).jeton_mort());
        assert!(r(Some(400), Some("BadDeviceToken")).jeton_mort());
        assert!(!r(Some(200), None).jeton_mort());
        assert!(!r(Some(429), Some("TooManyRequests")).jeton_mort());
        assert!(!r(None, None).jeton_mort());
    }
}
