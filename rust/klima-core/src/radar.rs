//! Le radar : ce qui tombe au-dessus de la ville, et où ça va.
//!
//! La source est la mosaïque européenne OPERA d'EUMETNET — les radars de
//! Météo-France et de ses voisins, assemblés toutes les cinq minutes, au
//! kilomètre, en accès libre (licence CC BY 4.0). Le relais la télécharge et
//! la découpe (`klima-relay/src/radar.rs`) ; ce module tient tout ce qui est
//! pur : la projection de la grille, la conversion des réflectivités en
//! millimètres par heure, le déplacement des averses entre deux images et
//! leur extrapolation sur les deux heures du guetteur.
//!
//! Les modèles ratent les cellules d'orage ; le radar, lui, les voit. Pour
//! l'heure qui vient, prolonger ce qu'il voit dans la direction où ça va bat
//! n'importe quel modèle — c'est ce que font les services météo sous le nom
//! de « prévision immédiate ». Au-delà, la prévision reprend la main, et le
//! guetteur fond l'un dans l'autre (`veille::radariser`).
//!
//! Les horodatages sont en millisecondes UTC : le relais ne connaît pas le
//! fuseau des villes, les interfaces le posent.

/// Les seuils, au même endroit que leur raison.
pub mod seuils {
    /// En deçà, l'écho ne mouille pas (dBZ) : environ 0,1 mm/h.
    pub const DBZ_MIN: f64 = 7.0;
    /// Une image toutes les cinq minutes (ms).
    pub const PAS_IMAGE_MS: i64 = 5 * 60_000;
    /// L'écart entre les deux images qui donnent le déplacement (ms).
    pub const ECART_MOUVEMENT_MS: i64 = 15 * 60_000;
    /// La demi-largeur de la fenêtre découpée autour de la ville (km) : assez
    /// pour voir venir une averse qui roule à 100 km/h pendant deux heures.
    pub const DEMI_FENETRE_KM: usize = 160;
    /// Le déplacement le plus grand cherché entre deux images, en kilomètres
    /// par quart d'heure : 40 km, soit 160 km/h.
    pub const DEPLACEMENT_MAX_KM: i64 = 40;
    /// Il faut assez de pluie dans la fenêtre pour mesurer un déplacement.
    pub const PIXELS_MOUILLES_MIN: usize = 30;
    /// Autour de la ville, on lit un carré de 3 km de côté.
    pub const VOISINAGE_KM: i64 = 1;
    /// Un quart d'heure (ms), comme le guetteur.
    pub const QUART_MS: i64 = 900_000;
    /// Huit quarts : les deux heures du guetteur.
    pub const HORIZON_QUARTS: usize = 8;
    /// À partir de ce débit au-dessus de la ville, il tombe quelque chose
    /// (mm/h) : le seuil d'un quart mouillé du guetteur, ramené à l'heure.
    pub const DEBIT_MOUILLE: f64 = crate::veille::seuils::PLUIE_QUART_MM * 4.0;
}

use seuils::*;

/// La grille de la mosaïque : une projection azimutale équivalente de Lambert
/// centrée sur 55° N, 10° E, sur l'ellipsoïde WGS 84, au kilomètre. Ces
/// constantes sont celles que porte le fichier (`GeoKeyDirectory`) ; le relais
/// vérifie qu'elles n'ont pas changé avant de lire quoi que ce soit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grille {
    pub largeur: usize,
    pub hauteur: usize,
    /// Coin haut gauche, en mètres projetés.
    pub x0: f64,
    pub y0: f64,
    /// Taille d'un pixel (m).
    pub pas: f64,
}

/// Les paramètres de la projection, tels que le fichier les déclare.
pub const LAT_CENTRE: f64 = 55.0;
pub const LON_CENTRE: f64 = 10.0;
pub const FAUX_EST: f64 = 1_950_000.0;
pub const FAUX_NORD: f64 = -2_100_000.0;

const DEMI_GRAND_AXE: f64 = 6_378_137.0;
const APLATISSEMENT: f64 = 1.0 / 298.257_223_563;

