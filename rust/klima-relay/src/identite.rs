//! Qui demande : l'identité prouvée par Apple, et la session qu'en tire le relais.
//!
//! Deux questions distinctes, et ce module ne répond qu'à la première.
//!
//! - **Qui est-ce ?** Apple le prouve. « Se connecter avec Apple » remet à
//!   l'application un jeton signé par Apple, qui dit : cette personne contrôle
//!   cette adresse. Le relais vérifie la signature avec les clés publiques
//!   d'Apple, l'émetteur, le destinataire, l'échéance. Rien ne se devine ni ne
//!   se tape : une adresse invitée ne suffit plus, il faut en être le titulaire.
//! - **A-t-elle le palier ?** C'est `KLIMA_PRO` qui le dit, comme avant, dans
//!   `pro.rs`. Retirer quelqu'un de la liste lui retire l'accès à la question
//!   suivante, session ou pas.
//!
//! Le jeton d'Apple vit dix minutes : impossible de le garder pour redemander
//! le palier demain. Le relais l'échange donc, une fois vérifié, contre une
//! **session** qu'il signe lui-même, avec un secret qui vit dans
//! l'environnement du serveur et nulle part ailleurs. La session dit seulement
//! « Apple a prouvé cette adresse » ; elle n'accorde rien par elle-même.
//!
//! Pas de base de données, pas de mot de passe, pas de compte à créer. La
//! liste d'invités reste la seule chose à gérer, une adresse par personne.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header};
use serde::{Deserialize, Deserializer, Serialize};

/// Celui qui signe les jetons d'identité d'Apple.
pub const APPLE_EMETTEUR: &str = "https://appleid.apple.com";

/// Où Apple publie ses clés publiques.
pub const APPLE_CLES: &str = "https://appleid.apple.com/auth/keys";

/// Le destinataire attendu des jetons d'Apple : l'identifiant de l'application.
/// Un jeton émis pour une autre application ne vaut rien ici.
pub const AUDIENCE_PAR_DEFAUT: &str = "com.kliima.app";

/// Le nom que le relais signe dans ses propres sessions.
const EMETTEUR_SESSION: &str = "klima-relais";

/// Une session vaut six mois. Au-delà, l'application redemande une connexion
/// à Apple — une tape, sans rien à retaper.
pub const DUREE_SESSION_S: i64 = 180 * 24 * 3600;

/// Ce qu'Apple a prouvé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identite {
    /// L'identifiant stable qu'Apple donne à cette personne pour cette
    /// application. Il ne change pas si l'adresse change.
    pub sujet: String,
    /// L'adresse vérifiée, normalisée comme la liste d'invités.
    pub courriel: String,
}

/// Pourquoi un jeton ne prouve rien.
///
/// Ces motifs restent dans le relais — journal et tests. La réponse au client
/// n'en dit pas plus qu'un refus : détailler pourquoi un jeton est rejeté
/// aiderait surtout qui en fabrique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refus {
    Illisible,
    CleInconnue,
    Signature,
    Expire,
    Emetteur,
    Audience,
    SansCourriel,
    CourrielNonVerifie,
}

/* ---- les clés d'Apple ---- */

/// Les clés publiques d'Apple, telles que `auth/keys` les publie.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cles {
    pub keys: Vec<Cle>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cle {
    pub kid: String,
    /// Module et exposant, en base64url — la forme JWK, que `jsonwebtoken`
    /// lit directement.
    pub n: String,
    pub e: String,
}

impl Cles {
    pub fn lire(corps: &str) -> Option<Cles> {
        serde_json::from_str(corps).ok()
    }

    fn par_kid(&self, kid: &str) -> Option<&Cle> {
        self.keys.iter().find(|cle| cle.kid == kid)
    }
}

/* ---- le jeton d'Apple ---- */

#[derive(Deserialize)]
struct RevendicationsApple {
    sub: String,
    exp: i64,
    #[serde(default)]
    email: Option<String>,
    #[serde(default, deserialize_with = "booleen_souple")]
    email_verified: Option<bool>,
}

/// Apple écrit `email_verified` tantôt `true`, tantôt `"true"`.
fn booleen_souple<'de, D: Deserializer<'de>>(d: D) -> Result<Option<bool>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Souple {
        Booleen(bool),
        Texte(String),
    }
    Ok(match Option::<Souple>::deserialize(d)? {
        Some(Souple::Booleen(b)) => Some(b),
        Some(Souple::Texte(t)) => Some(t.eq_ignore_ascii_case("true")),
        None => None,
    })
}

