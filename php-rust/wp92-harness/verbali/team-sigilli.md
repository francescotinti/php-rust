# TEAM-SIGILLI — Concilio WP-92 (relatore su verbali 1-Hoare, 2-Matsakis)

Fonti VINCOLANTI: `verbale-1-hoare.md`, `verbale-2-matsakis.md`. Questo
documento riconcilia e ordina; non emenda né attenua i verbali.

Verdetti individuali: entrambi **CONCORDO CON EMENDAMENTI**; entrambi
dichiarano **nessuna refutazione capitale** — le cifre di S-90.0 restano
in piedi, i buchi sono nelle GUARDIE IN AVANTI. Il team conferma: nulla
qui declassa una figura già emessa; tutto qui deve mordere PRIMA di
S-91.0.

## CONVERGENZE

1. **Il pin `==2` sui siti arm() non è un census.** Stesso vizio da due
   lati. Hoare Q2.1: `narm` (gate-lever-pins 1346) riconosce solo
   `let <nome> = ProbeWindow::arm()` — `let mut w =`, `let w: ProbeWindow =`,
   `let (w,_) =`, `drop(ProbeWindow::arm())`, arm in posizione
   d'espressione sono invisibili sia a `narm` sia a `nsil`: un TERZO sito
   passa con narm==2/nsil==0. Il pin morde la CONVERSIONE dei 2 siti noti
   (2→1 FAIL), non l'AGGIUNTA. Matsakis Q3: la cintura A-MS41 conta le
   grafie `.set(`, non i call-site — formalmente regge, ma la semantica
   si è indebolita (`probe_active=true` non identifica più UN solo
   strumento). Rimedi **complementari, non alternativi**: A-TH-57 (census
   totale lessicale, `grep -c 'ProbeWindow::arm'`, ==2) chiude
   l'aggiunta a VISTA; A-MS-54 (nesting-tooth: `debug_assert!(!flag)` o
   contatore in `arm()`) chiude l'annidamento a ESECUZIONE. Nessuno dei
   due copre il buco dell'altro: un terzo sito scritto in grafia esotica
   sfugge al dente runtime finché non è annidato; un annidamento tra i
   due siti già contati sfugge al census. Kill-switch coerenti e
   cumulativi: KS-TH-92-1 (ntot≠narm ⇒ figure con probe VOID) e
   KS-MS-92-3 (nuovo sito arm senza dente anti-nesting nella STESSA
   delibera ⇒ attribuzione VOID).

2. **Ciò che il compilatore può uccidere non va inseguito con la regex —
   ma la belt resta finché il recinto non c'è.** Hoare Q2.2 trova la
   QUINTA grafia su `CENSUS_PROBE_ACTIVE` (`with(Cell::set)`,
   `with(|f| Cell::set(f,true))`, corpo a graffe `|f| { f.set(true) }`)
   invisibile alle quattro reti A-MS48 (A-MS-emend). Matsakis conferma
   dal lato tipo/visibilità (linea A-MS46, WP-91): il perimetro di
   privacy è ancora l'intero `mod implementation` (~1900 righe, test
   inclusi). Convergenza: la quinta rete è una toppa a vita corta, ma
   dovuta finché A-MS46 non è in albero.

3. **Un abort/una guardia devono essere ATTRIBUIBILI al sito.** Matsakis
   Q2.1: i due `eprintln` di abort (teardown r.386-412 e `/__census_self`
   r.521-535) hanno stringhe IDENTICHE ("A-MS31; instrument, not
   worker") ⇒ forensics di campagna abortita cieca sul sito (A-MS-52,
   KS-MS-92-2). Stessa famiglia di Hoare Q3.1: `npdef` conta solo
   `^probe_in()` a colonna 0 ⇒ una ridefinizione indentata o in forma
   `function probe_in {` passa (A-TH-60, KS-TH-92-2): in entrambi i casi
   **due entità distinte condividono la stessa impronta osservabile** e
   il ledger non può separarle. Regola comune: ogni sito/definizione
   duplicabile porta un tag proprio (stringa distinta, `&'static str`),
   e il conteggio è di CENSUS (tutte le grafie), non di forma canonica.

