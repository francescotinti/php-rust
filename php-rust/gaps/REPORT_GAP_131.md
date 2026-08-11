# REPORT_GAP_131 — SOLO S-131 (2026-08-11). Coppia WP full+media RIMISURATA sul pin NUOVO s130 0fdf1c49 + server s130 (post-F4; criterio s131-criterio-pair.md pre-registrato). Warm-up media-only dichiarato (mai giudicato) + 4 gambe intercalate off1→on1→off2→on2.

## Cifre (raw: wp131-harness/pair-out/leg*-*/; verdetto: wp131-harness/s131-pair-verdetto.out; quiescenza rc=0 ×5 in header)
| gamba | ictx/s oracle/phpr | full cpu ratio proprio | media user-only | peak phpr MiB | esito gate |
|---|---|---|---|---|---|
| leg1-off | 1466 / 163 | 1,797 | 2,479 | 1788,1 | pulita |
| leg1-on | 1248 / 171 | 1,767 | 2,452 | 1848,1 | pulita |
| leg2-off | 1233 / 175 | 1,785 | 2,441 | 1887,0 | pulita |
| leg2-on | 1248 / 205 | 1,757 | 2,461 | 1830,3 | pulita |

Mediane ictx/s PER MOTORE: oracle 1248 · phpr 173 (addendum rev. S-129); **4/4 gambe pulite** — il warm-up leg ha assorbito l'effetto prima-di-sequenza (S-129: 2 prime-di-sequenza segnalate; qui la prima gamba giudicata è la 2ª della sera).

## Riferimento NUOVO @ pin s130 (matrice 16 celle nel verdetto)
**full = 1,757–1,797** (coppie proprie = intervallo pulito; user-only 1,766–1,809) ·
**media CANONICA user-only = 2,441–2,479** (companion user+sys 2,394–2,447) · peak 1788–1887 MiB.
Sostituisce 1,758–1,805 · 2,447–2,463 misurati @ s127b. Parità per NOME 4/4 (media 0 nomi; full solo `wp_is_stream data set #2`).

## Lettura (direzione, non attribuzione firmata — REGOLE §4)
Intervallo full 1,757–1,797 vs 1,758–1,805 @ s127b: riferimento CONFERMATO con bordo
alto −0,008; le coppie on (1,767/1,757) stanno sotto le off (1,797/1,785) come in
S-129. Il contributo F4 (−80 ns/objdatains micro) su WP full è DENTRO il rumore del
denominatore a N=2 per modo: direzione coerente, magnitudine non ripartita.
