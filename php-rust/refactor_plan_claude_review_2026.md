---
name: php-rust-refactor-plan
description: "PIANO refactoring architetturale phpr (segmentazione monoliti vm/mod.rs 24k righe, compile.rs 5.3k, lower/mod.rs 6.5k). Da eseguire in SESSIONI DEDICATE. Basato su analisi Gemini (analysis_and_suggestions_v5.md + refactor_plan_for_claude_2026.md) + rilievi Claude. SOLO spostamenti, 0 cambi logici, gate corpus phpt."
metadata:
  node_type: memory
  type: project
  originSessionId: 77b21d67-4404-4714-a21c-615f443cf5b6
---

# Piano refactoring architetturale phpr — SESSIONI DEDICATE

Origine: analisi Gemini in `php-rust/analysis_and_suggestions_v5.md` +
`php-rust/refactor_plan_for_claude_2026.md` (nel repo), riviste da Claude 2026-07-08.
**Obiettivo: segmentare i monoliti. SOLO spostamenti strutturali, ZERO cambi di
comportamento. Ogni fase = 1 sessione dedicata + gate corpus + commit.**

## Stato attuale verificato (2026-07-08)
- `vm/mod.rs` = **24.235 righe / 1.1 MB** (il monolite; ~200 metodi `ho_*` + `run_loop`).
  vm/ ha già estratti: arrays/calls/coroutines/dom/exceptions/oop/pdo.
- `compile.rs` = **5.358 righe** (HIR→bytecode; `expr` ~800 righe, `compile_class` ~400).
- `lower/mod.rs` = **6.588 righe**; lower/ ha già `stmt.rs`(23K)/class.rs/expr.rs/curl_consts.rs.
- `php-builtins`: lib.rs 62K, file.rs 85K, string.rs 76K, mbstring 44K, array 50K, date 51K.

## PRINCIPI NON NEGOZIABILI (regole di ingaggio)
1. **Zero bug-fix, zero cambi logici** — solo `mv`/estrazione. Se durante il refactor
   si trova un bug, ANNOTARLO e fixarlo in una sessione separata (non mescolare).
2. **Validazione = corpus phpt byte-identico**, NON solo `cargo check`. Prima di ogni
   fase: baseline `phpt-runner --isolate Zend/tests` (atteso 2316 al momento) +
   `cargo test -p php-runtime -p php-builtins`. Dopo: DEVE dare gli stessi numeri.
3. **`git mv`** (non `mv`) per preservare blame/history. Commit dopo OGNI sotto-modulo
   estratto (bisectabile).
4. **Lifetime intatti** (`'m` modulo, `'a`). I metodi spostati restano `impl<'m> Vm<'m>`
   / `impl<'a> FnCompiler<'a>` in un secondo blocco impl nel nuovo file.
5. **Un solo monolite per sessione** — non aggredire vm+compile insieme (troppo diff).

## SEQUENCING CONSIGLIATO (Claude — diverge da Gemini che mette vm 1°)
**Fase A = compile.rs come PILOTA** (rischio minore, valida il pattern), poi **Fase B =
vm/mod.rs** (massimo valore ma massimo rischio), poi **Fase C = lower/mod.rs** (parziale).

---
## FASE A — compile.rs → compile/ (PILOTA, sessione 1)
1. `git mv crates/php-runtime/src/compile.rs crates/php-runtime/src/compile/mod.rs`.
   Verifica `lib.rs` `pub mod compile;` ancora valido.
2. In `compile/mod.rs`: rendi `pub(super)` i campi di `FnCompiler<'a>` e `ProgramCtx<'a>`
   usati dai sotto-moduli. (FnCompiler ha molti campi: ops/lines/consts/loops/ctx/
   cur_class/exc_regions/finally_scopes/labels/static_vars/… — tutti pub(super).)