/// La projection d'un point (EPSG 9820, forme ellipsoïdale) : mètres est et
/// nord dans le repère de la mosaïque.
pub fn projeter(latitude: f64, longitude: f64) -> (f64, f64) {
    let e2 = 2.0 * APLATISSEMENT - APLATISSEMENT * APLATISSEMENT;
    let e = e2.sqrt();
    let q = |phi: f64| {
        let s = phi.sin();
        (1.0 - e2) * (s / (1.0 - e2 * s * s) - (1.0 / (2.0 * e)) * ((1.0 - e * s) / (1.0 + e * s)).ln())
    };
    let qp = q(std::f64::consts::FRAC_PI_2);
    let rq = DEMI_GRAND_AXE * (qp / 2.0).sqrt();
    let phi0 = LAT_CENTRE.to_radians();
    let beta0 = (q(phi0) / qp).asin();
    let d = DEMI_GRAND_AXE * (phi0.cos() / (1.0 - e2 * phi0.sin().powi(2)).sqrt()) / (rq * beta0.cos());

    let beta = (q(latitude.to_radians()) / qp).asin();
    let dl = (longitude - LON_CENTRE).to_radians();
    let b = rq * (2.0 / (1.0 + beta0.sin() * beta.sin() + beta0.cos() * beta.cos() * dl.cos())).sqrt();
    let x = FAUX_EST + b * d * beta.cos() * dl.sin();
    let y = FAUX_NORD + (b / d) * (beta0.cos() * beta.sin() - beta0.sin() * beta.cos() * dl.cos());
    (x, y)
}

impl Grille {
    /// Le pixel (colonne, rangée) d'un point, `None` hors de la mosaïque.
    pub fn pixel(&self, latitude: f64, longitude: f64) -> Option<(usize, usize)> {
        let (x, y) = projeter(latitude, longitude);
        let c = ((x - self.x0) / self.pas).floor();
        let r = ((self.y0 - y) / self.pas).floor();
        let dedans = c >= 0.0 && r >= 0.0 && (c as usize) < self.largeur && (r as usize) < self.hauteur;
        dedans.then_some((c as usize, r as usize))
    }
}

/// Ce qu'un pixel dit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Echo {
    /// Hors de portée des radars : on ne sait pas.
    Inconnu,
    /// Un débit (mm/h), zéro compris.
    Debit(f64),
}

/// La réflectivité d'un pixel en débit, par la relation de Marshall et Palmer
/// (Z = 200 R^1,6). `NaN` — « rien détecté » — est un ciel sec ; une valeur
/// de remplissage (très négative) est un pixel que personne ne voit.
pub fn echo(dbz: f32) -> Echo {
    if dbz.is_nan() {
        return Echo::Debit(0.0);
    }
    if dbz < -9_000.0 {
        return Echo::Inconnu;
    }
    let dbz = f64::from(dbz);
    if dbz < DBZ_MIN {
        return Echo::Debit(0.0);
    }
    Echo::Debit((10f64.powf(dbz / 10.0) / 200.0).powf(1.0 / 1.6))
}

/// Le code météo d'un débit (mm/h) : pluie faible, modérée ou forte, aux
/// seuils du guetteur.
pub fn code_du_debit(mm_h: f64) -> u16 {
    if mm_h >= crate::veille::seuils::FORTE_MM_H {
        65
    } else if mm_h >= crate::veille::seuils::MODEREE_MM_H {
        63
    } else {
        61
    }
}

/// Ce que le radar voit tomber sur la ville à l'heure de l'image, en code de
/// l'OMM ; `None` s'il n'y voit rien, ou ne la voit pas.
pub fn tombe(maintenant: Option<f64>) -> Option<u16> {
    maintenant.filter(|d| *d >= DEBIT_MOUILLE).map(code_du_debit)
}

/// La direction où va la pluie, en clé de catalogue : `dir.n`, `dir.ne`…
pub fn direction(cap: f64) -> &'static str {
    const ROSE: [&str; 8] = ["dir.n", "dir.ne", "dir.e", "dir.se", "dir.s", "dir.so", "dir.o", "dir.no"];
    ROSE[((cap.rem_euclid(360.0) + 22.5) / 45.0) as usize % 8]
}

/// Une fenêtre carrée découpée autour de la ville : des débits, `None` là où
/// aucun radar ne voit.
#[derive(Debug, Clone, PartialEq)]
pub struct Champ {
    /// Demi-côté, en pixels : la ville est au centre.
    pub demi: usize,
    pub valeurs: Vec<Option<f64>>,
}

