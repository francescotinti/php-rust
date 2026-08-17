# TEAM-STRUTTURA S-151 (KLABNIK·HEJLSBERG·HOARE) — verdetti 3× CONCORDO CON EMENDAMENTI; i verbali individuali restano la fonte VINCOLANTE (citazioni per emendamento).

## §Convergenze (unanimi salvo nota, citate per emendamento)
1. **Perimetro A2 = chirurgia-first, NON tutto-il-monolite** — il conflitto atteso NON esiste:
   tranche obbligatorie = SOLO i moduli che A3 toccherà (touch-map dal census A1); host.rs/resto
   mappa Gemini = backlog non bloccante; anomalia da 4–6 a ~3 sessioni (Klabnik R2 · Hejlsberg
   Q1b · Hoare Q1b). «Chirurgia prima del refactor» REFUTATA (Hoare Q1: nessun unit test
   isolabile in 25,7k righe); «census dopo il refactor» respinta (Klabnik Q1: ritarda A3).
2. **Chiavi census SIMBOLICHE, mai file:riga** (canale+simbolo/opcode/helper), o A2 invalida A1
   (Klabnik R1 · Hejlsberg Q1a · Hoare Q1a); totale per canale = gate di invarianza
   «census-eco 0,000%» del refactor (Klabnik R1, KS-C).
3. **«Rischio Zero» Gemini Fase 1 REFUTATO**: FR1 = +3,00 ns/iter da delta puramente strutturale
   +3180 B/+26 bl; ogni tranche cambia il binario per costruzione (Klabnik R4 · Hejlsberg ref. 2 ·
   Hoare Q2, precedente S-104 bl 1101→0). I gate attendono derive ENTRO banda-layout, non zero.
4. **«Build dimezzata» Gemini NON FONDATA sotto CGU=1+fat-LTO** (Hejlsberg ref. 1); misura
   companion non-gate `--timings` pre/post A2 (Hejlsberg Q5-1, KS-H4 · Klabnik Q5-2).
   **SAME-CRATE vincolante**: vietati crate nuovi in A2 (Hejlsberg Q5-2); Fase 5 Gemini fuori
   ordine, veto WP-44 sta (Hoare R8).
5. **Sostituto della byte-identità = FASCIO di gate meccanici**: batteria rc=0 + corpus 1412×2
   ZERO-FLIP per NOME (anche un PASS nuovo è rosso, Klabnik R3) + fixture bilaterali + micro R=5
   solo-regressione soglia max(4; rumore; banda-layout) + disasm run_loop (istr/bl) con delta
   DICHIARATO (tutti); in più: inventario firme-simboli e census-eco (Klabnik R3), nm-census
   set+size e inventario attributi inline (Hejlsberg Q2, KS-H2 — run.rs ha never×3/always×6
   MISURATI da non perdere). 1 tranche = 1 commit revertibile al byte; pure code-motion, zero
   rinomini (Hejlsberg Q2).
6. **run_loop non è un modulo qualsiasi**: non si smembra in A2 (Klabnik R2); run.rs per ULTIMO
   col disasm come gate, o resta intero se il disasm muove (Hoare R4 · Hejlsberg: freddi prima).
7. **Coppia WP a OGNI pin nuovo, mai diradata di nascosto** (regola utente 2026-08-12; tutti).
8. **Tensione tetto-movimenti 1,27 s ≈ 3,4%: sciolta CONTRO Gemini, unanime** — il movimento va
   ESCLUSO dalla somma pro-A3; A3 si motiva su borrow/refcount-accesso/gc_note/alloc-props o
   non si motiva (Klabnik Q5-3 · Hejlsberg Q3 · Hoare R5).
9. **A3 CONDIZIONATA ai numeri, kill-switch convergente**: conteggio×prezzo unitario per canale
   ⇒ TETTO in secondi (binario census, ORM); soglia pre-registrata PRIMA di leggere i numeri;
   somma canali Object < ~15% del gap ⇒ A3 refutata, restano leve per NOME (Klabnik R6+KS-D ·
   Hejlsberg Q3+KS-H3 · Hoare R5+KS-H1). Object DISTINTO da Str/Array: se il churn 32% è
   dominato da Str/Array, ObjectsStore non lo tocca (Hoare R5).
10. **Sigilli A3** (base = Hoare R1–R3, spec più stringente; compatibile con Klabnik Q3 e
    Hejlsberg Q3): handle LINEARE NON Copy/NON Clone, mint solo dello store (schema VmGate),
    dup solo `incref`, morte solo `Vm::release(&mut Vm)` col dtor sincrono LÌ; drop-bomb in
    debug; sigillo: `derive(Clone)` su Zval NON COMPILA (variante Cell+Clone refutata, Hoare R1).
    Indice GENERAZIONALE, check ON in release (stale id = oggetto sbagliato silenzioso, Hoare R2;
    costo 1–2 branch da confrontare onestamente col flag RefCell — Hejlsberg Q3). **Divieto
    doppia rappresentazione**: flip atomico compile-driven (Hoare R3, KS-H4-Hoare).
