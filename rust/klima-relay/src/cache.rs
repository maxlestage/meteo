//! Le cache mutualisé.
//!
//! Deux mécanismes, et le second compte autant que le premier :
//!
//! 1. **La péremption.** Une entrée sert pendant `ttl`, puis on la refait.
//!    C'est ce qui découple la facture du nombre d'utilisateurs : une cellule
//!    coûte le même nombre d'appels qu'elle soit ouverte par une personne ou
//!    par mille.
//!
//! 2. **La coalescence.** Cent appareils qui réveillent la même cellule à la
//!    même seconde ne doivent produire qu'une interrogation, pas cent. Sans
//!    cela, un pic de trafic passe à travers le cache et arrive entier chez le
//!    fournisseur — exactement ce qu'on voulait éviter.
//!
//! En cas de panne du fournisseur, une entrée périmée reste préférable à une
//! erreur : la météo d'il y a deux heures reste utilisable, l'absence de météo
//! ne l'est pas. `stale` dit jusqu'où on accepte de servir du périmé.
//!
//! Le cache est complet et éprouvé, mais aucune route ne l'appelle encore :
//! `serve` attend les routes `/v1/…`, qui attendent elles-mêmes le client
//! HTTP. D'où l'autorisation ci-dessous, qui part avec elles — mieux vaut un
//! `allow` commenté qu'une douzaine d'avertissements qu'on apprend à ne plus
//! lire.
#![allow(dead_code)]

//! La coalescence s'écrit ici autrement qu'en TypeScript, et c'est une
//! différence de langue, pas de règle. Là-bas on partage une promesse ; ici un
//! verrou par clé, et celui qui l'obtient vérifie d'abord si un autre n'a pas
//! déjà rempli l'entrée pendant qu'il attendait. Le nombre d'appels sortants
//! est le même, et c'est lui que les tests mesurent.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Horloge injectable, pour que les tests n'aient pas à attendre.
pub type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

#[derive(Clone)]
pub struct CacheOptions {
    /// Durée pendant laquelle une entrée est servie sans être refaite.
    pub ttl_ms: i64,
    /// Au-delà du TTL, durée pendant laquelle une entrée périmée peut encore
    /// dépanner si le fournisseur ne répond plus.
    pub stale_ms: i64,
    pub now: Clock,
}

/// Ce qu'on a servi, et d'où ça vient — l'interface le renvoie en en-tête.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Frais,
    Cache,
    Perime,
}

impl Freshness {
    /// Le libellé exact que le relais TypeScript pose en en-tête.
    pub fn label(self) -> &'static str {
        match self {
            Freshness::Frais => "frais",
            Freshness::Cache => "cache",
            Freshness::Perime => "perime",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Served<T> {
    pub value: T,
    pub freshness: Freshness,
    /// Âge de la donnée servie, en secondes.
    pub age_seconds: i64,
}

#[derive(Clone)]
struct Entry<T> {
    value: T,
    stored_at: i64,
}

pub struct ForecastCache<T> {
    options: CacheOptions,
    entries: Mutex<HashMap<String, Entry<T>>>,
    locks: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
    upstream_calls: AtomicU64,
}

impl<T: Clone + Send + Sync> ForecastCache<T> {
    pub fn new(options: CacheOptions) -> Self {
        Self {
            options,
            entries: Mutex::new(HashMap::new()),
            locks: Mutex::new(HashMap::new()),
            upstream_calls: AtomicU64::new(0),
        }
    }

    /// Interrogations qui ont réellement atteint le fournisseur.
    pub fn calls(&self) -> u64 {
        self.upstream_calls.load(Ordering::SeqCst)
    }

    /// Cellules gardées en mémoire.
    pub fn size(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// L'entrée en place et son âge, si elle existe.
    fn peek(&self, key: &str) -> Option<(T, i64)> {
        let now = (self.options.now)();
        self.entries
            .lock()
            .unwrap()
            .get(key)
            .map(|e| (e.value.clone(), now - e.stored_at))
    }

    /// Sert la clé demandée : depuis le cache s'il est frais, sinon en
    /// interrogeant `fetcher` — une seule fois, même si l'on est plusieurs à
    /// arriver ensemble.
    pub async fn serve<F, Fut, E>(&self, key: &str, fetcher: F) -> Result<Served<T>, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        if let Some((value, age)) = self.peek(key) {
            if age < self.options.ttl_ms {
                return Ok(Served { value, freshness: Freshness::Cache, age_seconds: age / 1000 });
            }
        }

        let lock = {
            let mut locks = self.locks.lock().unwrap();
            locks.entry(key.to_string()).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))).clone()
        };
        let _guard = lock.lock().await;

        // Quelqu'un a pu remplir l'entrée pendant qu'on attendait le verrou.
        // C'est ici que se joue la coalescence : cent arrivées simultanées,
        // une seule interrogation.
        if let Some((value, age)) = self.peek(key) {
            if age < self.options.ttl_ms {
                return Ok(Served { value, freshness: Freshness::Cache, age_seconds: age / 1000 });
            }
        }

