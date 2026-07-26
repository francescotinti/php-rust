# WP_SESSION_62 — Fase 0.5 compile-cache: P0/M2 DECISO COL METRO (prefisso 89,5% ⇒ rotta Hejlsberg, leva SOSPESA, stub-elision = leva nominata WP-63) + metro TARATO + osservabilità unit-cache consegnata + refactor reloc marker-espliciti (gate62 VERDE COMPLETO)

> ⚡ **WP-62 (2026-07-26, `d7a9e85`→`c20a42d`)** — design62 ha recepito
> TUTTI gli emendamenti della sintesi a 9 (COUNCIL_WP62_REVIEWS);
> scaletta Klabnik M0→M2 eseguita; il decision point M2 si è chiuso col
> metro PRIMA di scrivere la leva, come da direttiva.

## M0 — igiene (tutto verde)

- **M0a uploads-guard (Gregg R7)**: `wp62-harness/uploads-guard.sh`
  (tar+manifest count+sha256 su drive esterno, ABORT se il backup
  fallisce, restore verificato col manifest, stato `current` che
  blocca doppi backup) — self-test 5 casi verde; integrato in
  `wp16-harness/run-full-detached.sh` e nell'orchestratore serale.
  I 48 file di media utente sono passati da backup/restore verificato.
- **M0b guard anti-annidamento (Leijen R3/Gregg R1)**: le 3 finestre
  net-compile sono ora `CensusNetWindow` RAII (depth rilasciato anche
  su early-return: un compile fallito non avvelena il contatore);
  `tag=netguard nested_windows` = **0** su sintetico E su --list-tests.
- **M0c taratura del metro (Leijen R4)**: unit sintetiche a taglia
  analitica nota → ratio **1,007-1,058** (banda ±10% ✓); catena
  include a 3 livelli (1/2/3MB): ogni unit riporta il SUO net, nessun
  assorbimento del figlio; net_tot --list-tests **riproduce WP-61
  alla cifra** (648.774.801). ⇒ la banda dup 1.130,2MB NET è CITABILE.

## M1 — osservabilità unit-cache (chiave INVARIATA)

- Contatori sempre-compilati (path include = freddo, costo ~0 misurato:
  11,91s user / 1,44GB = WP-61): tassonomia del miss
  (cold / fp / double-check / nostat), hit intra-VM vs cross-VM via
  `VM_EPOCH` (Pedersen P3), inserts / fp_replaced / ways_evictions /
  metadata_calls; `superseded_*` a dump-time (Leijen R5).
- Log eventi dietro `PHPR_UNIT_CACHE_LOG` (WP-44: la prova positiva
  vive nel log) — è il meccanismo con cui le sentinelle M4 asseriranno
  `cachehit>0` (K5/KK1).
- `tag=cachehit` per-path con n/somma/MEDIANA del net del hit-path
  (finestra chiusa PRIMA del run linked).
- **BASELINE (mai misurata prima)** su --list-tests: **hit 0+0**,
  miss_cold 1925, miss_fp 6 (version.php ×2, blocks-json ×4),
  miss_dc 0, metadata_calls 1931 — la cache WP-20 sul CLI non serve
  nulla, il canale è interamente miss_fp/cold.

## M2 — DECISION POINT (chiuso col metro, zero codice di leva)

- **M2.1 Hejlsberg A1 SCATTATA**: contatore prefix-vs-proper
  consegnato (CompileSplit census-only: tables/stub/fnshare/proper
  per-unit su unittop/unitpath2 + tag=prefixsum). Il numero decisivo è
  l'A/B seed-vuoto vs seed-pieno sui 3 bersagli del full:
  version.php 437,7KB/inc vs 4,1KB proper fresco ≈ **99%**;
  script-modules 483KB vs ~66KB ≈ **86%**; script-loader 546KB vs
  ~128KB ≈ **77%** ⇒ **prefisso ≈ 851,6MB dei 951,6MB = 89,5% ≫ 60%**.
  KS-B (<30%) NON scattato: il dissenso Hejlsberg REGGE. Le colonne
  per-unit sotto-attribuiscono per costruzione (il "proper" a seed
  grande contiene le ricompile dei condizionali del seed): prefix-cols
  24,5% su --list-tests = lower bound.
- **M2.2 forma A (copia-leggera Hoare) MORTA per pre-quota**:
  bytes_copied ≈ owned_priv del census v2 = **86,2% (listtests) /
  87,3% (full)** del net ≫ KS-H2 (50%) e KS1-Leijen (5%). La
  predizione di Matsakis è confermata dai contatori REALI. → NON
  riproporre.
- **M2.3 forma B (id-relativi+base Matsakis) SOPRAVVIVE alla sua
  pre-quota**: binario `relbase-probe` (feature mai-parità: indirezione
  identity + base-add zero negli arm caldi — Alloc/InvokeMethod/
  InstanceOf/ClassConst/EnumCase/target_class_id/StaticGuard-Store-
  Alias). Micro advisory: canale invoke +1,0-1,3%, alloc+iof +1,0%,
  static ≈0. **K-M1 (verdetto): media A/B interleaved B 54,17/54,37 vs
  A 54,74/56,03 user = B −2,0% ⇒ la guardia ≤+1% NON scatta.**
