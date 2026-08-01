# WP_SESSION_85.md — S-85.0 "IL CANARY CHE DISCRIMINA" — ✅ 8 punti Concilio WP-86 + VERDICT85 PASS + ×W verdict-grade + pin peak RITIRATO

**In una frase**: abbiamo dimostrato con un esperimento mirato che ogni
worker del server PHP-in-Rust paga la propria quota fissa di memoria
(~7 MB a testa, quindi il consumo cresce in modo prevedibile col numero
di worker), e nel farlo abbiamo reso più severi i controlli automatici
che impediscono di pubblicare misure o conclusioni sbagliate.

**Data**: 2026-08-01 sera
**Scope**: ordine vincolante Concilio WP-86 §Sintesi (8 punti) eseguito.
Modello verificato all'apertura: Fable 5.
**Commit**: 675e7dc → e718f3f → 2c3d52a → 7ccfe90 → 368c91d → **d660fd7**
(tutti su main, pushati).
**Binari**: phpr **bf278d55fd5efb0a** (nuova baseline parità, stash
additivo `phpr-wp85`; bit mossi da A-TH32 mod gate + A-MS29 probe
lifetime-bound + A-TH35 guardia + A-DS30 sig + A-DL29 + A-PP30; corpus
1418 + refl 290 per NOME IDENTICI in battery-85pre); campagna a matrix
368c91d; union a8b65c0578c42fb7 · mem-census a3c901dfddd474c0.

## Ordine WP-86 eseguito (p1-p8)

