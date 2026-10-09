//! Les formes du film : ce que dessinent les grains, station par station.
//!
//! Chaque station est une liste de tracés dans un repère fixe — x de -1,6 à
//! 1,6, y de -1 à 1, y vers le haut — que l'on change en un nuage de points :
//! les contours d'abord, nets et lumineux ; l'intérieur des formes pleines,
//! plus pâle ; et une poussière éparse autour, pour que l'image respire.
//! Toutes les stations ont exactement le même nombre de grains : d'une
//! station à l'autre, chaque grain va d'une place à une autre.
//!
//! Rien ici ne touche au navigateur : la géométrie se teste comme le reste.

/// Les teintes des grains ; le moteur les traduit en couleurs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Teinte {
    Creme = 0,
    Pluie = 1,
    Soleil = 2,
    Feuille = 3,
}

/// Comment un grain bouge une fois en place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Genre {
    /// Il tient sa place, à peine agité.
    Fixe = 0,
    /// Il tombe, en boucle, sous le nuage : la pluie.
    Chute = 1,
    /// Il tourne autour du centre : le balayage du radar.
    Rotation = 2,
    /// Il file de sa place vers le centre : le vote des sources.
    Flux = 3,
}

/// Un tracé : une ligne brisée, fermée ou non, et ce qu'on en fait.
#[derive(Clone, Debug)]
pub struct Trace {
    pub points: Vec<(f32, f32)>,
    pub ferme: bool,
    /// Remplir l'intérieur (seulement pour un tracé fermé).
    pub plein: bool,
    pub teinte: Teinte,
    pub genre: Genre,
    /// Le poids du tracé dans le partage des grains, à longueur égale.
    pub poids: f32,
}

impl Trace {
    fn nouveau(points: Vec<(f32, f32)>, ferme: bool) -> Self {
        Trace { points, ferme, plein: false, teinte: Teinte::Creme, genre: Genre::Fixe, poids: 1.0 }
    }
    pub fn ligne(points: Vec<(f32, f32)>) -> Self {
        Self::nouveau(points, false)
    }
    pub fn boucle(points: Vec<(f32, f32)>) -> Self {
        Self::nouveau(points, true)
    }
    pub fn plein(mut self) -> Self {
        self.plein = self.ferme;
        self
    }
    pub fn teinte(mut self, teinte: Teinte) -> Self {
        self.teinte = teinte;
        self
    }
    pub fn genre(mut self, genre: Genre) -> Self {
        self.genre = genre;
        self
    }
    /// Le même tracé, réduit de `facteur` puis déplacé de `decalage`.
    pub fn transformer(mut self, facteur: f32, decalage: (f32, f32)) -> Self {
        for p in &mut self.points {
            *p = (p.0 * facteur + decalage.0, p.1 * facteur + decalage.1);
        }
        self
    }
    pub fn poids(mut self, poids: f32) -> Self {
        self.poids = poids;
        self
    }

