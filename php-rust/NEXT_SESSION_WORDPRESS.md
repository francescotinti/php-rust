# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **coppia bimodale
IN VOLO da S-105** (chain off→on detached 23:03 del 2026-08-06; finché la
lettura non è fatta, riferimento = WP-102, 3 sessioni fa; lettura dei
ratios = PRIMO ATTO S-106, con la banda PRE-registrata KS-GR-107-3:
full∈[1,84;1,89] · media∈[2,57;2,64] — fuori banda in meglio si legge con
la formula f̂, non con l'entusiasmo) · ultima campagna sull'OGGETTO =
**S-105 (LEVA H-D args PROMOSSA: calls 7,6→6,3)** ·
**sessioni-senza-Δ-rapporti = 0** (azzerato da S-105; era 3).

**Ultima sessione (S-105, 2026-08-06 sera)**: ordine WP-106 §1 consumato
INTERO. 1) **LEVA H-D args SPEDITA (forma 2 direct-bind)**: Δ=+23,00
ns/iter su calls (5/5, rumore ~3,5; ns/iter 164→141), census co-primario
**0,0000 alla quarta cifra** (alloc E free), admission-disasm SENZA flip
(bl 5408→5416, dropZval 1109→1111, rust_alloc/dealloc IDENTICI, +324 B).
La **forma 1** (SmallVec letterale del mandato) misurata come CONTROLLO e
CADUTA: Δ=−14,00 (0/5) con census pure 0,0000 ⇒ il contenitore costa più
della coppia alloc+free che elimina; il divario tra le forme è INDIZIO
direzionale, MAI cifra (estimatore post-hoc tra binari a layout diverso,
KS-HE-107-1 ≡ KS-GR-107-2; i confondenti — reverse, transito bind_params —
sono parte del contrasto, A-GR-107-1). 2) Gate d'apertura: G1 probe
cap-bump = sito attribuito PER MISURA (19,9M eventi spostati esatti); G2
arità reale wptests **73,1% ≤2** (⚠️ Bak R-BA-107: è l'arità a
bind_params, NON la copertura del predicato fast `Op::Call ∧ simple_call ∧
arità esatta` — builtin/metodi/default MAI censiti: serve il contatore
hit/miss A-BA-107-1 prima di nominare il prossimo sito); G3 audit-fuga ok
+ 🔵 **§3.15 NUOVA a catalogo** (variadic by-ref diretto: solo il 1° arg
aliasa; sito confermato Matsakis in expr.rs:1615). 3) PIN-106 SALDO
(sotto; ⚠️ Klabnik: il claim «zero code» dei commit post-pin è FALSO al
diff — 766d3d8 tocca 2 .rs mai compilati post-pin ⇒ A-KL-107-1 cargo
check a HEAD = primo atto S-106). 4) fx20 → banda due soglie (guardia
75/cap 150); sigilli Copy+doc (⚠️ Hoare∧Klabnik: il sigillo Copy così
com'è è VACUO/tautologico — «Bool→boxed» lo passerebbe; riscrivere sui
costruttori di variante, A-HO-107-1 ≡ A-KL-107-2); catalogo §3.14
(memory_get_usage stub). Dettaglio: `sessions/WP_SESSION_105.md`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-105, micro R=5 SUL PIN DI CHIUSURA d4d0fa52, N emessi)

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **12,4** | candidata alternativa 1° slot S-106 (Bak A-BA-107-3: se il braccio contatori non entra, arith prima di prop) |
| proprietà | **11,5** | fusione prop-RMW H-C3 (porta §3.12-i; contatori L1I = prerequisito di TESI se il criterio cita icache) |
| chiamate | **6,3** ↓ | leva args forma 2 SPEDITA (S-105); prossimo sito SOLO dopo contatore hit/miss del fast path (A-BA-107-1) |
| stringhe 6,7 · array 4,5 · regex 3,6 | | — (regex = parte sana) |

(le 5 categorie non toccate ferme entro ±0,1 sul pin S-105)

## Regole di metodo

