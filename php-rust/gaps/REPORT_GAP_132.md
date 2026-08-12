# REPORT_GAP_132 — SOLO S-132 (2026-08-12). Coppia WP full+media RIMISURATA sul pin NUOVO s131 ff66cb84 + server s131 97ed6e06 (post-E1-KO; criterio s132-criterio-pair.md pre-registrato, az.rev. S-131 #3/#5 cablate). Warm-up media-only dichiarato + 4 gambe intercalate off1→on1→off2→on2.

## Cifre (raw: wp132-harness/pair-out/t1-leg*-*/; verdetto: wp132-harness/s132-pair-verdetto-t1.out; quiescenza rc=0 ×5 in header)
| gamba | ictx/s oracle/phpr | firma (ictx%med o/p · rank cpu oracle) | full cpu ratio proprio | media user-only | peak phpr MiB | esito gate |
|---|---|---|---|---|---|---|
| leg1-off | 2181 / 651 | 161% / 245% · 1/4 | 1,797 | 2,475 | 1795,2 | **SEGNALATA** |
| leg1-on | 1374 / 270 | 101% / 102% · 2/4 | 1,768 | 2,481 | 1754,1 | pulita |
| leg2-off | 1335 / 262 | 98% / 98% · 3/4 | 1,783 | 2,458 | 1810,5 | pulita |
| leg2-on | 1338 / 243 | 99% / 91% · 4/4 | 1,752 | 2,453 | 1766,0 | pulita |

Mediane ictx/s PER MOTORE: oracle 1356 · phpr 266. **3/4 gambe pulite**: leg1-off
esclusa dal gate 1,5× CON la firma prevista dalla revisione S-131 (ictx oracle
+61%, oracle CPU la più veloce della sera) — la firma per gamba (az.rev. #3) ha
confermato sul campo il reperto secondario #2 di S-131.

## Riferimento NUOVO @ pin s131 (az.rev. #3: PER CONFIGURAZIONE; matrice 9 celle nel verdetto)
**full ON-ONLY CANONICO = 1,752–1,768** (coppie proprie N=2; intervallo on 1,751–1,769) ·
full off-only 1,783 (N=1, gamba unica pulita) · companion misto pulito: coppie
proprie 1,752–1,783, intervallo 1,751–1,797 · **media CANONICA user-only =
2,453–2,481** (companion user+sys 2,406–2,439; gambe pulite) · peak 1754–1810 MiB.
Sostituisce (e per la prima volta SEPARA per configurazione) il riferimento S-131
@ s130: full misto 1,757–1,797 (on 1,757–1,767 · off 1,785–1,797), media 2,441–2,479.
Parità per NOME 4/4 (media 0 nomi; full solo `wp_is_stream data set #2`).

## Lettura (direzione, non attribuzione firmata — REGOLE §4)
On-only 1,752–1,768 vs 1,757–1,767 @ s130: bordo basso −0,005, bordo alto +0,001 —
riferimento CONFERMATO, il contributo E1-KO (−23,3 ns/objdatains micro) su WP full
è DENTRO il rumore del denominatore a N=2 per modo: direzione coerente (minimo
storico 1,751 sull'intervallo), magnitudine non ripartita. Off 1,783 vs
1,785–1,797: coerente, N=1. Media invariata (2,45–2,48 vs 2,44–2,48): il gruppo
media non è objdatains-denso, atteso.
