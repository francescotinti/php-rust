# REPORT_GAP_134 — SOLO S-134 (2026-08-13). Coppia WP full+media sul pin NUOVO s134 61896da1 + server s134 461bfb55 (post-leva IC non-plain; criterio s132-criterio-pair.md RIUSATO invariato). Warm-up media-only dichiarato + 4 gambe intercalate off1→on1→off2→on2.

## Cifre (raw: wp134-harness/pair-out/t1-leg*-*/; verdetto: wp134-harness/s134-pair-verdetto-t1.out; quiescenza rc=0 ×5 in header; CI locale SOSPESA nella finestra via lock)
| gamba | ictx/s oracle/phpr | firma (ictx%med o/p · rank cpu oracle) | full cpu ratio proprio | media user-only | peak phpr MiB | esito gate |
|---|---|---|---|---|---|---|
| leg1-off | 1500 / 521 | 78% / 63% · 4/4 | 1,748 | 2,405 | 1877,5 | pulita |
| leg1-on | 1999 / 861 | 104% / 104% · 3/4 | 1,769 | 2,467 | 1817,6 | pulita |
| leg2-off | 1954 / 807 | 101% / 97% · 2/4 | 1,789 | 2,446 | 1883,8 | pulita |
| leg2-on | 1905 / 851 | 99% / 103% · 1/4 | 1,769 | 2,435 | 1840,4 | pulita |

Mediane ictx/s PER MOTORE: oracle 1930 · phpr 829 (assolute più alte del run
s133 per ENTRAMBI i motori — fondo macchina diverso, il gate è relativo per
costruzione). **4/4 gambe pulite** — prima coppia senza esclusioni dal s131.

## Riferimento @ pin s134 (PER CONFIGURAZIONE; matrice 16 celle nel verdetto)
**full ON-ONLY CANONICO = 1,769 (N=2 coppie proprie CONCORDI 1,769/1,769 —
prima banda propria, non più un punto)** · full off-only 1,748–1,789 (N=2) ·
companion misto pulito: coppie proprie 1,748–1,789, intervallo 1,730–1,791 ·
**media CANONICA user-only = 2,405–2,467** (companion user+sys 2,324–2,411;
4 gambe) · **peak 1818–1884 MiB (bordo alto ≈ +80 vs s131/s132 PERSISTE —
da tenere d'occhio, non gated)**. Parità per NOME 4/4 (media 0 nomi; full
solo `wp_is_stream data set #2`).

## Lettura (direzione, non attribuzione firmata — REGOLE §4)
On-only 1,769 (N=2) vs 1,754 (N=1) @ s133: il punto s133 era SENZA banda; il
riferimento nuovo cade dentro la fascia storica 1,75–1,79 e le off si
sovrappongono (1,748–1,789 vs 1,781–1,808) — la leva IC non-plain NON muove
WP full, COERENTE col modello: morde il cammino non-plain (prop tipizzate,
profilo ORM), le classi WP sono in larga parte plain già servite da IC
WP-29/fast-path WP-25. Il guadagno sta dove il modello lo colloca: micro-ORM
objalloc 7,5→6,6 (−133,4 ns/iter, A/B D=+136,7) · objdatains 7,2→6,4 ·
churn 8,2→7,4 · dropdef 8,9→7,9 · allocni 9,4→7,9. Il soffitto object-dense
(dbal 8,6 / ORM 8,5 @ pin vecchi) è il bersaglio diretto di DUE leve
consecutive (ctor resolve-once + IC non-plain): la rimisura bilaterale
dbal/ORM è DOVUTA in testa a S-135 (finestra S-134 insufficiente per la
ricetta pulita: untar per gamba, 2/lato, watchdog).
