# WP_SESSION_71 — debiti d'apertura CHIUSI (🔴 M-71.1 walker PROP-write/unset/DIM-unset a disciplina di confine + S-71.1 readonly-extends + S-71.2 autoload lista viva) + 🔴 ATTRIBUZIONE DEL RESIDUO CHIUSA A MACCHINA: cicli Rc non raccolti al teardown (grafo WP_Metadata_Lazyloader) + GATE71 PASS al 1° run ATTESTATO

> ⚡ **WP-71 (2026-07-29, `e0094c6`→`076ed4b`)** — sintesi a 9 recepita
> INTEGRALE in `wp71-harness/design71.md` PRIMA del codice; pre-registro
> G-71.1 lockato PRIMA dei letti (05faefcc → addendum strumento
> e61f07df → amplificazione 744e4b00/27f512d3, tutti ADDITIVI
> pre-letto). GATE71 **PASS fails=0 al 1° run — ATTESTATO
> dall'attempt-counter K-71.2 (attempts=1)**. Stash **phpr-wp71
> (38af5eaa…)**; census phpr-memgc71/71b/71c.

## 🔴 M-71.1 ≡ H-71.1 (opposizione Hoare+Matsakis) — CHIUSO (e0094c6)

`field_write`, `field_unset`, `unset_into` convertiti alla disciplina
di confine (token `WriteWalk`/`UnsetWalk`/`UnsetIntoWalk`, driver a
loop, guard mai vivo attraverso un confine; firma pubblica invariata
⇒ zero modifiche ai chiamanti). **6/6 repro del concilio BYTE-ID**
(m1-m6) + **7° panic scoperto e chiuso** (t6 panicava anche su wp70,
arrays.rs:229; la divergenza residua è la famiglia pre-esistente
STRING-OFFSET-BIND, probe non-ciclico a conferma). Batteria
**gate-h71walk 22/22** (t1-t8 + varianti verbo v1-v8; t4/t6/t7 pin
phpr con PROVENANCE-h71). **K-M71.2 PER COSTRUZIONE**:
`gate-walkborrow` (estrazione dei 23 corpi di walk-path + FAIL su
borrow nudo senza marker; 6 `BORROW-OK` motivati — 5 shared-read
approvati Matsakis + store_slot con invariante M-71.3 documentata).
H-71.3: tutti i rami Err dei drain RefLeaf tripwirati
(debug_assert + DRAIN_FAIL contato in OGNI build); M-71.2:
CELL_PARK/DRAIN_FAIL sempre compilati, census `tag=cellpark`.
H-71.4: il park RefLeaf nel drain nested-Descend è RAMO MORTO per
costruzione (probe h714 BYTE-ID); H-71.5: t7 pinnato (quiet-fetch).
**KH71-2 ESERCITATO: corpus 1421→1420 = −1 ESATTO, `ns_064.phpt` ORA
PASSA** (è la famiglia M-71.1: `$this->e[]=$this` + prop-write via
`__get`); prima run corpus RIGETTATA (contaminata da build
concorrenti) e rigiocata su binario fermo; ri-pin con
PROVENANCE-corpus71. Belt test nuovo (cargo 1651/0).

## 🔴 S-71.1 + S-71.2 (Stogov) — CHIUSI (e0094c6 + 4a12b85)

S-71.1: `ClassDecl.is_readonly` (fluisce nel SEED ⇒ un solo check per
eager e re-lower deferito) + `check_readonly_extends` BIDIREZIONALE
(Non-readonly/Readonly cannot extend); il `LowerError::Fatal` dal
defer ora surfaced come `PhpError::FatalAt` Zend-shaped (mai più il
wrapper "Failed to compile"). S-71.2: `spl_autoload_call` a lista
VIVA — cursore element-stable su Vm (`autoload_cursors`, stack per
lookup annidate), register(prepend) shifta, unregister riposiziona al
successore AL MOMENTO della delete con STICKY-END (l'oracle ha
rivelato che il sostituto appeso dopo self-unregister NON fira nella
stessa lookup). Batteria **gate-s71 15/15** (vi_trait = backlog
S-71.3, pin phpr + target oracle salvato). E-71.H1/H2 (audit predicato
+ caso-negativo): NON eseguiti — restano a WP-72.

