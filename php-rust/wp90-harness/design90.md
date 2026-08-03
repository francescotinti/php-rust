# design90 — delibere S-90.0 p7 (Concilio WP-91): design v8 residui + status delibere ereditate

Ordine Concilio WP-91 §Sintesi p4 (coda design) e p7. NESSUNA riga di
runtime cambia con questo documento; l'attuazione di ogni item è la
prima sessione che ne consuma l'esito.

## A-MS46 — `mod probe` annidato (design, Matsakis WP-91 Q1)

**Problema**: il sigillo di visibilità A-MS43 è completo verso
l'ESTERNO ma sovradimensionato verso l'interno — il perimetro di privacy
è l'intero `mod implementation` (~1900 righe, test inclusi): ogni riga
futura del modulo può scrivere il flag senza errore di compilazione, e lì
giudica solo la belt (che A-MS48 ha rinforzato ma resta lessicale).

**Design vincolante**: flag + ProbeWindow + getter in un `mod probe`
annidato (~40 righe), export `pub(super)` del solo necessario; anche i
test del modulo muoiono a compilazione se scrivono il flag. Dopo il
refactor: A-MS41/45/47/48 restano come belt; KS-MS-91-1 decade a
compilazione. Attuazione = sessione engine (va col codice, non con gli
script — nota team-sigilli).

## A-TH53 — bando `paste!`/`concat_idents` (design, Hoare WP-91)

**Problema**: le macro a incollaggio di identificatori sono fuori
portata di QUALSIASI sigillo lessicale (grafia 7 di Hoare Q1) — la
chiusura vera resta A-MS27 (rustc giudice).

**Design vincolante**: pin sui Cargo.toml di php-runtime/php-server —
occorrenze di `paste`/`concat_idents` nelle dipendenze ==0 finché A-MS27
non chiude; limite lessicale DICHIARATO nel commento dei sigilli v8
(fatto in gate-lever-pins: residui dichiarati). Attuazione del pin
Cargo.toml = prossima revisione dei sigilli.

## A-TH55 — ordine canonico dei KIND intra-putord (design, Hoare WP-91 Q3)

**Problema**: A-TH51 v2 (non-decrescente) + adiacenza coprono la coppia,
ma la riga `supersede` dello stesso put può migrare da prima a dopo la
coppia senza morso (intra-put invisibile).

**Design vincolante**: in a_ds26/a_ds38, dentro ogni run di putord
UGUALE, asserire l'ordine canonico dei KIND: `supersede*` →
`main_evicted` → `evict-fp`. Attuazione = prossima sessione che tocca
quei test (stesso commit del dente, disciplina A-PP48).

## A-MS50 — STATUS: ATTUATA in S-90.0 (era design nel team-sigilli)

Visitor per-theap senza allocazione (capacità pre-riservata, bins ad
array fisso classe BinTab, overflow dichiarati, `&mut` con scope chiuso
prima della visita annidata) — commit 8da340c. KS-MS-91-3 sollevabile
sul canale v90; le righe mi_theap_* di m89 restano ADVISORY
(pre-A-MS50, mai riqualificate retroattivamente).

## A-BB60 / A-PP49 — invariati (design89)

Nested-guard v2 e `worker_dispatch path=` in-band restano DESIGN come
deliberato (design89.md); nessuna scomposizione additiva su coppie
annidate e nessun claim di composizione warm per-worker in grado VERDICT
sono stati emessi in S-90.0 (la campagna m90 non ne consuma l'esito).

## Deferred invariati

A-MS27 (rustc giudice) · A-PP18 · A-PP27 · A-AH38 — backlog, invariati.
KS-DS-80-3 invariata (mai innescata).
