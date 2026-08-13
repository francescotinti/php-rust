# REPORT_GAP_133 — SOLO S-133 (2026-08-13). Coppia WP full+media sul pin NUOVO s133 c87439a9 + server s133 d447f828 (post-leva ctor resolve-once; criterio s132-criterio-pair.md RIUSATO invariato). Warm-up media-only dichiarato + 4 gambe intercalate off1→on1→off2→on2.

## Cifre (raw: wp133-harness/pair-out/t1-leg*-*/; verdetto: wp133-harness/s133-pair-verdetto-t1.out; quiescenza rc=0 ×5 in header; CI locale SOSPESA nella finestra)
| gamba | ictx/s oracle/phpr | firma (ictx%med o/p · rank cpu oracle) | full cpu ratio proprio | media user-only | peak phpr MiB | esito gate |
|---|---|---|---|---|---|---|
| leg1-off | 1385 / 194 | 103% / 90% · 3/4 | 1,781 | 2,447 | 1897,8 | pulita |
| leg1-on | 1294 / 230 | 97% / 107% · 2/4 | 1,754 | 2,487 | 1816,0 | pulita |
| leg2-off | 1430 / 199 | 107% / 93% · 1/4 | 1,808 | 2,428 | 1885,8 | pulita |
| leg2-on | 1294 / 486 | 97% / 226% · 4/4 | 1,756 | 2,449 | 1828,1 | **SEGNALATA** |

Mediane ictx/s PER MOTORE: oracle 1339 · phpr 215. **3/4 gambe pulite**:
leg2-on esclusa dal gate 1,5× per contesa sul MOTORE phpr (ictx 226% della
mediana; oracle CPU rank 4/4) — il raw della gamba esclusa (1,756) resta
DENTRO l'intervallo on del riferimento, l'esclusione è protocollare.

## Riferimento @ pin s133 (PER CONFIGURAZIONE; matrice 9 celle nel verdetto)
**full ON-ONLY CANONICO = 1,754 (N=1 coppia propria — UN punto, non banda)** ·
full off-only 1,781–1,808 (N=2) · companion misto pulito: coppie proprie
1,754–1,808, intervallo 1,750–1,817 · **media CANONICA user-only = 2,428–2,487**
(companion user+sys 2,366–2,454; gambe pulite) · **peak 1816–1898 MiB (≈ +80
MiB sul bordo alto vs 1754–1810 @ s131/s132 — da tenere d'occhio, non gated)**.
Parità per NOME 4/4 (media 0 nomi; full solo `wp_is_stream data set #2`).

## Lettura (direzione, non attribuzione firmata — REGOLE §4)
On-only 1,754 vs 1,752–1,768 @ s131: DENTRO il riferimento precedente — la leva
ctor (−46,7 ns/objalloc micro) NON muove WP full, ed è COERENTE col modello: la
leva morde il fallback non-plain di PropSet (classi con prop tipizzate/hooked
— il profilo ORM), mentre le classi WP sono in larga parte plain → IC WP-29 e
fast-path WP-25 le servivano già senza resolve doppie. Off 1,781–1,808 vs 1,783
@ s131: coerente, bordo alto +0,025 con N=2 (rumore denominatore). Media
invariata (2,43–2,49 vs 2,45–2,48). Il guadagno della leva sta dove il modello
lo colloca: micro-ORM objalloc 7,8→7,5 · objdatains 7,2 (1183,3) · churn 8,2
(1440,0) — il soffitto object-dense (dbal 8,6 / ORM 8,5) è il bersaglio a cui
la leva parla, la rimisura di quelle suite resta ai prossimi pin.