/// Vérifie un jeton d'identité d'Apple.
///
/// L'échéance se compare à `maintenant` (secondes) plutôt qu'à l'horloge du
/// système : c'est ce qui rend l'expiration vérifiable dans un test.
pub fn verifier_apple(
    jeton: &str,
    cles: &Cles,
    audience: &str,
    maintenant: i64,
) -> Result<Identite, Refus> {
    let entete = decode_header(jeton).map_err(|_| Refus::Illisible)?;
    // Apple signe en RS256 et en rien d'autre. Accepter l'algorithme que le
    // jeton annonce, c'est la faille classique : un « none » ou un HS256 signé
    // avec la clé publique passerait.
    if entete.alg != Algorithm::RS256 {
        return Err(Refus::Signature);
    }
    let kid = entete.kid.ok_or(Refus::CleInconnue)?;
    let cle = cles.par_kid(&kid).ok_or(Refus::CleInconnue)?;
    let cle = DecodingKey::from_rsa_components(&cle.n, &cle.e).map_err(|_| Refus::CleInconnue)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false; // comparée plus bas, à l'horloge injectée
    validation.validate_aud = false; // idem, pour distinguer les motifs
    validation.required_spec_claims.clear();

    let lu = decode::<serde_json::Value>(jeton, &cle, &validation).map_err(|_| Refus::Signature)?;
    let brut = lu.claims;

    if brut.get("iss").and_then(|v| v.as_str()) != Some(APPLE_EMETTEUR) {
        return Err(Refus::Emetteur);
    }
    let audience_ok = match brut.get("aud") {
        Some(serde_json::Value::String(a)) => a == audience,
        Some(serde_json::Value::Array(a)) => a.iter().any(|v| v.as_str() == Some(audience)),
        _ => false,
    };
    if !audience_ok {
        return Err(Refus::Audience);
    }

    let revendications: RevendicationsApple =
        serde_json::from_value(brut).map_err(|_| Refus::Illisible)?;
    if revendications.exp <= maintenant {
        return Err(Refus::Expire);
    }

    let courriel = revendications
        .email
        .map(|c| c.trim().to_lowercase())
        .filter(|c| !c.is_empty())
        .ok_or(Refus::SansCourriel)?;
    // Une adresse qu'Apple dit non vérifiée ne prouve rien : c'est le seul
    // endroit où elle compte, et on ne transige pas.
    if revendications.email_verified == Some(false) {
        return Err(Refus::CourrielNonVerifie);
    }

    Ok(Identite { sujet: revendications.sub, courriel })
}

/* ---- la session du relais ---- */

#[derive(Serialize, Deserialize)]
struct RevendicationsSession {
    iss: String,
    sub: String,
    email: String,
    iat: i64,
    exp: i64,
}

/// Signe une session pour cette identité.
pub fn emettre_session(identite: &Identite, secret: &[u8], maintenant: i64) -> String {
    let revendications = RevendicationsSession {
        iss: EMETTEUR_SESSION.to_owned(),
        sub: identite.sujet.clone(),
        email: identite.courriel.clone(),
        iat: maintenant,
        exp: maintenant + DUREE_SESSION_S,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &revendications,
        &EncodingKey::from_secret(secret),
    )
    // HS256 sur une structure sérialisable ne peut pas échouer ; si la
    // bibliothèque changeait d'avis, mieux vaut le savoir tout de suite.
    .expect("signature HS256")
}