- **VERDETTO DI PROGRAMMA (design62 §6, da ratificare al concilio)**:
  la condizione utente "entrambe bocciate" non si è data (B è viva),
  MA A1+KS-A Hejlsberg autorizzano il riordino: la banda della cache
  si riscrive come "prefisso evitato sul hit"; costruire l'indirezione
  di B oggi = pagare arm caldi per moduli che la stub-elision renderà
  position-independent GRATIS. ⇒ **leva compile-cache SOSPESA al
  decision point; WP-63 = stub-elision (tetto ≥851,6MB misurati sui 3
  bersagli; 1,2-1,6GB plausibile); la cache si riapre DOPO su moduli
  PI, riusando matrice sentinelle M4 + contatori M1 già consegnati.**
- M2.4 (ns/hit budget Bak): non eseguito — moot con la leva sospesa;
  si applicherà alla forma post-stub-elision.

## Refactor di sicurezza (F, indipendente dalla leva) + il bug che il gate ha preso

- Il pun `Rc::get_mut` nella relocation è ora CLASSIFICATO dai marker
  espliciti (`file == b"prelude"` / `b"seed-stub"`): owned ⇒ SEMPRE
  relocate; shared+marcato ⇒ skip contato (`tag=reloc`); shared
  NON-marcato ⇒ **RUMOROSO** (log::error sempre, panic nei build
  census) — la classe IpUtils non può più tacere.
- 🔴 **La prima stesura del refactor era SBAGLIATA e il gate l'ha
  presa**: saltava la relocation per QUALSIASI entry marcata prelude —
  ma un corpo prelude ricompilato FRESCO (mismatch/conditional) è
  uniquely-owned con id unit-locali e DEVE rilocare. Sintomo: hk
  `KernelTest::testWarmupIsNotRunOnSubsequentBoot` 1F (solo a livello
  suite/file, in isolamento passava). Fix `c20a42d`: la decisione
  resta a get_mut, il marker classifica solo il caso None.
- SealedModule (M3-Matsakis): NON introdotto — la pubblicazione
  `&'static Module` già impedisce ogni `&mut` a valle (borrow
  checker); il newtype ha valore solo contro futuri owned-copy e resta
  nel backlog della cache riaperta.

## Gate62 (VERDE COMPLETO, binario 2f5220c7…)

cargo **1645/0** · sentinelle 5 assi **BYTE-ID** · corpus **1421
IDENTICO** per nome · refl **290 IDENTICO** · ORM 3484 **3E/13F
fail-set IDENTICO** · hk 1665 **0E/0F** (2 PHPUnit-warnings note).
**Stash additivo: phpr-wp62 = `2f5220c7…`** (phpr-old-target/release/).

## Catena serale (orchestrate62.sh, daemonizzata, uploads-guard attorno)

gap62 media (oracle+phpr) → census62-full con memgc62 (colonne prefix
sul bordo vero = quota stub-elision) → coppia full run49 new (wp62) vs
old (phpr-wp61). Esiti: → REPORT_GAP_62 + §sotto.

<!-- SLOT-ESITI-CATENA (compilato a fine catena) -->

## ⭐ Lezioni

- ⭐⭐ **Le pre-quote possono chiudere un conflitto di design senza
  scrivere la leva**: forma A è morta coi contatori census v2 GIÀ
  esistenti (owned_priv 86-87%), il prefisso è emerso da un A/B
  seed-vuoto da 30 secondi — il concilio aveva ragione a pretendere i
  numeri PRIMA della forma.
- ⭐⭐ **Il "proper" di un contatore per-colonne a seed grande non è il
  proper dell'unit**: contiene le ricompile dei condizionali del seed e
  il compile contro ctx grande — la definizione autoritativa di
  prefisso è l'A/B seed-vuoto-vs-pieno sullo stesso file (le colonne
  sono il lower bound e dicono DOVE vive).
- ⭐⭐ **Un refactor "di sicurezza" nella relocation è un cambio engine a
  tutti gli effetti**: la semantica skip-se-shared vs skip-se-marcato
  diverge esattamente sul caso raro (prelude ricompilato fresco) e
  solo il gate PIENO la vede (hk suite-level; il test isolato passa).
- ⭐ Un guard RAII sulle finestre census (depth in Drop) sopravvive agli
  early-return dei compile falliti — un contatore "aperto/chiuso" a
  mano si avvelenerebbe.
- ⭐ `daemonize.pl <log> <cmd>`: il log è il PRIMO argomento e la dir
  del log deve esistere — due errori di lancio evitabili.

## Prossimo (WP-63) — vedi NEXT_SESSION §WP-63

1. **Stub-elision** (leva NOMINATA, rotta Hejlsberg): compilare le unit
   CONTRO la symbol table del VM senza materializzare il prefisso del
   seed nel Module per-unit. Quota inchiodata da census62-full
   (colonne prefix sul full). Sentinelle: il contratto di compilazione
   di TUTTE le unit cambia ⇒ gate PIENO + coppia full.
2. Cache re-link: riaperta DOPO stub-elision su moduli PI (matrice M4 +
   contatori M1 pronti in design62).
3. php-server front-end axum (richiesta utente, sessione dedicata) —
   nota Pedersen: UNIT_CACHE thread_local ⇒ ×N worker.