| # | Esito | Commit |
|---|---|---|
| 1 | **Sanatorie**: KL-85-2 (3.605.572 B = 3,44 MiB/worker [derivata] dai sei raw time -l 82p; sorgente corretta: VP time -l, NON vmmap) · A-PP32 · A-MS30 (+sweep nome vecchio ==0) · A-BG37 · A-DS33 · A-MS31 | 675e7dc |
| 2 | **Battery/equivalenza v3**: A-SK40 (cifre-gate `--all` su TUTTI i MEASURE8[4-9] — buco GRAVE sanato NEL 15/15; unità case-insensitive, check per-FIGURA, companion VERIFICATO numericamente) · A-SK36 (riga PASS ANCORATA terminale+unica; .done solo-PASS con sha256(OUT) ricomputato — 4 percorsi REFUSE bite-tested) · A-SK37 (corner index --cached; claim PROVVISORIO fino a commit append) · A-SK39 (F16 pinna '1 passed'+rustc -V; dichiara test-bin) · A-AH39 (manifest nomina crates/php-runtime/) · battery-85pre.sh nuovo | e718f3f |
| 3 | **Sigilli v3**: A-TH32 micro-modulo foglia `mod gate` (campo privato al modulo: rustc giudica ANCHE dentro vm/mod.rs; pin classe VmGate( ==3 + region-check) · A-MS29 probe `vm_gate_probe(anchor: &())` lifetime-bound + sweep VmGate<'static> ==0 · A-TH33 grafie eluse ==0 (⚠️ awk -v mangia le backslash → `[.]`) · A-TH34 split per-GRAFIA (prod 2/2/1, selftest 0/7/2, decoy anti-swap) · A-TH35/KH86-3 guardia rientranza UC_EMIT_GUARD (put+get; test a_th35 MORDE) · KH86-1 `gate-binary-noprobe.sh` | 2c3d52a |
| 4 | **sig() esaustiva**: A-DS30 destructuring SENZA `..` di Module+CompiledClass (derived esclusi PER NOME; mappe/set ORDINATI); 3 mutanti nuovi MORDONO (deferred-body, enum case value, final-flag) con guardie anti-vacuità · A-DS31 ordine eventi per-put = API (doc UC_LOG_EVENTS + pin main_evicted<evict-fp nella metà armata di a_ds26/F16) | 7ccfe90 |
| 5a+6 | **Strumenti + eredità**: canary A-DL28 forma Leijen (fixture contenuto hello_pad85.php 9.276 B, calibrazione W=1 esatta) · A-DL29 burst-control VERDE · measure85-campaign/verdict85 (A-SK38 per-blocco) · A-PP31 sweep reqns nei pin (verdict83 escluso PER NOME) · KG-86-1 gate-slope-verdict.sh (4 denti) · A-PP30 pre-warm con include REALE · A-DS32 N/A dichiarata | 368c91d |
| 5b | **CAMPAGNA + VERDICT85 PASS** (battery 15/15 anchored, KH86-1 su ogni arm): vedi `wp85-harness/MEASURE85_RESULTS.md` — delibera peak ESEGUITA | d660fd7 |
| 7 | A-DL27: metà CONTATA scomposta al byte (residuo thread 7.343.135 B [derivata] + hello-own 6.842 B + pad-own 460.146 B — regalo del raw VOID); metà FISICA (A-DL31: mi_bin thr=, vmmap purge=0, ±5%) NON eseguita, dichiarata | — |
| 8 | ROADMAP: non ripresa (dichiarato) | — |

## Misure (verdict85.out, fail-closed)

- **VDL28 — PER-THREAD CONFERMATO (canary monolaterale)**: cal W=1:
  NET_H = 7.349.977 B = 7,01 MiB · NET_P = 7.803.281 B = 7,44 MiB; W=2
  una-richiesta-per-worker: hello→thr0 = NET_H ESATTO e pad→thr1 = NET_P
  ESATTO (byte-exact, attribuzione per PATH in-band). **×W PROMOSSO a
  verdict-grade** (KL-86-2/KB-86-2). NET_H identico al byte ai record
  WP-83/84.
- **VP — R=9**: pin 232±1 MOSSO confermato; r1 NOMINATO (232.079.360 B =
  221,3 MiB, wall 4,94s — stavolta è il MINIMO); spread r2..r9 =
  16.891.904 B = 16,11 MiB [derivata] NON attribuito ⇒ **pin identità
  RITIRATO** (KL-86-1); envelope max **252.526.592 B = 240,8 MiB**
  (SALITO vs 84).
- **Delibera peak**: ×W ACCETTATO verdict-grade; pin ritirato
  (envelope-only fino ad attribuzione spread); registry BLOCCATA su
  A-MS27 (KS-MS-86-2).

## 🔵 Scoperte

1. **KH86-1: il dente nm è necessario ma NON sufficiente** — il binario
   costruito con `php-runtime/vm-gate-probe` ha hash DIVERSO (b55e2f78 vs
   15fb6b46) ma NESSUN simbolo `vm_gate_probe`: il linker lo dead-strippa
   (nessun caller nel bin). Il discriminatore che morde è hash==matrix
   (cablato come seconda metà del gate). Refutazione del MECCANISMO, non
   dell'intento → Concilio WP-87.
2. **Il bracket del net avvolge SOLO `lower_source`** (vm/mod.rs:16121):
   il sito path→CString del piecewise VW è FUORI — il canary len≥384 di
   A-BB41 NON avrebbe mosso il net; operativa la forma contenuto di
   Leijen con calibrazione esatta. La verifica per NOME batte l'ipotesi
   di sede (di nuovo).
3. **La scomposizione additiva chiude al byte**: dal raw VOID (pad come
   ord=2): residuo one-time del THREAD 7.343.135 B + main-own per fixture
   (hello 6.842 B, pad 460.146 B) — somme ESATTE con le calibrazioni. La
   metà contata di A-DL27 è scomposta per costruzione.
4. **Una richiesta pulita banale NON semina entry include**: il commento
   storico di a_pp20 («request 1 seeds the prelude include entries») era
   FALSO — scoperto dal pin A-PP30 (t_warm>e0) che ha morso; il pre-warm
   ora fa un `include` REALE.
5. **Il parent dichiarato DOPO viene hoistato**: non va in `m.deferred`
   (la guardia anti-vacuità del mutante 4 ha morso); il late-bound vero
   esige il parent CONDIZIONALE.
6. **Il path del repo contiene uno spazio** ("Extreme Pro") — ogni parser
   che chiude con `(\S+)$` sul path è rotto silenziosamente; fix
   dichiarato nel verdict85.

## ⭐ Lezioni

1. ⭐⭐ **Due input identici non discriminano; due input calibrati sì** —
   l'identità al byte di WP-84 era compatibile con lo specchio; il canary
   monolaterale con Δ calibrato ex-ante l'ha falsificata in un run.
2. ⭐⭐ **Un protocollo rotto con contatori onesti insegna comunque**: il
   raw VOID (hello dispatchato 2×) ha regalato la scomposizione additiva
   del residuo — VOID per il verdetto, ORO per il modello.
3. ⭐⭐ **Il simbolo assente non prova la feature assente**: dead-strip ≠
   cfg-out; l'identità del binario (hash vs matrix) è l'unico giudice
   post-build.
4. ⭐ Un pin identità con spread tornato non si rifonda a colpi di R: a
   R=9 lo spread è ancora 16,11 MiB — prima si ATTRIBUISCE, poi si pinna.
5. ⭐ awk -v processa le backslash: `\.` nel pattern diventa `.` — i
   decoy dei pin l'hanno scoperto prima che mentisse in produzione.

## Residui / NON fatto (dichiarati, per NOME)

- **A-DL31 (metà fisica di A-DL27)**: mi_bin thr= + spiegazione
  phys_peak<phys (A-DL30, righe mi_proc fuori corpus fino ad allora,
  KL-86-3) + vmmap purge=0, chiusura ±5% — prossima campagna.
- **Attribuzione spread VP** (purge in-band, first-touch): senza, il pin
  peak resta envelope-only.
- **A-MS27** (CachedMain/CachedInclude): precondizione registry, backlog.
- **A-SK38 su finestre census steady**: cablata in verdict85 per i
  blocchi usati; le finestre MEASURED=30 con controlli arriveranno con la
  prossima VA — il template c'è.
- **A-PP18/A-PP27**: invariati (nessun twin-pair nuovo).
- **ROADMAP** ([[php-rust-todo-master]]): non ripresa, primo candidato
  WP-86(sessione).