1. Il giudice è la micro-categoria. 2. WordPress è collaudo di PARITÀ —
la coppia S-105 è IN VOLO: leggerla è primo atto S-106 (banda
pre-registrata KS-GR-107-3). 3. Criterio scritto PRIMA. 4. Apparato solo
se blocca; timebox ½ sessione. 5. **SCOREBOARD in testa a ogni report +
almeno una leva TENTATA (A/B eseguito) per sessione**
([[feedback-scoreboard-lever-pacing]]). 6. **PIN-106**: `build → hash₁ →
batteria → re-hash₂ → STASH(hash₂) → fixture → corpus×2`; build dopo lo
stash = gate VOID; churn documentato; commit post-pin su .rs esigono
`cargo check` a HEAD (A-KL-107-1). 7. **Admission di OGNI leva
(KS-GR-106-2)**: admission-disasm (bl-count prima/dopo) + smoke R=2 prima
del pieno; **early-stop PRE-registrato: smoke 2/2 a segno OPPOSTO
all'attesa ⇒ si può fermare il pieno e cambiare forma** (KS-GR-107-1);
mai componenti di costo da A/B che flippa il codegen (KS-BA-106-1); mai
differenze tra A/B DISTINTI come cifra (KS-HE-107-1 ≡ KS-GR-107-2).
8. Parity-null sempre col PERIMETRO nominato (KS-KL-106-1). 9. Nessun
verdetto su memory_get_usage finché è stub (KS-MA-106-1, §3.14).

## Stato gate

- **phpr (pin release)**: **d4d0fa5217515dd9** @ d569a56 (codice leva;
  HEAD di chiusura ha in più commit doc/sigilli/harness dichiarati
  parity-null — ⚠️ 766d3d8 tocca 2 .rs (const-seal + doc): mai compilati
  post-pin, il cargo check a HEAD è DOVUTO all'apertura S-106; churn
  5e8c84c9→d4d0fa52 di relink batteria documentato; fa fede HEAD) —
  DEFAULT flag-ON. Batteria **1740/0** · fixture
  **13+5+19a/b+fx20v2+fx21** (pin BILATERALE, oracle 07b0df8d; ⚠️ 19a/19b
  PASS = SOLO byte-parity, non arbitrano meccanismi finché OBS-8 non
  decide) · corpus **1417 per NOME ×2 modi** IDENTICO a wp82 (runner
  rilinkato verificato) · micro sul pin (baseline sopra). Stash ADDITIVO
  `phpr-s105`.
- **php-server**: **de67cb6466acb030** @ stesso HEAD del pin, stash
  `php-server-s105` — ricostruito in S-105 ma **grado PIENO NON
  eseguito**: NESSUNA cifra server finché non è gradato (KS-PE-106-1).
  Il vecchio pin 31aa7c2e è STORICO.
- **IN VOLO — coppia WP bimodale S-105**: chain off→on detached
  (daemonizer, partita 23:03; pin-check phpr nel chain — ⚠️ Klabnik
  A-KL-107-4/5: il chain NON verifica server/oracle e non esce
  GATE_VOID: la lettura deve validare l'identità dai `pair105.identity`
  di ciascuna gamba). Marker: `wp105-harness/pair-chain/pair-chain.done`
  (rc_off/rc_on); ratios in `wp105-harness/pair105-ratios-{off,on}.out`;
  raw in `pair-out-{off,on}/`.
- **Launcher S-105** (`wp105-harness/`): `s105-hd-ab.sh` · 
  `s105-admission-disasm.sh` (conteggi sul .dis con awk, MAI rtk grep) ·
  `s105-corpus-gate.sh` · `s105-pair-chain.sh` · fixture
  `fixtures/fx21-args-window.php` (⚠️ A-KL-107-3: da promuovere a gate
  fail-closed con golden della riga 5 in repo) · criterio/verdetti
  `hd-*.out`, `pin106-gate-verdetto.out`.
