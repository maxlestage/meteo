//! Le radar européen, lu par le relais.
//!
//! EUMETNET publie toutes les cinq minutes la mosaïque OPERA — les radars de
//! Météo-France et de ses voisins assemblés au kilomètre — dans un seau
//! public, sans clé, sous licence CC BY 4.0 :
//!
//! ```text
//! https://s3.waw3-1.cloudferro.com/openradar-24h/2026/10/06/OPERA/COMP/OPERA@20261006T1900@0@DBZH.tiff
//! ```
//!
//! Chaque image est un GeoTIFF de 3 800 × 4 400 pixels, en tuiles de 512
//! compressées une à une (deflate) : deux bandes de flottants, la réflectivité
//! puis sa qualité. Le relais télécharge une image quand on la lui demande —
//! une seule pour toutes les villes —, garde les quatre dernières, et ne
//! décompresse que les tuiles où des villes se trouvent. Il en tire, pour
//! chaque ville, ce que dit `klima_core::radar` : le débit au-dessus d'elle,
//! le déplacement des averses entre deux images à un quart d'heure
//! d'intervalle, et les deux heures qui viennent.
//!
//! Un téléphone ne pourrait pas en faire autant (3 Mo toutes les cinq
//! minutes) ; un navigateur non plus. C'est donc ici, et seulement ici.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use klima_core::calendar::civil_from_ms;
use klima_core::radar::{
    Champ, Echo, FAUX_EST, FAUX_NORD, Grille, LAT_CENTRE, LON_CENTRE, Prevision, deplacement, echo, prevoir,
    seuils::{DEMI_FENETRE_KM, ECART_MOUVEMENT_MS, PAS_IMAGE_MS},
};

use crate::cache::Clock;
use crate::upstream::UpstreamError;

/// Ce qui télécharge une image, en octets. Injectable : les tests n'ont pas
/// de réseau, et fabriquent leurs propres images.
pub type Charge =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, UpstreamError>> + Send>> + Send + Sync>;

/// Le seau des dernières vingt-quatre heures.
const SEAU: &str = "https://s3.waw3-1.cloudferro.com/openradar-24h";

/// Combien d'images on garde : la plus récente, celle d'un quart d'heure
/// avant, et de quoi passer d'une image à la suivante sans tout refaire.
const IMAGES_GARDEES: usize = 4;

/// Une image absente (pas encore publiée) n'est redemandée qu'après une
/// minute : sinon chaque ville relancerait la même requête vaine.
const ABSENCE_MS: i64 = 60_000;

/// L'adresse de l'image d'une heure donnée (ms UTC, sur un multiple de cinq
/// minutes).
pub fn adresse(heure: i64) -> String {
    let c = civil_from_ms(heure);
    format!(
        "{SEAU}/{:04}/{:02}/{:02}/OPERA/COMP/OPERA@{:04}{:02}{:02}T{:02}{:02}@0@DBZH.tiff",
        c.year, c.month, c.day, c.year, c.month, c.day, c.hour, c.minute
    )
}

/// Une image lue : sa grille, et ses tuiles, décompressées à la demande.
pub struct Image {
    pub heure: i64,
    pub grille: Grille,
    tuile_l: usize,
    tuile_h: usize,
    tuiles_x: usize,
    echantillons: usize,
    morceaux: Vec<(usize, usize)>,
    octets: Vec<u8>,
    decodees: Mutex<HashMap<usize, Arc<Vec<f32>>>>,
}

/// Les étiquettes TIFF qu'on lit.
mod etiquette {
    pub const LARGEUR: u16 = 256;
    pub const HAUTEUR: u16 = 257;
    pub const BITS: u16 = 258;
    pub const COMPRESSION: u16 = 259;
    pub const ECHANTILLONS: u16 = 277;
    pub const PLANAIRE: u16 = 284;
    pub const PREDICTEUR: u16 = 317;
    pub const TUILE_L: u16 = 322;
    pub const TUILE_H: u16 = 323;
    pub const TUILE_DEBUTS: u16 = 324;
    pub const TUILE_TAILLES: u16 = 325;
    pub const FORMAT: u16 = 339;
    pub const ECHELLE: u16 = 33550;
    pub const ANCRAGE: u16 = 33922;
    pub const GEO_REELS: u16 = 34736;
}