11. **Rotture A3 da spec-are PRIMA**: re-entrancy (dtor/call_method_sync — senza RefCell il
    rischio migra da panic a bug logico SILENZIOSO; il census conta i siti re-entranti: tutti) ·
    spl_object_id con RIUSO id compatibile-PHP, generazione interna (Klabnik · Hoare) · weak refs
    su store a indici · GC-cicli/possible_roots (oggi leggono i conti Rc) · due regimi di
    conteggio al confine (Resource/Closure/Ref restano Rc, Hoare) · RetainSet server: store
    per-VM, NO bump-reset, sweep con promozione esplicita, binding output-capture sovrano
    (Hoare R6 · Klabnik R6). A3 = cambio di TIPO whole-program: il conteggio dei SITI di clone
    (44% inline in run_loop) è deliverable di A1 ed è la cifra di costo (Hoare Q5 · Klabnik R1).

## §Conflitti-e-dissensi
- **C1 — Cadenza pin/coppia (unico conflitto reale)**: Klabnik R3 e Hejlsberg Q2 = pin per
  SESSIONE impacchettando 2–3 tranche (4–6 coppie totali); Hoare Q2 = «ogni tranche promossa =
  pin nuovo ⇒ coppia». Maggioranza 2-1 per pin-per-sessione; da SCIOGLIERE in plenaria/utente —
  unanime che il costo si dichiara, non si dirada di nascosto.
- **C2 — Slack del ratchet A4**: Hejlsberg Q4 = cap esistenti +50; Klabnik R5 e Hoare R7 = cap
  = conteggi ODIERNI esatti, mai crescere. Maggioranza 2-1 per cap esatti; Hejlsberg a registro
  come minoritaria (lo slack legittimo lo dà la regola anti-slack sotto).
- Dissensi senza opposizione (additivi): census-eco (Klabnik) · nm-census/attributi inline
  (Hejlsberg) · KS-B Klabnik «modulo HOT → solo dentro A3» · Hoare KS-H3 (gen-check mai via alla cieca).

## §Spec-dente-A4 (UNIFICATA — le posizioni lo consentono, minoranza C2 a registro)
- **Sede: BATTERIA** (`cargo test --release`), file `crates/php-runtime/tests/loc_dente.rs`,
  telaio di rczval_funnel_dente (unanime; CI = specchio TARDIVO ~3 gg; pre-commit hook cintura
  facoltativa, MAI sede di record). **Misura**: `src.lines().count()` ≡ wc -l, NIENTE pattern
  componibili ⇒ auto-morso bea7ea3 escluso (Klabnik R5 · Hejlsberg Q4 · Hoare R7).
- **Regola nuovi**: file .rs fuori allowlist ⇒ ≤ 2.000 righe (unanime).
- **Ratchet**: allowlist `(path, cap, motivo)` = 21 conteggi ODIERNI verificati (lista in
  Klabnik R5; big5.rs GENERATO a cap fisso). Anti-slack: `cap − n ≤ 200` ⇒ chi snellisce abbassa
  il cap nello STESSO commit; cap solo in discesa; salita = diff dichiarato a verbale, mai
  silenziosa (KS-E); cap scendono a valle di ogni tranche promossa (Hejlsberg).
- **Collaudo in NEGATIVO nell'atto di armamento**: violazione sintetica che DEVE fallire, pena
  dente NON armato + incidente contato (Hoare R7, KS-H5; forge-silent-failure). **Timing**: il
  dente entra PRIMA della tranche 1 di A2 (Klabnik R5).

## §Priorità-per-l'ordine-S-151+
1. **A1 census tranche-5**: chiavi simboliche; canali per-TIPO (Object≠Str/Array): borrow ·
   refcount · gc_note · drop · clone-per-sito · alloc-props · siti re-entranti · oggetti
   cross-richiesta; conteggio×prezzo ⇒ tetti in secondi; soglia A3 (~15% gap) PRE-REGISTRATA
   prima di leggere i numeri (KS-D/KS-H3/KS-H1).
2. **Armare il dente A4** (spec sopra, controllo positivo) prima della tranche 1. **Pre-misure
   A2**: `--timings` pre-registrato (KS-H4-Hejlsberg); same-crate a verbale; partizione dalla
   touch-map A3, freddi→caldi, run_loop ultimo/mai.
3. **A2 tranche promo-gated** col fascio §Convergenze-5; kill-switch: revert al byte su
   flip/fixture (KS-A), 2 tranche rosse consecutive ⇒ stop e concilio (KS-H2-Hoare).
4. **A3 fuori agenda** finché A1 non produce i numeri (Klabnik R6 · Hoare R5); spec sigilli
   Hoare R1–R3 = base della futura chirurgia.
5. In plenaria: sciogliere C1 (cadenza pin) e ratificare C2 a maggioranza.