/// Relit une session émise par ce relais.
pub fn lire_session(jeton: &str, secret: &[u8], maintenant: i64) -> Result<Identite, Refus> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.required_spec_claims.clear();
    validation.set_issuer(&[EMETTEUR_SESSION]);

    let lu = decode::<RevendicationsSession>(jeton, &DecodingKey::from_secret(secret), &validation)
        .map_err(|erreur| match erreur.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => Refus::Emetteur,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => Refus::Signature,
            _ => Refus::Illisible,
        })?;
    if lu.claims.exp <= maintenant {
        return Err(Refus::Expire);
    }
    Ok(Identite { sujet: lu.claims.sub, courriel: lu.claims.email })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod essai {
    //! Un faux Apple : une paire de clés fabriquée à l'exécution, et de quoi
    //! signer des jetons comme Apple les signe. Aucune clé privée n'est
    //! commitée — elle naît et meurt avec la suite de tests.

    use std::sync::OnceLock;

    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::traits::PublicKeyParts;

    use super::{Cle, Cles};

    pub const KID: &str = "cle-essai";

    struct Paire {
        prive: EncodingKey,
        publique: Cle,
    }

    fn paire() -> &'static Paire {
        static PAIRE: OnceLock<Paire> = OnceLock::new();
        PAIRE.get_or_init(|| {
            let mut alea = rand::thread_rng();
            let cle = rsa::RsaPrivateKey::new(&mut alea, 2048).expect("clé d'essai");
            let der = cle.to_pkcs1_der().expect("DER");
            Paire {
                prive: EncodingKey::from_rsa_der(der.as_bytes()),
                publique: Cle {
                    kid: KID.to_owned(),
                    n: URL_SAFE_NO_PAD.encode(cle.n().to_bytes_be()),
                    e: URL_SAFE_NO_PAD.encode(cle.e().to_bytes_be()),
                },
            }
        })
    }

    pub fn cles() -> Cles {
        Cles { keys: vec![paire().publique.clone()] }
    }

    /// Un jeton signé comme Apple le signe, avec les revendications données.
    pub fn jeton(revendications: serde_json::Value) -> String {
        jeton_avec_kid(revendications, KID)
    }

    pub fn jeton_avec_kid(revendications: serde_json::Value, kid: &str) -> String {
        let mut entete = Header::new(Algorithm::RS256);
        entete.kid = Some(kid.to_owned());
        jsonwebtoken::encode(&entete, &revendications, &paire().prive).expect("jeton d'essai")
    }

    /// Les revendications d'un jeton d'Apple valide à l'instant `maintenant`.
    pub fn apple(courriel: &str, maintenant: i64) -> serde_json::Value {
        serde_json::json!({
            "iss": super::APPLE_EMETTEUR,
            "aud": super::AUDIENCE_PAR_DEFAUT,
            "sub": "000123.abc.0456",
            "iat": maintenant,
            "exp": maintenant + 600,
            "email": courriel,
            "email_verified": "true",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::essai::{apple, cles, jeton, jeton_avec_kid};
    use super::*;

    const T: i64 = 1_790_000_000;
    const SECRET: &[u8] = b"un secret d'essai suffisamment long pour HS256";

    #[test]
    fn un_jeton_dapple_valide_donne_une_identite() {
        let id = verifier_apple(&jeton(apple("Max@Ferme.FR", T)), &cles(), AUDIENCE_PAR_DEFAUT, T)
            .expect("valide");
        assert_eq!(id.courriel, "max@ferme.fr", "normalisée comme la liste");
        assert_eq!(id.sujet, "000123.abc.0456");
    }

    #[test]
    fn email_verified_se_lit_en_booleen_comme_en_texte() {
        let mut r = apple("max@ferme.fr", T);
        r["email_verified"] = serde_json::json!(true);
        assert!(verifier_apple(&jeton(r), &cles(), AUDIENCE_PAR_DEFAUT, T).is_ok());
    }

    #[test]
    fn un_jeton_expire_ne_prouve_rien() {
        let j = jeton(apple("max@ferme.fr", T));
        assert_eq!(
            verifier_apple(&j, &cles(), AUDIENCE_PAR_DEFAUT, T + 601),
            Err(Refus::Expire)
        );
    }

    #[test]
    fn un_jeton_pour_une_autre_application_ne_vaut_rien_ici() {
        let mut r = apple("max@ferme.fr", T);
        r["aud"] = serde_json::json!("com.autre.app");
        assert_eq!(
            verifier_apple(&jeton(r), &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::Audience)
        );
    }

    #[test]
    fn un_autre_emetteur_est_refuse() {
        let mut r = apple("max@ferme.fr", T);
        r["iss"] = serde_json::json!("https://faux.example");
        assert_eq!(
            verifier_apple(&jeton(r), &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::Emetteur)
        );
    }

    #[test]
    fn une_cle_inconnue_est_refusee() {
        let j = jeton_avec_kid(apple("max@ferme.fr", T), "une-autre-cle");
        assert_eq!(
            verifier_apple(&j, &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::CleInconnue)
        );
    }

    #[test]
    fn une_signature_retouchee_est_refusee() {
        // On change une revendication sans resigner : l'adresse d'un autre,
        // glissée dans un jeton authentique.
        let j = jeton(apple("max@ferme.fr", T));
        let mut morceaux: Vec<String> = j.split('.').map(str::to_owned).collect();
        let mut r = apple("pirate@ailleurs.fr", T);
        r["sub"] = serde_json::json!("000123.abc.0456");
        use base64::Engine;
        morceaux[1] = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&r).unwrap());
        let falsifie = morceaux.join(".");
        assert_eq!(
            verifier_apple(&falsifie, &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::Signature)
        );
    }

    #[test]
    fn un_jeton_hs256_signe_avec_la_cle_publique_est_refuse() {
        // La faille classique : annoncer HS256 et signer avec la clé
        // publique, que tout le monde connaît.
        let cle_publique = cles().keys[0].n.clone();
        let mut entete = Header::new(Algorithm::HS256);
        entete.kid = Some(super::essai::KID.to_owned());
        let j = jsonwebtoken::encode(
            &entete,
            &apple("max@ferme.fr", T),
            &EncodingKey::from_secret(cle_publique.as_bytes()),
        )
        .unwrap();
        assert_eq!(
            verifier_apple(&j, &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::Signature)
        );
    }

    #[test]
    fn sans_adresse_ou_avec_une_adresse_non_verifiee_rien_nest_prouve() {
        let mut sans = apple("max@ferme.fr", T);
        sans.as_object_mut().unwrap().remove("email");
        assert_eq!(
            verifier_apple(&jeton(sans), &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::SansCourriel)
        );

        let mut non_verifiee = apple("max@ferme.fr", T);
        non_verifiee["email_verified"] = serde_json::json!("false");
        assert_eq!(
            verifier_apple(&jeton(non_verifiee), &cles(), AUDIENCE_PAR_DEFAUT, T),
            Err(Refus::CourrielNonVerifie)
        );
    }

    #[test]
    fn nimporte_quoi_est_illisible() {
        for j in ["", "abc", "a.b.c", "max@ferme.fr"] {
            assert!(verifier_apple(j, &cles(), AUDIENCE_PAR_DEFAUT, T).is_err(), "{j}");
        }
    }

    #[test]
    fn les_cles_se_lisent_telles_quapple_les_publie() {
        let corps = r#"{"keys":[{"kty":"RSA","kid":"W6WcOKB","use":"sig","alg":"RS256","n":"2Zc5","e":"AQAB"}]}"#;
        let lues = Cles::lire(corps).expect("lisible");
        assert_eq!(lues.keys[0].kid, "W6WcOKB");
        assert_eq!(Cles::lire("<html>"), None);
    }

    /* ---- la session ---- */

    fn identite() -> Identite {
        Identite { sujet: "000123.abc.0456".into(), courriel: "max@ferme.fr".into() }
    }

    #[test]
    fn une_session_se_relit() {
        let s = emettre_session(&identite(), SECRET, T);
        assert_eq!(lire_session(&s, SECRET, T + 10), Ok(identite()));
    }

    #[test]
    fn une_session_expire_au_bout_de_six_mois() {
        let s = emettre_session(&identite(), SECRET, T);
        assert!(lire_session(&s, SECRET, T + DUREE_SESSION_S - 1).is_ok());
        assert_eq!(lire_session(&s, SECRET, T + DUREE_SESSION_S), Err(Refus::Expire));
    }

    #[test]
    fn une_session_signee_avec_un_autre_secret_ne_vaut_rien() {
        // C'est aussi ce qui se passe quand on change le secret du relais :
        // toutes les sessions tombent, et chacun se reconnecte d'une tape.
        let s = emettre_session(&identite(), b"un autre secret, tout aussi long que l'autre", T);
        assert_eq!(lire_session(&s, SECRET, T), Err(Refus::Signature));
    }

    #[test]
    fn un_jeton_dapple_nest_pas_une_session() {
        // Même s'il était signé avec le bon secret, l'émetteur n'est pas le
        // relais : on ne confond pas les deux jetons.
        let faux = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &apple("max@ferme.fr", T),
            &EncodingKey::from_secret(SECRET),
        )
        .unwrap();
        assert!(lire_session(&faux, SECRET, T).is_err());
    }
}