## 🔴 CACCIA — ATTRIBUZIONE CHIUSA A VERDETTO MACCHINA

- **P71-T (memgc71, scope-heap L-71.1 via allocator-routing, addendum
  pre-letto)**: 3 leg 1000/1000/1000, assert G-71.3/G-71.4 TUTTI verdi,
  **KL71-1 PASS: totale 20,000 obj/req ESATTO (spread 0,000!) /
  2,1109 KiB/req**, firma STRADDLE 5/5 (96|112→9,00; L-71.3 era la
  forma giusta), cellpark=0 drainfails=0 (K-M71.3 ok). **H1-C1 =
  MORTA**: src=defer slope 0,000 su 3 leg CON controllo positivo
  (defermini 16,00/req + working-set costante 37k righe nel
  defer-heap) ⇒ **falsificati IN BLOCCO tutti i candidati interni a
  run_deferred** (C1 park, C3 attempted, C5 per-id, C6 accumulate_seed).
- **KL71-2 (escalation pre-registrata)**: block-content dump di tutti
  i bin ≤1024 a R500/R1000 + istogramma DELTA-DI-CONTEGGIO
  (address-free): **Σ delta = 10.000 ESATTO** = 500 req × 20 obj.
  Inseguimento puntatori fino alle foglie: chiavi **"filter"/"callback"
  strong=3001** (3/req+1), hook **"get_comment_metadata"** (len 20,
  1001) — è il grafo di **WP_Metadata_Lazyloader** (`callback =
  [$this,…]` dentro `$this->settings`) = **CICLO Rc auto-referente
  NON raccolto al teardown della richiesta** (VM per-request muore,
  il Drop libera i raggiungibili, i cicli restano).
- **AMPLIFICAZIONE (verdict da script committato)**: mu-plugin K
  cicli sintetici/req — K=3 → +23,996 (token NON-CONCLUSIVO per banda
  mal dimensionata: 8 obj/ciclo; refutazione ESCLUSA); nuovo lock
  K=10 banda [64,96] ⇒ **delta = 80,000 obj/req ESATTO =
  MECCANISMO-CONFERMATO**. Libro mastro finale: causa unica
  "cicli-Rc-al-teardown" = 100% della firma.
- **L-71.4 ladder PASS: LINEARE** (fp 184,3/192,0/200,3 MiB a
  1k/5k/9k; ratio 1,078; rate phys ~2,0 KiB/req ≈ census — la coppia
  singola WP-70 a 1,45 era il caso sfavorevole del metro lossy).
- **Leva ora LEGITTIMA (post-attribuzione)**: cycle-collect al Drop
  della VM / confine richiesta — candidata d'apertura WP-72;
  validazione P-71.3 (two-boot post-fix < 2 MiB) = sblocco axum.

## B-70.1/B-70.4 (debito Bak) — CONSEGNATI PER-EVENTO

Nuovo evento probe-API `evict` (UC_LOG_EVENTS, con vittima fp+path).
Run cron-ON canonica: **8 evict OSSERVATI** — 4 nel segmento cron
(set front sfrattato dai 4 fp del contesto cron su version.php) + 4
nel primo front successivo (rimbalzo); B-70.4: firma version.php ×4
per segmento, set front {3917…,3eb5…,736f…,c672…} stabile e disgiunto
dal set cron. La ricostruzione WP-69 "4+4=8" è ora evidenza
per-evento: il concilio ways/fp può deliberare (KB71-4 soddisfatto).

## Igiene di metro (Klabnik/Gregg/Pedersen) — FATTA