impl Champ {
    /// Découpe une fenêtre autour de `centre` en lisant chaque pixel par
    /// `lire` — le relais sait où sont les pixels, ce module ne le sait pas.
    pub fn decouper(centre: (usize, usize), demi: usize, mut lire: impl FnMut(i64, i64) -> Echo) -> Champ {
        let cote = 2 * demi + 1;
        let mut valeurs = Vec::with_capacity(cote * cote);
        for dy in 0..cote {
            for dx in 0..cote {
                let c = centre.0 as i64 + dx as i64 - demi as i64;
                let r = centre.1 as i64 + dy as i64 - demi as i64;
                valeurs.push(match lire(c, r) {
                    Echo::Debit(d) => Some(d),
                    Echo::Inconnu => None,
                });
            }
        }
        Champ { demi, valeurs }
    }

    fn cote(&self) -> i64 {
        2 * self.demi as i64 + 1
    }

    /// Le débit à (dx, dy) du centre, `None` hors de la fenêtre ou hors de
    /// portée.
    pub fn a(&self, dx: i64, dy: i64) -> Option<f64> {
        let d = self.demi as i64;
        let (c, r) = (dx + d, dy + d);
        if c < 0 || r < 0 || c >= self.cote() || r >= self.cote() {
            return None;
        }
        self.valeurs[(r * self.cote() + c) as usize]
    }

    /// Le débit moyen d'un petit carré autour de (dx, dy) ; `None` si aucun
    /// de ses pixels n'est vu.
    pub fn autour(&self, dx: f64, dy: f64) -> Option<f64> {
        let (cx, cy) = (dx.round() as i64, dy.round() as i64);
        let vus: Vec<f64> = (-VOISINAGE_KM..=VOISINAGE_KM)
            .flat_map(|j| (-VOISINAGE_KM..=VOISINAGE_KM).map(move |i| (i, j)))
            .filter_map(|(i, j)| self.a(cx + i, cy + j))
            .collect();
        (!vus.is_empty()).then(|| vus.iter().sum::<f64>() / vus.len() as f64)
    }
}

/// Le déplacement de la pluie entre deux images, en pixels (km) par minute :
/// le décalage qui superpose le mieux l'image d'avant à celle d'après.
///
/// La recherche se fait d'abord sur une grille quatre fois plus grossière,
/// puis s'affine autour du meilleur décalage. `None` s'il n'y a pas assez de
/// pluie pour que la mesure veuille dire quelque chose : on ne déplace pas un
/// ciel vide.
pub fn deplacement(avant: &Champ, apres: &Champ, ecart_min: f64) -> Option<(f64, f64)> {
    let mouilles = apres.valeurs.iter().filter(|v| v.is_some_and(|d| d > 0.0)).count();
    if mouilles < PIXELS_MOUILLES_MIN || ecart_min <= 0.0 {
        return None;
    }
    let portee = (DEPLACEMENT_MAX_KM as f64 * ecart_min / 15.0).ceil() as i64;
    let demi = apres.demi as i64 - portee;
    if demi <= 0 {
        return None;
    }
    // L'écart quadratique moyen entre l'image d'après et celle d'avant
    // décalée de (sx, sy), sur les pixels vus des deux côtés, en ne lisant
    // qu'un pixel sur `pas`.
    let ecart = |sx: i64, sy: i64, pas: i64| -> Option<f64> {
        let (mut somme, mut n, mut pluie) = (0.0, 0usize, 0usize);
        let mut y = -demi;
        while y <= demi {
            let mut x = -demi;
            while x <= demi {
                if let (Some(b), Some(a)) = (apres.a(x, y), avant.a(x - sx, y - sy)) {
                    // Les débits sont comparés en racine : une forte averse ne
                    // doit pas écraser tout le reste.
                    let d = b.sqrt() - a.sqrt();
                    somme += d * d;
                    n += 1;
                    if a > 0.0 || b > 0.0 {
                        pluie += 1;
                    }
                }
                x += pas;
            }
            y += pas;
        }
        (n > 0 && pluie > 0).then(|| somme / n as f64)
    };
    let meilleur = |candidats: Vec<(i64, i64)>, pas: i64| {
        candidats
            .into_iter()
            .filter_map(|(sx, sy)| ecart(sx, sy, pas).map(|e| (e, sx, sy)))
            // À égalité, le plus petit déplacement.
            .min_by(|a, b| a.0.total_cmp(&b.0).then((a.1.abs() + a.2.abs()).cmp(&(b.1.abs() + b.2.abs()))))
    };

    let grossier: Vec<(i64, i64)> = (-portee..=portee)
        .step_by(4)
        .flat_map(|sy| (-portee..=portee).step_by(4).map(move |sx| (sx, sy)))
        .collect();
    let (_, gx, gy) = meilleur(grossier, 4)?;
    let fin: Vec<(i64, i64)> =
        (gy - 3..=gy + 3).flat_map(|sy| (gx - 3..=gx + 3).map(move |sx| (sx, sy))).collect();
    let (_, sx, sy) = meilleur(fin, 2)?;
    Some((sx as f64 / ecart_min, sy as f64 / ecart_min))
}