4. **Il cap/la forma DICHIARATA deve essere quella che morde.** Matsakis
   Q1.1: la guardia theap confronta `capacity()`, non `MAX_THEAPS`
   (memcensus r.1147) — `reserve_exact` può concedere >64 slot, quindi
   `heaps_total` può superare 64 con `heap_overflow=0`: il "declared cap
   64" non è il cap effettivo (A-MS-51, KS-MS-92-1). Hoare Q3.2 dice la
   stessa cosa sul form-pin noprobe: `nform==1` è di FORMA, non di
   RAGGIUNGIBILITÀ (`false && strings … | grep -q` lo soddisfa a
   rilevazione svuotata) — residuo fuori portata lessicale, ma **va
   DETTO** nel testo dei sigilli, non lasciato implicito.

5. **I contratti assunti vanno scritti come pin, non raccontati.**
   Matsakis Q1.3: identità heap = `last_mut()` è corretta solo se visita
   heap e visita aree sono strettamente sequenziali — contratto C
   assunto, mai dichiarato; e lo scarto del puntatore `_h` rende
   IMPOSSIBILE il join con `mi_bin_thr_sum` (A-MS-53, rilevante per la
   collision VSELF / iter-3). Hoare Q4: la doc TLS A-TH54 omette il
   terzo stato (chiave RefCell MAI inizializzata sul thread, primo
   accesso in teardown — std non garantisce) e il percorso silente è un
   canale di **scrittura persa** (uc_log in buffer che muore senza
   flush), non solo di mancato panic (A-TH-61). Convergenza: enumerazioni
   incomplete e contratti impliciti sono lo stesso difetto.

6. **Il pin A-MS47 non è l'unica lettera esposta al refactor** (nodo
   emerso dalla riconciliazione, vedi CONFLITTI/§ordine): il `narm`
   attuale esige `ProbeWindow::arm\(\)` a parentesi VUOTE.

## CONFLITTI E TENSIONI DI METODO

**Nessun conflitto frontale.** Entrambe le sedie assolvono le cifre di
S-90.0 e attaccano solo le guardie. Restano tre tensioni, tutte
risolvibili per composizione:

- **T1 — Belt vs recinto (ricorrente da WP-91).** *Hoare*: estendere la
  belt lessicale (A-TH-58 use monoriga `as`, A-TH-59 dot-path tollerante
  ai commenti + stato nome-pendente su tre righe, A-TH-57 census,
  A-TH-60 census probe_in): il rilevatore di produzione deve riconoscere
  la grafia PIÙ NATURALE, che oggi è l'unica scoperta (`use crate::vm::CachedUnit as CU;`).
  *Matsakis*: spostare il giudizio dove muore a compilazione o a
  esecuzione (A-MS46 mod probe annidato; A-MS-51 `Box<[TheapAgg]>` a
  lunghezza fissa; A-MS-54 debug_assert), tenendo la belt come cintura.
  Composizione: la belt copre il residuo, il recinto riduce il residuo.
  Non è una scelta, è un ordine.

