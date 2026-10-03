//! Le balayage qui a servi à régler la typographie sur celle d'`Intl`.
//!
//! `format.rs` ne peut pas appeler `Intl` : il n'existe pas en WASM. Les
//! règles y sont donc écrites à la main, et c'est ce programme qui vérifie
//! qu'elles rendent la même chose. Pour refaire la comparaison :
//!
//! ```sh
//! bun run /tmp/sweep.ts > /tmp/ts.txt            # voir le commit qui l'ajoute
//! cargo run -q -p klima-core --example balayage > /tmp/rs.txt
//! diff /tmp/ts.txt /tmp/rs.txt
//! ```
//!
//! Soixante valeurs, six fonctions, trois langues : sortie identique, demis
//! et séparateurs invisibles compris.

use klima_core::format::Formats;
use klima_core::i18n::LANGUAGES;

fn main() {
    let valeurs = [
        (0.0, "0"), (0.04, "0.04"), (-0.04, "-0.04"), (0.5, "0.5"), (-0.5, "-0.5"),
        (1.25, "1.25"), (2.1, "2.1"), (-1.2, "-1.2"), (4.8, "4.8"), (16.4, "16.4"),
        (-1.4, "-1.4"), (26.6, "26.6"), (61.5, "61.5"), (99.95, "99.95"), (100.0, "100"),
        (1234.5, "1234.5"), (-1234.5, "-1234.5"), (1234567.5, "1234567.5"),
        (0.265, "0.265"), (-2.45, "-2.45"),
    ];
    for language in LANGUAGES {
        let f = Formats::new(language);
        for (v, texte) in valeurs {
            println!(
                "{} {texte} d0={} d1={} d2={} p={} t={} s={}",
                language.code(),
                f.decimal(v, 0), f.decimal(v, 1), f.decimal(v, 2),
                f.percent(v), f.temperature(v), f.signed_unit(v, "mm", 1),
            );
        }
    }
}
