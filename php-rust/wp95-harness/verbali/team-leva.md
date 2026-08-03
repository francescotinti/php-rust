# Team-leva (fase 2) — Concilio WP-95

**Relatore**: sedia-relatore team-leva. **Sedie in perimetro**: Matsakis (2),
Stogov (8), Hejlsberg (4, parte rank/precompilato), Hoare (1, parte leva
per-file e semantica bumpalo). **Mandato**: riconciliare o REGISTRARE i
dissensi. Nessuna benedizione: sotto ogni cifra c'è un giudice o la cifra non
si scrive.

---

## 1. Convergenze (unanimi, senza sconti)

1. **La leva per-file è semanticamente percorribile e safe-only.** Il borrow
   checker prova già che nulla della `LoweredPrelude` borrows dall'arena
   (Matsakis Q1: alias senza lifetime, `Box<[u8]>`/`Vec` owned, parent =
   `ClassId` indice; zero `unsafe`/`Box::leak`/`transmute` in lower/mod.rs);
   la sequenza `{ Bump; parse; hoist; drop }` per unità compila com'è
   (Matsakis Q3); il Drop di fine funzione è già la prova (Hejlsberg Q1);
   il preludio è GIÀ multi-unit — 7 `File::ephemeral` tutti col nome
   `b"prelude"` su UNA sola `Bump` (Hoare Q3).
2. **La cifra pubblicata in `wp93-harness/huge-sites.out:89-90` è SBAGLIATA**
   («peak arena = max file ~74 KB invece del cumulativo ~39.5 MB»): confonde
   byte di SORGENTE con byte di ARENA. Refutazione capitale concorde di
   Matsakis (RC-MS-95-1) e Stogov (RC#1), corroborata da Hoare (Q3-7). Il
   fattore è ~2 ordini di grandezza: **la riduzione NON è ~500×.**
3. **Il rischio primario NON è «l'ordine globale di hoist»** (Stogov Q1,
   audit vivo: zero forward-ref extends/implements CROSS-file; core 52 decl,
   spl 28, reflection 25, tidy 0) **ma la sentinella `b"prelude"`** —
   uguaglianza al byte in ~20 siti portanti (host_reflect.rs:1511,
   vm/mod.rs:853 take_while sul PREFISSO, :13221, :16793, :6598/12145,
   calls.rs:1324, host.rs:3014, hir.rs:203, bytecode.rs:1769).
4. **Nessuna chiusura di fronte su per-file** (Stogov A-DS-70, Hejlsberg rank,
   legge no-front-closure): per-file è palliativo del solo picco transiente;
   lo slope ~18,8 MB/worker resta intatto e non nominato.
5. **Il rank parte da per-file** in entrambe le proposte (Hejlsberg 1, Stogov
   1): nessun conflitto sulla prima mossa.

### Verifica a macchina fatta dal relatore (nuova, non nei verbali)

- **Sorgenti**: 15 file, Σ = **405.628 B**; l'unità concat (9 file) =
  **309.488 B**; le 6 unità extra (ns/bcmath/gmp/mysqli/gd/fileinfo) =
  96.140 B. File massimo = **prelude/reflection.php 74.019 B**; secondo =
  dom.php 68.048 B.
- **Catena dei chunk bumpalo ricostruita a residuo ZERO**: 11 chunk piccoli
  (112 → 155.632, Σ = **310.944**) + 6 chunk huge (622.576 · 1.245.168 ·
  2.490.352 · 4.980.720 · 9.961.456 · **19.922.928**, Σ = **39.223.200**);
  **310.944 + 39.223.200 = 39.534.144 = `allocated_bytes` ESATTO**, e i primi
  cinque huge = 19.300.272 = esattamente la cifra di Hoare (Q2). La catena è
  `x·2+16` verificata su ogni anello.
  ⇒ **`allocated_bytes` = Σ CAPACITÀ di tutti i chunk; `chunk_capacity` =
  RESIDUO LIBERO del chunk corrente (13.738.592 su 19.922.928)**; nell'ultimo
  chunk sono toccati 6.184.336 B. **Touched fisico = 39.534.144 − 13.738.592
  = 25.795.552 B.** Hoare Q2 è CONFERMATO per aritmetica chiusa; il commento
  lower/mod.rs:1013-1015 («allocated_bytes … è il touched fisico») è FALSO.
