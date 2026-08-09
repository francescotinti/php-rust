# REPORT_GAP_120 — SOLO S-120 (2026-08-09). Coppia WP sul pin s120 885d2c64 (phpr; server non esercitato), pair109 INVARIATA ×4 gambe INTERCALATE stessa sera (off1→on1→off2→on2), rc=0 ×4.

## Cifre (raw: wp120-harness/pair-out/leg*-*/; verdetto: wp120-harness/s120-pair-verdetto.out)
| voce | OFF (gamba1 / gamba2) | ON (gamba1 / gamba2) |
|---|---|---|
| full master CPU ratio (coppie proprie) | **1,873 / 1,853** | **1,850 / 1,825** |
| full oracle → phpr (s) | 436,05→816,55 · 441,30→817,86 | 432,93→801,02 · 442,65→807,67 |
| media user CPU ratio | 2,511 / 2,524 | 2,534 / 2,527 |
| full peak phpr (MiB) | 1909,2 / 1983,1 | 1936,6 / 1862,5 |
| parità | full == solo wp_is_stream; media 0 fail | idem ×2 |

## Lettura (N=2 per modo, stessa sera, oracle campionato N=4)
- **Oracle spread stessa-sera = 2,2%** (432,93–442,65) contro il 7,2% tra le gambe
  S-119: il rumore era TRA-SERE, non di modo. **Matrice 16 celle: full = 1,810–1,889**.
- **Lo split ON/OFF resta piccolo ma stanotte è CONCORDE** (ON < OFF su entrambe
  le gambe, phpr 801/808 vs 817/818): direzione modo-ON favorevole, N=2.
- **Media-ON 2,636 SCIOLTA** (az. rev. S-119): stanotte ON 2,534/2,527 ≈ OFF
  2,511/2,524 — l'anomalia era N=1 dell'oracle-media.
- **Peak +94 MiB SCIOLTO come effetto-modo**: 1862–1983 MiB su 4 gambe senza
  pattern di modo — banda run-to-run ~120 MiB; il confronto s118 (1839) è tra-sere.
- phpr full 801–818 s contro 826/827 s del pin s119: coerente con L-RE1 (WP è
  preg-heavy) ma TRA-SERE e tra-pin ⇒ direzione-solo, magnitudine non ripartita.

## Metodo
DEVIAZIONE DICHIARATA dal piano «sul pin s119»: il pin s119 è stato superato in
sessione (L-RE1 → pin s120); l'intervallo si scioglie campionando l'oracle N=4
stessa sera sul pin CORRENTE. Riferimento nuovo della famiglia = QUESTA coppia
(pin s120): **full 1,81–1,89 (intervallo 16 celle), punto mediano ~1,85**;
il rif s119 1,71–1,84 decade («il riferimento segue il pin»). Il server s120
6b822369 è pin al minimo (smoke): grado pieno = debito S-121.
