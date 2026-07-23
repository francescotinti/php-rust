# REPORT_GAP_43 — gap perf oracle↔phpr, cumulativo fino a WP-43

> Convenzione (decisione utente 2026-07-23): a fine sessione N si crea
> `gaps/REPORT_GAP_<N>.md` = copia del report precedente + UNA riga nuova
> in tabella (l'ultimo file è la tabella viva; i precedenti sono snapshot).
> ⚠️ confrontare RAPPORTI, mai i tempi assoluti di giornate diverse.

## Metodo di misura (invariato dalla regola ricorrente)
1. **Media group**: oracle 1 run `/usr/bin/time -l` (DB reset + uploads
   azzerati, MIMALLOC_PURGE_DELAY=0) vs phpr → rapporto **user CPU** e
   **peak footprint**.
2. **Full-suite**: CPU del processo master phpr dal tail del `.rss` della
   runN di sessione vs oracle (baseline 5:39) → rapporto; wall indicativo.

## Tabella cumulativa

| sessione | media CPU (phpr/oracle) | media footprint | full-suite master-CPU | full-suite wall |
|---|---|---|---|---|
| WP-26 (baseline) | 85,8/21,0 = **4,1×** | 5,0/0,4GB = **12,7×** | (wall, non comparabile) | ~1,9× |
| WP-27 | 82,7/21,1 = **3,9×** | 4,78/0,40GB = **12,0×** | 16:11/5:39 = **2,9×** | ~22/11,5 min = **1,9×** |
| WP-28 | 87,6/23,0 = **3,8×** | 4,83/0,40GB = **12,2×** | 16:43/5:39 = **3,0×** | ~22/11,5 min = **1,9×** |
| WP-29 | 82,4/23,0 = **3,6×** | 4,84/0,40GB = **12,1×** | 15:27/5:39 = **2,7×** | ~22/11,5 min = **1,9×** |
| WP-30 | 80,7/21,0 = **3,8×** ⚠️ | 4,80/0,40GB = **12,1×** | 15:12/5:39 = **2,7×** | ~20/11,5 min = **1,7×** |
| WP-31 | 72,4/20,95 = **3,5×** | 4,82/0,40GB = **12,1×** | 13:02/5:39 = **2,3×** | ~17,5/11,5 min = **1,5×** |
| WP-32 | 69,0/20,91 = **3,3×** | 4,75/0,39GB = **12,0×** | 12:54/5:39 = **2,3×** | ~19,5/11,5 min = **1,7×** |
| WP-33 | 66,9/20,97 = **3,19×** | 4,75/0,39GB = **12,0×** | 12:20/5:39 = **2,18×** | ~16,5/11,5 min = **1,4×** |
| WP-34 | 65,1/20,92 = **3,11×** | 4,73/0,39GB = **12,0×** | ~12:35/5:39 = **2,2×** (rumore) | ~17,5/11,5 min = **1,5×** |
| WP-35 | 59,6/20,99 = **2,84×** ⭐ | 4,73/0,39GB = **12,0×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-36 | 61,4/21,06 = **2,92×** ⚠️ | 4,78/0,40GB = **12,1×** | ~12:05/5:39 = **2,14×** | ~17/11,5 min = **1,5×** |
| WP-37 | 60,07/20,94 = **2,87×** | 4,72/0,39GB = **12,0×** | ~12:30/5:39 = **2,2×** (rumore) | ~17/11,5 min = **1,5×** |
| WP-38 | 59,75/20,955 = **2,85×** (SSO revertato: neutro) | 4,72/0,39GB = **12,0×** (invariato) | ~12:29/5:39 = **2,2×** | ~17/11,5 min = **1,5×** |
| WP-39 | 56,79/20,93 = **2,71×** ⭐ (fast-shutdown + sweep fast-path) | 4,20/0,435GB = **9,7×** ⚠️ maxrss stesso-giorno (old 8,9×; il +9% new = accounting MADV_FREE, picco reale identico — caveat WP-20) | 11:56/5:39 = **2,11×** | ~17,4/11,5 min = **1,5×** |
| WP-40 | 56,05/20,95 = **2,68×** ⭐ (GC marks in-object; old stesso-giorno 57,52 = 2,75×) | non rimisurato (maxrss MADV-inquinato; Object +8B/istanza ≈ +20MB teorici su picco multi-GB) | ~11:39/5:39 = **2,06×** | ~16,6/11,5 min = **1,4×** |
| WP-41 | 56,08/20,91 = **2,68×** (invariato: shim gc_note BOCCIATO su A/B 4 round, +0,62% consistente → revert; A/B odierno old 56,08 vs new 56,43) | non rimisurato (zero delta codice) | ~11:39/5:39 = **2,06×** (run31 resta baseline: zero delta codice) | invariato **1,4×** |
| WP-42 | 57,74/20,93 = **2,76×** ⚠️ giornata rumorosa: old stesso-giorno 57,56 = 2,75× → warm-up by-borrow **FLAT su 6 round** (keep, leva chiusa); riferimento resta ~2,7× | 4,75/0,37GB = **12,7×** raw maxrss (old==new nei 6 round; oracle di giornata basso 0,36-0,38) | run32 ~12:50 NOMINALE stessa giornata rumorosa; fail-set **byte-id a run31** → riferimento resta **2,06×** (11:39 WP-40) | invariato **1,4×** |
| WP-43 | 55,70/20,77 = **2,68×** (stadio 1 registri = infra spenta, A/B 6 round RUMORE ZERO vs old 56,38 = 2,71×; in linea col riferimento WP-40/41) | 3,73/0,44GB = **8,5×** raw maxrss ⚠️ non comparabile alle righe sopra: oracle di giornata alto (0,44 vs 0,37) e maxrss new/old MADV-rumorosi (old 3,31 con outlier 2,10; old==new entro rumore) — il riferimento strutturale resta ~12× | full-suite NON rilanciata (delta zero provato: bytecode byte-id + gate22 verde + A/B flat; run32 resta baseline) → riferimento resta **2,06×** | invariato **1,4×** |

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).
