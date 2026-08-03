# Verbale sedia 3 — Steve Klabnik (Concilio WP-93, revisione S-91.0)

## VERDETTO: **GATE v2 NON VERDICT-GRADE** — 6 forge nuovi COSTRUITI, 6 LANDED (6/6), ognuno con controllo negativo che FALLISCE.

Il gate v2 ha chiuso i quattro forge WP-92 (T13-T16 mordono davvero). Ne ho
costruiti sei nuovi contro v2: **tutti passano**. Tre sono refutazioni capitali.

## Forge tentati (comando + esito)

Baseline: `bash wp81-harness/gate-measure-cifre.sh <doc>` (advisory, ~25 s,
corpus card=24232/budget 24232). Docs in scratchpad, repo mai toccato.

| # | Forge | Riga piantata | Esito | Controllo negativo |
|---|---|---|---|---|
| F1 | **Identità rev DECIMALI** (Q1) | `b_peak rivisto = 23.330.397 B per worker` | **PASS** (rc=0) | `23.330.398` → FAIL |
| F2 | **%ALLOW come lavanderia** (Q3) | `regressione CPU full: 2,8 volte la media` + `46,25 percento` | **PASS** | `2,9` → FAIL |
| F3 | **Unità incollata = troncamento** | `b_work rivisto = 1.024.999B per worker` | **PASS** | `1.024.999 B` (spazio) → FAIL |
| F4 | **Storia del budget** (Q2) | `outstanding al checkpoint: 23.999 allocazioni` | **PASS** | `23.998` → FAIL |
| F5 | **prov v2 su valore rotondo** (Q5) | `18.000.000 B [derivata: prov 18287875@…axum.83cr.lever.n2000.r1.log:3 − 287875@…:3]` | **PASS** (stampa `provenance-verified`) | — |
| F6 | **Giudice-COPIA** | copia patchata `wp81-harness/zzforge-judge93.sh --all` con doc perimetrale NON committato presente | **PASS verdict-grade** con `judge_sha=3b035cc737887991` = blob HEAD del giudice **pristine** | giudice pristine `--all` → FAIL |

**Misure a corredo.**
- Q1: 1431 antenati, **53 con prefisso decimale a 7 cifre, 32 a 8** → ≥53 valori
  fabbricabili in 1-9,9 M e 32 in 10-99 M **esattamente la finestra dei byte**.
  La normalizzazione italiana (`23.330.397`→`23330397`) porta la cifra dentro
  la porta: il residuo «raro» dichiarato in commento è una popolazione nota.
- Q2: 8 blob storici del budget, 7 valori (20097…24232); il budget è **autorità
  E sorgente di corpus**, non è output macchina, ed è già escluso dal pool prov
  (A-SK-73, T14d): contraddizione interna.
- Q5: chiusura same-file sulla finestra 15-22 M scesa da 46,25% a **1,270%** —
  ma sul sottospazio **pubblicabile**: multipli di 1 MiB **6/6 = 100%**, di
  0,5 MiB **13/13 = 100%**, di 100.000 22,5%. La chiusura uniforme è la metrica
  sbagliata: il forger pubblica MiB tondi, non interi a caso.
- Q4: `run_gate` (battery-91pre r.83) decide **solo sul codice di uscita**;
  ADVISORY-PASS esce **0** ⇒ indistinguibile da PASS (batteries 82/83/84-pre
  invocano senza `--all`). In advisory un **giudice sporco** produce solo NOTE.
- Q4 perimetro: `class_rx` non ammette sottodirectory ⇒ **questo stesso verbale**
  (`wp93-harness/verbali/*.md`) è fuori; fuori anche 75 .md committati, tra cui
  **FOOTPRINT_CPU_ROADMAP 53 cifre, COVERAGE 50, TODO 34, README 16** — i file
  PUBBLICATI su GitHub. E i verbali del concilio sono `judge=no`: le voci ALLOW
  2.8/46.25 sono cifre **non giudicate** promosse a costanti legali ovunque.

## Emendamenti (da A-SK-74)

- **A-SK-74** — porta rev decimali ABOLITA: niente `rev-parse` vivo; le identità
  decimali citate diventano righe committate `rev=<sha40>` pinnate per NOME.
- **A-SK-75** — `%ALLOW` è un'AUTORITÀ: ogni voce porta provenienza
  `path@riga` di un blob committato **e** la dichiarazione «non è una misura».
  **2.8 e 46.25 REVOCATE** (o resi `judge=yes` i verbali che le producono).
- **A-SK-76** — tokenizzazione fail-closed: un run di cifre seguito da
  `[A-Za-z]` non si TRONCA, si **rifiuta** (mai giudicare `1.024` per `1.024.999B`).
- **A-SK-77** — il budget esce dal corpus (fonte non-macchina), coerenza con A-SK-73.
- **A-SK-78** — **self-tether del giudice**: all'avvio `hash-object($0)` ==
  blob HEAD di `JUDGE_REL`, altrimenti REFUSE. Il `judge_sha` firma il codice
  che gira, non un omonimo.
- **A-SK-79** — il GRADO nel codice d'uscita: ADVISORY-PASS esce 64; `run_gate`
  pretende la riga `PASS … judge_sha=`.
- **A-SK-80** — perimetro per COMPLEMENTO: ogni `.md` committato sotto
  `php-rust/` (sottodirectory incluse) esige una riga di manifest, default
  `judge=yes`.
- **A-SK-81** — prov: operandi ETICHETTATI e della stessa grandezza (stessa
  chiave), non digit-run qualunque della stessa riga.

## Kill-switch

- **KS-SK-93-1** — nessuna PASS verdict-grade senza A-SK-78 dimostrato dal
  bite-test con copia patchata.
- **KS-SK-93-2** — i sei forge WP-93 diventano denti permanenti T17-T22; se uno
  non morde, il gate non è verdict-grade.
- **KS-SK-93-3** — nessuna voce ALLOW nuova senza A-SK-75; 2.8/46.25 fuori entro S-92.0.
- **KS-SK-93-4** — ADVISORY-PASS non chiude una riga di battery.

## Refutazioni capitali: **SÌ (3)**

1. «Un giudice patchato non può produrre un PASS anonimo» (A-SK-67) — **REFUTATA**:
   il PASS non è anonimo, è firmato con l'identità di un ALTRO file (F6).
2. «Il perimetro è ogni doc che pubblica cifre» (A-SK-71) — **REFUTATA**: 153
   cifre nei quattro doc più letti restano fuori.
3. «A-SK-69/A-SK-73 chiudono il forge prov» — **REFUTATA sul sottospazio che conta**:
   100% dei multipli di MiB nella finestra plausibile.

*Repo lasciato porcelain; nessun commit; forge rimossi.*