- **T2 — A-MS46 riduce la superficie dei sigilli? SÌ, ma solo su UNA
  delle tre famiglie.** Valutazione del team:
  * **Scrittura del flag** (A-MS41 `.set(` ==2, A-MS48 quattro reti +
    A-MS-emend quinta): superficie da ~1900 righe a ~40 ⇒ riduzione
    REALE; dopo A-MS46 le grafie point-free/UFCS/graffe restano
    tecnicamente eludibili ma dentro un modulo che un umano legge
    interamente, e i test del modulo muoiono a compilazione. La belt
    scende da giudice a cintura.
  * **Siti `arm()`**: nessuna riduzione. `arm()` resta esportato
    `pub(super)` e i call-site (2 oggi: worker_pool 385, 520) vivono
    nell'intero `implementation`. A-TH-57 e A-MS-54 NON sono assorbiti
    da A-MS46 e vanno fatti comunque.
  * **`probe_in` / noprobe** (A-TH-60, KS-TH-92-2, form-pin): superficie
    bash dell'harness, ortogonale al refactor Rust. Zero riduzione.
  Conclusione: A-MS46 è un moltiplicatore per la famiglia 1, neutro per
  le altre due ⇒ **non è un motivo per rinviare i sigilli-subito**.

- **T3 — Vincolo di SEQUENZA scoperto in riconciliazione (nuovo, non in
  nessuno dei due verbali).** A-MS-52 e A-MS-54 convergono su una sola
  modifica di firma: `arm()` che porta un `&'static str` di sito (lo dice
  Matsakis stesso in A-MS-52) più il dente anti-nesting. Ma quella firma
  **rompe la lettera dei pin attuali**: `narm` (r.1346) esige
  `ProbeWindow::arm\(\)` a parentesi vuote ⇒ con l'argomento diventa 0 e
  il gate FAIL-CLOSED (comportamento corretto, ma il gate va aggiornato
  nella STESSA delibera, disciplina A-PP48). Corollario vincolante per
  A-TH-57: il census va scritto in forma **prefisso** (`ProbeWindow::arm`
  senza parentesi), altrimenti nasce già cieco alla propria evoluzione.
  Stesso vincolo su A-MS46: spostando flag/ProbeWindow in `mod probe`,
  le finestre awk e i pin `npriv` di A-MS43/A-MS48 sono ancorati alla
  forma odierna e vanno riancorati nello stesso commit.

