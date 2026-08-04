# Team «giudice» — Concilio WP-99 (relatore)
Fonti: verbale-3-klabnik.md, verbale-4-hejlsberg.md. Nessun altro verbale letto.

## 1. Convergenze — lista MINIMA che sblocca «gate PASS» e il collaudo dello strumento

Le due sedie convergono sullo stesso principio, già lezione di S-97.1: **nessuna evidenza senza controllo positivo che il dente ha morso**.

Riparazioni del giudice (manifest), minimo per il claim «gate PASS»:
- **M1 (A-KL-99-1)**: delibera che ripari il manifest — WP_SESSION_97 a `judge=yes` (o carta A-SK-71 emendata), WP_SESSION_94/95 declassate, e per i tre `.out` di S-97.0 (`micro-baseline`, `arith-decomposition`, `ha2-sweep`) o la riga o l'esclusione DICHIARATA di classe. Vincolo KS-KL-99-2: senza questo, la riga «gate cifre PASS» NON si scrive nel report di chiusura S-98.
- **M2 (A-KL-99-4)**: gli `.out` flag-ON devono autodescriversi (stato del flag nell'header, non solo nel nome file).

Collaudo dello strumento census (pin 9,87 ns = grado VERDICT su strumento mai ri-collaudato dopo WP-44):
- **M3 (A-KL-99-2)**: smoke flag-ON in CI di sessione: `PHPR_REG_LOWER=1` su arith_small + controllo positivo `PHPR_DUMP_OPS` che DEVE contenere `BinarySSDst`.
- **M4 (KS-KL-99-1)**: nessun futuro `.out` VERDICT flag-ON senza parità corpus per NOME flag-ON sullo STESSO albero.
- **M5 (A-KL-99-5)**: il test flag-off non si auto-skippa in silenzio se l'ambiente esporta la variabile: fallire rumorosamente.

## 2. Batteria fedele al punto di pipeline di produzione

RC-1 (Hejlsberg): `lowered()` del harness applica il pass POST-cessione WP-65 (`{main}.slot_names` vuoti ⇒ `fold_slot` non matcha), mentre la produzione lo applica in `compile_body` PRE-cessione. I 13 snippet top-level sono quindi potenzialmente vacui sulle gambe `{main}`.

**Nota di contesto vincolante**: il census di PRODUZIONE (CLI, flag-on) HA foldato il top-level di arith_small (19→11 op/iter, dump con BinarySC/CmpJmpSC). La vacuità eventuale riguarda la COPERTURA DEL TEST, non il comportamento di produzione.

Piano (A-HE-99-1 + A-KL-99-3, convergenti):
- **B1 — chiusura del buco**: spostare la batteria sul funnel VERO (pass invocato al punto di produzione, env-flag in-process) E asserire ≥1 forma fusa in `{main}` — il controllo positivo, non l'uno o l'altro. Un semplice assert sul modulo post-cessione non basta: pinnerebbe il punto sbagliato. KS-HE-99-3: finché B1 non è verde, le gambe `{main}` non contano come evidenza.
- **B2 — estensione Klabnik**: target a metà finestra, finestra multi-linea + parità riga del warning, Spaceship const-first non-fold, TypeError operand-order nei due ordini, guardia u16 esercitata, test lowered() ≡ funnel sull'insieme dei corpi (property hook inclusi).

## 3. Conflitti

Nessun conflitto: perimetri disgiunti (Klabnik = giudice/manifest/strumento; Hejlsberg = pipeline/batteria) e principio comune. Unica tensione minore: Klabnik chiede più casi nella batteria attuale, Hejlsberg la vuole spostata di punto di pipeline — l'ordine giusto è B1 PRIMA di B2 (estendere una batteria vacua è lavoro sprecato).

## 4. Priorità S-98.0 (regola di ammissione: entra SOLO ciò che blocca il prossimo passo sull'oggetto; timebox mezza sessione)

1. **M3+M5** (smoke flag-ON + no-skip) — **BLOCCA**: il prossimo passo (H-B1 e ogni misura flag-ON) poggia sul census; senza collaudo il pin 9,87 ns e i futuri numeri non sono ammissibili (M4).
2. **B1** (batteria al punto di produzione + assert `{main}`) — **BLOCCA**: ogni evoluzione del pass (H-B*) sarebbe verificata da un test vacuo sui top-level.
3. **M1+M2** (manifest + header `.out`) — **BLOCCA il CLAIM, non l'oggetto**: senza, il report S-98 non può scrivere «gate PASS» (KS-KL-99-2). Costo minutario, si fa nella stessa mezza sessione.
4. **B2** (estensione batteria) — **NON blocca**: entra solo se avanza tempo nel timebox; altrimenti in coda per NOME.
5. **M4** (KS parità flag-ON stesso albero) — regola permanente da verbalizzare, costo zero.

Timebox complessivo apparato: mezza sessione; il resto di S-98.0 va all'oggetto (programma H-B1 con criterio in ns/op derivato dal 3×, A-KL-99-6).
