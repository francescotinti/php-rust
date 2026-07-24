# REPORT_GAP_48 — gap perf oracle↔phpr, cumulativo fino a WP-48

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
| WP-44 | 55,21/20,785 = **2,66×** (= old e3c8e0b su giornata pulitissima, oracle 20,74-20,80 su 6 serie; **stadio 2 registri BOCCIATO in TRE forme e REVERTATO**: +1,17% v1 enum / +1,28% v2 enum-singola-risoluzione / +1,01% v3 raw-monomorfa, 18/18 round new>old → main resta il binario WP-43; epitaffio: contano i CORPI HANDLER CALDI, non lo stile di estrazione) | 4,78/0,476GB = **10,0×** raw maxrss (old di giornata; MADV-rumoroso, riferimento strutturale resta ~12×) | full-suite NON rilanciata (tree revertato BYTE-IDENTICO a e3c8e0b già gated; run32 resta baseline) → riferimento resta **2,06×** | invariato **1,4×** |

| WP-45 | non rimisurato (nessuna leva CPU; binario di parità = WP-43) | **11,9× su PEAK FOOTPRINT FISICO** (4,67-4,78G vs 0,39-0,40G, 18+3 run; `/usr/bin/time -l` "peak memory footprint" = nuova metrica ufficiale, MADV-immune) — **ATTRIBUITO (Fase 0 roadmap)**: ~3,08G cicli Rc irraggiungibili (gc_note object-only: cicli Ref/Array/Closure mai rooted) + ~1,2G non censito (rounding/tabelle) + 0,30G unit ritenute + 0,15G stato PHP. Bersaglio WP-46: root-tracking esteso modello Zend | full non rilanciata (nessun cambio al binario di parità) → riferimento **2,06×** | invariato **1,4×** |

| WP-46 | 59,39/20,82 = **2,85×** ⚠️ **REGRESSIONE +7,0% TENUTA per direttiva no-revert** (old dd148a0 stesso-giorno 55,50 = 2,67×; 6/6 round new>old, oracle 20,79-20,85; costo = 11 collect × classify su ~726k root VIVI + rooting container nel note-path, ripagati da freed 543 = nulla) — in cambio: **parità funzionale migliore di sempre** (corpus 1447→1421, famiglia gc 36→14, tutte le 18 probe gc oracle byte-id) | 4,55/0,37GB = **12,3×** peak fisico (old di giornata 4,43-4,46 = 12,0×; +2,5% = buffer ctr + walks) — **il bersaglio ~3G NON è caduto: collector operativo ma i root notati sono VIVI (mem-census: arr 3,113M/1,9G e str 23,8M/1,29G INVARIATI alla cifra). La diagnosi WP-45 "cicli irraggiungibili" va ri-attribuita: WP-47 = owner-tracer + root-walk esteso (FramePool, tabelle VM)** | run33: ~21:00/5:39 = **3,71×** ⚠️⚠️ (+80% vs 11:39 run31/32: il collect-walk — HashMap handles/in_edges/children dell'INTERO grafo raggiungibile a ogni collect — scala col grafo vivo della full; i collect "efficaci" riabbassano la soglia a 50k ⇒ ri-walk continui; recupero = classify a 2 passate / mark intrusivi / collect al confine test, DOPO l'attribuzione WP-47) — **fail-set BYTE-IDENTICO a run32** (88 nomi; 30.472 test 0E/2F/86W/73S) | ~26/11,5 min = **2,3×** |

| WP-47 | 63,60/21,21 = **3,00×** di giornata ⚠️ (+1,3% vs old e6af390 stesso-giorno 62,77 = 2,96×, 6/6 round new>old, TENUTO per direttiva: la ricostruzione dei descrittori evitti costa più di quanto il classify 2-passate recuperi sul media; giornata lenta, oracle 21,12-21,30 — il riferimento comparabile resta ~2,85× WP-46) | **1,77/0,41GB = 4,3×** ⭐⭐⭐ peak fisico (old di giornata 4,88-4,92G = 11,9×: **−63,8%, il più grande salto footprint della storia del progetto**) — attribuzione owner-level RIUSCITA: root-walk esteso a tutti i campi Vm riconcilia arr 100,0%/str 96,7%, owner = `reflect_method_info_cache` 456k entry/2,48G (mock PHPUnit = ClassId freschi, memo mai evitta) → epoch eviction cap 8192 (`fa100ad`) | run34 ≈19:20/5:39 = **3,42×** (da 21:00 = 3,71× WP-46, −8% da isteresi soglia + classify 2-passate `d684cd7`; riferimento WP-40 2,06× non ancora recuperato) — **fail-set BYTE-IDENTICO a run33** (88 nomi; 30.472 test 0E/2F/86W/73S) | ~23/11,5 min = **2,0×** |

| WP-48 | 61,05/20,86 = **2,93×** di giornata (+0,04% vs old ce4be6e stesso-giorno 61,02 = RUMORE, round 2/6; **guardia CPU ≤+0,5% della roadmap Fase 1 rispettata — prima leva memoria a costo CPU zero**; oracle 20,79-20,93) | **1,747/0,395 = 4,42×** peak fisico (old di giornata 1,756 = 4,45×, −0,52%; oracle di giornata basso 0,394-0,396 vs 0,41 WP-47 — confrontare i rapporti con cautela) — **Fase 1.1 shrink unit LANDED col mechanism-check più pulito della roadmap: canale unit −70.625.796 byte = il 100,0% ESATTO dello slack cap−len predetto dal contatore `unit_slack`** (299,9→229,3MB, 2046 moduli; slack = 23,5% del canale, il resto è bytecode len reale); il peak fisico ne vede ~9MB perché il picco è mid-run coi moduli non ancora tutti caricati (proxy-peak census −67,2MB conferma il meccanismo pieno) | run35 ≈19:10/5:39 = **3,39×** (≈ run34 3,42×: nessuna leva CPU applicata, rumore) — **ATTRIBUZIONE CPU CHIUSA (Ob.2): `classify_ms` 494,9s ≈ 8:15 su 1005 collect / 9,93M root / 14,16M freed = l'INTERO residuo vs 2,06× WP-40 (≈7:41); reflect-cache cap 8192 in thrash (890k miss / 16% hit / 108 eviction) ma SECONDARIO (ordine 10× sotto)** — fail-set **BYTE-IDENTICO a run33** (88 nomi; 30.472 test 0E/2F/86W/73S); RSS transiente mid-run 5431MB invariato da run34 | ~24/11,5 min = **2,1×** |

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).
