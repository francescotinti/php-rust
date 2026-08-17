# Verbale KLABNIK — S-151 (lente: chiarezza, spec dei gate, testabilità)

VERDETTO: **CONCORDO CON EMENDAMENTI** sull'impianto A1..A4.

## Q1 — Sequenza A1→A2→A3
Giusta NELL'ORDINE, fragile nella GIUNTURA A1↔A2: un census «per SITO» chiavato
file:riga viene invalidato dal refactor che sposta i siti. Ratifico solo con
R1 (chiavi simboliche + census-eco). L'alternativa «census dopo il refactor»
ritarda la decidibilità di A3 di 4–6 sessioni: respinta. L'interleaving è
recepito come R2 (perimetro del refactor guidato da ciò che A3 toccherà).

## Q2 — Gate per tranche (A2) e cosa sostituisce la byte-identità
La byte-identità è comunque IMPOSSIBILE qui (spostare item tra moduli cambia
simboli e partizione CGU), oltre che vietata. Il sostituto non è UN gate ma un
fascio, ciascuno meccanico (R3). E refuto il «Rischio Zero» di Gemini Fase 1:
FR1 ha appena prezzato +3,00 ns/iter un delta PURAMENTE strutturale
(+3180 B/+26 bl) — ogni tranche può costare layout, quindi micro-guardie e
coppia non sono cerimonia (R4). Coppia WP: il mandato chiede se diradarla a
fine refactor — NO senza ratifica esplicita dell'utente (regola 2026-08-12:
coppia a OGNI pin nuovo; il diradamento estrapolato fu già respinto). La
mitigazione lecita è: pin per SESSIONE, non per tranche.

## Q3 — Forma di A3 e numeri che la rendono DECIDIBILE
La forma «store centralizzato + refcount esplicito senza RefCell + props
inline» rispetta il vincolo distruttori solo se il decref è l'UNICO punto di
morte. Rotture da spec-are PRIMA (R6): spl_object_id (PHP RIUSA gli id dopo la
free: chiavi generazionali pure divergono osservabilmente — serve riuso
id-compatibile); weak refs (WeakReference/WeakMap su store a indici); ordine
__destruct e GC cicli (possible_roots oggi legge i conteggi Rc); RetainSet
per-richiesta vs reset-arena (binding rule output-capture/request_end);
re-entrancy (destructor o call_method_sync dentro una scrittura di proprietà):
senza RefCell il rischio migra da panic a bug logico silenzioso — il borrow su
`&mut vm.objects[i]` va rilasciato prima di ogni rientro, e il census deve
CONTARE i siti re-entranti. Decidibile = tetto in SECONDI per canale (borrow ·
refcount · gc_note · drop · clone-per-sito · alloc-props) su binario census,
workload ORM, + attesa pre-registrata col PAVIMENTO dichiarato (lezione S-150).

## Q4 — Spec del dente A4 (proposta meccanica, sede: BATTERIA)
Sede: **batteria** (`cargo test --release`), file
`crates/php-runtime/tests/loc_dente.rs`, stesso telaio di scansione di
`rczval_funnel_dente.rs` (rs_files su tutto `crates/`, sanity >100 file).
La CI la eredita come allarme TARDIVO (backlog ~3 gg, ~20 min/commit: non
morde in tempo — confermato); il pre-commit hook NON è sede di record
(bypassabile, e i cap vivrebbero in due giudici). Spec (R5):
- Misura: `src.lines().count()` (≡ wc -l a meno del newline finale; niente
  pattern testuali da comporre ⇒ il rischio auto-morso bea7ea3 non si applica
  per costruzione; il file stesso è nel perimetro e sta ≤2000).
- Regola 1 (nuovi): file .rs NON in allowlist ⇒ ≤ **2000** righe.
- Regola 2 (ratchet): allowlist `(path, cap, motivo)` coi cap = conteggi
  ODIERNI verificati (21 voci): vm/mod.rs 25704 · vm/host.rs 7626 ·
  vm/run.rs 6786 · php-runtime/tests/eval.rs 4773 ·
  php-builtins/tests/builtins.rs 4772 · lower/mod.rs 3838 · vm/dom.rs 3641 ·
  php-types/src/big5.rs 3372 (GENERATO: cap fisso, esente Regola 3) ·
  string.rs 2865 · file.rs 2758 · compile/expr.rs 2590 · date.rs 2458 ·
  bytecode.rs 2306 · preg.rs 2290 · php-types/src/memcensus.rs 2268 ·
  lower/class.rs 2168 · vm/arrays.rs 2167 · lsp_check.rs 2094 ·
  fileinfo.rs 2083 · php-server/src/worker_pool.rs 2074 · lower/expr.rs 2029.
- Regola 3 (anti-slack, ratchet DECRESCENTE meccanico): `n ≤ cap` E
  `cap − n ≤ 200` ⇒ chi snellisce DEVE abbassare il cap nello stesso commit;
  lo slack residuo non consente ricrescita.