/// Une entrée du répertoire TIFF, ses valeurs déjà lues.
enum Valeurs {
    Entiers(Vec<u64>),
    Reels(Vec<f64>),
    Autre,
}

fn u16_a(o: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(o.get(i..i + 2)?.try_into().ok()?))
}
fn u32_a(o: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(o.get(i..i + 4)?.try_into().ok()?))
}
fn f64_a(o: &[u8], i: usize) -> Option<f64> {
    Some(f64::from_le_bytes(o.get(i..i + 8)?.try_into().ok()?))
}

/// Lit le premier répertoire d'un TIFF petit-boutiste.
fn repertoire(o: &[u8]) -> Option<HashMap<u16, Valeurs>> {
    if o.get(0..4)? != b"II*\0" {
        return None;
    }
    let debut = u32_a(o, 4)? as usize;
    let n = u16_a(o, debut)? as usize;
    let mut entrees = HashMap::new();
    for k in 0..n {
        let e = debut + 2 + 12 * k;
        let (etiquette, genre, nombre) = (u16_a(o, e)?, u16_a(o, e + 2)?, u32_a(o, e + 4)? as usize);
        let largeur = match genre {
            3 => 2,
            4 => 4,
            12 => 8,
            _ => {
                entrees.insert(etiquette, Valeurs::Autre);
                continue;
            }
        };
        let place = if largeur * nombre <= 4 { e + 8 } else { u32_a(o, e + 8)? as usize };
        let valeurs = match genre {
            3 => Valeurs::Entiers((0..nombre).map(|i| u16_a(o, place + 2 * i).map(u64::from)).collect::<Option<_>>()?),
            4 => Valeurs::Entiers((0..nombre).map(|i| u32_a(o, place + 4 * i).map(u64::from)).collect::<Option<_>>()?),
            _ => Valeurs::Reels((0..nombre).map(|i| f64_a(o, place + 8 * i)).collect::<Option<_>>()?),
        };
        entrees.insert(etiquette, valeurs);
    }
    Some(entrees)
}