/// Ce que le radar dit pour une ville.
#[derive(Debug, Clone, PartialEq)]
pub struct Prevision {
    /// L'heure de l'image (ms UTC).
    pub image: i64,
    /// Le débit au-dessus de la ville à l'heure de l'image (mm/h) ; `None` si
    /// aucun radar ne la voit.
    pub maintenant: Option<f64>,
    /// Les quarts d'heure qui viennent : début (ms UTC) et débit prévu
    /// (mm/h). Vides si aucun radar ne voit la ville.
    pub quarts: Vec<(i64, f64)>,
    /// Le déplacement de la pluie : vitesse (km/h) et direction où elle va
    /// (degrés, 0 = nord, 90 = est). `None` sans pluie à suivre.
    pub deplacement: Option<(f64, f64)>,
}

/// Prolonge ce que voit l'image la plus récente dans la direction où va la
/// pluie : le débit d'un quart est celui qui, à l'heure de l'image, se
/// trouvait là d'où le vent l'amène — lu au milieu du quart. Sans
/// déplacement mesurable, la pluie est supposée rester sur place.
pub fn prevoir(image: i64, champ: &Champ, vitesse: Option<(f64, f64)>) -> Prevision {
    let maintenant = champ.autour(0.0, 0.0);
    if maintenant.is_none() {
        return Prevision { image, maintenant: None, quarts: Vec::new(), deplacement: None };
    }
    let (vx, vy) = vitesse.unwrap_or((0.0, 0.0));
    let premier = image.div_euclid(QUART_MS) * QUART_MS;
    let quarts = (0..HORIZON_QUARTS as i64)
        .map(|k| {
            let debut = premier + k * QUART_MS;
            let minutes = ((debut + QUART_MS / 2 - image) as f64 / 60_000.0).max(0.0);
            // Ce qui arrive sur la ville venait d'en amont : centre − v·t.
            let debit = champ.autour(-vx * minutes, -vy * minutes).unwrap_or(0.0);
            (debut, debit)
        })
        .collect();
    let deplacement = vitesse.map(|(vx, vy)| {
        let km_h = (vx * vx + vy * vy).sqrt() * 60.0;
        // Les rangées descendent vers le sud : le nord est −y.
        let cap = vx.atan2(-vy).to_degrees().rem_euclid(360.0);
        (km_h, cap)
    });
    Prevision { image, maintenant, quarts, deplacement }
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    const OPERA: Grille = Grille { largeur: 3800, hauteur: 4400, x0: -500.0, y0: 500.0, pas: 1000.0 };

    #[test]
    fn la_projection_place_les_villes_ou_les_radars_les_voient() {
        // Valeurs relevées sur la mosaïque du 6 octobre 2026 : Brest sous la
        // pluie au pixel (883, 2726), Bordeaux au sec au (1114, 3167).
        assert_eq!(OPERA.pixel(48.39, -4.49), Some((883, 2726)));
        assert_eq!(OPERA.pixel(44.84, -0.58), Some((1114, 3167)));
        assert_eq!(OPERA.pixel(48.857, 2.352), Some((1389, 2753)));
        // Le centre de la projection tombe sur son faux est / faux nord.
        let (x, y) = projeter(LAT_CENTRE, LON_CENTRE);
        assert!((x - FAUX_EST).abs() < 1e-6 && (y - FAUX_NORD).abs() < 1e-6);
        assert_eq!(OPERA.pixel(-10.0, -60.0), None, "hors de la mosaïque");
    }

    #[test]
    fn ce_qui_tombe_et_ou_ca_va() {
        assert_eq!(tombe(Some(1.2)), Some(61));
        assert_eq!(tombe(Some(0.2)), None, "trop peu pour mouiller un quart");
        assert_eq!(tombe(None), None);
        assert_eq!(direction(0.0), "dir.n");
        assert_eq!(direction(350.0), "dir.n");
        assert_eq!(direction(75.0), "dir.e");
        assert_eq!(direction(225.0), "dir.so");
        assert_eq!(direction(-45.0), "dir.no");
    }

    #[test]
    fn les_reflectivites_deviennent_des_debits() {
        assert_eq!(echo(f32::NAN), Echo::Debit(0.0), "rien détecté : sec");
        assert_eq!(echo(-9_999_000.0), Echo::Inconnu, "hors de portée");
        assert_eq!(echo(5.0), Echo::Debit(0.0));
        let Echo::Debit(d) = echo(23.0) else { panic!() };
        assert!((d - 1.0).abs() < 0.1, "23 dBZ ≈ 1 mm/h : {d}");
        let Echo::Debit(d) = echo(40.0) else { panic!() };
        assert!((d - 11.5).abs() < 0.5, "40 dBZ ≈ 11,5 mm/h : {d}");
        assert_eq!(code_du_debit(0.8), 61);
        assert_eq!(code_du_debit(4.0), 63);
        assert_eq!(code_du_debit(12.0), 65);
    }

    /// Une averse ronde de rayon 8 km centrée en (cx, cy).
    fn averse(cx: f64, cy: f64) -> Champ {
        let demi = 60;
        Champ::decouper((demi, demi), demi, |c, r| {
            let (x, y) = (c as f64 - demi as f64, r as f64 - demi as f64);
            let d2 = (x - cx).powi(2) + (y - cy).powi(2);
            Echo::Debit(if d2 <= 64.0 { 6.0 } else { 0.0 })
        })
    }

    #[test]
    fn le_deplacement_se_mesure_entre_deux_images() {
        // L'averse a avancé de 12 km vers l'est et 6 km vers le sud en un
        // quart d'heure.
        let (vx, vy) = deplacement(&averse(-30.0, -10.0), &averse(-18.0, -4.0), 15.0).unwrap();
        assert!((vx * 15.0 - 12.0).abs() < 1.01, "{vx}");
        assert!((vy * 15.0 - 6.0).abs() < 1.01, "{vy}");
        // Un ciel vide ne se déplace pas.
        let sec = Champ::decouper((60, 60), 60, |_, _| Echo::Debit(0.0));
        assert_eq!(deplacement(&sec, &sec, 15.0), None);
    }

    #[test]
    fn la_pluie_qui_vient_arrive_a_son_heure() {
        // Une averse à 24 km à l'ouest, qui roule vers l'est à 48 km/h
        // (0,8 km/min) : sur la ville une demi-heure plus tard.
        let image = 1_791_313_200_000; // 19 h 00 UTC, pile sur un quart
        let champ = averse(-24.0, 0.0);
        let p = prevoir(image, &champ, Some((0.8, 0.0)));
        assert_eq!(p.maintenant, Some(0.0));
        assert_eq!(p.quarts.len(), 8);
        assert_eq!(p.quarts[0].0, image);
        assert_eq!(p.quarts[0].1, 0.0, "milieu du premier quart : encore à 18 km");
        assert!(p.quarts[2].1 > 5.0, "19 h 30 – 19 h 45 : dessus");
        assert_eq!(p.quarts[6].1, 0.0, "puis passée");
        let (km_h, cap) = p.deplacement.unwrap();
        assert!((km_h - 48.0).abs() < 1e-9);
        assert!((cap - 90.0).abs() < 1e-9, "vers l'est");

        // Sans déplacement mesuré, ce qui est là reste là.
        let p = prevoir(image + 7 * 60_000, &averse(0.0, 0.0), None);
        assert!(p.quarts.iter().all(|(_, d)| *d > 5.0));
        assert_eq!(p.deplacement, None);

        // Une ville que personne ne voit.
        let aveugle = Champ::decouper((5, 5), 5, |_, _| Echo::Inconnu);
        let p = prevoir(image, &aveugle, None);
        assert_eq!((p.maintenant, p.quarts.len()), (None, 0));
    }
}