- Regola 4: cap solo in discesa; un aumento = diff visibile sull'allowlist +
  dichiarazione a verbale (revisore) — mai silenzioso.
- Effetto dichiarato: anche i test-monoliti (eval.rs, builtins.rs) sono
  cappati ⇒ i test NUOVI nascono in file nuovi ≤2000. È l'intento, non un bug.

## Q5 — Cosa manca (e mandato inverso, Gregg)
(1) Il costo coppia-WP-per-pin delle sessioni di refactor non è a budget
nell'anomalia 4–6 sessioni: va dichiarato pin-per-sessione (R3) o chiesto
all'utente. (2) «Tempi di build dimezzati» (Gemini) è una promessa non
misurata: misura companion non-gate per tranche, o si tace. (3) Mandato
inverso: BT1 ha mosso ORM di 6+ s con UNA cura di fedeltà NOMINATA — il
soffitto «solo strutturale» non è provato; A1 può di nuovo trovare un canale
nominato economico, quindi A3 resta CONDIZIONATA ai numeri (KS-D).
Tensione obbligatoria (tetto movimenti 1,27 s ≈ 3,4% vs Gemini «azzerare i
movimenti ripaga»): si scioglie contro Gemini — il beneficio-vetrina di
ObjectId-Copy (movimenti gratis) attacca un canale già TETTATO al 3,4%; le
cifre 30–45%/−35% restano NON firmate; il caso A3 va costruito sui canali
borrow/refcount/gc_note/alloc-props che il census tranche-5 deve prezzare.

## Emendamenti
- **R1 (A1)**: chiavi del census SIMBOLICHE (canale + simbolo/funzione), mai
  file:riga; e il totale per canale diventa gate di invarianza del refactor
  («census-eco», repliche 0,000% già precedente S-147/S-149).
- **R2 (A2)**: perimetro chirurgia-first: tranche OBBLIGATORIE = solo i moduli
  che A3 tocca (object/props/gc_note/exec-objects da mod.rs+run.rs);
  host.rs e il resto della mappa Gemini = backlog non bloccante. Riduce
  l'anomalia da 4–6 a ~3 sessioni. run_loop NON si smembra in A2.
- **R3 (A2, gate per tranche)**: 1 tranche = 1 commit revertibile al byte;
  gate: inventario simboli identico pre/post (script: set di firme fn, cambia
  solo il prefisso di modulo) · batteria rc=0 dal comando · corpus 1412×2
  ZERO-FLIP per NOME (anche un PASS nuovo è rosso: un refactor che «cura» un
  test ha cambiato semantica) · fixture bilaterali · micro R=5
  solo-regressione · census-eco 0,000% · disasm run_loop (istr/bl) a verbale
  come tripwire dichiarativo. Pin per SESSIONE via pin-*.sh; a ogni pin:
  coppia WP + gate ORM 16 nomi + hk 0E/0F (il vm tocca ref/arg/reflection
  per costruzione: la ricetta ORM è OBBLIGATORIA qui).
- **R4 (A2)**: refutato «Rischio Zero» Fase 1 Gemini (precedente FR1:
  +3,00 ns da +3180 B/+26 bl strutturali). Ogni tranche è un atto misurato.
- **R5 (A4)**: spec del dente come in Q4; sede batteria; l'allowlist nasce coi
  21 conteggi verificati oggi, non «~». Il dente entra PRIMA della tranche 1.
- **R6 (A3)**: A3 non entra in agenda finché il census non produce: tetti in
  secondi per canale su binario census (ORM) · conteggio siti re-entranti ·
  spec id-reuse spl_object_id · piano weak/GC-cicli/RetainSet. Attesa con
  pavimento pre-registrata prima della prima riga di codice.
- **R7 (metodo)**: ogni sessione di refactor dichiara «leve spedite: 0»
  (anomalia accettata ma CONTATA, REGOLE §8); niente chiusura di fronte sul
  verdetto delle singole tranche.

## Kill-switch pre-registrabili
- **KS-A**: flip corpus (qualsiasi direzione) o fixture non bilaterale su una
  tranche → revert del commit al byte + incidente contato.
- **KS-B**: micro fuori soglia (max(4; rumore; banda-layout)) su 2 tentativi
  dello stesso modulo → modulo dichiarato HOT: si rifattora solo dentro A3.
- **KS-C**: census-eco ≠ 0,000% → la tranche NON è code-motion: revert, o si
  ridichiara come LEVA con criterio pieno pre-registrato.
- **KS-D**: Σ tetti canali Zval dal census (binario census, ORM) < soglia da
  fissare in fase 2 (proposta: 15% del gap) → A3 forma piena REFUTATA, si
  torna a leve per NOME.
- **KS-E**: cap del dente in salita senza emenda dichiarata → batteria rossa
  per costruzione (nessun atto umano richiesto).