        self.upstream_calls.fetch_add(1, Ordering::SeqCst);
        match fetcher().await {
            Ok(value) => {
                let now = (self.options.now)();
                self.entries
                    .lock()
                    .unwrap()
                    .insert(key.to_string(), Entry { value: value.clone(), stored_at: now });
                Ok(Served { value, freshness: Freshness::Frais, age_seconds: 0 })
            }
            Err(error) => {
                // Le fournisseur n'a pas répondu. Une prévision d'il y a deux
                // heures vaut mieux qu'un écran vide ; passé `stale`, on renonce.
                if let Some((value, age)) = self.peek(key) {
                    if age < self.options.ttl_ms + self.options.stale_ms {
                        return Ok(Served {
                            value,
                            freshness: Freshness::Perime,
                            age_seconds: age / 1000,
                        });
                    }
                }
                Err(error)
            }
        }
    }

    /// Oublie les entrées que même le mode dépannage ne servirait plus.
    pub fn sweep(&self) -> usize {
        let limit = self.options.ttl_ms + self.options.stale_ms;
        let now = (self.options.now)();
        let mut entries = self.entries.lock().unwrap();
        let avant = entries.len();
        entries.retain(|_, e| now - e.stored_at < limit);
        avant - entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicI64;

    /// Une horloge qu'on avance à la main : les tests n'attendent pas.
    fn horloge() -> (Clock, Arc<AtomicI64>) {
        let instant = Arc::new(AtomicI64::new(0));
        let lu = instant.clone();
        (Arc::new(move || lu.load(Ordering::SeqCst)), instant)
    }

    fn cache(now: Clock) -> ForecastCache<u32> {
        ForecastCache::new(CacheOptions { ttl_ms: 3_600_000, stale_ms: 7_200_000, now })
    }

    #[tokio::test]
    async fn la_deuxieme_demande_vient_du_cache() {
        let (now, _) = horloge();
        let c = cache(now);
        let a = c.serve("cell", || async { Ok::<_, ()>(1) }).await.unwrap();
        let b = c.serve("cell", || async { Ok::<_, ()>(2) }).await.unwrap();
        assert_eq!(a.freshness, Freshness::Frais);
        assert_eq!(b.freshness, Freshness::Cache);
        assert_eq!(b.value, 1);
        assert_eq!(c.calls(), 1);
    }

    #[tokio::test]
    async fn passe_le_ttl_on_refait() {
        let (now, instant) = horloge();
        let c = cache(now);
        c.serve("cell", || async { Ok::<_, ()>(1) }).await.unwrap();
        instant.store(3_600_001, Ordering::SeqCst);
        let apres = c.serve("cell", || async { Ok::<_, ()>(2) }).await.unwrap();
        assert_eq!(apres.value, 2);
        assert_eq!(c.calls(), 2);
    }

    /// La propriété qui fait tenir le modèle économique, reprise telle quelle
    /// du test TypeScript : une cellule ouverte par tout un village coûte le
    /// même nombre d'appels qu'ouverte par une seule personne.
    #[tokio::test]
    async fn mille_neuf_cent_vingt_consultations_font_vingt_quatre_appels() {
        let (now, instant) = horloge();
        let c = cache(now);
        for minute in 0..1440 {
            instant.store(minute * 60_000, Ordering::SeqCst);
            // Plusieurs appareils réveillent la cellule dans la même minute.
            for _ in 0..(if minute % 3 == 0 { 2 } else { 1 }) {
                c.serve("cell", || async { Ok::<_, ()>(42) }).await.unwrap();
            }
        }
        assert_eq!(c.calls(), 24);
    }

    #[tokio::test]
    async fn une_panne_sert_du_perime_plutot_quune_erreur() {
        let (now, instant) = horloge();
        let c = cache(now);
        c.serve("cell", || async { Ok::<_, &str>(7) }).await.unwrap();
        instant.store(3_600_001, Ordering::SeqCst);
        let servi = c.serve("cell", || async { Err("fournisseur muet") }).await.unwrap();
        assert_eq!(servi.freshness, Freshness::Perime);
        assert_eq!(servi.value, 7);
    }

    #[tokio::test]
    async fn passe_le_delai_de_depannage_on_renonce() {
        let (now, instant) = horloge();
        let c = cache(now);
        c.serve("cell", || async { Ok::<_, &str>(7) }).await.unwrap();
        instant.store(3_600_000 + 7_200_001, Ordering::SeqCst);
        let echec = c.serve("cell", || async { Err("fournisseur muet") }).await;
        assert!(echec.is_err());
    }

    #[tokio::test]
    async fn le_balayage_oublie_ce_que_meme_le_depannage_ne_sert_plus() {
        let (now, instant) = horloge();
        let c = cache(now);
        c.serve("cell", || async { Ok::<_, ()>(1) }).await.unwrap();
        assert_eq!(c.size(), 1);
        instant.store(3_600_000 + 7_200_001, Ordering::SeqCst);
        assert_eq!(c.sweep(), 1);
        assert_eq!(c.size(), 0);
    }

    #[tokio::test]
    async fn les_libelles_sont_ceux_que_le_relais_pose_en_entete() {
        assert_eq!(Freshness::Frais.label(), "frais");
        assert_eq!(Freshness::Cache.label(), "cache");
        assert_eq!(Freshness::Perime.label(), "perime");
    }
}
