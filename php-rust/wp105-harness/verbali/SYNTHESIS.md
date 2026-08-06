# Concilio WP-105 — SINTESI DI CONVERGENZA (su S-103 e programma S-104)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in S-103**: conoscenza NUOVA e nominata —
audit che ribalta un verdetto meccanico (A/B VOID, rerun R=7 in volo);
11 DropS + 3 DropC/iter ESATTI; cifra calls **1 alloc × 32,0 B** (con la
scoperta che il denominatore S-102 era doppio); generator-in-cycle
PROVATO perdere; §3.12 typed-LVALUE. Rapporti del giudice: **FERMI per
la seconda sessione** (prop 11,5). Gregg (mandato inverso): sessione
onesta, ma **soglia anomalia RAGGIUNTA (contatore = 2, non superata)**.

**(b) Contatore full/media**: WP-102 (1 sessione fa); S-103 runtime
parity-null dichiarato — Klabnik REFUTA la dichiarazione come non
verificata (vedi R-KL sotto): la definizione verificabile entra in regola.

**(c) Rischio d'oggetto più trascurato**: aprire la leva H-C2 col
predicato SBAGLIATO — il rischio è stato COLTO dal concilio (refutazione
capitale convergente, sotto). **KS-GR-105-1 (testuale, vincolante): se
S-104 chiude senza l'A/B della leva H-C2 ESEGUITO (qualunque verdetto),
contatore = 3 ⇒ anomalia DICHIARATA; WP-106 apre con riallocazione
obbligatoria fondamentali-first.**

## Verdetti fase 1 (9/9: nessun MI OPPONGO; tutti CON EMENDAMENTI)

Verbali VINCOLANTI in `verbali/verbale-*.md`; team in `verbali/team-*.md`.

**REFUTAZIONE CAPITALE CONVERGENTE (Hoare ∧ Bak, indipendenti)**: il
criterio H-C2 chiavava il fast-out su `is_gc_container` — ma il predicato
distingue container/non-container, NON trivial-drop: `Str`/`Resource`/
`Generator` rispondono `false` e hanno Drop NON banale ⇒ un fast-out
letterale LEAKA ogni stringa poppata, invisibile al giudice prop (quasi
solo Long) fino a corpus/WordPress. EMENDA D'OBBLIGO PRIMA di aprire la
leva (atto zero): predicato NUOVO **`is_trivial_drop`** (true SOLO
Undef/Null/Bool/Long/Double + classificazione ESPLICITA di
ArgPlace/WeakHandle, match esaustivo) + **`dispose(v)` unico** (scalare→
forget; refcounted-non-container→glue; container→note+glue; due
valutazioni del predicato sulla stessa morte = reject, KS-BA-105-2) +
`debug_assert!(is_trivial_drop ⇒ !is_gc_container)` + fixture
stringhe-in-Pop nel gate. **KS-HO-105-1 ≡ KS-BA-105-1: fast-out chiavato
su `!is_gc_container` = reject senza appello.**

Refutazioni maggiori per nome:
1. **Stogov ×2**: (i) §3.12 ha TRE regimi (strict_types e `.=` su typed
   CONSERVANO — parità già oggi; il censimento 4/4 era tutto nel regime
   op-throws+weak): il catalogo si corregge SUBITO; (ii) 🔵 la marca
   §3.13 porta (unit, line) — su include/eval la riga è giusta ma
   l'UNITÀ è del flush: divergenza NUOVA provata su HEAD (A-ST-105-3);
   §3.13 non è «chiusa» senza le fixture include+eval. A-ST-104-4:
   SCIOLTA (la refutazione S-103 regge).
2. **Klabnik**: «parity-null» solo DICHIARATO — definizione verificabile
   (A-KL-105-3): funzionale (gate sul pin di chiusura) E strumentale
   (micro sul PIN DI CHIUSURA, non su un binario diverso: la baseline
   S-103 è su d0b01362 ≠ f45a5d19 e va dichiarata nella riga). Regola
   **PIN-105**: pin bilaterale (anche ORACLE_PIN_ATTESO), sequenza
   atomica di chiusura build→hash→stash→gate, «gradato» senza stash
   contestuale = retroattivamente NON-gradato.
3. **Gregg**: banda tra-sere — 2 punti nello stesso GIORNO = 1 punto
   (≥3 punti su ≥2 giorni distinti); N del giudice EMESSO dal run (mai
   più denominatori a memoria: prevenzione del 2×).
4. **Leijen**: «32,0 B esatti» regge solo lato alloc via argomento del
   soffitto; il free è ASSUNTO (senza istogramma) — prima del SiteTag:
   free-hist + attese byte-per-tipo pre-registrate (40 B =
   `Rc<RefCell<Zval>>` ⇒ ret_cell ESCLUSO per layout: l'indiziato
   principale resta args-Vec). Lettura R=7: tetto R-coerente o
   auto-VOID; sign test CO-PRIMARIO (7/7 ⇒ p=0,0078).