- **Il forward-ref citato nel commento del `concat!` (mod.rs:740-747),
  «PhpToken prima di Stringable», è INTRA-file**: entrambi in
  `prelude/core.php` (righe 295 e 348). Corrobora l'audit di Stogov e
  l'argomento statico di Hejlsberg Q1(a): il conteggio decl per file
  coincide col suo (core 52, spl 28, reflection 25).

---

## 2. Conflitti (registrati, non lisciati)

**C1 — Unità della cifra: capacità vs touched.** Stogov e Matsakis calcolano
sulla CAPACITÀ (39.534.144 ⇒ ~97× sorgente→arena ⇒ ~7 MB / 5-8 MB per il file
massimo); Hoare pinna il numeratore al TOUCHED (25,8 MB). *Riconciliazione
del relatore*: **non è lo stesso oggetto e vanno etichettati entrambi.** La
valuta del FOOTPRINT è il touched (le pagine mai toccate non sono committed:
la coda di 13.738.592 B è indirizzo, non memoria fisica). La valuta
dell'IGIENE d'arena è la capacità. Regola: *ogni cifra della leva porta la
sua unità o è nulla*.
- capacità/sorgente = 39.534.144 / 405.628 = **97,5×** (Stogov, esatto);
- touched/sorgente = 25.795.552 / 405.628 = **63,6×** (derivato).

**C2 — `Bump::with_capacity` (A-MS-63, Hejlsberg «coda evitabile
13.738.592»).** *Parziale refutazione del relatore*: la coda mai toccata
molto probabilmente **non è nel footprint fisico**, quindi il pre-size può
valere **0 MB** sul denominatore che ci interessa. Vale su spazio
d'indirizzamento, su mmap/CPU e sull'igiene. **Non può essere venduto come
parte del numeratore della leva.** ⇒ pre-size in **commit e misura
SEPARATI** dalla leva, altrimenti si attribuisce alla leva un risparmio che
non c'è (violazione della predizione-misurata WP-48).

**C3 — Forma della leva: N arene vs una arena + `reset()`.** Stogov (A-DS-69)
nomina la variante `reset()`; Matsakis/Hejlsberg parlano di arena per unità.
*Riconciliazione*: **default = una `Bump` + `reset()` fra le unità**
(bumpalo trattiene il chunk maggiore ⇒ niente ri-raddoppio, niente churn di
mmap/commit, e la reuse delle pagine è garantita dall'arena invece che
sperata dall'allocatore — decisivo perché LEVER-2 ha provato che mimalloc non
decommitta). **Fallback = arena per unità** se `reset()` non è compatibile con
i borrow del lowering. Il picco atteso è lo stesso; cambia il rischio.

**C4 — Rank 2 vs 3.** Hejlsberg: 2 = precompilato embedded, 3 = condiviso
Arc (il condiviso aiuta SOLO il server, l'oggetto è il CLI 4,42×; ripple
Rc→Arc + atomics sui path clone-caldi). Stogov: 2 = condiviso per-processo
(è l'omologo Zend, ed è l'unico che attacca lo slope 18,8 MB/worker), 3 =
precompilato. *Riconciliazione*: entrambe le posizioni dipendono da DUE
misure che **oggi non esistono**. Ordine unico condizionale in §4.

