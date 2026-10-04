//! Servir la vitrine et l'application depuis le relais.
//!
//! Le relais tourne déjà quelque part de public : lui faire servir les
//! fichiers construits évite un second hébergement, et l'application se
//! retrouve sur la même origine que ses appels — donc plus de CORS à accorder,
//! et rien à configurer pour qu'elle trouve le relais.
//!
//! C'est un serveur de fichiers minuscule, et il n'a pas besoin d'être plus :
//! quelques dizaines de fichiers construits, tous connus d'avance.

use std::path::{Component, Path, PathBuf};

const TYPES: [(&str, &str); 14] = [
    ("html", "text/html;charset=utf-8"),
    ("js", "text/javascript;charset=utf-8"),
    ("css", "text/css;charset=utf-8"),
    ("json", "application/json;charset=utf-8"),
    ("svg", "image/svg+xml"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("webp", "image/webp"),
    ("ico", "image/x-icon"),
    ("woff2", "font/woff2"),
    ("woff", "font/woff"),
    ("txt", "text/plain;charset=utf-8"),
    ("webmanifest", "application/manifest+json"),
    ("wasm", "application/wasm"),
];

/// Le chemin demandé, ramené à un fichier sous la racine — ou rien.
///
/// Deux pièges, et le second est le sérieux :
///
/// - Un dossier ne se sert pas : `/` et `/app/` désignent leur `index.html`.
/// - `%2e%2e/` remonte l'arborescence aussi bien que `../`. On décode donc
///   *avant* de normaliser, puis on vérifie que le résultat est toujours sous
///   la racine. Sans cette vérification finale, un relais public sert ses
///   propres secrets.
pub fn resolve_within(root: &Path, pathname: &str) -> Option<PathBuf> {
    let decoded = percent_decode(pathname)?;
    if decoded.contains('\0') {
        return None;
    }

    // On résout depuis la racine, sans barre de tête : « /../x » devient alors
    // « ../x », qui sort vraiment de la racine, et se fait refuser.
    let base = normalise(root, "");
    let cible = normalise(&base, decoded.trim_start_matches('/'));

    // La comparaison porte le séparateur : sans lui, « …/publicité » passerait
    // pour un enfant de « …/public ».
    if cible != base && !cible.starts_with(&base) {
        return None;
    }

    let sans_extension = cible.extension().is_none();
    if decoded.ends_with('/') || sans_extension {
        return Some(cible.join("index.html"));
    }
    Some(cible)
}

/// `decodeURIComponent`, qui refuse un pourcentage mal formé plutôt que de
/// deviner — et refuse aussi une suite d'octets qui n'est pas de l'UTF-8.
fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = value.get(index + 1..index + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(out).ok()
}

/// Normalise un chemin sans toucher au disque, comme `path.resolve`.
///
/// Les `..` remontent réellement, y compris au-dessus de la racine : c'est
/// justement ce qui permet de s'en apercevoir ensuite.
fn normalise(base: &Path, relative: &str) -> PathBuf {
    let mut pile: Vec<Component> = base.components().collect();

    for composant in Path::new(relative).components() {
        match composant {
            Component::ParentDir => {
                if pile.len() > 1 {
                    pile.pop();
                }
            }
            Component::CurDir => {}
            autre => pile.push(autre),
        }
    }

    pile.iter().collect()
}

/// Le type d'un fichier d'après son extension, `application/octet-stream`
/// sinon.
pub fn content_type(chemin: &Path) -> &'static str {
    let extension = chemin
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    TYPES
        .iter()
        .find(|(suffixe, _)| *suffixe == extension)
        .map_or("application/octet-stream", |(_, type_mime)| *type_mime)
}

/// Combien de temps garder le fichier.
///
/// Les outils de construction posent une empreinte dans le nom des fichiers
/// d'`assets/` : leur contenu ne changera jamais, ils peuvent donc être gardés
/// un an. Le `index.html`, lui, désigne ces noms — s'il était gardé, une
/// nouvelle version resterait invisible.
pub fn cache_control(chemin: &Path) -> &'static str {
    let empreinte = chemin.components().any(|c| c.as_os_str() == "assets");
    if empreinte { "public, max-age=31536000, immutable" } else { "no-cache" }
}

/// Un fichier servi : son contenu, son type, sa durée de garde.
pub struct Fichier {
    pub contenu: Vec<u8>,
    pub content_type: &'static str,
    pub cache_control: &'static str,
}

/// Sert un fichier depuis `root`, ou rend `None` si la route n'en désigne
/// aucun — à l'appelant de décider ce que vaut une absence.
pub async fn servir(root: &Path, pathname: &str) -> Option<Fichier> {
    let chemin = resolve_within(root, pathname)?;
    let contenu = tokio::fs::read(&chemin).await.ok()?;

    Some(Fichier {
        contenu,
        content_type: content_type(&chemin),
        cache_control: cache_control(&chemin),
    })
}