impl Image {
    /// Lit l'en-tête d'une image et vérifie qu'elle est bien ce qu'on croit :
    /// la grille OPERA, des flottants, des tuiles en deflate. Une image qui
    /// aurait changé de forme est refusée plutôt que lue de travers.
    pub fn lire(heure: i64, octets: Vec<u8>) -> Result<Image, &'static str> {
        let r = repertoire(&octets).ok_or("en-tête TIFF illisible")?;
        let entier = |t: u16| match r.get(&t) {
            Some(Valeurs::Entiers(v)) => v.first().copied(),
            _ => None,
        };
        let entiers = |t: u16| match r.get(&t) {
            Some(Valeurs::Entiers(v)) => Some(v.clone()),
            _ => None,
        };
        let reels = |t: u16| match r.get(&t) {
            Some(Valeurs::Reels(v)) => Some(v.clone()),
            _ => None,
        };
        use etiquette::*;
        if entier(COMPRESSION) != Some(8) {
            return Err("compression inattendue");
        }
        if entier(BITS) != Some(32) || entier(FORMAT) != Some(3) {
            return Err("échantillons inattendus");
        }
        if entier(PREDICTEUR).unwrap_or(1) != 1 || entier(PLANAIRE).unwrap_or(1) != 1 {
            return Err("disposition inattendue");
        }
        let geo = reels(GEO_REELS).ok_or("projection absente")?;
        if geo.len() < 4 || geo[..4] != [LAT_CENTRE, LON_CENTRE, FAUX_EST, FAUX_NORD] {
            return Err("projection inattendue");
        }
        let echelle = reels(ECHELLE).ok_or("échelle absente")?;
        let ancrage = reels(ANCRAGE).ok_or("ancrage absent")?;
        if echelle.len() < 2 || ancrage.len() < 5 || (echelle[0] - echelle[1]).abs() > 1e-6 {
            return Err("géoréférencement inattendu");
        }
        let largeur = entier(LARGEUR).ok_or("largeur")? as usize;
        let hauteur = entier(HAUTEUR).ok_or("hauteur")? as usize;
        let tuile_l = entier(TUILE_L).ok_or("tuiles")? as usize;
        let tuile_h = entier(TUILE_H).ok_or("tuiles")? as usize;
        let debuts = entiers(TUILE_DEBUTS).ok_or("tuiles")?;
        let tailles = entiers(TUILE_TAILLES).ok_or("tuiles")?;
        let tuiles_x = largeur.div_ceil(tuile_l);
        if tuile_l == 0 || tuile_h == 0 || debuts.len() != tailles.len() || debuts.len() != tuiles_x * hauteur.div_ceil(tuile_h)
        {
            return Err("tuiles incohérentes");
        }
        let morceaux: Vec<(usize, usize)> =
            debuts.iter().zip(&tailles).map(|(d, t)| (*d as usize, *t as usize)).collect();
        if morceaux.iter().any(|(d, t)| d + t > octets.len()) {
            return Err("fichier tronqué");
        }
        Ok(Image {
            heure,
            grille: Grille {
                largeur,
                hauteur,
                // Le coin haut gauche du premier pixel.
                x0: ancrage[3],
                y0: ancrage[4],
                pas: echelle[0],
            },
            tuile_l,
            tuile_h,
            tuiles_x,
            echantillons: entier(ECHANTILLONS).unwrap_or(1).max(1) as usize,
            morceaux,
            octets,
            decodees: Mutex::new(HashMap::new()),
        })
    }

    /// La première bande d'une tuile, décompressée une fois pour toutes.
    fn tuile(&self, rang: usize) -> Option<Arc<Vec<f32>>> {
        if let Some(t) = self.decodees.lock().unwrap().get(&rang) {
            return Some(t.clone());
        }
        let (debut, taille) = *self.morceaux.get(rang)?;
        let mut brut = Vec::with_capacity(self.tuile_l * self.tuile_h * self.echantillons * 4);
        flate2::read::ZlibDecoder::new(&self.octets[debut..debut + taille]).read_to_end(&mut brut).ok()?;
        let pas = 4 * self.echantillons;
        if brut.len() < self.tuile_l * self.tuile_h * pas {
            return None;
        }
        let bande: Vec<f32> = brut
            .chunks_exact(pas)
            .map(|p| f32::from_le_bytes([p[0], p[1], p[2], p[3]]))
            .collect();
        let bande = Arc::new(bande);
        self.decodees.lock().unwrap().insert(rang, bande.clone());
        Some(bande)
    }

    /// Ce que dit un pixel ; hors de l'image ou illisible, on ne sait pas.
    pub fn echo(&self, c: i64, r: i64) -> Echo {
        if c < 0 || r < 0 || c as usize >= self.grille.largeur || r as usize >= self.grille.hauteur {
            return Echo::Inconnu;
        }
        let (c, r) = (c as usize, r as usize);
        let rang = (r / self.tuile_h) * self.tuiles_x + c / self.tuile_l;
        match self.tuile(rang) {
            Some(t) => echo(t[(r % self.tuile_h) * self.tuile_l + c % self.tuile_l]),
            None => Echo::Inconnu,
        }
    }

    /// Oublie les tuiles décompressées : une image qui n'est plus la plus
    /// récente n'a plus à tenir de place.
    fn alleger(&self) {
        self.decodees.lock().unwrap().clear();
    }
}

/// Les images en mémoire, et de quoi en charger de nouvelles.
pub struct Radar {
    charge: Charge,
    horloge: Clock,
    images: tokio::sync::Mutex<BTreeMap<i64, Arc<Image>>>,
    absentes: Mutex<HashMap<i64, i64>>,
}

impl Radar {
    pub fn new(charge: Charge, horloge: Clock) -> Radar {
        Radar {
            charge,
            horloge,
            images: tokio::sync::Mutex::new(BTreeMap::new()),
            absentes: Mutex::new(HashMap::new()),
        }
    }

    /// Un radar qui ne charge rien : pour un relais sans réseau.
    pub fn eteint(horloge: Clock) -> Radar {
        Radar::new(Arc::new(|_| Box::pin(async { Err(UpstreamError { status: None }) })), horloge)
    }

