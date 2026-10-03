// Le pendant TypeScript de `balayage.rs` : la même sortie, produite par
// `Intl`. Lancé depuis la racine du dépôt :
//
//   bun run rust/klima-core/examples/balayage.ts
//
// Ce n'est pas un test — c'est l'outil qui a servi à relever les règles de
// typographie que `format.rs` applique à la main.
import { formats } from '../../../core/src/format'
const VALEURS = [0, 0.04, -0.04, 0.5, -0.5, 1.25, 2.1, -1.2, 4.8, 16.4, -1.4, 26.6, 61.5, 99.95, 100, 1234.5, -1234.5, 1234567.5, 0.265, -2.45]
const out: string[] = []
for (const l of ['fr','en','es'] as const) {
  const f = formats(l)
  for (const v of VALEURS) {
    out.push(`${l} ${v} d0=${f.decimal(v,0)} d1=${f.decimal(v,1)} d2=${f.decimal(v,2)} p=${f.percent(v)} t=${f.temperature(v)} s=${f.signedUnit(v,'mm')}`)
  }
}
console.log(out.join('\n'))
