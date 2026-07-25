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

⚠️ riga WP-36: NON è una regressione — l'old-binary (WP-35) rimisurato lo
STESSO giorno dà 61,1s (2,90×): la giornata di WP-35 era favorevole; il
confronto interleaved new/old dà phpr −0,5/−1% (rumore/flat).

⚠️ riga WP-30: phpr media in calo ASSOLUTO (82,4→80,7) ma l'oracle del giorno
gira −9% (23,0→21,0) → il rapporto sale per rumore dell'oracle, non per una
regressione phpr (2 coppie consistenti: 80,42/21,03 e 80,97/21,02).

Dettaglio e contesto di ogni riga: `gaps/REPORT_GAP_<N>.md` (per-sessione).