    /// L'image d'une heure : en mémoire, ou téléchargée. `None` si elle n'est
    /// pas (encore) publiée, ou illisible.
    async fn image(&self, heure: i64) -> Option<Arc<Image>> {
        let maintenant = (self.horloge)();
        if let Some(depuis) = self.absentes.lock().unwrap().get(&heure) {
            if maintenant - depuis < ABSENCE_MS {
                return None;
            }
        }
        let mut images = self.images.lock().await;
        if let Some(image) = images.get(&heure) {
            return Some(image.clone());
        }
        let lue = match (self.charge)(adresse(heure)).await {
            Ok(octets) => Image::lire(heure, octets).map_err(|motif| eprintln!("radar illisible ({motif})")).ok(),
            Err(_) => None,
        };
        let Some(image) = lue.map(Arc::new) else {
            self.absentes.lock().unwrap().insert(heure, maintenant);
            return None;
        };
        images.insert(heure, image.clone());
        while images.len() > IMAGES_GARDEES {
            images.pop_first();
        }
        // Seules les deux plus récentes servent : les autres rendent leurs
        // tuiles.
        for vieille in images.values().rev().skip(2) {
            vieille.alleger();
        }
        Some(image)
    }

    /// La plus récente des images publiées. Une image paraît sept à dix
    /// minutes après son heure : on part de cinq minutes en arrière, et on
    /// recule jusqu'à vingt.
    async fn recente(&self) -> Option<Arc<Image>> {
        let maintenant = (self.horloge)();
        let derniere = (maintenant - PAS_IMAGE_MS).div_euclid(PAS_IMAGE_MS) * PAS_IMAGE_MS;
        for k in 0..4 {
            if let Some(image) = self.image(derniere - k * PAS_IMAGE_MS).await {
                return Some(image);
            }
        }
        None
    }

    /// Ce que le radar dit pour un point. `Err` si aucune image n'est
    /// disponible ; une ville hors de portée a une prévision vide.
    pub async fn prevision(&self, latitude: f64, longitude: f64) -> Result<Prevision, UpstreamError> {
        let recente = self.recente().await.ok_or(UpstreamError { status: None })?;
        let Some(centre) = recente.grille.pixel(latitude, longitude) else {
            return Ok(Prevision { image: recente.heure, maintenant: None, quarts: Vec::new(), deplacement: None });
        };
        let apres = Champ::decouper(centre, DEMI_FENETRE_KM, |c, r| recente.echo(c, r));
        let vitesse = match self.image(recente.heure - ECART_MOUVEMENT_MS).await {
            Some(avant) => {
                let avant = Champ::decouper(centre, DEMI_FENETRE_KM, |c, r| avant.echo(c, r));
                deplacement(&avant, &apres, ECART_MOUVEMENT_MS as f64 / 60_000.0)
            }
            None => None,
        };
        Ok(prevoir(recente.heure, &apres, vitesse))
    }
}