- **T4 — Classificazione di A-TH53 (disaccordo con la bozza d'ordine).**
  Il brief lo mette tra i DESIGN; design90.md §A-TH53 assegna però
  l'attuazione del pin Cargo.toml alla «prossima revisione dei sigilli».
  Posizione del team: la **ratifica di policy** è design (già fatta in
  WP-91), il **pin** è meccanico (due `grep -c` su
  php-runtime/php-server Cargo.toml, ==0) ⇒ va nei sigilli-subito. Il
  residuo dichiarato (grafia 7, macro a incollaggio) resta fuori portata
  lessicale fino ad A-MS27.

## PRIORITÀ PER S-91.0

### SIGILLI-SUBITO (meccanici, solo script/doc, chiudono kill-switch attivi)
Nessuno tocca runtime; tutti eseguibili senza il ciclo di build.

1. **A-TH-57 — census totale arm**, `ntot = grep -c 'ProbeWindow::arm'`
   su righe di codice, pin **==2**, in forma PREFISSO (no `\(\)`), con
   FAIL se `ntot != narm`. Chiude Q2.1 e arma KS-TH-92-1. *Primo perché
   è il buco che lascia entrare un TERZO strumento senza accorgersene.*
2. **A-TH-60 — census delle definizioni `probe_in`**,
   `grep -cE '^[[:space:]]*(function[[:space:]]+)?probe_in'` ==1.
   Chiude Q3.1 e arma KS-TH-92-2 (decoy-front/gutting-behind).
3. **A-MS-emend — quinta rete su `CENSUS_PROBE_ACTIVE`**: point-free
   `with(Cell::set)`, UFCS-Cell `with(|f| Cell::set(f,…))`, corpo a
   graffe `|f| { f.set(true) }`. Toppa a vita corta (cade con A-MS46) ma
   dovuta nell'interim.
4. **A-TH-58 — use monoriga con `as`**: ramo `use [^;{]*(CachedUnit|VmGate) as `.
   È la grafia più naturale ed è l'unica scoperta.
5. **A-TH-59 — dot-path tollerante + tre righe**: skip commenti/blank
   sul ramo dot e stato nome-pendente in TH49ML_PROG (Q1.2-1.4).
6. **A-TH53 (pin) — bando `paste`/`concat_idents`** nei Cargo.toml di
   php-runtime/php-server, ==0 (vedi T4).
7. **A-TH-61 — doc A-TH54, terzo stato**: chiave TLS mai inizializzata
   al teardown + canale di scrittura PERSA (uc_log senza flush). Costo
   zero, KH91-1 già attivo sui claim.
8. **Residui DICHIARATI** (non chiudibili lessicalmente, da scrivere nel
   commento dei sigilli accanto alla lane A-TH53/A-MS27): form-pin
   noprobe non è raggiungibilità (Q3.2); `::` spaziato in posizione di
   TIPO (firma/return/turbofish) non è mint diretto ma è canale d'alias
   (Q1.5).

**Regola di ordine interna**: 1-2 prima di ogni altra cosa (sono i due
che governano kill-switch che rendono VOID figure di campagna); 3-5 in
qualunque ordine; 6-8 in coda, costo nullo.

### DESIGN (vanno COL CODICE, stesso commit del dente — disciplina A-PP48)

Ordine proposto, dal recinto che riduce superficie al dente che misura:

- **D1 — A-MS46 `mod probe` annidato** (~40 righe, `pub(super)`).
  PRIMO tra i design: è l'unico che RIDUCE la superficie dei sigilli
  (famiglia 1, vedi T2), e fa decadere KS-MS-91-1 a compilazione.
  Vincolo: riancorare finestre awk / pin `npriv` di A-MS43/A-MS48 nello
  stesso commit; A-MS41/45/47/48 restano come belt.
- **D2 — A-MS-52 + A-MS-54 insieme** (unica modifica di firma: `arm(site:
  &'static str)` + `debug_assert!(!flag)`/contatore; tag di sito nei due
  eprintln e nella riga del panic-hook). Vincolo T3: aggiornare `narm`
  (r.1346) e A-TH-57 nello stesso commit; senza dente anti-nesting il
  terzo sito è VOID per KS-MS-92-3.
- **D3 — A-MS-51 cap effettivo**: guardia su `len()==MAX_THEAPS` o
  `heaps: Box<[TheapAgg]>` a lunghezza fissa; arma KS-MS-92-1. Piccolo,
  ma è una LETTERA in-band oggi falsa ("declared cap 64").
- **D4 — A-MS-55 REQ_NS esclude `/__census_self`**: `req_t0` non deve
  avvolgere il ramo census_self (r.507-552). Tocca il canale delle
  figure per-request ⇒ da coordinare col team-misura.
- **D5 — A-MS-53 `heap=<ptr>` in-band nelle righe `mi_theap_pages`**:
  prerequisito del join anti-collisione con `mi_bin_thr_sum` ⇒ serve a
  VSELF iter-3. Rinviato al team-misura per la priorità relativa (è un
  ingrediente di misura, non un sigillo).
- **D6 — A-TH55 ordine canonico KIND intra-putord** in a_ds26/a_ds38:
  invariato, va con la prossima sessione che tocca quei test.
- **Aperti fuori perimetro sigilli**: contratto di sequenzialità della
  visita C (Q1.3) e headroom di stack per `TheapAgg::zeroed()` ~3,9 KB
  su frame `extern "C"` con `RUST_MIN_STACK` non pinnato al blocco
  (Q1.2) — da instradare al team-misura come pin di canale.

## NOTA AL TEAM-MISURA
A-MS-53 (heap ptr in-band, VSELF iter-3), A-MS-55 (inquinamento REQ_NS
da census_self), A-MS-51 (cap dichiarato ≠ cap effettivo ⇒ KS-MS-92-1
sul canale theap) e i due pin di canale sopra toccano figure, non
sigilli: la priorità relativa spetta a loro.

— relatore team-sigilli, Concilio WP-92
