# CONCILIO S-151 — MANDATO (cambio di rotta: fronte ZVAL-FIRST)

Convocazione: utente, 2026-08-17 sera. Mandato di ogni sedia: **REFUTARE, mai
benedire**. Fase 1 = verbali INDIPENDENTI (nessuna sedia vede le altre).

## Input (leggere dal repo `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/`)
1. `REGOLE.md` (processo vigente, 25 righe)
2. `NEXT_SESSION_WORDPRESS.md` (stato S-150 + ordine S-151)
3. `doc/gemini/20260817-gemini.md` (parere esterno da vagliare criticamente)
4. `PERF_MAP.md` (mappa misure; regola: rapporti PER workload, mai aggregato)
5. `sessions/WP_SESSION_150.md` (ultima sessione)
6. Dove serve: il CODICE (Serena per il Rust; vm/mod.rs 25.704 righe ·
   host.rs 7.626 · run.rs 6.786 — conteggi verificati indipendentemente).

## Oggetto da ratificare o refutare (rotta utente, sequenza in 3 atti)
- **A1 — Census tranche-5 @ pin s150** orientato al ciclo di vita Zval:
  partizione per canale (clone per SITO · drop · gc_note · refcount · borrow)
  + testa nuova hostcall + none.other 94,6M; probe NUOVO a sorgente s150 con
  ricetta ESATTA (env incluso); dentro: TETTO scommessa da rifondare +
  re-istruzione scarto census +3,2% (probe s149 conservato f3a111ac).
- **A2 — Refactoring dei monoliti a TRANCHE promo-gated** (mod.rs → sottomoduli;
  ~4–6 sessioni SENZA leve = anomalia dichiarata e accettata dall'utente).
- **A3 — Chirurgia Zval/Object COI NUMERI del census**: store centralizzato +
  refcount ESPLICITO senza RefCell + props inline. VINCOLO SEMANTICO
  (irrinunciabile): il timing dei distruttori è osservabile in PHP ⇒ MAI
  `ObjectId` «Copy senza refcount» alla Gemini §2.2-1.
- **A4 — Dente ANTI-RICRESCITA righe .rs** (soglia ~2.000): dove vive?
  FATTO FRESCO S-151: la CI locale replica un backlog di ~3 giorni
  (~20 min/commit, oggi è ai commit dell'era S-145) ⇒ un gate solo-CI non
  morde in tempo; il precedente `rczval_funnel_dente.rs` vive nella BATTERIA
  (cargo test) e morde a ogni promozione — e ha già mostrato il rischio
  auto-morso (pattern composto a pezzi, sanato in bea7ea3).

## Registro delle refutazioni GIÀ acquisite (vincolano: non ri-litigare, verificare che il piano le rispetti)
- TakeSlot chiuso in OGNI forma (S-147) · ponte slot-load 0,83× = ZERO codice
  sul ponte · canale movimenti TETTO 1,27 s ≈ 3,4% del gap ORM (tetto su
  binario census) · NaN-boxing a VETO senza census · registri ibridi
  (enum Operand a match runtime) refutati WP-44 (+1,28%).
- Cifre Gemini NON firmate dai nostri strumenti: «30–45% CPU in Rc/RefCell/
  memcpy», «-35% CPU su Doctrine», «40% del tempo in Rc churn». Il nostro
  census S-140 dice: profilo SUITE = CHURN 32% (clone/drop visibile
  multi-%), DIMPROP 6%, 44% dei clone INLINE da run_loop. Il TETTO movimenti
  1,27 s ≈ 3,4% sembra in tensione con «azzerare il traffico di movimento
  ripaga»: le sedie DEVONO pronunciarsi su questa tensione.
- Veti di metodo (NON riproporre, NEXT_SESSION righe 55-72): tra cui
  byte-identità come gate di edit .rs post-pin · promozione sotto banda ·
  componenti prezzate · pin/stash senza collaudo-nell'atto.

## Stato misure (pin s150, per contesto)
WP full mediana 1,781 (COMPATIBILE) · media 2,48–2,56 · compoff 1,86–1,89 ·
hf 2,55 · hk 4,3 · dbal 7,28–7,49 · ORM 7,10–7,15 · micro: arith 5,5 ·
prop 5,5 · calls 4,8 · str 4,3 · arr 3,3 · re 2,5 · hintcall 7,3 ·
dimread 4,3 · objchurn 6,7 · objmap 11,7 · evalcls 316,9 · refl 42,4.
Corpus congelato 1412×2 · batteria 1747/0/2 · gate ORM 16 nomi · hk 0E/0F.

## Domande OBBLIGATORIE (ogni sedia risponde a tutte, poi la propria lente)
- **Q1**: la SEQUENZA A1→A2→A3 è giusta? Alternative: census dopo il refactor
  (il refactor sposta i siti censiti?); chirurgia prima del refactor;
  interleaving (refactor solo dei moduli che la chirurgia toccherà).
- **Q2**: A2 promo-gated senza leve per 4–6 sessioni: quali GATE rendono
  ogni tranche sicura (batteria+corpus+fixture+micro? coppia WP a ogni
  tranche o solo a fine refactor? cosa sostituisce la byte-identità, vietata
  come gate post-edit)? Quale partizione in tranche minimizza il rischio?
- **Q3**: A3 «store centralizzato + refcount esplicito senza RefCell, props
  inline» — è la forma giusta? Cosa si rompe (weak refs, __destruct order,
  GC cicli, spl_object_id, Resource)? Quali numeri il census DEVE produrre
  perché A3 sia decidibile (non «utile», DECIDIBILE)?
- **Q4**: il dente A4: soglia, perimetro (file nuovi vs esistenti?
  ratchet decrescente?), sede (batteria vs CI vs pre-commit hook)?
- **Q5**: cosa manca dall'ordine S-151 che invalida il resto se trascurato?
  (Gregg, mandato inverso: cosa sappiamo oggi di phpr che ieri non sapevamo?)

## Output per sedia (FILE, non in chat)
Scrivere `wp151-harness/concilio/verbale-<cognome>.md`, ≤120 righe:
`VERDETTO:` CONCORDO / CONCORDO CON EMENDAMENTI / MI OPPONGO (sull'impianto
A1..A4) · emendamenti numerati R1..Rn (cosa/perché/misura o gate) ·
kill-switch pre-registrabili (condizione meccanica → atto). In chat SOLO una
ricevuta ≤80 parole: verdetto + titoli degli emendamenti.