    /// Les segments du tracé, celui qui le referme compris.
    fn segments(&self) -> impl Iterator<Item = ((f32, f32), (f32, f32))> + '_ {
        let n = self.points.len();
        let fin = if self.ferme { n } else { n.saturating_sub(1) };
        (0..fin).map(move |i| (self.points[i], self.points[(i + 1) % n]))
    }

    pub fn longueur(&self) -> f32 {
        self.segments().map(|(a, b)| distance(a, b)).sum()
    }

    /// L'aire d'un tracé fermé (formule du lacet).
    pub fn aire(&self) -> f32 {
        if !self.ferme || self.points.len() < 3 {
            return 0.0;
        }
        let n = self.points.len();
        let double: f32 = (0..n)
            .map(|i| {
                let (a, b) = (self.points[i], self.points[(i + 1) % n]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum();
        double.abs() / 2.0
    }

    /// Le point à la distance `d` le long du tracé.
    fn au_long(&self, mut d: f32) -> (f32, f32) {
        for (a, b) in self.segments() {
            let l = distance(a, b);
            if d <= l && l > 0.0 {
                let t = d / l;
                return (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            }
            d -= l;
        }
        *self.points.last().unwrap_or(&(0.0, 0.0))
    }

    /// Vrai si `p` est à l'intérieur du tracé fermé (règle pair-impair).
    pub fn contient(&self, p: (f32, f32)) -> bool {
        let n = self.points.len();
        let mut dedans = false;
        let mut j = n.wrapping_sub(1);
        for i in 0..n {
            let (a, b) = (self.points[i], self.points[j]);
            if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
                dedans = !dedans;
            }
            j = i;
        }
        dedans
    }

    fn boite(&self) -> (f32, f32, f32, f32) {
        self.points.iter().fold(
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            |(x0, y0, x1, y1), &(x, y)| (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        )
    }
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

// MARK: - Les briques

/// Un arc de cercle, de l'angle `a0` à `a1` (radians, sens trigonométrique).
pub fn arc(c: (f32, f32), r: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    let pas = (((a1 - a0).abs() * r * 40.0).ceil() as usize).max(8);
    (0..=pas)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / pas as f32;
            (c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

pub fn cercle(c: (f32, f32), r: f32) -> Vec<(f32, f32)> {
    let mut p = arc(c, r, 0.0, std::f32::consts::TAU);
    p.pop();
    p
}

/// Un rectangle aux coins arrondis, de `(x0, y0)` à `(x1, y1)`.
pub fn rectangle(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Vec<(f32, f32)> {
    use std::f32::consts::{FRAC_PI_2, PI};
    let r = r.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
    if r <= 0.0 {
        return vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
    }
    let mut p = Vec::new();
    p.extend(arc((x1 - r, y0 + r), r, -FRAC_PI_2, 0.0));
    p.extend(arc((x1 - r, y1 - r), r, 0.0, FRAC_PI_2));
    p.extend(arc((x0 + r, y1 - r), r, FRAC_PI_2, PI));
    p.extend(arc((x0 + r, y0 + r), r, PI, 1.5 * PI));
    p
}

/// Une courbe de Bézier cubique, sans son premier point.
pub fn bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> Vec<(f32, f32)> {
    (1..=24)
        .map(|i| {
            let t = i as f32 / 24.0;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            (
                a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
                a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
            )
        })
        .collect()
}

// MARK: - Les stations

/// Les stations du film, dans l'ordre ; chacune a sa ligne de texte, sous la
/// clé `film.<nom>` des catalogues.
pub const STATIONS: [&str; 9] =
    ["signe", "nuage", "heures", "radar", "sources", "parapluie", "soleil", "appareils", "fin"];

pub fn station(nom: &str) -> Vec<Trace> {
    match nom {
        "signe" => signe(),
        // Le signe de la fin, plus petit et plus haut : la phrase a un
        // bouton sous elle, et prend plus de place.
        "fin" => signe().into_iter().map(|t| t.transformer(0.74, (0.0, 0.2))).collect(),
        "nuage" => nuage(),
        "heures" => heures(),
        "radar" => radar(),
        "sources" => sources(),
        "parapluie" => parapluie(),
        "soleil" => soleil(),
        "appareils" => appareils(),
        _ => Vec::new(),
    }
}

/// Le signe de Klima, depuis le gabarit de 1024 : la hampe, les deux lames,
/// la goutte, et le carreau en contour.
fn signe() -> Vec<Trace> {
    // Du repère du gabarit (y vers le bas) à celui du film.
    let k = |x: f32, y: f32| ((x - 512.0) / 540.0, (512.0 - y) / 540.0);
    let lame = |c1: (f32, f32), c2: (f32, f32), fin: (f32, f32), d1: (f32, f32), d2: (f32, f32), retour: (f32, f32)| {
        let mut p = vec![k(418.0, 556.0)];
        p.extend(bezier(k(418.0, 556.0), k(c1.0, c1.1), k(c2.0, c2.1), k(fin.0, fin.1)));
        p.extend(bezier(k(fin.0, fin.1), k(d1.0, d1.1), k(d2.0, d2.1), k(retour.0, retour.1)));
        p
    };
    let (hx0, hy1) = k(322.0, 276.0);
    let (hx1, hy0) = k(418.0, 836.0);
    let (cx0, cy1) = k(0.0, 0.0);
    let (cx1, cy0) = k(1024.0, 1024.0);
    vec![
        Trace::boucle(rectangle(cx0, cy0, cx1, cy1, 229.0 / 540.0)).poids(0.55),
        Trace::boucle(rectangle(hx0, hy0, hx1, hy1, 48.0 / 540.0)).plein(),
        Trace::boucle(lame((536.0, 520.0), (632.0, 432.0), (704.0, 312.0), (726.0, 460.0), (634.0, 580.0), (498.0, 628.0))).plein(),
        Trace::boucle(lame((536.0, 592.0), (632.0, 680.0), (704.0, 800.0), (726.0, 652.0), (634.0, 532.0), (498.0, 484.0)))
            .plein()
            .teinte(Teinte::Feuille),
        Trace::boucle(cercle(k(370.0, 240.0), 52.0 / 540.0)).plein().teinte(Teinte::Pluie).poids(1.6),
    ]
}

/// Un nuage, et la pluie qui en tombe.
fn nuage() -> Vec<Trace> {
    use std::f32::consts::PI;
    // Trois bosses sur un fond plat.
    let mut contour = arc((-0.62, 0.3), 0.36, PI * 1.5, PI * 0.35);
    contour.extend(arc((0.02, 0.42), 0.5, PI * 0.9, PI * 0.1));
    contour.extend(arc((0.66, 0.3), 0.36, PI * 0.65, -PI * 0.5));
    let mut traces = vec![Trace::boucle(contour).plein()];
    // Les filets de pluie : des colonnes de grains qui tombent sous le nuage.
    for i in 0..13 {
        let x = -0.8 + i as f32 * (1.6 / 12.0);
        traces.push(
            Trace::ligne(vec![(x, -0.02), (x - 0.12, -0.98)])
                .teinte(Teinte::Pluie)
                .genre(Genre::Chute)
                .poids(1.4),
        );
    }
    traces
}

/// Douze heures en barres : celles qui mouillent sont pleines, et bleues.
fn heures() -> Vec<Trace> {
    const PLUIE: [f32; 12] = [0.08, 0.1, 0.14, 0.2, 0.42, 0.78, 0.95, 0.88, 0.5, 0.26, 0.14, 0.1];
    let mut traces = vec![Trace::ligne(vec![(-1.45, -0.72), (1.45, -0.72)]).poids(0.8)];
    for (i, &p) in PLUIE.iter().enumerate() {
        let x = -1.35 + i as f32 * 0.245;
        let haut = -0.72 + 1.5 * p;
        let mouille = p >= 0.4;
        let mut barre = Trace::boucle(rectangle(x, -0.72, x + 0.15, haut, 0.05));
        if mouille {
            barre = barre.plein().teinte(Teinte::Pluie);
        }
        traces.push(barre);
    }
    traces
}

/// Le radar : des cercles, une croix, le balayage qui tourne et deux échos.
fn radar() -> Vec<Trace> {
    let mut traces: Vec<Trace> = [0.32, 0.62, 0.92]
        .iter()
        .map(|&r| Trace::boucle(cercle((0.0, 0.0), r)).poids(0.7))
        .collect();
    traces.push(Trace::ligne(vec![(-0.98, 0.0), (0.98, 0.0)]).poids(0.4));
    traces.push(Trace::ligne(vec![(0.0, -0.98), (0.0, 0.98)]).poids(0.4));
    traces.push(Trace::ligne(vec![(0.0, 0.0), (0.92, 0.0)]).genre(Genre::Rotation).poids(2.2).teinte(Teinte::Pluie));
    let echo = |c: (f32, f32), r: f32| {
        let p: Vec<_> = (0..40)
            .map(|i| {
                let a = i as f32 / 40.0 * std::f32::consts::TAU;
                let bosse = 1.0 + 0.18 * (3.0 * a).sin() + 0.1 * (5.0 * a + 1.0).cos();
                (c.0 + r * bosse * a.cos(), c.1 + 0.8 * r * bosse * a.sin())
            })
            .collect();
        Trace::boucle(p).plein().teinte(Teinte::Pluie).poids(1.3)
    };
    traces.push(echo((-0.42, 0.34), 0.2));
    traces.push(echo((0.3, -0.42), 0.14));
    traces
}

/// Neuf sources en couronne, qui votent vers le centre.
fn sources() -> Vec<Trace> {
    let mut traces = vec![Trace::boucle(cercle((0.0, 0.0), 0.2)).plein().teinte(Teinte::Pluie).poids(1.5)];
    for i in 0..9 {
        let a = std::f32::consts::FRAC_PI_2 - i as f32 / 9.0 * std::f32::consts::TAU;
        let c = (0.86 * a.cos(), 0.86 * a.sin());
        traces.push(Trace::boucle(cercle(c, 0.09)).plein().poids(1.2));
        traces.push(Trace::ligne(vec![(0.74 * a.cos(), 0.74 * a.sin()), (0.24 * a.cos(), 0.24 * a.sin())]).genre(Genre::Flux).poids(0.9));
    }
    traces
}

/// Un parapluie ouvert, et quelques gouttes autour.
fn parapluie() -> Vec<Trace> {
    use std::f32::consts::PI;
    // Le dôme, puis le bord festonné : quatre arcs qui remontent.
    let mut toile = arc((0.0, 0.0), 0.9, 0.0, PI);
    for i in 0..4 {
        let cx = -0.675 + i as f32 * 0.45;
        toile.extend(arc((cx, 0.0), 0.225, PI, 0.0).into_iter().map(|(x, y)| (x, y * 0.5)));
    }
    let mut manche = vec![(0.0, 0.0), (0.0, -0.7)];
    manche.extend(arc((-0.14, -0.7), 0.14, 0.0, -PI));
    let mut traces = vec![
        Trace::boucle(toile).plein().teinte(Teinte::Pluie),
        Trace::ligne(manche).poids(1.2),
        Trace::ligne(vec![(0.0, 0.9), (0.0, 1.0)]),
    ];
    for &(x, y) in &[(-1.3, 0.7), (-1.15, -0.2), (1.2, 0.55), (1.35, -0.35), (-1.4, -0.7), (1.05, -0.7)] {
        traces.push(Trace::ligne(vec![(x, y), (x - 0.04, y - 0.22)]).teinte(Teinte::Pluie).genre(Genre::Chute).poids(1.6));
    }
    traces
}

/// Le soleil et ses rayons, au-dessus de l'échelle des UV.
fn soleil() -> Vec<Trace> {
    let mut traces = vec![Trace::boucle(cercle((0.0, 0.28), 0.36)).plein().teinte(Teinte::Soleil).poids(1.2)];
    for i in 0..12 {
        let a = i as f32 / 12.0 * std::f32::consts::TAU;
        traces.push(
            Trace::ligne(vec![(0.5 * a.cos(), 0.28 + 0.5 * a.sin()), (0.68 * a.cos(), 0.28 + 0.68 * a.sin())])
                .teinte(Teinte::Soleil)
                .poids(1.4),
        );
    }
    // L'échelle des UV : cinq paliers, du faible à l'extrême.
    for (i, largeur) in [0.5f32, 0.5, 0.36, 0.5, 0.4].iter().enumerate() {
        let x0 = -1.13 + [0.0f32, 0.5, 1.0, 1.36, 1.86][i];
        traces.push(Trace::boucle(rectangle(x0 + 0.02, -0.82, x0 + largeur - 0.02, -0.66, 0.03)).plein().poids(0.7));
    }
    traces
}

/// Un iPhone, une montre et une fenêtre de navigateur.
fn appareils() -> Vec<Trace> {
    let mut traces = vec![
        // Le navigateur.
        Trace::boucle(rectangle(-1.55, -0.55, -0.35, 0.45, 0.06)),
        Trace::ligne(vec![(-1.55, 0.32), (-0.35, 0.32)]).poids(0.6),
        Trace::boucle(rectangle(-1.4, -0.3, -0.95, 0.15, 0.04)).plein().teinte(Teinte::Pluie).poids(0.8),
        // Le téléphone.
        Trace::boucle(rectangle(-0.24, -0.82, 0.38, 0.82, 0.12)),
        Trace::ligne(vec![(-0.02, 0.72), (0.16, 0.72)]).poids(0.5),
        Trace::boucle(rectangle(-0.14, 0.1, 0.28, 0.5, 0.06)).plein().teinte(Teinte::Pluie),
        Trace::boucle(rectangle(-0.14, -0.62, 0.05, -0.2, 0.04)).poids(0.6),
        Trace::boucle(rectangle(0.09, -0.62, 0.28, -0.2, 0.04)).poids(0.6),
        // La montre et son bracelet.
        Trace::boucle(rectangle(0.62, -0.38, 1.18, 0.3, 0.14)),
        Trace::ligne(vec![(0.72, 0.3), (0.76, 0.62), (1.04, 0.62), (1.08, 0.3)]).poids(0.6),
        Trace::ligne(vec![(0.72, -0.38), (0.76, -0.7), (1.04, -0.7), (1.08, -0.38)]).poids(0.6),
    ];
    traces.push(Trace::boucle(cercle((0.9, -0.04), 0.18)).plein().teinte(Teinte::Pluie).poids(0.8));
    traces
}

// MARK: - Les grains

/// Un grain posé : sa place, sa teinte, sa façon de bouger, et s'il est un
/// grain de contour (vif) ou de remplissage et de poussière (pâle).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    pub x: f32,
    pub y: f32,
    pub teinte: Teinte,
    pub genre: Genre,
    pub vif: bool,
}

/// Une suite pseudo-aléatoire fixe : le film est le même à chaque visite.
pub struct Graine(pub u32);

impl Graine {
    pub fn suivant(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Part des grains sur les contours, dans les formes pleines, et en
/// poussière autour.
const PART_CONTOURS: f32 = 0.62;
const PART_POUSSIERE: f32 = 0.1;

/// La part des contours d'une station : le signe veut des pleins plus
/// denses, sans quoi ses deux lames croisées se lisent mal.
pub fn part_contours(nom: &str) -> f32 {
    match nom {
        "signe" | "fin" => 0.42,
        _ => PART_CONTOURS,
    }
}

/// Change une station en exactement `n` grains.
#[cfg(test)]
pub fn grains(traces: &[Trace], n: usize, graine: &mut Graine) -> Vec<Grain> {
    grains_selon(traces, n, PART_CONTOURS, graine)
}

/// Comme `grains`, avec une part des contours choisie.
pub fn grains_selon(traces: &[Trace], n: usize, part_contours: f32, graine: &mut Graine) -> Vec<Grain> {
    let mut sortie = Vec::with_capacity(n);
    let poussiere = (n as f32 * PART_POUSSIERE) as usize;
    let pleins: Vec<&Trace> = traces.iter().filter(|t| t.plein).collect();
    let aire: f32 = pleins.iter().map(|t| t.aire() * t.poids).sum();
    let remplissage = if aire > 0.0 { (n as f32 * (1.0 - part_contours - PART_POUSSIERE)) as usize } else { 0 };
    let contours = n - poussiere - remplissage;

    // Les contours, au prorata de la longueur pondérée.
    let longueurs: Vec<f32> = traces.iter().map(|t| t.longueur() * t.poids).collect();
    let total: f32 = longueurs.iter().sum();
    if total > 0.0 {
        let mut restant = contours;
        for (i, (trace, l)) in traces.iter().zip(&longueurs).enumerate() {
            let part = if i + 1 == traces.len() { restant } else { ((contours as f32) * l / total).round() as usize };
            let part = part.min(restant);
            restant -= part;
            let longueur = trace.longueur();
            for j in 0..part {
                // Réparti régulièrement, avec un peu de hasard : un trait de
                // craie plutôt qu'une ligne de pointillés.
                let d = (j as f32 + graine.suivant()) / part as f32 * longueur;
                let (x, y) = trace.au_long(d);
                let ecart = 0.008;
                sortie.push(Grain {
                    x: x + (graine.suivant() - 0.5) * ecart,
                    y: y + (graine.suivant() - 0.5) * ecart,
                    teinte: trace.teinte,
                    genre: trace.genre,
                    vif: true,
                });
            }
        }
    }

    // L'intérieur des formes pleines, au prorata de l'aire.
    let mut restant = remplissage;
    for (i, trace) in pleins.iter().enumerate() {
        let part = if i + 1 == pleins.len() {
            restant
        } else {
            ((remplissage as f32) * trace.aire() * trace.poids / aire).round() as usize
        }
        .min(restant);
        restant -= part;
        let (x0, y0, x1, y1) = trace.boite();
        let mut poses = 0;
        let mut essais = 0;
        while poses < part && essais < part * 60 {
            essais += 1;
            let p = (x0 + graine.suivant() * (x1 - x0), y0 + graine.suivant() * (y1 - y0));
            if trace.contient(p) {
                sortie.push(Grain { x: p.0, y: p.1, teinte: trace.teinte, genre: trace.genre, vif: false });
                poses += 1;
            }
        }
    }

    // Le reste en poussière, sur tout le cadre.
    while sortie.len() < n {
        sortie.push(Grain {
            x: (graine.suivant() * 2.0 - 1.0) * 1.9,
            y: (graine.suivant() * 2.0 - 1.0) * 1.25,
            teinte: Teinte::Creme,
            genre: Genre::Fixe,
            vif: false,
        });
    }
    sortie.truncate(n);
    sortie
}

/// Toutes les stations, à la suite : `n` grains par station, quatre nombres
/// par grain — x, y, teinte (plus 0,5 si le grain est pâle) et genre. C'est
/// le tampon que lit le moteur.
pub fn film(n: usize) -> Vec<f32> {
    let mut graine = Graine(20_261_009);
    let mut tampon = Vec::with_capacity(STATIONS.len() * n * 4);
    for nom in STATIONS {
        let mut g = grains_selon(&station(nom), n, part_contours(nom), &mut graine);
        // Mélangés : sans cela, les grains d'un même tracé partiraient
        // ensemble vers un même tracé de la station suivante.
        for i in (1..g.len()).rev() {
            let j = (graine.suivant() * (i + 1) as f32) as usize;
            g.swap(i, j.min(i));
        }
        for grain in g {
            tampon.extend([
                grain.x,
                grain.y,
                grain.teinte as u8 as f32 + if grain.vif { 0.0 } else { 0.5 },
                grain.genre as u8 as f32,
            ]);
        }
    }
    tampon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_station_a_exactement_le_meme_nombre_de_grains() {
        let mut graine = Graine(1);
        for nom in STATIONS {
            let g = grains(&station(nom), 4000, &mut graine);
            assert_eq!(g.len(), 4000, "station {nom}");
        }
        assert_eq!(film(1000).len(), STATIONS.len() * 1000 * 4);
    }

    #[test]
    fn chaque_station_a_des_traces() {
        for nom in STATIONS {
            assert!(!station(nom).is_empty(), "station {nom} vide");
        }
    }

    #[test]
    fn les_formes_tiennent_dans_le_cadre() {
        for nom in STATIONS {
            for trace in station(nom) {
                for &(x, y) in &trace.points {
                    assert!((-1.6..=1.6).contains(&x) && (-1.0..=1.0).contains(&y), "{nom} : ({x}, {y}) déborde");
                }
            }
        }
    }

    #[test]
    fn les_grains_de_remplissage_sont_dans_leur_forme() {
        let carre = Trace::boucle(rectangle(-0.5, -0.5, 0.5, 0.5, 0.0)).plein();
        let mut graine = Graine(7);
        let g = grains(std::slice::from_ref(&carre), 2000, &mut graine);
        let pales: Vec<_> = g.iter().filter(|g| !g.vif && g.x.abs() <= 0.5 && g.y.abs() <= 0.5).collect();
        assert!(pales.len() >= 500, "{} grains pâles dans le carré", pales.len());
        assert!(g.iter().filter(|g| g.vif).all(|g| (g.x.abs() - 0.5).abs() < 0.01 || (g.y.abs() - 0.5).abs() < 0.01));
    }

    #[test]
    fn aire_et_appartenance() {
        let carre = Trace::boucle(rectangle(0.0, 0.0, 2.0, 1.0, 0.0));
        assert!((carre.aire() - 2.0).abs() < 1e-5);
        assert!(carre.contient((1.0, 0.5)));
        assert!(!carre.contient((2.5, 0.5)));
        assert!((carre.longueur() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn la_pluie_tombe_et_le_radar_tourne() {
        assert!(station("nuage").iter().any(|t| t.genre == Genre::Chute));
        assert!(station("radar").iter().any(|t| t.genre == Genre::Rotation));
        assert!(station("sources").iter().any(|t| t.genre == Genre::Flux));
    }

    #[test]
    fn neuf_sources_comme_sur_la_page() {
        let cercles = station("sources").iter().filter(|t| t.plein && t.teinte == Teinte::Creme).count();
        assert_eq!(cercles, 9);
    }
}