/* ---------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    const RACINE: &str = "/var/klima/public";

    fn resolu(pathname: &str) -> Option<String> {
        resolve_within(Path::new(RACINE), pathname)
            .map(|chemin| chemin.to_string_lossy().into_owned())
    }

    #[test]
    fn la_racine_designe_son_index() {
        assert_eq!(resolu("/").as_deref(), Some("/var/klima/public/index.html"));
    }

    #[test]
    fn un_dossier_aussi_avec_ou_sans_barre_finale() {
        assert_eq!(resolu("/app/").as_deref(), Some("/var/klima/public/app/index.html"));
        assert_eq!(resolu("/app").as_deref(), Some("/var/klima/public/app/index.html"));
    }

    #[test]
    fn un_fichier_reste_lui_meme() {
        assert_eq!(
            resolu("/assets/index-abc123.js").as_deref(),
            Some("/var/klima/public/assets/index-abc123.js")
        );
    }

    #[test]
    fn on_ne_remonte_pas_au_dessus_de_la_racine() {
        // Le cas qui compte : un relais public sert sinon ses propres secrets.
        assert_eq!(resolu("/../../etc/passwd"), None);
        assert_eq!(resolu("/app/../../../../etc/passwd"), None);
    }

    #[test]
    fn ni_en_encodant_les_points() {
        assert_eq!(resolu("/%2e%2e/%2e%2e/etc/passwd"), None);
    }

    #[test]
    fn un_pourcentage_mal_forme_ne_se_devine_pas() {
        assert_eq!(resolu("/%zz"), None);
        assert_eq!(resolu("/%2"), None);
    }

    #[test]
    fn un_octet_nul_non_plus() {
        assert_eq!(resolu("/index.html%00.png"), None);
    }

    #[test]
    fn une_suite_doctets_qui_nest_pas_de_lutf8_non_plus() {
        assert_eq!(resolu("/%ff%fe.png"), None);
    }

    #[test]
    fn une_racine_voisine_nest_pas_la_racine() {
        // « /var/klima/publicité » commence par « /var/klima/public » : c'est
        // exactement le genre de préfixe qui passe quand on compare sans
        // séparateur.
        assert_eq!(resolu("/../publicité/secret.txt"), None);
    }

    #[test]
    fn les_types_que_la_construction_produit() {
        let type_de = |chemin: &str| content_type(Path::new(chemin));
        assert_eq!(type_de("/x/index.html"), "text/html;charset=utf-8");
        assert_eq!(type_de("/x/index-abc.js"), "text/javascript;charset=utf-8");
        assert_eq!(type_de("/x/index-abc.css"), "text/css;charset=utf-8");
        assert_eq!(type_de("/x/marque.svg"), "image/svg+xml");
        assert_eq!(type_de("/x/police.woff2"), "font/woff2");
        // Celui que le TypeScript n'avait pas à connaître.
        assert_eq!(type_de("/x/klima-web_bg.wasm"), "application/wasm");
    }

    #[test]
    fn linconnu_reste_des_octets() {
        assert_eq!(content_type(Path::new("/x/chose.quoi")), "application/octet-stream");
    }

    #[test]
    fn les_fichiers_empreintes_se_gardent_un_an() {
        assert_eq!(
            cache_control(Path::new("/var/klima/public/assets/index-abc123.js")),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn lindex_jamais() {
        // S'il était gardé, il continuerait à désigner les anciens fichiers
        // empreintés : la nouvelle version resterait invisible.
        assert_eq!(cache_control(Path::new("/var/klima/public/index.html")), "no-cache");
    }

    #[tokio::test]
    async fn sert_un_fichier_present_avec_son_type() {
        let racine = std::env::temp_dir().join("klima-site-present");
        tokio::fs::create_dir_all(&racine).await.unwrap();
        tokio::fs::write(racine.join("index.html"), b"<!doctype html>").await.unwrap();

        let fichier = servir(&racine, "/").await.unwrap();
        assert_eq!(fichier.content_type, "text/html;charset=utf-8");
        assert_eq!(fichier.cache_control, "no-cache");
        assert_eq!(fichier.contenu, b"<!doctype html>");
    }

    #[tokio::test]
    async fn rend_rien_sur_un_fichier_absent_pour_que_lappelant_decide() {
        let racine = std::env::temp_dir().join("klima-site-absent");
        tokio::fs::create_dir_all(&racine).await.unwrap();
        assert!(servir(&racine, "/inconnu.png").await.is_none());
    }

    #[tokio::test]
    async fn rend_rien_sur_une_tentative_de_remontee_sans_toucher_au_disque() {
        // Le fichier existe pour de vrai : c'est la résolution qui refuse.
        let racine = std::env::temp_dir().join("klima-site-remontee");
        tokio::fs::create_dir_all(racine.join("vrai")).await.unwrap();
        tokio::fs::write(racine.join("secret.txt"), "clé").await.unwrap();

        assert!(servir(&racine.join("vrai"), "/../secret.txt").await.is_none());
    }
}
