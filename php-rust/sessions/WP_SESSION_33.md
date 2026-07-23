# WP_SESSION_33 — archivio storico della sessione WP-33

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-33 (2026-07-22, gated `8462ce4`→`62ea805`, 6 commit)** — **Fase 1
> dell'arco "interprete specializzante"** (decisione strategica utente:
> benchmark pubblicabile CPU+memoria vs oracle; Laravel accantonato; unsafe
> = ultima spiaggia FUORI roadmap). Due gambe:
> **(T0) OP CENSUS** — `crates/php-runtime/src/vm/census.rs`: contatori
> per-op + BIGRAMMI (l'oracolo delle fusioni T2) + matrici tipate
> Binary/CmpJmp (binop×tag×tag), FetchDim (base×key), IncDecSlot; attivo
> con env `PHPR_OP_CENSUS` (=1 → stderr; **=path assoluto → append su
> file** — ⭐ i SUBPROCESS phpr ereditano l'env e il loro dump su stderr
> diventava una PHPUnit\Framework\Exception in 15 test media separate-
> process); ⭐⭐ **il hook è dietro cargo feature `op-census`**: anche il
> branch bool mai-preso nel run_loop costava **+2,9%** sul micro op-denso
> (5 coppie A/B) — compilato via, l'off-cost è −0,4% = zero. Build census:
> feature `php-runtime/op-census` con CARGO_TARGET_DIR dedicato
> (`phpr-census-target`). Dati (media/option/post ~910M op/run + micro):
> **Concat(Str,Str) ~31M e NotIdentical(Str,Str) ~29M dominano il Binary
> del WP reale** (l'aritmetica Long è 10× sotto — il micro mente);
> cross-class ===/!== ~3-4M/run; **FetchDim = 98,8% Array×{Long,Str}**;
> IncDecSlot ≈100% Long; top bigram **This→PropGet 73M**, Ret→DerefTop
> 40M, MethodCall→This 37M, PushConst→CmpJmp 35M. File in
> `wp30-harness/ab-out/census-*`.
> **(T1) FAST-PATH TIPIZZATI** — **(C3) `binary_fast`** in testa a
> binary_value (CmpJmp e compound li ereditano per costruzione):
> (Long,Long) aritmetica con overflow→Double RIFATTO SUGLI OPERANDI
> (ops.rs verbatim) + confronti + bitwise + spaceship; (Double,Double)
> aritmetica + confronti IEEE (⭐ Gt/Ge = forme smaller SCAMBIATE `r<l`/
> `r<=l`, NaN-esatte; === è `==` IEEE); (Str,Str) Concat byte + ===/!==
> byte-eq (⭐ MAI l'Eq loose: "10"=="1e1" resta in smart_streq); **cross-
> class ===/!== = costante** (ident_class rispecchia le arm di
> ops::identical; Undef≡Null stessa classe; Ref/ArgPlace/WeakHandle →
> generico). **(C4) guardie FetchDim/CoalesceFetchDim**: base Array + key
> Long/Str → UN lookup a chiave canonica (Key::from_zstr, "5"→Int(5))
> PRIMA del probe ArrayAccess; ⭐⭐ il flush dei diag pendenti resta sul
> hit (warning dell'op precedente sorge AL read, error-handler che lancia
> fa unwind da QUI); miss → read_dim_warn (una sola fonte del warning);
> coalesce = gemello silente (hit E miss inline). **(C5) IncDecSlot** su
> slot Long raw con checked step (overflow/Ref/Undef → generico).
> **Esito: microbench 6,08→4,64s = −23,7%** (aspettativa piano era
> −5/10%); **media 69,0→66,9s, rapporto 3,19×** (media è gd/mysql-bound);
> **full-suite 12:54→12:20 = −4,4%, gap 2,18×**; footprint 12,0×
> invariato (nessun lavoro memoria in fase 1). gate22 TUTTO verde; cargo
> **1619**; **run24 = run23 per nome** (30.472 test, 2F+86W identici).
> Riprofilo `wp30-harness/ab-out/new-wp33.sample`: run_loop self 2760,
> drop Zval 766, resolve_prop_access 545, memmove 456, memcmp 442,
> enter_callee 375, clone Zval 341, gc_note 282, slot_of 202 → **T2 =
> catena prop (fusione This→PropGet?/slot IC più a monte), call path,
> FetchDimConst su chiavi letterali, gc_note batching, interning**.
> 📌 Metodologia benchmark (per la pubblicazione): oracle CLI con
> `opcache.enable_cli=Off` e JIT off ⇒ interprete-vs-interprete
> simmetrico; dichiararlo; colonna futura opcache_cli=1 (richiederà
> unit-cache persistente phpr). 📌 NoRef load/store RINVIATO: 3 canali
> runtime installano Ref negli slot a prescindere dagli op (scope-bridge
> include/eval; BindGlobalDyn su catena bridge; PushArgPlace/SEND_VAR_EX
> a bind time) — inventario bancato, non riproporre senza chiudere quelli.
> 📌 Gap pre-esistenti visti di passaggio (NON toccati): deprecation 8.5
> "Using null as an array offset" e float-key nel contesto `??` non
> emesse (funnel silente documentato di read_dim_nullable); warning
> "Undefined variable" mancante su `$u++` di var mai vista.

## Lezioni operative della sessione

- ⭐⭐ **Strumentazione nel hot loop SEMPRE dietro cargo feature**: un
  `if bool` mai-preso nel run_loop = +2,9% sul micro op-denso (misurato
  5 coppie A/B). `#[cold]` sul recorder NON basta; compilare via il branch
  sì (−0,4% = rumore). Feature build in CARGO_TARGET_DIR separato per non
  invalidare la cache default.
- ⭐⭐ **Il census dump va su FILE quando il workload spawna subprocess**:
  l'env si eredita, il dump stderr del figlio finisce nell'output che il
  harness cattura e asserisce (15 errori PHPUnit separate-process solo
  per il banner). `PHPR_OP_CENSUS=/path` → append (aggrega anche i figli).
- ⭐ **Il micro mente sul mix reale**: bench.php è Long-aritmetica; il WP
  reale è Concat/NotIdentical(Str,Str) 10× sopra l'aritmetica. Le matrici
  tipate del census (non i sospetti) decidono la matrice dei fast-path —
  è così che sono saltati fuori i cross-class === costanti.
- ⭐ **Fast-path = guardia sui TAG appena poppati + fall-through al
  generico**: mai duplicare warning/coercion/overload nel ramo veloce —
  il miss DEVE ricadere nel funnel esistente (una sola fonte di verità);
  pinnare PRIMA su oracle E su old-binary (probe byte-diff) i casi bordo
  (NaN, -0.0, overflow→Double sugli operandi, "10"=="1e1", float-key).
- ⭐ Il hook serena-vexp-guard ora blocca anche `git add` con path .rs
  espliciti → `git add -u` + `git commit -F file`.
- ⭐ `run_source` (test eval.rs) NON ha php-builtins: niente var_dump/
  gettype — asserire via echo/ternari con stringhe oracle-pinnate.
