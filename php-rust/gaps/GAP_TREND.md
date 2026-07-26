# GAP_TREND — tabella cumulativa del gap perf oracle↔phpr (file unico VIVO)

> Convenzione (decisione utente 2026-07-25): i `REPORT_GAP_<N>.md` contengono
> SOLO le misure della sessione N; il trend cumulativo vive QUI, una riga
> nuova a ogni chiusura di sessione.
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
| WP-39 | 56,79/20,93 = **2,71×** ⭐ | 4,20/0,435GB = **9,7×** ⚠️ maxrss (accounting MADV_FREE) | 11:56/5:39 = **2,11×** | ~17,4/11,5 min = **1,5×** |
| WP-40 | 56,05/20,95 = **2,68×** ⭐ | non rimisurato | ~11:39/5:39 = **2,06×** | ~16,6/11,5 min = **1,4×** |
| WP-41 | 56,08/20,91 = **2,68×** (invariato) | non rimisurato | ~11:39/5:39 = **2,06×** | **1,4×** |
| WP-42 | 57,74/20,93 = **2,76×** ⚠️ giornata rumorosa | 4,75/0,37GB = **12,7×** raw maxrss | riferimento **2,06×** | **1,4×** |
| WP-43 | 55,70/20,77 = **2,68×** | 3,73/0,44GB = **8,5×** raw maxrss ⚠️ non comparabile | riferimento **2,06×** | **1,4×** |
| WP-44 | 55,21/20,785 = **2,66×** (stadio 2 registri BOCCIATO ×3) | 4,78/0,476GB = **10,0×** raw maxrss | riferimento **2,06×** | **1,4×** |
| WP-45 | non rimisurato | **11,9×** PEAK FISICO (nuova metrica ufficiale) | riferimento **2,06×** | **1,4×** |
| WP-46 | 59,39/20,82 = **2,85×** ⚠️ +7,0% TENUTA | 4,55/0,37GB = **12,3×** peak fisico | run33 ~21:00 = **3,71×** ⚠️⚠️ | ~26/11,5 min = **2,3×** |
| WP-47 | 63,60/21,21 = **3,00×** di giornata ⚠️ | **1,77/0,41GB = 4,3×** ⭐⭐⭐ | run34 ≈19:20 = **3,42×** | ~23/11,5 min = **2,0×** |
| WP-48 | 61,05/20,86 = **2,93×** (+0,04% flat) | **1,747/0,395 = 4,42×** peak fisico | run35 ≈19:10 = **3,39×** | ~24/11,5 min = **2,1×** |
| WP-49 | 63,69/21,18 = **3,01×** (A/B −1,19%) | **1,715/0,394 = 4,35×** (−2,22%) | run36 ≈13:55 = **2,46×** ⭐⭐ | ~18/11,5 min = **1,6×** |
| WP-50 | 59,84/20,935 = **2,86×** (leve flat) | **1,755/0,394 = 4,45×** (+2,64% attribuito) | run37c ≈13:37 = **2,41×** ⭐ | ~18/11,5 min = **1,6×** |
| WP-51 | 59,17/20,82 = **2,84×** ⭐ (A/B +0,01% flat, guardia ok) | **1,721/0,396 = 4,34×** ⭐ (A/B −0,32%) | run39 ≈13:03 = **2,31×** ⭐⭐ (old 2,44× stessa notte; classify fuso −4,1% + boundary-collect gratis) | ~17/11,5 min = **1,5×** |
| WP-52 | 58,84/20,82 = **2,83×** ⭐ (A/B −0,57%, new 6/6) | **1,720/0,395 = 4,36×** (A/B +0,61% = +10,4MB, size-class del +8B/nodo — tenuta e verbalizzata) | run40 ≈12:05 = **2,14×** ⭐⭐ (old 2,32× stesso-giorno = −7,9% raw; in-node marks: classify −42,7% + cold-box Object −56B/ist) | ~16,3/11,5 min = **1,4×** |
| WP-53 | 59,05/20,90 = **2,83×** (A/B −0,88%, new 6/6 stretti) | **1,628/0,376 = 4,33×** (A/B −0,69% — guardia Fase 2 ≤+2% con margine) | run41 ≈12:19 −0,2% raw vs old stesso-giorno; **riferimento resta run40 2,14×** (giornata +2,1% d'ambiente; Fase 2.1+2.2: −40,2M DerefTop + Sweep-elision, −5,66% dispatch = ⭐ solo −0,9% CPU) | ~16,5/11,5 min = **1,4×** |
| WP-54 | 54,51/20,90 = **2,61×** ⭐⭐ minimo assoluto (A/B **−7,42%**, new 6/6 netti — reflect re-key (decl,mname): hit-rate 11,7%→96,7%) | **1,564/0,376 = 4,16×** ⭐ (A/B −5,79%: descrittori inherited collassati) | run43 ≈11:57 = **2,12×** ⭐ (−2,71% vs old stesso-giorno, old replica run41: ambiente stabile; residuo vs WP-40 2,06× ≈18s) | ~17/11,5 min = **1,4×** |
| WP-55 | 55,06/21,07 = **2,61×** (A/B FLAT: mediane +0,16%, r2-6 −0,04% — leva full-only) | **1,566/0,377 = 4,15×** (A/B **+1,27%** = +8B/stringa 5,7MB + slack append ~14MB — guardia ≤2% ok, tenuto) | run44 ≈11:57 = **2,11×** ⭐ (**−2,56%** vs old stessa-sera 735,7s — PhpStr growable + append-in-place `.=`: probe 250×→=oracle; ⚠️ deriva pomeriggio→sera ~2,6%: fa fede solo la coppia; residuo vs WP-40 ≈14s) | ~16/11,5 min = **1,4×** |
| WP-56 | 54,665/20,945 = **2,61×** (A/B **−0,33%**, new 5/6 — pilota GRATIS in CPU, checkpoint ≤+2% superato alla prima sessione) | **1,536/0,376 = 4,08×** ⭐ (A/B **−1,79% = −28,1MB**, 6/6 separazione pulita — indice keyless PhpArray, Key duplicata eliminata) | run45 ≈11:38 = **2,06×** ⭐⭐ NUOVO MINIMO (**−2,66%** vs old stessa-sera 716,9s=replica run44 alla cifra; residuo vs WP-40 CHIUSO; churn indice −296MB/run; ⚠️ banda census −25..−45% NON verificata: estimatore arr 5,7× — reached −7,4%) | ~16/11,5 min = **1,4×** |
| WP-57 | non rimisurato (sessione di QUOTA, binari parità intatti = phpr-wp56) | non rimisurato — **ri-quota su metro ESATTO: arr peak 66,4MB = 4,3% del fisico 1.536MB** (l'estimatore WP-55 "39% del proxy" era 6,0× over; tranche 2 arena ⇒ banda onesta −10..−20MB ~−1% fisico) | riferimento **2,06×** — quota `.=` non-locali sul FULL: 48.934 ev / 0,70MB lhs ≈ **23µs** ⇒ canale MORTO, fuso esteso NON si apre | **1,4×** |
| WP-58 | 54,84/21,04 = **2,61×** (A/B **−0,13% FLAT** — guardia pilota ≤+2% ok) | **1,602/0,394 = 4,07×** ⭐ (A/B **−1,05% = −17MB**, dentro la banda −10..−20MB — dieta header 104→96B/array + scan-mode ≤8 slot + block arena; mechanism-check leva C alla cifra) | run46 ≈11:54 = **2,11×** ⚠️ (**+1,0..+2,5%** vs old stessa-sera 696,6s raw+troncamento — regressione SOLO-full, media flat, TENUTA no-revert; fail-set ×2 byte-id a run33 = fix yield_from VALIDATO; cumulato Fase 3 full resta −) | ~16/11,5 min = **1,4×** |
| WP-59 | non rimisurato (sessione di MISURA, parità intatta = phpr-wp58) — **LA MAPPA: frag mimalloc 2% al picco (ipotesi concilio FALSIFICATA), non-censito VIVO 65% = compile-side (HIR seeds ~2/3 + payload op ~1/3, ≈800MB; `--list-tests` da solo = 818MB); leak template-include CONFERMATO; obj peak de-fantasmato 56,1→48,7MB (fix next_obj_id, recon 22.141==22.141)** | non rimisurato — **4,07×** riferimento | riferimento **2,11×**; Ob.3 stessa-sera metro NUOVO (tree-user time -l): new **790,83s** (fail-set 88 BYTE-ID a run33 ✓), pool-off **−0,56%** (unico costo identificato ⇒ ⚖️ verbale revert B), scan4 **+0,66%** (scan-mode ASSOLTO, fail-set 88 byte-id); spread serata ±0,6% ⇒ la banda WP-58 +1..+2,5% era in buona parte rumore del campionatore | **1,4×** |

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).

Dettaglio e contesto di ogni riga: `gaps/REPORT_GAP_<N>.md` (per-sessione).