- Census build (con GA_ARITY al choke-point bind_params) in
  `phpr-census-target`. GATE72 CLI baseline trasversale invariata
  (corpus 1417 · refl 290 · ORM 3E/13F · hk 1665).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA: rimuoverla nel
  pre-flight.

## Voci APERTE per NOME

- **Lettura coppia WP S-105** (primo atto S-106): pair-chain.done +
  identity per gamba + ratios vs banda PRE-registrata KS-GR-107-3
  (full∈[1,84;1,89], media∈[2,57;2,64], f̂=(1−r/1,89)/0,14 per la quota
  calls); se il chain è fallito ⇒ fallback KS-PE-106-2: rilancio entro
  chiusura S-106 o il riferimento WP-102 DECADE.
- **Completamento Concilio WP-107**: sedie Leijen+Stogov (cadute per
  limite API) + eventuale Pedersen + FASE 2 team + SYNTHESIS — primo
  atto dopo la lettura coppia; i 6 verbali consegnati sono GIÀ vincolanti.
- **cargo check a HEAD** (A-KL-107-1) + **sigillo Copy riscritto sui
  costruttori di variante** (A-HO-107-1 ≡ A-KL-107-2) + **dente
  ordine-di-drop del direct-bind** (A-MA-107-1: «decay pure-read» va
  provato, non dichiarato) + **fx21 a gate fail-closed** (A-KL-107-3).
- **Contatore hit/miss del fast path** (A-BA-107-1): quota REALE di
  copertura del predicato `Op::Call ∧ simple_call ∧ arità esatta` su
  carico WP — PRIMA di nominare il prossimo sito calls.
- **Grado PIENO php-server de67cb64** (KS-PE-106-1).
- **Terza mutazione OBS-8 su 19a/19b** (A-MA-106-1/A-MA-107-3, sito
  mod.rs:4965, −2→−1, target-dir SEPARATO) + **mutante leak-PARZIALE
  fx20** (deve rompere la guardia 75).
- **Contatori L1I/INST_RETIRED** (checkout 4ea2cff): finché mancano,
  «icache-bound» resta ipotesi NON citabile come premessa.
- **§3.15 variadic by-ref diretto** (expr.rs:1615; cura = flag di vslot
  per posizioni ≥ vslot; gate ORM/hk obbligatorio al cambio ref) ·
  **memory_get_usage §3.14** (cura due gradini, fuori finestra leva) ·
  **generator get_gc** + **§3.13 unit** · §3.12 regime-i (con H-C3) ·
  terzo punto banda micro su GIORNO distinto · banda-layout N≥3
  (occasione S-105 persa, rilievo Hejlsberg A-HE-107-3: micro su hash₁ E
  hash₂ = punto gratis dal churn).

## Che cosa è SOSPESO (non abbandonato)

- A-ZV2 (liveness+TakeSlot); design per-fase A-LE-105-5; PGO/outlining
  A-HE-106-4 (+ text-budget run_loop A-HE-107-4: a +4 KB cumulativi
  scatta il bilancio); H-ICS cold-out; 21,2% run_loop = prefisso
  targeting leva icache (A-HE-106-5); roadmap footprint (S-102: full
  peak on 1942,05 / off 1989,88 MiB); disposizione mutanti (A-MA-106-2);
  retention uploads (A-PE-106-3); srotolamento per-arità RESPINTO
  (A-BA-107-2: icache-lesson).

## NON riproporre

Tutti i NON-riproporre WP-83..104 restano. Nuovi da S-105/WP-107:

- **SmallVec (o qualunque CONTENITORE) per gli args del call path**:
  misurato e caduto (Δ=−14,00, 0/5, census 0,0000); l'estensione ad
  altri siti si progetta senza contenitori intermedi — e il verdetto
  vale per FORMA-E-CONFINE misurati, non «contenitori» in astratto
  (KS di Bak).
- **differenze tra A/B DISTINTI usate come cifra** (KS-HE-107-1 ≡
  KS-GR-107-2): il «~37 ns/iter di contenitore» resta indizio.
- **allargare simple_call/il predicato fast senza dente VM + fx21**
  (KS-HO-107-1: = VOID) · **direct-bind su op dinamiche senza check
  ArgPlace** (KS di Matsakis).