/// Le `Charge` qui parle vraiment au réseau.
pub fn http_charge(client: reqwest::Client) -> Charge {
    Arc::new(move |adresse: String| {
        let client = client.clone();
        Box::pin(async move {
            let reponse = client
                .get(&adresse)
                .send()
                .await
                .map_err(|_| UpstreamError { status: None })?;
            if !reponse.status().is_success() {
                return Err(UpstreamError { status: Some(reponse.status().as_u16()) });
            }
            reponse.bytes().await.map(|b| b.to_vec()).map_err(|_| UpstreamError { status: None })
        })
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Write;

    /// Fabrique une petite image au format OPERA : `l × h` pixels en tuiles de
    /// 16, deux bandes, la réflectivité donnée par `dbz(c, r)`. Le coin haut
    /// gauche est placé pour que la grille soit celle de la vraie mosaïque.
    pub(crate) fn image_opera(l: usize, h: usize, x0: f64, y0: f64, dbz: impl Fn(usize, usize) -> f32) -> Vec<u8> {
        let t = 16;
        let (tx, ty) = (l.div_ceil(t), h.div_ceil(t));
        let mut tuiles = Vec::new();
        for j in 0..ty {
            for i in 0..tx {
                let mut brut = Vec::new();
                for y in 0..t {
                    for x in 0..t {
                        let (c, r) = (i * t + x, j * t + y);
                        let v = if c < l && r < h { dbz(c, r) } else { -9_999_000.0 };
                        brut.extend_from_slice(&v.to_le_bytes());
                        brut.extend_from_slice(&0f32.to_le_bytes());
                    }
                }
                let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
                z.write_all(&brut).unwrap();
                tuiles.push(z.finish().unwrap());
            }
        }

        // En-tête, répertoire, données annexes, puis les tuiles.
        let n = tuiles.len();
        let entrees: u16 = 14;
        let dir = 8;
        let annexe = dir + 2 + 12 * entrees as usize + 4;
        let mut o = b"II*\0".to_vec();
        o.extend_from_slice(&(dir as u32).to_le_bytes());
        let mut ann: Vec<u8> = Vec::new();
        let mut champs: Vec<(u16, u16, u32, u32)> = Vec::new();
        let mut loin = |octets: &[u8]| -> u32 {
            let p = (annexe + ann.len()) as u32;
            ann.extend_from_slice(octets);
            p
        };
        let reels = |v: &[f64]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let deux = |a: u16, b: u16| u32::from(a) | (u32::from(b) << 16);
        champs.push((256, 3, 1, l as u32));
        champs.push((257, 3, 1, h as u32));
        champs.push((258, 3, 2, deux(32, 32)));
        champs.push((259, 3, 1, 8));
        champs.push((277, 3, 1, 2));
        champs.push((284, 3, 1, 1));
        champs.push((322, 3, 1, t as u32));
        champs.push((323, 3, 1, t as u32));
        let debuts_p = loin(&vec![0u8; 4 * n]);
        champs.push((324, 4, n as u32, debuts_p));
        let tailles: Vec<u8> = tuiles.iter().flat_map(|z| (z.len() as u32).to_le_bytes()).collect();
        champs.push((325, 4, n as u32, loin(&tailles)));
        champs.push((339, 3, 2, deux(3, 3)));
        champs.push((33550, 12, 3, loin(&reels(&[1000.0, 1000.0, 0.0]))));
        champs.push((33922, 12, 6, loin(&reels(&[0.0, 0.0, 0.0, x0, y0, 0.0]))));
        champs.push((34736, 12, 4, loin(&reels(&[LAT_CENTRE, LON_CENTRE, FAUX_EST, FAUX_NORD]))));
        o.extend_from_slice(&entrees.to_le_bytes());
        for (e, g, nb, v) in &champs {
            o.extend_from_slice(&e.to_le_bytes());
            o.extend_from_slice(&g.to_le_bytes());
            o.extend_from_slice(&nb.to_le_bytes());
            o.extend_from_slice(&v.to_le_bytes());
        }
        o.extend_from_slice(&0u32.to_le_bytes());
        let debut_tuiles = annexe + ann.len();
        let mut p = debut_tuiles;
        let debuts: Vec<u8> = tuiles
            .iter()
            .flat_map(|z| {
                let d = p as u32;
                p += z.len();
                d.to_le_bytes()
            })
            .collect();
        let rel = debuts_p as usize - annexe;
        ann[rel..rel + 4 * n].copy_from_slice(&debuts);
        o.extend_from_slice(&ann);
        for z in &tuiles {
            o.extend_from_slice(z);
        }
        o
    }

    /// Une petite fenêtre de la vraie grille, autour de Brest (883, 2726) : le
    /// coin haut gauche de l'image d'essai est le pixel (700, 2550) de la
    /// mosaïque.
    pub(crate) const COIN: (usize, usize) = (700, 2550);

    pub(crate) fn coin_xy() -> (f64, f64) {
        (-500.0 + COIN.0 as f64 * 1000.0, 500.0 - COIN.1 as f64 * 1000.0)
    }

    #[test]
    fn l_adresse_d_une_image() {
        assert_eq!(
            adresse(1_791_313_200_000),
            "https://s3.waw3-1.cloudferro.com/openradar-24h/2026/10/06/OPERA/COMP/OPERA@20261006T1900@0@DBZH.tiff"
        );
    }

    #[test]
    fn une_image_se_lit_pixel_par_pixel() {
        let (x0, y0) = coin_xy();
        let octets = image_opera(400, 360, x0, y0, |c, r| if c == 5 && r == 7 { 30.0 } else { f32::NAN });
        let image = Image::lire(0, octets).unwrap();
        assert_eq!(image.grille.largeur, 400);
        assert_eq!(image.echo(5, 7), echo(30.0));
        assert_eq!(image.echo(6, 7), Echo::Debit(0.0));
        assert_eq!(image.echo(400, 0), Echo::Inconnu);
        assert_eq!(image.echo(-1, 0), Echo::Inconnu);
        // Brest tombe là où la vraie mosaïque la met.
        assert_eq!(image.grille.pixel(48.39, -4.49), Some((883 - COIN.0, 2726 - COIN.1)));
    }

    #[test]
    fn une_image_d_une_autre_forme_est_refusee() {
        assert!(Image::lire(0, b"pas un tiff".to_vec()).is_err());
        let (x0, y0) = coin_xy();
        let mut octets = image_opera(16, 16, x0, y0, |_, _| 0.0);
        // Le centre de projection a bougé : on ne lit pas de travers.
        let p = octets.windows(8).position(|w| w == LAT_CENTRE.to_le_bytes()).unwrap();
        octets[p..p + 8].copy_from_slice(&52.0f64.to_le_bytes());
        assert_eq!(Image::lire(0, octets).err(), Some("projection inattendue"));
    }

    #[tokio::test]
    async fn le_radar_suit_l_averse_qui_vient_sur_brest() {
        // Deux images à un quart d'heure d'écart : une averse ronde qui
        // avance de 12 km vers l'est, et se trouve à 36 km à l'ouest de Brest
        // à 19 h 00.
        let (bx, by) = (883 - COIN.0, 2726 - COIN.1);
        let (x0, y0) = coin_xy();
        let image = |decalage: f64| {
            image_opera(400, 360, x0, y0, move |c, r| {
                let d2 = (c as f64 - (bx as f64 - decalage)).powi(2) + (r as f64 - by as f64).powi(2);
                if d2 <= 100.0 { 35.0 } else { f32::NAN }
            })
        };
        let dix_neuf: i64 = 1_791_313_200_000;
        let (recente, avant) = (image(36.0), image(48.0));
        let charge: Charge = Arc::new(move |adresse: String| {
            let reponse = if adresse.ends_with("T1900@0@DBZH.tiff") {
                Ok(recente.clone())
            } else if adresse.ends_with("T1845@0@DBZH.tiff") {
                Ok(avant.clone())
            } else {
                Err(UpstreamError { status: Some(404) })
            };
            Box::pin(async move { reponse })
        });
        // Il est 19 h 08 : l'image de 19 h 05 n'est pas encore publiée.
        let radar = Radar::new(charge, Arc::new(move || dix_neuf + 8 * 60_000));
        let p = radar.prevision(48.39, -4.49).await.unwrap();
        assert_eq!(p.image, dix_neuf);
        assert_eq!(p.maintenant, Some(0.0));
        let (km_h, cap) = p.deplacement.unwrap();
        assert!((km_h - 48.0).abs() < 4.1, "{km_h}");
        assert!((cap - 90.0).abs() < 10.0, "vers l'est : {cap}");
        // 36 km à 48 km/h : sur Brest vers 19 h 45.
        let mouilles: Vec<i64> = p.quarts.iter().filter(|(_, d)| *d > 1.0).map(|(t, _)| (t - dix_neuf) / 60_000).collect();
        assert!(mouilles.contains(&45), "{mouilles:?}");
        assert!(!mouilles.contains(&0));

        // Hors de la mosaïque d'essai : une prévision vide, pas une erreur.
        let p = radar.prevision(40.0, -30.0).await.unwrap();
        assert_eq!((p.maintenant, p.quarts.len()), (None, 0));
    }

    #[tokio::test]
    async fn sans_image_publiee_le_radar_se_tait() {
        let radar = Radar::eteint(Arc::new(|| 1_791_313_200_000));
        assert!(radar.prevision(48.39, -4.49).await.is_err());
    }
}
