# s144-progettazione-B — via B su carta (deliberato S-143: B sola / B-poi-A; fascicolo COUNCIL_S143_REVIEWS vincolante)

## 1. Stato di fatto (dal SORGENTE, non a memoria)
- `Zval` = enum 14 varianti, **16 B / align 8**, assert compilate in
  `array.rs` (KS-HE-104-1/105-1). **`Option<Zval>` == 16 B: la NICHE è GIÀ
  ATTIVA** (assert a array.rs, riga «packed representation»). ⇒ Dei tre
  bersagli storici di B (taglia · niche · Rc-traffic) i primi DUE sono già
  incassati dalla rappresentazione corrente: **B rimirata = SOLO il ciclo
  clone/drop per-movimento** (coerente col reperto S-143 zval_size=16).
- `Clone` DERIVATA (dispatch 14 bracci; heap = inc non-atomico: ZStr rc
  custom, gli altri `Rc`). Drop = glue automatica per-variante + trampoline
  `drop_bounded`; Hejlsberg a verbale: ~34,5% dei subtree Zval-glue in
  vm_inline.
- `gc_note` per-MOVIMENTO (vm/mod.rs:3919): guard `is_gc_container()`
  inline (scalari/str non pagano — H-C1a); container ⇒ `gc_note_slow` =
  borrow + flag + `Rc::clone` nel buffer. 238,6M eventi/run (dossier S-141;
  obj 56,5M). Zend annota solo possible-root a DELREF≠0, bufferizzato
  (Stogov R3).
- `ZStr` è GIÀ single-alloc con header `{rc,hash,len,cap}` a offset 0
  (S-124). `Array/Ref/Object/Closure/Generator/Resource/ArgPlace` = `Rc`
  standard (RcBox strong+weak).

## 2. Bersagli (canale · grade · cosa NON è bersaglio)
- **churn_zval ~4,4 s** (INDIZIO, un lato solo): prezzo per-movimento =
  dispatch variante + inc/dec + glue. Bersaglio primario.
- **memops ~5,4 s**: SOLO la parte Zval-move (attribuzione memcpy PENDENTE
  — Hoare R2); realloc/Vec-growth (19,5M eventi, 9,6 GB) FUORI bersaglio.
- **nota GC per-movimento** (0,5–1,2 s + parte famiglia gc): VOCE PROPRIA
  del budget (Stogov R3), criterio proprio — non si somma a B1 di nascosto.
- NON bersaglio: taglia (già 16), niche (già attiva), SSO/PhpStr (veti),
  dispatch (~9–10 ns invariante, S-103), stringhe/array condivisi.

## 3. Fette (compiler-driven; ciascuna nasce col SUO criterio ≤10 righe)
- **B1 «uniform-rc»**: ogni variante heap con header refcount a offset
  fisso (estensione del precedente ZStr S-124: `ZArr`, `ZRef`, `ZObj`, …)
  ⇒ clone/drop = 1 test is-heap + inc/dec SENZA dispatch per-variante;
  glue compressa (attesa secondaria: icache in run_loop — disasm bl-count
  prima/dopo obbligatorio, lezione H-C2). Ordine interno delle sotto-fette
  deciso dal MOLTIPLICATORE di movimenti (propget 29,9M · recv_clone
  14,8M · quota rczval dalla tranche-2), non dalle quote di allocazione.
  Safe-only: unsafe incapsulato nel crate types come ZStr (precedente
  S-124); le assert 16/8/niche restano il sigillo — una fetta che le rompe
  non compila.
- **B2 «possible-root-at-decrement»**: la nota GC si sposta dal movimento
  al DELREF che non arriva a 0, bufferizzata (mirror
  `gc_check_possible_root`). PERIMETRO SEMANTICO: il `__destruct` resta
  refcount-driven nel punto esatto di fine-vita (binding output-capture
  INTATTO); la §3.22 non si allarga: fixture a gate. Sotto-noting ritarda
  un destructor ⇒ il gate è il corpus per NOME, non solo la batteria.
- **B3 (condizionale)**: se la sonda-B dà churn memcpy-dominato ≥60%
  (clausola Klabnik), il collo è il CONTEGGIO di movimenti, non il prezzo:
  B1/B2 retrocedono e si apre il filone conteggi (TakeSlot S-140) — torna
  al concilio, non si improvvisa.

## 4. Prerequisiti che PREZZANO (senza questi numeri, NIENTE codice B)
1. **Sonda-B monobinaria** (Klabnik R1c, classe S-138): ripartizione del
   churn in memcpy / inc-dec / nota + prezzo pair alloc/free (voce c
   istruttoria S-143).
2. **Profilo ORACLE per famiglia** (Stogov R4, Gregg K3): comprabile =
   phpr−oracle per canale; **la promozione della prima fetta ASPETTA
   questo profilo** (criterio proprio alla sua apertura, s143-criterio p.8).
3. **quota_obj_max dalla tranche-2** (revisione S-143 az.1): se il
   maggiorante misurato entra in 25–40% ⇒ RICONVOCA (la regola S-143
   resta arbitra della via; B non si apre nel dubbio).

## 5. Giudici e kill-switch (armonizzati; dissensi citati, verbali vincenti)
- Giudici di fetta: famiglia churn_zval+memops dal profilo campionario
  (2 repliche) + micro churn (m-objchurn/objdatains) R=5 ABAB soglia
  REGOLE §3 + disasm bl-count su run_loop per fette che lo toccano.
- Giudice della scommessa: **coppia suite ORM 2/lato net** (il giudice che
  ha ucciso 4 micro-leve; Gregg R4/Bak KS2).
- **KS-B1** (Stogov): a fette spedite, churn_zval+memops NON calano ≥25%
  relativo E la coppia ORM resta in banda ±0,7% ⇒ B falsificata — entro
  3 sessioni dalla prima fetta spedita.
- **KS-B2** (Klabnik K4): 4 sessioni di fette con ORM fermo in banda ⇒
  revert al pin + riconvoca. (Dissenso Gregg K4: ≥5% entro 5 sessioni
  dalla prima promozione — si applica il PIÙ SEVERO che scatta prima.)
- **KS-B3** (Pedersen K3): segno 5/5 per fetta sui micro churn; ORM in
  banda a B completa ⇒ B ridimensionata, non estesa.
- **KS-B4** (sonda): churn memcpy-dominato ≥60% ⇒ B1/B2 non si aprono (B3).
- Gate semantici invariati: batteria · corpus 1414×2 per NOME · ORM 3E/13F
  · fixture bilaterali; B non tocca RetainSet/identità/destruct per
  costruzione — un fail NUOVO per NOME in weakrefs/destructor ⇒ STOP.