- **fixture su memory_get_usage** (KS-MA-106-1) · **claim «icache-bound»
  come premessa firmata** (contatori mancanti).
- ereditati: leve micro-costi drop-call; pool args; threaded-dispatch;
  estimatori post-hoc; banda dal proprio run; denominatori a memoria;
  output di run nel repo; .rs nei comandi git (Write + commit -F).

---
**Riscritto**: rotazione S-105 il 2026-08-06 notte. Apertura/chiusura =
skill `apri-sessione`/`chiudi-sessione`. Harness di sessione:
`wp105-harness/`; concilio in `wp107-harness/`.

## §S-106 — ORDINE (Concilio WP-107 — stato: COLLEGIO PARZIALE 6-7/9, fase 2 NON tenuta)

⚖️ **Concilio WP-107 (2026-08-06 notte)**: fase 1 convocata a 9 sedie;
consegnati **7/9** — Hoare, Matsakis, Klabnik, Hejlsberg, Bak, Pedersen,
Gregg (verbali in `wp107-harness/verbali/`, VINCOLANTI); **Leijen e
Stogov cadute per LIMITE API settimanale (reset 3:00 Europe/Rome)**.
**La FASE 2 (team) e la SYNTHESIS NON sono state tenute**: completare il
collegio è atto dovuto di S-106 («se manca, primo atto della successiva =
convocarlo»). Pedersen ricalibra il rinvio del grado server:
**A-PE-107-1 = grado PIENO primo atto S-106 PRIMA di cifre e build**
(proroga SPESA; KS-PE-107-3: seconda proroga ⇒ il pin server decade);
lettura ratios con precondizioni failnames-VUOTI (A-PE-107-3).

Ordine PROVVISORIO S-106 (da RATIFICARE con la SYNTHESIS WP-107):

1. **Lettura coppia WP S-105** (pair-chain.done + identity per gamba +
   precondizioni failnames A-PE-107-3 + banda KS-GR-107-3) — salda o
   rilancia il debito KS-PE-106-2.
2. **Grado PIENO php-server de67cb64** (A-PE-107-1: PRIMA di cifre e
   build; collaudo sullo stash `php-server-s105`; seconda proroga ⇒ pin
   server decade, KS-PE-107-3).
3. **Completamento Concilio WP-107** (Leijen+Stogov fase 1 + fase 2 team
   + SYNTHESIS + ratifica di quest'ordine).
4. **Igiene del pin (breve)**: cargo check a HEAD (A-KL-107-1) + sigillo
   Copy riscritto (A-HO-107-1) + dente ordine-di-drop (A-MA-107-1) +
   fx21 a gate fail-closed (A-KL-107-3).
5. **LEVA di sessione** (regola di ritmo): candidata H-C3 fusione
   prop-RMW (prop 11,5) SE il braccio contatori entra come prerequisito
   di tesi; ALTRIMENTI arith 12,4 (A-BA-107-3). Criterio scritto PRIMA;
   admission + early-stop KS-GR-107-1; PIN-106 in chiusura.
6. **Denti nella finestra**: terza mutazione OBS-8 (mod.rs:4965, −2→−1,
   target-dir separato) + mutante leak-parziale fx20 (deve rompere la
   guardia 75) + contatore hit/miss fast path (A-BA-107-1, census-gated).
7. Fedeltà timeboxata: generator get_gc > §3.13 unit > §3.15 (ordine da
   ratificare con Stogov al completamento).

Pre-flight S-106: pin phpr **d4d0fa5217515dd9** @ HEAD S-105 (fa fede
HEAD, la build churna) · server de67cb64 stash `php-server-s105` (NON
gradato: niente cifre) · corpus 1417 per NOME ×2 SUL PIN · default
flag-ON · debug/ si rigenera: rimuoverla · MySQL wp8 con l'elenco dei
database · uploads sotto guardia · **coppia S-105: leggere
pair-chain.done PRIMA di qualunque run pesante**.