5. **Hejlsberg**: `==1` globale conflaziona tre cause (per-CORPO);
   braccio `=0` senza controllo POSITIVO (recidiva); `size==16` non
   pinna align/repack (KS-HE-105-1: align+fingerprint nel criterio);
   banda-layout 0,67 = campione N=1, non citabile come banda.
6. **Matsakis**: «un arbitro mai visto fallire non arbitra» —
   mutation-check con ROSSO ARCHIVIATO per 19a/19b (e ogni
   fixture-arbitro) prima di usarle come gate di promozione
   (KS-MA-105-1: senza rosso, RC-MA-104 si riapre); fixture 19c hook.
7. **Pedersen**: perimetro del collaudo RISTRETTO nel registro
   («cross-richiesta stesso-worker», MAI «cross-worker»: cbE mai in
   interleave né a freddo); stash meccanico NEL launcher; symlink-docroot
   a catalogo; PIENO mai speso su 31aa7c2e — si grada PIENO il pin che
   porterà le cifre.

## Regola di LETTURA A/B R=7 (pre-registrata QUI, run in volo)

Composta team-misura (Leijen+Gregg), PRIMA del verdetto delle ~21:00:
1. audit del tetto: 51,96 è tarato su R=5 fase-1 — a R=7 il range è
   monotono non-decrescente ⇒ se il range sfonda, la magnitudine si
   legge su IQR/percentili, non sul range (il tetto range-vs-range è
   auto-VOID per costruzione);
2. verdetti CO-PRIMARI: magnitudine (|Δ mediane| vs banda fase-1 34,64)
   E sign test (7/7 B>A ⇒ p one-sided 0,0078 firma la DIREZIONE);
   direzione senza magnitudine NON autorizza bisect;
3. STOP: R=7 è l'ULTIMO tentativo full-peak — qualunque VOID ⇒ metrica
   inadatta, ridisegno per-fase (design A-LE-105-5) DOPO la leva H-C2,
   MAI un terzo rerun;
4. la lettura è atto BREVE d'apertura S-104; il ridisegno no.

## Ordine DEFINITIVO S-104 (regola di ammissione applicata)

1. **Verdetto A/B R=7** con la regola di lettura pre-registrata sopra.
2. **LEVA H-C2 — NON NEGOZIABILE (KS-GR-105-1)**, nella sua finestra:
   a. atto zero (~30′): emenda criterio — `is_trivial_drop` + `dispose`
      unico + align/fingerprint Zval (KS-HE-105-1) SCRITTI nel criterio;
   b. prefisso disasm (~30′, A-BA-105-2): `drop_in_place::<Zval>`
      outlined? l'esito RINOMINA la banda [8,22] (mai misurata al
      numeratore); A/B senza verdetto disasm = VOID;
   c. denti DENTI-105 nella finestra: mutation-check 19a/19b (rosso
      archiviato, pre-atto) + `==1` per-corpo + `=0` positivo + dente
      fast-out (panic su container nel fast path, A-MA-105-4);
   d. implementazione → **A/B DA SOLA** (promozione: Δ≥8 ns/iter a R=5;
      Δ∈[4,8) ⇒ R≥9 + segno stabile; caduta sotto max(banda-layout,
      rumore ~3) ⇒ registra e chiudi il braccio);
   e. gate pieno con regola PIN-105: fixture 13+5+19(+stringhe-in-Pop)
      + batteria + corpus 1417×2 + **coppia WP bimodale** (salda il
      debito S-103).
3. **H-D**: free-hist + attese byte-per-tipo (A-LE-105-1/2) → SiteTag
   residuo≡0 → attribuzione; leva SOLO da criterio suo.
4. **Igiene (timebox ½)**: catalogo SUBITO (tre regimi §3.12; unit
   §3.13 + probe include/eval; symlink-docroot; N emesso dal giudice);
   terzo punto banda su GIORNO distinto; A-HO-105-2/3/4 (assert issato,
   doc, braccio rosso); 19c/zoo hook se entra.
5. **BACKLOG per NOME** (non bloccano): A-MA-105-3 ledger fine-vita;
   banda-layout N≥3 (solo se Δ marginale, dentro la campagna);
   allineamento rc 0/1/66; design per-fase A-LE-105-5; grado PIENO
   server (A-PE-105-1/3/4) sul pin post-leva; 21,2% run_loop (resta
   nominato; il concilio lo subordina alla leva).

## Conflitti registrati

- Nessun conflitto sostanziale intra-team (leva: complementarità
  Hoare/Bak; fedeltà: convergenza «fedeltà o assenza consapevole»).
- Unica tensione: Gregg vuole la leva SUBITO (KS-GR-105-1) vs denti
  DENTI-105 che la precedono — composta: i denti vivono DENTRO la
  finestra della leva (non sono prefissi di sessione), e il KS scatta
  solo se l'A/B non viene ESEGUITO.
