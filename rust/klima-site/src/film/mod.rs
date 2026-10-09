//! Le film de la vitrine : des grains qui dessinent, au fil du défilement,
//! ce que Klima regarde avant qu'on sorte.

pub mod formes;
pub mod moteur;

/// Où en est le film, d'après l'avancée dans sa section (de 0 à 1) : la
/// station que l'on quitte, celle où l'on va, et l'avancée du passage. Chaque
/// station tient l'image la plus grande part de son temps, puis passe la main.
pub fn moment(avancee: f32, stations: usize) -> (usize, usize, f32) {
    let dernier = stations.saturating_sub(1);
    let p = avancee.clamp(0.0, 1.0) * dernier as f32;
    let de = (p.floor() as usize).min(dernier);
    let vers = (de + 1).min(dernier);
    let t = ((p - de as f32 - 0.5) / 0.5).clamp(0.0, 1.0);
    (de, vers, t)
}

/// La phrase à montrer : celle de la station dont on est le plus près.
pub fn ligne(avancee: f32, stations: usize) -> usize {
    let dernier = stations.saturating_sub(1);
    ((avancee.clamp(0.0, 1.0) * dernier as f32 + 0.25).floor() as usize).min(dernier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_film_tient_puis_passe() {
        assert_eq!(moment(0.0, 9), (0, 1, 0.0));
        // Le début de chaque station tient l'image.
        assert_eq!(moment(0.4 / 8.0, 9).2, 0.0);
        let (de, vers, t) = moment(0.75 / 8.0, 9);
        assert_eq!((de, vers), (0, 1));
        assert!((t - 0.5).abs() < 1e-4);
        // À la fin, la dernière station tient.
        assert_eq!(moment(1.0, 9), (8, 8, 0.0));
    }

    #[test]
    fn la_phrase_change_au_milieu_du_passage() {
        assert_eq!(ligne(0.0, 9), 0);
        assert_eq!(ligne(0.7 / 8.0, 9), 0);
        assert_eq!(ligne(0.8 / 8.0, 9), 1);
        assert_eq!(ligne(1.0, 9), 8);
    }
}