3. Estrai in sotto-moduli con `impl<'a> super::FnCompiler<'a>` + `mod xyz;` in mod.rs:
   - `compile/expr.rs`: metodo `expr` (~800 righe) + helper espressioni.
   - `compile/stmt.rs`: `stmt`, `block`, loop/try/finally, `StmtKind::*` (incl. StaticVar).
   - `compile/class.rs`: `compile_class`, `stub_class`, PropInfo build, static_props/consts.
   - `compile/func.rs`: `compile_fndecl`, `compile_body`, `stub_func`, `compile_default_thunk`,
     `compile_const_thunk`, `default_const_name`, `const_eval`.
   `cargo check -p php-runtime` dopo OGNI estrazione.
4. Gate finale: corpus Zend + cargo test. Commit.
   **NB**: i costruttori `Func` (compile_body/stub_func/prop-init/default-thunk/
   const_thunk/const_stub) sono ~6 e sparsi — se si spostano, tenerli coerenti.

## FASE B — vm/mod.rs → estrai gli ho_* (sessione 2, la GRANDE)
1. `crates/php-runtime/src/vm/host.rs` con `impl<'m> super::Vm<'m> { … }`. `mod host;` in
   mod.rs (accanto ad arrays/calls/…).
2. Sposta TUTTI i ~200 metodi `fn ho_*`. **Il macro `host_builtins!` RESTA in mod.rs**
   (genera il dispatch `b"name" => vm.ho_name(args)`; i metodi in host.rs sono in scope
   via impl Vm). NON spostare `run_loop` (~4k righe, hot-path) né i metodi core non-ho_.
3. **Esplosione visibilità**: rendi `pub(super)` i campi/metodi privati di Vm che gli
   ho_* toccano (classes, frames, statics, static_props, class_index, diags,
   reflect_object_bound, module, modules, generators, fibers, …) e gli helper
   (deref_clone/alloc_object/drive_to_return/find_method_reflect/run_value_thunk/…).
   Molti sono già pub(super) per gli altri sotto-moduli vm/.
4. `cargo check` iterando su use/visibilità finché verde. Gate corpus + cargo test. Commit.
   **Opzionale**: se host.rs diventa esso stesso enorme, categorizzare in
   `vm/host/{fs,net,string,array,reflect,pdo,json,…}.rs` (ogni file un impl Vm block).

## FASE C — lower/mod.rs (sessione 3, parziale, bonus)
- lower/stmt.rs ESISTE già. Estrarre ancora: `lower/decl.rs` (costanti, function defs,
  namespace) e le utility AST-mago più lunghe. Stesso paradigma pub(super) su
  Lowerer<'f>. Gate + commit.

## PHP-BUILTINS (sessione 4, bonus, indipendente dal runtime)
- `lib.rs` (62K) ospita impropriamente var_dump/print_r/gettype/is_* → estrarre in
  `var.rs`/`type_info.rs` (lasciare in lib.rs solo definizioni modulo + `add(...)`).
- file.rs (85K)/string.rs (76K): splittare solo se crescono ancora (`file/{io,stat,dir}.rs`).
- Dispatch resta la tabella `add(b"name", module::fn)` in lib.rs.

## ⚠️ NON FARE come parte del refactor meccanico
- **"Bridge ho_* → php-builtins"** (proposto in v5): è un cambio ARCHITETTURALE (confine
  VmCtx/API), NON uno spostamento. Molti ho_* necessitano stato VM (gen/pdo/reflect/
  coroutine) e non possono vivere in un crate puro. Progetto a sé, sessione separata,
  design-first. Non mescolare con la segmentazione.

## Checklist per-fase
- [ ] baseline corpus+cargo PRIMA (registrare i numeri)
- [ ] `git mv` non `mv`; commit per sotto-modulo
- [ ] `cargo check -p php-runtime` verde dopo ogni estrazione
- [ ] gate corpus Zend byte-identico DOPO (stessi pass/fail)
- [ ] `cargo test -p php-runtime -p php-builtins` verde
- [ ] 0 warning nuovi; lifetime intatti
- [ ] nessun cambio logico (diff = solo mosse + visibilità + use)