**C5 — Posto di battery61 in S-94.0.** Hoare la mette **prima** della leva
(criterio 5); Stogov dice esplicitamente «resta debito separato, non
caricarlo sulla leva»; Hejlsberg mette la leva prima (la leva È l'oggetto).
*Non riconciliato — dissenso registrato*: il team a maggioranza (3 su 4) mette
battery61 DOPO, sotto la condizione-4 (apparato solo se blocca l'oggetto).
**Clausola di ribaltamento**: se la misura di ricezione della leva (§5.4) non
è riproducibile senza battery61 nativa, allora battery61 blocca e sale al
posto 0. Hoare mantiene la sua priorità a verbale.

**C6 — Obbligo Hoare-1 (interleaving classi/funzioni e doppio parse).**
L'audit di Stogov + la verifica PhpToken/Stringable intra-file riducono il
rischio, **ma non cancellano l'obbligo**: resta da provare che
`hoist_function` non leghi classi a hoist-time. Registrato come obbligo
vivo, non come rischio chiuso.

**C7 — «Riduzione ~5,5×» (Matsakis).** Corretta **sull'arena**; sul
footprint CLI la stessa leva porta il rapporto 4,42× a ~2,3-2,7× (≈1,8-1,9×
di miglioramento). Due numeri veri, oggetti diversi: **vietato citare 5,5×
senza nominare l'oggetto.**

---

## 3. Cifra difendibile della leva (predizione ex-ante)

### 3.1 Numeratore — cosa la leva toglie

**Valuta = touched d'arena, per-thread, al picco del preludio.**

```
N = T_tot − T_max
T_tot = 25.795.552 B          (MISURATO: 39.534.144 − 13.738.592, catena chiusa)
T_max = touched del file più grande   (DA MISURARE, §3.4)
stima lineare: T_max ≈ 63,6 × 74.019 = 4.707.608 B (~4,71 MB)
⇒ N ≈ 21.087.944 B (~21,1 MB), banda dichiarata 19–22 MB
```

In valuta di CAPACITÀ (per l'igiene d'arena, non per il footprint):
39.534.144 → ~7,2 MB, cioè **~5,5×** (Matsakis) / «5-8 MB» (Stogov):
entrambi CONFERMATI come cifre di capacità.

### 3.2 Denominatore — su cosa il risparmio si legge

**`peak_footprint` del CLI, binario di parità d5ce86e3342f3926 (pinnato):**

```
D  = 44.630.520 B   (phpr hi.php)      | controllo: 44.597.728 (refl.php)
Dr = 10.093.048 B   (oracle php -n)    | rapporto oggi = 4,42×
```

**Argomento a supporto che il picco È la fase preludio** (nuovo, dai dati già
committati): due script semanticamente lontanissimi — `hi.php` e `refl.php` —
hanno lo stesso picco entro lo **0,073%**. Un picco indipendente dallo script
è fissato da una fase indipendente dallo script: lo spike del preludio. Questo
è ciò che rende la leva la leva.

### 3.3 Predizione firmata (WP-48), con il coefficiente di trasferimento

Il numeratore d'arena **non è automaticamente** numeratore di footprint:
serve α = Δfootprint / Δtouched.

```
peak_post = 44.630.520 − α × N,     α ∈ [0,8 ; 1,0] atteso
⇒ peak_post ≈ 23,5 – 27,7 MB       ⇒ rapporto vs oracle 2,33× – 2,74×
FALSIFICATA se peak_post > 40 MB (α < 0,25: il canale non è quello nominato)
FALSIFICATA se peak_post < 21 MB (α > 1,1: risparmio non spiegato dalla leva)
```

α ≈ 1 è atteso perché con `reset()` la reuse delle pagine è interna
all'arena; con N arene separate α dipende dalla reuse di mimalloc (che NON
decommitta — LEVER-2 — quindi le pagine liberate restano committed e
riusabili: favorevole al picco, che è un high-water).

**Sul multi-worker: nessuna predizione firmabile.** Lo slope 18,8 MB/worker
non è ancora decomposto (committed-free dei sei chunk / live PRELUDE_CACHE /
live UNIT_CACHE-STUBS / residuo theap — Matsakis Q2). Predizione **solo
condizionale**: se una quota f dello slope è residuo committed dello spike,
la leva ne toglie f × (1 − T_max/T_tot) ≈ f × 0,82. Firmarla prima di
A-MS-62/m91 è vietato (KS-MS-95-1).

### 3.4 Come si misura PRIMA di scrivere una riga di leva

Il relatore **non** costruisce il contatore in questa sede: il repo non si
tocca durante il concilio e un build strumentato usato per cifre di footprint
è già vietato (KS-TH-95-1, KB-78-5). La cifra si misura nella sessione della
leva, **prima del codice della leva**, così:

- **M1 — contatore per-unità, parse-only** (A-AH-72 v2 + A-DS-69). Dietro
  `PHPR_PRELUDE_STATS=2`, build ADVISORY: per ciascuna delle 15 unità una
  `Bump` propria, **solo `parse_file`** (nessun hoist ⇒ zero rischio
  semantico, e l'arena tiene l'AST che è il 100% del canale), stampa
  `unit=<file> src=<B> allocated=<B> chunk_remaining=<B> touched=<B>`.
  Fornisce `T_max` e ogni `T_i` REALI, che rimpiazzano la stima lineare.
  **Controllo positivo obbligatorio**: `Σ T_i ≈ 25.795.552 ± 10%`; se la
  somma non torna, il contatore è rotto e la predizione è nulla (lezione
  WP-72: un contatore che non ha un controllo positivo non è un contatore).
- **M2 — dente sulla semantica bumpalo** (A-TH-75). Unit test su
  bumpalo 3.20.3 che asserisce `allocated_bytes` = Σ capacità e
  `chunk_capacity` = residuo del chunk corrente, + correzione del commento
  lower/mod.rs:1013-1015. La ricostruzione della catena qui sopra chiude
  l'aritmetica a residuo zero, ma resta **inferenza** finché non è un dente.
- **M3 — α si misura DOPO, e solo in coppia BUILD-ADIACENTE** stessa sera
  (`/usr/bin/time -l`, MIMALLOC_PURGE_DELAY=0, binari non strumentati).
  Ex-ante α è dichiarato come banda falsificabile, non come numero.

---

## 4. Rank unico delle leve (Hejlsberg ∪ Stogov, con tie-break oggettivo)

| # | Leva | Costo | Deve provare | Guadagno atteso |
|---|------|-------|--------------|-----------------|
| **1** | **Per-file, forma `Bump` + `reset()`** (fallback: arena per unità) | 1 sessione, superficie media-alta (span, sentinella, ordine) | Obblighi O1-O16 (§5) integralmente | CLI 4,42× → **2,3-2,7×**; slope W ignoto |
| **1b** | **Pre-size `with_capacity`** — commit e misura SEPARATI | basso | che il suo Δfootprint sia ≠ 0 (C2) | capacità −13,7 MB; footprint forse 0 |
| **2** | **Precompilato embedded** (`include_bytes!` da build-tool) | alto (derive su tutta la foresta HIR, versioning, determinismo) | A-AH-73 (HIR plain-data, niente `Cell`/`Rc` interni), byte-determinismo dell'artefatto, KS-AH-95-2 (**mai** cache su disco a runtime), parità completa | resto dell'arena **+ il parse CPU** (tocca il 2,06-2,11× full) |
| **3** | **Condiviso per-processo** (Rc→Arc o `&'static`) | ripple su tutta la VM + atomics sui path clone-caldi (rischio full CPU, WP-81) | **PRIMA il numeratore** (A-MS-62/m91: quota preludio dello slope), poi immutabilità post-seed dei decl, poi costo full CPU in coppia. KS-MS-95-3: proposta che non nomina il costo Rc→Arc = vacua | solo server (W−1 thread); zero sul CLI |
| **4** | **Lazy per-unità** | massima superficie | rompe l'invariante id-contigui (mod.rs:793-796) e la parità reflection | duplicato della 1 |

**Tie-break 2↔3 (registrato come regola, non come gusto)**: le due misure
mancanti decidono l'ordine.
- Se il **residuo CLI post-leva-1 resta ≥2× oracle** ⇒ la 2 è la prossima.
- Se **m91 nomina ≥50% dei 18,8 MB/worker come live `PRELUDE_CACHE`** ⇒ la 3
  sale al posto 2 (il numeratore esiste e vale il ripple).
- Se **entrambe** ⇒ prima la 2 (paga anche il parse CPU, e non ha ripple).
- Se **nessuna** ⇒ il fronte footprint-preludio si sospende, non si chiude
  (A-DS-70 + legge no-front-closure).

Le due misure sono già negli obblighi (O3 e O17): il rank si risolve da sé
alla fine della sessione della leva.

---

## 5. Obblighi di prova, ordinati (unione senza duplicati)

Fonti fuse: i 7 obblighi di Hoare (A-TH-76), F1-F8 + A-DS-66/67/68/69 di
Stogov, A-MS-62/63/64 di Matsakis, A-AH-72/73 di Hejlsberg, tutti i KS delle
quattro sedie.

### Fase 0 — PRE-CODICE (niente riga di leva prima che questi siano verdi)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O1** | Dente sulla semantica bumpalo (`allocated_bytes` = Σ capacità; `chunk_capacity` = residuo) + correzione commento lower/mod.rs:1013-1015 | A-TH-75 | `cargo test --release`; **KS-TH-95-3** (predizione firmata col 39.534.144 senza questo dente ⇒ NULLA) |
| **O2** | Contatore per-unità parse-only (M1) con controllo positivo `Σ T_i ≈ 25,8 MB ±10%` | A-AH-72 v2 + A-DS-69 | il numero scritto nel design PRIMA del codice; **KS-AH-95-1**, **KS-DS-95-3** |
| **O3** | Predizione WP-48 **firmata** nel design: N = 25.795.552 − T_max; D = 44.630.520; α ∈ [0,8;1,0]; soglie di falsificazione | relatore (§3.3) | coppia BUILD-ADIACENTE, stessa sera |
| **O4** | Fixture **F1-F8** oracle-morse committate | A-DS-67 | **KS-DS-95-2** (codice prima delle fixture ⇒ STOP) |
| **O5** | Checker cross-file forward-ref (extends/implements) come gate **pre-nascita**, scope dichiarato (NON copre const-expr cross-file né attributi) | A-DS-68 + Hejlsberg Q1(a) | exit-code del checker committato |
| **O6** | Dente `assert_static<LoweredPrelude>` + assenza di `unsafe`/`Box::leak`/`transmute` nel modulo | A-MS-64 | `cargo test` + dente di pattern |
| **O7** | Se il probe tocca l'alloc-path: env letta in `main()`, nessun `thread::current()` nel `GlobalAlloc` | A-TH-73, A-TH-74 | build del probe |

### Fase 1 — ATTUAZIONE (stesso commit di tutti i gate)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O8** | Nome unità `b"prelude"` **identico al byte** per ognuna delle 15 unità + unit test che lo asserisce; alternativa ammessa: predicato unico migrato in **tutti** i ~20 siti — **mai a metà** | A-DS-66 | **KS-DS-95-1** (rinomina senza migrazione ⇒ STOP) |
| **O9** | `hoist_function` NON lega classi a hoist-time (oppure due passate = doppio parse, **da misurare**, non da assumere) | Hoare-1, C6 | audit + F6/F8 + costo CPU misurato |
| **O10** | Ogni frammento resta unità di compilazione con lo scoping odierno (declare/namespace: il file NS separato esiste per questo) | Hoare-5 | corpus per NOME |
| **O11** | `MAIN_CHAIN_FP`: input enumerati aggiornati e **falsifier che si muove nello stesso commit** (mutare uno qualsiasi dei file muove il fp) | Hoare-6 + A-AH22 esteso (F8) | il falsifier che MORDE, non che passa |
| **O12** | Zero `unsafe`, zero leak `'static` dell'arena | **KS-MS-95-2** | grep-dente + review |
| **O13** | Pre-size in commit **separato**, con misura propria | C2 / A-MS-63 declassato | coppia adiacente dedicata |

### Fase 2 — GATE (stesso commit dell'attuazione)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O14** | Gate parità **per NOME**: corpus 1418 + refl 290; ORM 3E/13F + hk 1665 per la ricertificazione della baseline | Hoare-3, Stogov Q3, ricetta ORM | **KS-TH-95-2**: UNA divergenza per NOME ⇒ **leva RESPINTA**; vietato adattare i gate |
| **O15** | F1-F3 liste INTERE `get_declared_classes/interfaces/functions` + count byte-id; F4 campione per file (`getFileName`/`getStartLine` === false, `isInternal` === true); F5 eccezione lanciata DA codice preludio (nessun `prelude:<riga>` affiora); F6 autoloader-logger a zero invocazioni; F7 "Cannot redeclare" senza file/riga; F8 snapshot name→id invariato | Stogov Q1 | baseline phpr d5ce86e3 per F1-F3/F8; oracle vivo per F4/F5/F7 |
| **O16** | **battery-91pre** (MAI girata: la ricompila la fa scattare) nella stessa sessione | Stogov Q4, Hejlsberg Q5 | la battery |

### Fase 3 — RICEZIONE E DEBITI

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O17** | Live-delta on-thread attorno a `get_or_init` di `PRELUDE_CACHE`, con `heap=<ptr>` in banda; slope decomposto per NOME (committed-free / PRELUDE_CACHE / UNIT_CACHE-STUBS / residuo theap) | A-MS-62, A-MS-53 | **KS-MS-95-1**: «slope attribuito» senza heap=`<ptr>` e senza separazione per NOME ⇒ vietato |
| **O18** | Misura di ricezione: hello + refl CLI pre/post in coppia BUILD-ADIACENTE, slope W{1,4}; α calcolato e confrontato con O3 | Hoare Q4, Stogov Q4 | **KS-TH-95-1**: nessun build strumentato per cifre di footprint |
| **O19** | Precondizione della via 2: dente che tiene `ClassDecl`/`FnDecl` **plain-data** | A-AH-73 | `cargo test` |

---

## 6. Ordine S-94.0 dal team-leva (FONDAMENTALI-first)

**Principio applicato**: l'oggetto è il footprint del CLI (4,42×) e la leva è
l'oggetto; l'apparato entra solo se blocca (condizione 4), con timebox.

0. **(≤1h, tutta misura)** O1 (dente bumpalo) → O2 (contatore per-unità
   parse-only + controllo positivo) → **O3: predizione firmata nel design**.
   In parallelo, se il canale è pronto: **O17 prima metà** (A-MS-62,
   mezz'ora, un thread) — nomina la metà retained prima di ogni leva
   (priorità 1 di Matsakis).
1. **Pre-nascita**: O4 (F1-F8) + O5 (checker cross-file) + O6 committati
   PRIMA del codice (priorità 1 di Stogov).
2. **Leva**: forma `Bump` + `reset()` per unità (9 file splittati + le 6
   unità esistenti), O8-O12 nello **stesso commit** dei gate O14-O16.
   Pre-size (O13) in commit separato con misura propria.
3. **Gate completi per NOME** + battery-91pre; ricertificazione baseline
   (ORM/hk).
4. **Ricezione**: O18 (coppia adiacente stessa sera, hello+refl, slope
   W{1,4}); α calcolato; predizione↔misurato a verbale. Se O3 è falsificata,
   la leva si dichiara REFUTATA con la sua cifra — non si riscrive la
   predizione a posteriori.
5. **Completamento** O17 (attribuzione slope su m91) ⇒ risolve il tie-break
   2↔3 del rank.
6. **Debiti separati, non caricati sulla leva**: battery61 nativa (mezza
   sessione — **con la clausola di ribaltamento C5**); gli emendamenti al
   checker (A-AH-69/70/71) restano a verbale, finestra apparato successiva
   (fuori perimetro team-leva: Q4c di Hejlsberg è capitale e va al
   team-cifre).

---

## 7. Conflitti con la posizione di ciascuna sedia (esplicito)

**Hoare (1)** — Accolto: numeratore = touched 25,8 MB (confermato per
aritmetica chiusa dal relatore); i 7 obblighi entrano tutti (O9-O11, O14);
A-TH-73/74/75 accolti. **Conflitto**: il suo ordine mette battery61 **prima**
della leva; il team la mette dopo con clausola di ribaltamento (C5). Suo
obbligo-1 mantenuto vivo (C6) contro chi lo darebbe per chiuso dall'audit.

**Matsakis (2)** — Accolto: RC-MS-95-1 (la cifra pubblicata è sbagliata),
A-MS-62/64, KS-MS-95-1/2/3. **Conflitti**: (a) il suo «~7 MB/thread» e la
«riduzione ~5,5×» sono cifre di **capacità** — sul footprint valgono ~4,7 MB
e 4,42×→~2,4× (C1, C7); (b) **A-MS-63 declassato**: il pre-size è igiene,
non numeratore, perché la coda mai toccata probabilmente non è footprint
fisico — commit e misura separati (C2).

**Stogov (8)** — Accolto: refutazione della cifra e del rischio inquadrato,
audit cross-file (corroborato dal relatore su PhpToken/Stringable),
F1-F8, A-DS-66/67/68/69/70, KS-DS-95-1/2/3, e la sua tesi «battery61 non si
carica sulla leva» (contro Hoare). **Conflitti**: (a) il suo «~97×» è
capacità/sorgente, il rapporto footprint-rilevante è 63,6× (C1); (b) il suo
rank 2 = condiviso è reso **condizionale** e non a priori (C4); (c) la sua
variante `reset()`, che lui offre come alternativa, è **promossa a forma
DEFAULT** della leva (C3).

**Hejlsberg (4)** — Accolto: rank come scheletro, A-AH-72 (specificato:
parse-only per unità, Σ come controllo positivo), A-AH-73, KS-AH-95-1/2
(cache su disco a runtime NEGATA), l'argomento statico su `extends`
risolto a `ClassId` al lowering. **Conflitti**: (a) «la coda 13.738.592 è il
costo evitabile» → corretto in «costo di capacità, effetto di footprint non
provato» (C2); (b) il rank 2↔3 diventa condizionale a due misure (C4);
(c) la sua refutazione capitale Q4c (append in-window mai autenticati) è
**fuori perimetro** di questo team e va inoltrata al team-cifre — qui è
registrata, non giudicata.

---

*Nessuna cifra di questo verbale è consumabile senza il suo giudice. La sola
cifra qui dichiarata MISURATA è `T_tot = 25.795.552 B` (aritmetica chiusa a
residuo zero sui dati già committati); tutto il resto — `T_max`, α,
`peak_post`, la quota di slope — è DA MISURARE, con il come già scritto.*