K-71.3 timbri retroattivi macchina (P70-0 NON-CALCOLABILE, P70-S
NON-ESEGUITA, P70-D NON-CALCOLABILE-in-forma, K-71.1 alias
ESISTE-SU-RELEASE≡DENTRO) in `wp70-harness/retro-out/k71-3.stamps` +
K-71.4 causa del superseded. K-71.2 attempt-counter nel gate71
(**PASS al 1° run ATTESTATO: attempts=1**). P-71.1: TUTTI i verdict
di sessione emessi da script committati (tripla, ladder, cron, amp
re-emesso da probe71-amp-analyze.sh). P-71.2 bonifica per-leg + curl
--fail contate; G-71.3/G-71.4 assert nel parser (analyze71.pl).

## GATE71 — PASS fails=0 AL 1° RUN (attests=1)

cargo 1651/0 · sentinelle 5 assi BYTE-ID · fixture WP-67..71 TUTTE
(+gate-h71walk, gate-s71, gate-walkborrow) · sentinels65 + KS-S6 +
seed_prefix_short=0 · KE-e · P1 · sem doppio pin · corpus **1420**
IDENTICO (baseline wp71) · refl 290 · ORM 3E/13F · hk 1665 ·
reverse 2F per nome. Trap self-test esercitata.

## ⭐ Lezioni

- ⭐⭐ **Uno scope-heap falsifica una FAMIGLIA intera in un letto**: il
  routing allocator su run_deferred ha ucciso in un colpo C1+C3+C5+C6
  (tutto ciò che alloca nello scope) — con controllo POSITIVO
  incorporato (il canale fira 16,00/req e tiene un working-set
  costante): un letto zero senza positivo sarebbe stato un falso morto.
- ⭐⭐ **Il contenuto dei blocchi è l'attributore finale**: il delta
  per-pattern address-free riconcilia alla cifra (10.000/10.000) dove
  il diff per puntatore annega nel riuso; le foglie stringa
  ("filter"/"callback"/hook) NOMINANO il canale senza strumentare
  l'engine.
- ⭐⭐ **L'amplificazione lineare è la prova regina di un meccanismo**:
  K cicli sintetici ⇒ K×8,000 obj/req intero-esatto — nessuna
  correlazione, una legge; e la banda sbagliata (K=3) si corregge con
  un NUOVO lock derivato dal letto, mai con l'aggiustamento post-hoc.
- ⭐⭐ **I teardown non raccolgono i cicli**: Rc + VM-per-request =
  ogni ciclo utente sopravvive al Drop; il collector si radica dai
  buffer (WP-49) ma al teardown nessuno lo invoca — la famiglia era
  invisibile a ogni census per-canale perché il ritenuto è "heap
  utente normale".
- ⭐ Il verbo è parte della batteria (unset/prop-write mancavano a
  h70cycle); il grep-gate chiude la proprietà per costruzione dove le
  batterie coprono solo le forme note.
- ⭐ L'oracle decide anche le semantiche di iterazione (sticky-end di
  spl_autoload_call: il sostituto NON fira — l'implementazione "prima
  non-chiamata" era plausibile e SBAGLIATA).

## Parità e stash

Release **phpr-wp71 (38af5eaa…, tree `076ed4b`)**, stash ADDITIVO —
27° in archivio (motivo ri-stash: fix engine M-71.1/S-71.1/S-71.2 +
evento evict log-gated). Census: phpr-memgc71 (2c4691d7, scope-heap),
71b (91038d46, +blockdump bin firma), 71c (+blockdump ≤1024) in
phpr-mem-target/. Delta engine vs wp70: WriteWalk/UnsetWalk/
UnsetIntoWalk + prop_step ×2 (walker) + cellpark/drainfail counters +
ClassDecl.is_readonly + check_readonly_extends + FatalAt dal defer +
autoload_cursors (lista viva) + evento uc_log `evict` + (census-only)
DeferHeapScope/scoped_alloc + PHPR_MI_BLOCKDUMP.

## Prossimo (WP-72) — vedi NEXT_SESSION §WP-72
