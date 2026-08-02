# COUNCIL_WP91_REVIEWS.md — Concilio a 9 sedie su S-89.0 (due forge chiusi + campagna measure89 + attribuzione refutata) + programma WP-90(sessione)

Protocollo A DUE FASI (decisione utente 2026-08-02): fase 1 = 9 verbali
INDIPENDENTI (nessuna sedia ha visto le altre; file in `verbali/`);
fase 2 = 4 team tematici con relatore (team-cifre Klabnik+Hejlsberg+
Gregg, team-misura Bak+Leijen, team-sigilli Hoare+Matsakis,
team-confine-engine Pedersen+Stogov; note in `verbali/team-*.md`).
I verbali individuali sono la fonte VINCOLANTE; le note di team
alimentano la sintesi. Token-lean: nel contesto di sessione sono passate
SOLO le ricevute ≤80 parole.

## ⚖️ SINTESI DI CONVERGENZA (in coda, dopo verbali e note di team)

## VERBALI INTEGRALI (fase 1)

---

# VERBALE — Tony Hoare, sedia 1, Concilio WP-91

VERDETTO: CONCORDO CON EMENDAMENTI.

## Q1 — A-TH49: i decoy mordono, ma sulla COPIA sbagliata
I 5 decoy mordono (self-test 1/1/1/2/1, exit 2 se no, stesso commit —
gate-lever-pins.sh:1144-1164). Vizio strutturale: il self-test esercita le
regex PER-GRAFIA; lo sweep (riga 1167) usa un'alternazione RI-DIGITATA.
TH33RE/TH39RE sono variabili condivise; TH49 (e TH44) no: un branch perso
nello sweep lascia il self-test verde — positivo che non esercita il
rilevatore di produzione (classe A-PP48). Grafie che ANCORA sfuggono
(verificate contro ogni pin/sweep del file):
1. punto a fine riga, nome alla riga DOPO (`u.` ⏎ `vm_gate(x)`): r2 copre
   solo nome-a-EOL; fuori dai 3 file pinnati nessuno sweep matcha
   `vm_gate(` senza punto (TH38_SWEEP esige il punto).
2. riga interposta nello split (`u.vm_gate` ⏎ `// x` ⏎ `(x)`): r2 azzera
   `pend` su ogni riga intermedia.
3. commento non adiacente al punto (`. /*c*/production_gate(` — r3 esige
   `[.]/[*]`), commento tra nome e parentesi (`.production_gate/*c*/(`,
   elude ANCHE i site-pin ==1/==2 nei file pinnati), commento multilinea.
4. `::` spaziato multi-segmento (`= crate :: vm :: CachedUnit;`) e
   `= :: vm :: CachedUnit`: r4 copre un solo segmento o global senza spazi.
5. use-group MULTILINEA e `use …CachedUnit` ⏎ `as CU;`: r5/ua line-based.
6. r# sui TIPI: `vm::r#CachedUnit` (r1 copre solo i metodi; pq/ua esigono
   `::CachedUnit` adiacente).
7. macro a incollaggio di identificatori (`paste!` `[<production _gate>]`):
   fuori portata di QUALSIASI sigillo lessicale — la chiusura vera resta
   A-MS27 (rustc giudice).

## Q2 — A-TH50: prima metà ESATTA, seconda metà INESATTA
Prima metà (vm/mod.rs:16533-16545) esatta rispetto a std sul path
`target_thread_local` (macOS/Linux/Windows): `const{Cell::new}` di tipo
senza Drop ⇒ EagerStorage senza stato né dtor, `with` infallibile. Caveat
non dichiarato: sul fallback os-TLS anche i no-Drop registrano il dtor che
libera il box. La SEDE dichiarata NON regge come scritta: «fails fast at
the first TLS key WITH a destructor it touches» vale SOLO se il distruttore
di quella chiave è GIÀ corso. L'ordine dei distruttori TLS è NON
SPECIFICATO (std: "may panic if the destructor has previously been run"):
un put da TLS-destructor che corre PRIMA dei teardown di
UNIT_CACHE/UC_STATS/UC_LOG_BUF trova le chiavi vive e COMPLETA in silenzio,
guard armato e disarmato regolarmente. Il vero sigillo è l'ultima frase
(«nessun sito di put in TLS destructor oggi»), non il fail-fast.

## Q3 — A-TH51: l'uguaglianza legale HA un buco; l'adiacenza NON copre
Sede reale: test `a_ds26_main_evicted_tripwire_bites_on_injection`
(19402; il report S-89.0 scrive "a_ds36" — drift di nome: A-DS36 è
l'invariante, a_ds26 il test). Uno STESSO put emette fino a TRE righe con
lo stesso putord: `supersede` (16675), `main_evicted` (16694), `evict fp`
(16701). L'adiacenza pinnata copre SOLO la coppia (19499-19507); la riga
supersede dello stesso put può migrare da prima a dopo la coppia senza
morso (non-decrescente ✓, adiacenza ✓). Cross-put morde (decremento);
intra-put no. Righe senza putord: invisibili al filtro, non dichiarato.

## Q4 — A-TH48: finestra fail-closed sul rename, eludibile in tre modi
Rename/brace-style anomalo ⇒ finestra vuota ⇒ FAIL: fail-closed ✓. Ma:
(a) DOPPIA definizione: awk finestra la PRIMA `^probe_in()`, bash esegue
l'ULTIMA — decoy davanti, gutting dietro; (b) senza `^}` a colonna 0 la
finestra ingoia il resto del file (decoy esterni la soddisfano); (c) un
`echo "grep … $PAYLOAD"` DENTRO probe_in soddisfa ancora il pin (il vizio
A-TH45 sopravvive nella finestra). Il belt --selftest NON discrimina la
riga-payload: il payload CONTIENE `vm_gate_probe` come sottostringa ⇒
tainted2 è rilevato dalla riga generica 38 anche gutting la 39 —
tainted2 oggi è forward-guard, non falsificatore.

## Emendamenti
- **A-TH52**: single-source TH49RE/TH44RE (variabile unica self-test↔sweep,
  modello TH33RE); il decoy DEVE passare per la regex composta dello sweep
  (≥6 morsi); branch nuovi per le grafie Q1(1-6); r2 con pend che morde le
  righe interposte.
- **A-TH53**: bando PER NOME della dipendenza `paste`/`concat_idents` in
  php-runtime/php-server (pin sui Cargo.toml ==0) finché A-MS27 non chiude;
  limite lessicale DICHIARATO nel commento dei sigilli.
- **A-TH54**: riscrivere la seconda metà di A-TH50: fail-fast
  ORDER-DEPENDENT (solo se la chiave RefCell è già distrutta), put
  silenzioso possibile nell'altro ordine; caveat os-TLS. Correggere
  "a_ds36"→a_ds26 nel session file.
- **A-TH55**: in a_ds26/a_ds38, dentro ogni run di putord uguale asserire
  l'ordine canonico dei KIND (supersede* → main_evicted → evict-fp).
- **A-TH56**: noprobe: pin `^probe_in()` ==1; finestra awk CHIUSA
  obbligatoria (ultima riga `}`); pin di FORMA sulla riga di rilevazione
  (`strings -- "$1" … grep -q '<payload>'` ==1 dentro probe_in); tainted2
  declassato a forward-guard DICHIARATO.

## Kill-switch
- **KH91-1**: ogni claim fail-fast-teardown fondato sulla lettera
  deterministica di A-TH50 è VOID; vale solo la forma order-dependent.
- **KH91-2**: finché A-TH52 non atterra, i sigilli-grafie TH44/TH49 sono
  ADVISORY (positivo che non esercita il rilevatore di produzione).
- **KH91-3**: doppia definizione di probe_in() o finestra non chiusa nel
  noprobe ⇒ gate noprobe ADVISORY e campagne appoggiate VOID.

Nessuna delle refutazioni tocca il PASS di VERDICT89 (nessun put da TLS
destructor oggi; noprobe attuale rileva comunque; il buco intra-put non ha
prodotto figure). Programma WP-90: confermo l'ordine proposto.

— Tony Hoare, sedia 1

---

# VERBALE — Niko Matsakis, sedia 2, Concilio WP-91

VERDETTO: CON EMENDAMENTI (una refutazione capitale su Q4).

## Q1 — A-MS43: sigillo di visibilità
Verificato: `crates/php-server/src/worker_pool.rs:74-77` — `thread_local!` PRIVATO, nessun `pub`. Il modulo contenitore NON è il file: è il `mod implementation` interno (riga 46, cfg `axum-server`), che si chiude a riga 1929 e contiene ~1880 righe INCLUSO `mod tests`. Il `pub use` (1931-1935) esporta solo `census_probe_active` (lettura, main.rs:512) — `ProbeWindow` è `pub(crate)` dentro un mod privato ⇒ irraggiungibile fuori dal file. Scrittori esterni: morti a compilazione, confermato (belt 1183-1194 coerente; nota: `pub(crate) static` farebbe npriv=0 ⇒ FAIL, corretto). Scrittori interni residui OGGI: nessuno oltre la coppia arm/Drop (righe 104/111, uniche `.set` nel file). MA il perimetro di privacy è l'intero `implementation`: ogni riga futura del modulo (test compresi) può scrivere il flag senza errore di compilazione — lì giudica solo la belt, che ha buchi (Q3). Il sigillo è completo verso l'ESTERNO, sovradimensionato verso l'interno → A-MS46.

## Q2 — A-MS44: #[must_use] e sito d'uso
Sito reale riga 386: `let probe_window = ProbeWindow::arm();` — binding NOMINATO, vivo fino al `drop(probe_window)` riga 406, A VALLE del `catch_unwind` (387-405). Corretto: il hook scatta sul thread in panico DENTRO la finestra; `abort()` a 411 non ripassa dal hook. Però KS-MS-90-3 NON è oggi macchina-distinguibile: `#[must_use]` morde solo l'espressione nuda `arm();` — `let _ = arm()` SILENZIA il lint E droppa subito (il wildcard non lega). La belt (1196) conta solo la PRESENZA dell'attributo, non la forma del sito. Distinzione a macchina esiste ma va armata: `clippy::let_underscore_must_use` (allow di default) oppure pin testuale sul sito → A-MS47.

## Q3 — A-MS45 belt + .done rc=
Belt (gate-lever-pins.sh 1203-1212), buchi PER NOME, tutti interni a worker_pool.rs (visibilità copre l'esterno):
1. `.with(|g| g.set(true))` — binder ≠ `f`: sfugge al conteggio A-MS41 (1105, letterale `|f| f.set(`) e ad A-MS45 (nrepl copre solo replace|update, NON set). Terzo scrittore ⇒ belt PASS. Buco capitale della belt.
2. Grafie `take()`/`swap()` (Cell E LocalKey::take scrivono false) assenti da entrambe le regex.
3. UFCS `LocalKey::set(&CENSUS_PROBE_ACTIVE, true)` — nessun prefisso puntato ⇒ invisibile.
4. Split multilinea (classe A-TH49) — grep per riga.
.done (gate-parity-83p1.sh 81): rc= in-band è giusto, ma il marker NON viene rimosso all'avvio: run uccisa prima della riga 81 ⇒ .done STANTIO della run precedente con rc/git vecchi; il campo git= permette la verifica ma nessuno la impone → A-MS49.

## Q4 — A-DL46-census: aliasing e visita annidata
Aliasing Rust: `theap_heap_cb` (1074-1082) crea `&mut *(arg)`, fa push, poi passa il RAW `arg` alla visita annidata — sotto Stacked/Tree Borrows è SANO perché `acc` non è più usato dopo il push (il raw padre sopravvive al reborrow); margine fragile: un uso di `acc` dopo la annidata sarebbe UB → riderivare da arg. Le callback sono sequenziali sullo stesso thread: nessun overlap di `&mut`.
REFUTAZIONE CAPITALE: il blocco viola il PROPRIO invariante dichiarato (righe 698-700: «the visitor MUST NOT allocate — it walks the very heap that would serve the allocation»; BinTab usa array fissi per questo). `theap_heap_cb` fa `Vec::push` e `theap_area_cb` fa `BTreeMap::entry().or_insert()`: ALLOCANO via mimalloc mentre `mi_subproc_visit_heaps` enumera i heap e mentre `mi_heap_visit_blocks` cammina le pagine del heap di default del thread visitante — che È tra i heap visitati. Mutazione della struttura sotto iterazione lato C: "non è crashato" in m89 non è soundness. La dichiarazione KL-90-3 copre win=0 post-teardown (quiescenza degli ALTRI thread) ma non copre l'auto-allocazione.

## Emendamenti
- **A-MS46**: restringere il perimetro — flag+ProbeWindow+getter in un `mod probe` annidato (~40 righe), export `pub(super)`; anche i test del modulo muoiono a compilazione.
- **A-MS47**: sito d'uso a macchina — belt: ==1 `let <ident> = ProbeWindow::arm()`, ==0 grafie `let _ =`/espressione nuda; in catena `-D clippy::let_underscore_must_use`.
- **A-MS48**: A-MS45 estesa: closure-set con binder generico, take/swap, UFCS LocalKey, decoy che mordono same-commit.
- **A-MS49**: `rm -f "$W/.done"` in testa a gate-parity-83p1.sh; consumer DEVE confrontare git= con HEAD.
- **A-MS50**: census per-theap senza allocazione nei visitor (capacità pre-riservata prima della visita, bins ad array fisso classe BinTab) e `acc` riderivato da arg dopo la annidata.

## Kill-switch
- **KS-MS-91-1**: terzo scrittore del flag in worker_pool.rs a revisione senza A-MS48 ⇒ attribuzione probe ADVISORY.
- **KS-MS-91-2**: righe mi_theap_* raccolte con worker NON joinati/quiescenti ⇒ righe VOID.
- **KS-MS-91-3**: righe mi_theap_* prodotte da visitor che alloca dal subproc visitato (pre-A-MS50) ⇒ ADVISORY, mai verdict-grade.

Firmato: Niko Matsakis, sedia 2, Concilio WP-91.

---

# VERBALE — Steve Klabnik, sedia 3, Concilio WP-91

VERDETTO: **CON EMENDAMENTI — con UNA REFUTAZIONE CAPITALE (A-SK56).**

Metodo: gate invocato SOLO su copie in scratchpad; albero verificato pulito
(`git status --porcelain` = solo `?? php-rust/wp91-harness/`). Copia intatta
di MEASURE89 → PASS (baseline).

## Q1a — A-SK55 TIENE
Forge non committati con nomi che matchano i glob del corpus
(`wp78-harness/measure-out/m89.zzklab-forge.log`, `wp89-harness/zzklab-forge.out`,
`forged=777444111`) + doc doctored che cita `777.444.111 B` → **FAIL** entrambi
("not in committed corpus"). Il corpus HEAD-only morde.

## Q1b — A-SK56 REFUTATA A MACCHINA (capitale)
Corpus estratto dal canale cache: **22.399 token**. Riga piantata su copia:
`b_base rivisto = 19.600.000 B = 18,69 MiB per worker [derivata: 23.000.000 − 3.400.000]`
→ **PASS**. Controllo senza tag → FAIL su ENTRAMBI i token. Gli operandi non
sono misure: sono cifre di **indirizzi esadecimali vmmap**
(`b22c00000-b23000000`, `9f3000000-9f3400000`); **5.262/22.399 token (23,5%)
del corpus esistono SOLO in file vmmap**. La verifica X−Y è esistenziale sulla
CHIUSURA DELLE DIFFERENZE: densità diretta in [19e6,20e6) = 4 valori; con
[derivata] i valori raggiungibili entro ±2% di b_base sono **89.881 (~11,5%)** —
amplificazione ~2×10⁴. Non serve N arbitrario (12.345.678 è irraggiungibile):
basta un N *plausibile*, e quelli abbondano.
Secondo morso, sulla forma REALE del doc: su riga [derivata] è vincolato solo
il token seguito da `B`. `2σ = [11.111.111, 20.745.049] B [derivata: companion /1048576]`
→ **PASS** (bound inferiore fabbricato); senza tag → FAIL. "Figure-scope" è in
realtà "token-seguito-da-B-scope". Nota: `--selftest` resta PASS (verificato,
>120 s: nessuna cache in quel modo) — il suo caso A-SK56 usa un tag SENZA
espressione, quindi morde solo il falsario ingenuo.

## Q2 — A-SK53-bis: tiene, ma due buchi
Fuori M85 `232±1` → FAIL (dente ok). (a) `3.605.572 B ±5%` (banda KL-85-2
**ritirata** da KB-90-2) è in allowlist GLOBALE: **PASS** come banda VIVA in un
doc MEASURE89 — la ritirata è prosa, non dente. (b) le grazie per-doc sono
keyed a una REGEX DI NOME: copia chiamata `MEASURE85_zzforge_RESULTS.md` (che
`--all` peschera dal glob) eredita `±5%` e `232±1` → **PASS**. Corollario:
`MEASURE100_RESULTS.md` con `232 MiB … ±7%` senza companion → **PASS**
(bytes-first spento da `/MEASURE8[4-9]|MEASURE9\d/`); stessa riga in MEASURE89
→ FAIL. Tutta la famiglia A-DL26/A-SK40/A-SK43/A-SK53-bis **scade a WP-100**.

## Q3 — cache AVVELENABILE
`GATE_CIFRE_CORPUS_CACHE` è input non autenticato: `cp corpus.cache poison2;
echo C777444111 >>` → lo stesso doc che senza cache FALLISCE ottiene **PASS**.
Chiave = solo HEAD rev; nessun digest della lista sorgenti, nessun nonce.
`--all` riassegna la var (mktemp) ⇒ battery schermata; ogni invocazione singola
no. Bonus: la cache è anche **esfiltratore** del corpus (l'ho usata per costruire
il forge Q1b).

## Q4 — gG: nessun dente, solo KS
`gate-measure-cifre` è l'UNICO gate che legge i doc (doc-purge scansiona .rs).
Copia di MEASURE89 con `g3`→`g1` (file citato e "generazione MASSIMA") → **PASS**.
Inoltre i .out di generazioni FAIL/superseded stanno nel corpus (`wp89-harness/*.out`):
a questo HEAD 0 token esclusivi di g1/g2 ⇒ buco **latente**, non vivo.

## Emendamenti
- **A-SK60**: abolire l'evaluator X−Y libero. Un [derivata] byte-figure è legale
  solo se ogni operando è risolto per PROVENIENZA (`file:riga` committato,
  riletto a HEAD) e il gate STAMPA la risoluzione; meglio ancora: la cifra
  derivata la emette l'emitter in un .out.
- **A-SK61**: igiene corpus — escludere i digit-run da indirizzi (righe vmmap
  `[0-9a-f]+-[0-9a-f]+`) e i flatten `nodot` dei decimali; stampare cardinalità
  del corpus e pinnarla (budget) per HEAD.
- **A-SK62**: su riga [derivata] giudicare OGNI token ≥3 cifre (bracket 2σ,
  se, conteggi), esentando solo i ricalcolabili a macchina.
- **A-SK63**: allowlist ± da MANIFEST committato (doc path + blob sha + bande
  concesse), non da regex di nome; `3.605.572B±5%` fuori dalla lista globale.
- **A-SK64**: perimetro e bytes-first dal manifest; `--all` FAIL bidirezionale
  (doc in manifest mancante / `MEASURE*_RESULTS.md` fuori manifest).
- **A-SK65**: cache via argv + nonce generato dal padre; env ignorato.
- **A-SK66**: dente generazioni — il doc deve citare `verdict<NN>.a<A>.g<G>.out`
  con G massimo per A e ultimo verdetto PASS; i .out superseded escono dal corpus.

## Kill-switch (serie WP-91)
- **KS-SK-91-1**: finché A-SK60/A-SK62 non atterrano, ogni PASS di
  gate-measure-cifre su doc con [derivata] è NON verdict-grade (forge dimostrato).
- **KS-SK-91-2**: cifra non giudicata perché il NOME del doc è fuori regex ⇒
  PASS del doc VOID; manifest obbligatorio prima del primo doc post-WP-99.
- **KS-SK-91-3**: PASS ottenuto in un process-tree con `GATE_CIFRE_CORPUS_CACHE`
  ereditata = VOID.
- **KS-SK-91-4**: doc che cita generazione non massima o FAIL senza dente ⇒
  la citazione di provenienza non è verdict-grade.

— Steve Klabnik, sedia 3

---

# VERBALE — Anders Hejlsberg, sedia 4, Concilio WP-91

VERDETTO: CONCORDO CON EMENDAMENTI.

## Q1 — battery-attempts.ledger e il dente KS-AH-90-1

La storia si racconta da sola e regge il confronto coi commit: FAIL a
692c697 (epoch 23:48:56, first_fail=lever-fixtures2) → fix 65086e6
(23:49:38) → REFUSE tree-not-porcelain a 65086e6 (23:49:47: il ledger
appena appeso era esso stesso lo sporco) → evidenza committata 05726aa →
PASS 00:07:55 (stamp 3b7effb) → emendamento allowlist 30f5a87 (che
invalida la finestra del terzo PASS: il checker stesso è delta
non-allowlistato) → PASS attempt 4 a 30f5a87 (stamp ed427f4). Epoche e
commit interfogliano senza contraddizioni; anche il porcelain-refuse è
ledgerato come A-AH50 esige.

Riga forgiata: da sola NON legalizza un PASS — il dente A-AH50
(battery-equivalence.sh:329-344) è necessario-non-sufficiente; il carico
positivo resta su stamp 4-campi committato + matrix committato +
sha256(OUT) ricomputato. MA l'allowlist HA aperto un canale: il
prefix-check a granularità di riga (A-SK57, righe 313-328) copre SOLO
battery-stamps.ledger. Il ledger attempts, ora ammesso in finestra, può
essere RISCRITTO in-window (cancellare un FAIL/REFUSE, fabbricare la
riga PASS che il dente esige) senza che nessun dente morda; e il dente
non confronta il campo sha256 della riga col DSHA dello stamp. Refuto
la lettura "allowlist innocua": è innocua solo per il forge, non per la
riscrittura di storia. → A-AH54, KS-AH-91-1.

## Q2 — judge_sha e tether lato giudice

g1=ae4f528f93095979, g2=24cd290ae0a9fc2b, g3=294898431ef72d16: tre sha
DIVERSI, tutti ledgerati (m89.campaign.ledger righe 61-63, esiti
FAIL(9)/FAIL(41)/PASS). Tether lato giudice presente e armato:
verdict89.sh:88-98 ri-verifica mem_hash contro bin[mem-census] del
matrix COMMITTED a HEAD (A-SK59). Il supersede g3>g2>g1 è leggibile dal
solo ledger SOLO applicando la convenzione fuori-banda max-generation
(KS-SK-90-3): epoch crescenti + generation= + esiti, nessun campo
supersedes=. Ho risolto le sha: g1 = verdict89.sh@418def8 (committato),
g3 = @778d96f == HEAD. g2 NON risolve ad ALCUN blob committato: il
giudice che emise FAIL(41) è identificato ma IRRECUPERABILE — un
dangling pointer che vanifica lo scopo di A-AH51 per quella
generazione. → A-AH56, KS-AH-91-2.

## Q3 — recorder rustc e diagnosi del comparatore

Recorder (gate-feature-matrix.sh:88-92): A-AH53 attuato — sample-first,
rifiuto esplicito del valore vuoto, la riga rustc= non nasce mai vuota.
cargo= (riga 94) NON è coperto: il fallback `|| echo unknown` è vivo
(niente pipeline che ne maschera lo status), ma scrive `cargo=unknown`
senza fallire e NESSUN dente a valle legge cargo= — disciplina
asimmetrica. Comparatore (battery-equivalence.sh:221-233): SÌ, ora
distingue — NRUSTC!=1 ⇒ "carries N rustc= rows" (header assente = 0
righe, A-AH47); NRUSTC==1 con valore vuoto ⇒ "header present but VALUE
EMPTY" (A-AH53). → A-AH55.

## Q4 — finestra 30f5a87..ed427f4

`git log 30f5a87..ed427f4` = UN solo commit, ed427f4; tre file: la
matrix-archive feature-matrix.30f5a87.20260803-000923.log (esattamente
quella nominata dallo stamp e dalla riga phase=identity), +1 riga
attempts, +1 riga stamps. SOLO evidenza, tutta allowlistata. La
campagna parte a git=ed427f4 (phase=start epoch 1785709081, 2 s dopo il
commit). Finestra PULITA. Nota: il verdetto SAME-REV CONSUMPTION LEGAL
vive solo nello stdout della campagna, non nel ledger. → A-AH57.

## Emendamenti

- **A-AH54**: estendere il prefix-check A-SK57 (st_judge, riga-granulare)
  a battery-attempts.ledger nella finestra --same-rev; il dente A-AH50
  esiga sha256==DSHA nella riga PASS consumata (triangolo
  attempts↔stamp↔OUT).
- **A-AH55**: A-AH53 esteso a cargo= — il recorder rifiuta valore
  vuoto/`unknown`; il comparatore dichiari cargo= almeno ADVISORY.
- **A-AH56**: judge_sha risolvibile — ogni generazione va giudicata con
  giudice COMMITTED, o la riga è citabile solo come
  "superseded/judge-unrecoverable"; g2 di m89 va così annotata nel doc.
- **A-AH57**: la consumazione --same-rev va ledgerata nel campaign
  ledger (phase=consume, BREV, sha del checker).

## Kill-switch

- **KS-AH-91-1**: delta in-window su battery-attempts.ledger che non sia
  append riga-granulare, O riga PASS consumata con sha256 != DSHA ⇒
  consumazione VOID, battery re-run.
- **KS-AH-91-2**: judge_sha della generazione MASSIMA non risolvibile a
  blob committato a HEAD ⇒ VERDICT VOID; generazioni intermedie dangling
  vanno dichiarate o il doc che le cita è VOID.
- **KS-AH-91-3**: matrix con cargo= vuoto o unknown ⇒ matrix NULL per le
  identità che citano cargo (rustc= resta il giudice primario).

Firmato: Anders Hejlsberg, sedia 4, Concilio WP-91.

---

# VERBALE — Lars Bak, sedia 5, Concilio WP-91

VERDETTO: **CONCORDO CON EMENDAMENTI** (cifre riprodotte al byte; tre refutazioni documentali, nessuna capitale).

## Q1 — Ricomputo dai raw: COMBACIA AL BYTE, con due nei

Riletti i 40 raw `m89.slope*/slope0*` (ultima riga `tag=mi_proc win=0 … commit=`; pin coppia pre/post ==2 verificato su TUTTI i 40 file). Modi dominanti: BASE {156.762.112, 228.065.280, 316.276.736, 388.366.336}, RET0 {154.992.640, 252.575.744, 323.485.696, 389.087.232} — identici al g3. LSQ: b_base=19.575.603,2, se=584.722,9, 2σ=[18.406.157, 20.745.049], a=76.611.584 ✓; b_ret0=19.329.843,2, se=1.319.393,3, 2σ=[16.691.057, 21.968.630] ✓. Formula dell'awk (verdict89.sh:349-357) = OLS testuale, dof=n−2=2 ✓. Nei: (1) a W4 BASE il census ha un **TIE x2/x2** (157.745.152 vs 156.762.112) risolto col min in silenzio (riga :340, deterministico ma non dichiarato; sensibilità Δb=−73.728, immateriale); (2) **i marginali RET0 OUT sono DUE**, non uno: 24.395.776 (W4→8) E 16.400.384 (W12→16, sotto fascia di 29.983 B). MEASURE89_RESULTS.md:50-52 dichiara «un marginale fuori fascia» — **REFUTATO dal g3 stesso** (righe 112/114). Grade ADVISORY comunque corretto.

## Q2 — A-BB57 giusto; la soglia 0,8 NON è nel header

se(b) e giudizio fascia: corretti; δ=0,15 nominata ex-ante (COUNCIL_WP90:134, mia sedia). RET0→ADVISORY corretto (bastava già il primo OUT). Ma il header campagna (measure89-campaign.sh:26-28,137) nomina **SOLO 0,5**: «se b_ret0 ~ b_base la ritenzione è esclusa» non è una soglia. La **0,8 vive solo nel giudice** (verdict89.sh:396, post-run per costruzione g1→g3). La riga «judged against the ex-ante P-RET0 **thresholds**» (plurale) è un **overclaim: REFUTATO**. L'esito però è insensibile alla scelta: ratio=0,987 — qualunque confine in (0,5; 0,987] dà NOT-attributed.

## Q3 — Clamped dt 1-5: lettura sottodeterminata

Con purge_delay=0 il decommit è sincrono al free: «teardown del primo dentro la finestra del secondo» e «purge aggressivo in finestra» sono lo STESSO meccanismo a dt 1-2 (dt1.afirst: primo net=34.024.441 gonfiato da swallow, secondo clamped). Ma **dt5.bfirst refuta la lettura pura**: ENTRAMBI i lati deflazionati di ~4,54 M senza clamp (dA=−4.536.912, dB=−4.555.509) — il PRIMO non può essere deflazionato dal proprio teardown; è il decommit di QUALUNQUE lato dentro la finestra aperta dell'altro, con esito sub-ms (coerente con discriminatore UNSTABLE). Esperimento minimo: rerun dt∈{1,5}, entrambi gli ordini, **MIMALLOC_PURGE_DELAY ≥ finestra** con read-back in-band; predizione ex-ante: clamp sparisce e i net tornano a regime overlap ⇒ deflazione=decommit da purge-al-free; persiste ⇒ colpa del bracket/counter. Complementare e risolutivo: A-BB50 net per-thread.

## Q4 — Semantica del NOT-attributed

«verdict-grade … grade inherits the slope grades above» è autocontraddittorio: se eredita, eredita **ADVISORY** (min dei bracci). KL-90-4 dà condizione NECESSARIA (census+braccio), non sufficiente. Regola giusta: il negativo è claim di soglia; è citabile verdict-grade **solo se robusto in-band**. Qui la robustezza C'È ma l'ho calcolata IO, fuori banda: 2σ-floor ret0 16.691.057 ≥ 0,8·b_base=15.660.482; anti-moda W8 (cluster 228M al posto del plateau 252.575.744 x2, patologia del dominante: 2 outlier ripetuti battono 3 vicini non identici) dà ratio 1,019; tie-alt immateriale. Quindi: P-RET0 REFUTATA regge, ma la CITAZIONE va scritta «NOT-attributed, verdict-grade previa robustezza (ora mostrata); label g3 da leggere ADVISORY-inherited».

## Emendamenti

- **A-BB61**: tie nel mode-census dichiarato IN-BAND (`tie=K tiebreak=min`); mai più tie silenzioso.
- **A-BB62**: blocco VATTR emette la robustezza a macchina: ratio, 2σ-floor vs soglie, stimatore alternativo (min-of-R e anti-moda). Solo allora «verdict-grade».
- **A-BB63**: esperimento discriminante clamped come da Q3 (purge_delay≥finestra, read-back, predizione ex-ante nel header).
- **A-BB64** (sanatoria): correggere MEASURE89_RESULTS.md:50-52 — DUE marginali OUT, per nome e byte.

## Kill-switch

- **KB-91-1**: etichetta «verdict-grade» che eredita un braccio ADVISORY senza riga di robustezza in-band ⇒ declassata ADVISORY.
- **KB-91-2**: soglia numerica usata dal giudice ma assente dal header pre-run committato ⇒ esito citabile solo con prova d'insensibilità in-band; senza, VOID.
- **KB-91-3**: doc di risultati con conteggio difforme dal verdict.out di generazione massima (marginali OUT, modi, clamp) ⇒ doc VOID finché corretto.

— Lars Bak, sedia 5, Concilio WP-91

---

# VERBALE — Pedersen, sedia 6, Concilio WP-91

VERDETTO: CONCORDO CON EMENDAMENTI.

## Q1 — A-PP46 failpath (measure89-campaign.sh)
I due handler sono reali: `subshell_failpath` (r.164-173) e `teardown_fail`
(r.177-188) uccidono l'albero, verificano gone best-effort 5s e ledgerano
`failpath=… server_gone=yes|no` con `attempt=$ATT phase=$label` — leggibile
per attempt. MA restano rami residui:
1. **wait_up fallito** (r.243, 280, 312): `kill_server_tree; fail` — TERM
   solo, NESSUNA verifica gone, NESSUNA riga `server_gone=`. È l'unico ramo
   di abbandono cieco rimasto: esattamente la classe che A-PP46(2) dichiara
   di chiudere.
2. **Nessuna escalation KILL**: se TERM è ignorato, tutti e tre gli handler
   ledgerano `server_gone=no`/`still alive` e ESCONO lasciando il superstite
   vivo (onesto ma incompleto; il pre-flight successivo lo paga con un VOID).
3. **Terminalità VOID invertita nel ramo subshell**: il fail() dentro la
   command-substitution scrive `verdict=VOID` PRIMA che il parent appenda la
   riga `failpath=… server_gone=`; KG-89-1 vuole la VOID terminale.
4. Censimento del report: i siti chiamanti sono 8 (196, 245, 256, 282, 287,
   314, 333, 334), non "7" come in WP_SESSION_89.md r.30. Off-by-one, non
   capitale. Doppio-fail: escluso (il fail del subshell scrive UNA sola VOID;
   il parent esce senza secondo fail).

## Q2 — A-BG51 pid-echo (main.rs r.358-380)
Il layer `map_response` avvolge il Router con `.fallback(php_handler)`:
copre TUTTE le risposte del router principale, incluso `__reqns` (r.211, è
dentro php_handler) e gli early-return d'errore. **Tier0 NO** (r.315-338,
Router separato senza layer) e l'esclusione NON è dichiarata da nessuna
parte: il commento dice "every response", la riga di sessione "ogni risposta
axum". La campagna non esercita tier0, quindi nessun assert falso in-band —
ma la deviation va nominata. Inoltre "asserts it on the FIRST request of
every phase" è letteralmente falso: i probe di `wait_up` (fino a 100 curl su
`/__reqns`, r.139-146) PRECEDONO `assert_http_pid`. Materialmente sano
(l'assert avviene prima di ogni richiesta misurata, contro il LISTEN-owner
già verificato), ma la dicitura va corretta.

## Q3 — A-PP47 due-lane (worker_pool.rs r.1443-1514)
Riqualificazione ONESTA: refutata a macchina (log armato: probe_fail×2, zero
put — F8c by design), non a parole; la lane fatale ora è PINNATA al contatore
(put==0 AND probe_fail==2), la publish osservabile vive sulla lane pulita.
Pool W=1 (r.1397) ⇒ "same worker" garantito. Lane pulita: file pid-suffisso
in temp_dir, creato SOLO se l'env è armato; nel gate il run armato è
`--test-threads=1` filtrato su a_pp38 ⇒ niente flakiness parallela. Residui:
(a) panic prima di `remove_file` lascia il fixture (benigno); (b) full-suite
parallela con PHPR_UNIT_CACHE_LOG nell'ambiente inquina il log condiviso ⇒
FAIL spurio (fail-closed, da dichiarare). Il gate arma e conta giusto: UCL
non-vuoto (dente F16b), put==1/hit==1 esterni — che mordono anche su
pubblicazione della fatale (put diventerebbe ≥2). Manca il contatore esterno
`probe_fail==2`.

## Q4 — A-PP48
NPASS==13: morde in ENTRAMBI i versi (prune⇒12, aggiunta⇒14, ignore⇒12),
ancorato alla suite unittests src/main.rs (r.32), non al MAX. Estrazione
prima del run armato che appende allo stesso LOG: ordine corretto.
I conteggi A-PP45==5/A-PP47==13 (gate-lever-pins.sh r.1217-1219, `grep -c`
su TUTTO il file): dente contro delete/aggiunta non dichiarata, **incenso
contro il gutting sostitutivo** — neutralizzare l'assert conservando la
stringa/commento con la sigla mantiene il conteggio. Mitigato dai contatori
esterni del gate-axum (put/hit/vacuità); l'unico pin SOLO in-test è
npfail==2: quello è il buco reale.

## Emendamenti
- **A-PP50**: wait_up fail-path ledgerato — nei 3 siti sostituire con
  handler che kill-tree + verifica gone + riga `failpath=wait_up
  server_gone=`; nessun ramo di abbandono senza riga server_gone.
- **A-PP51**: escalation KILL — dopo il loop 5s con gone=no, `kill -KILL`
  sull'albero + ri-verifica; ledger `server_gone=killed9|no`.
- **A-PP52**: contatore esterno fatal-lane — gate-axum-tests conta
  `main_probe_fail`==2 sull'UCL armato (chiude il gutting comment-preserving
  che A-PP48(b) non morde).
- **A-PP53**: dichiarare per NOME l'esclusione tier0 dal pid-echo (o
  aggiungere il layer) + correggere "FIRST request of every phase" in
  "prima request ASSERITA, post-wait_up, pre-workload"; dichiarare il
  vincolo "run armato = solo targeted" della lane pulita.

## Kill-switch
- **KS-PP-91-1**: nessuna campagna m90+ consumabile finché esiste un ramo di
  abbandono server senza riga `failpath=/server_gone=` (A-PP50 = gate di
  merge).
- **KS-PP-91-2**: ogni claim "teeth intact" fondato sul solo censimento
  sigle (A-PP48(b)) è ADVISORY finché il contatore esterno npfail (A-PP52)
  non è armato.

Firmato: Pedersen, sedia 6, Concilio WP-91.

---

# VERBALE — Daan Leijen, sedia 7, Concilio WP-91

VERDETTO: **CON EMENDAMENTI** (una refutazione capitale di premessa: la
metrica di b è un contatore CONGELATO AL PICCO su macOS).

## Q1 — census per-theap a win=0: STRUTTURALMENTE MUTO, e la premessa "il grosso vive negli abandoned" è FALSA

Dai raw (m89.slope.w12.r1): a win=0 il census vede theap post-collect
223.600 B (~250 KB, un heap), bins src=main 571.200 B, bins src=aband
200.512 B — **tutte** le righe aband hanno used_n≥1. **Abandoned free
(used==0): ZERO righe, 0 B, in TUTTI i raw m89** (verificato su ogni
finestra). Il commit di processo a win=0 è 327.417.856 B: il census copre
~0,8 MB, **il 99,76% non sta in NESSUNA pagina** (mi_arena pages
current=36, pages_abandoned current=10). La cifra chiesta non è
ricavabile dai tag: non esiste.

Perché: su macOS release `_mi_prim_decommit` (prim/unix/prim.c:493-508)
usa MADV_FREE_REUSABLE e pone `*needs_recommit=false`; in os.c:590-592
`mi_os_stat_decrease(committed,…)` gira SOLO se needs_recommit —
**quindi mai**. Il contatore committed è monotono: nei 20/20 raw slope
(e negli slope0) `commit == peak_commit` AL BYTE, mentre phys a win=0 è
3,5-4,8 MB piatto su tutti i W. **committed_postcollect_win0 ≡ commit
PEAK**: b = 19.575.603 B/worker è la pendenza del PICCO, non del
trattenuto. Controprova: phys_peak 52,0/131,4/217,8/299,5 MB (mediane
W4/8/12/16) ⇒ ~20,6 MB/worker ≈ b. KL-90-4 formalmente eseguita, ma la
gamba census era VACUA per l'attribuzione a quel timing; la refutazione
P-RET0 regge sul solo braccio (b_ret0≈b_base). **Sì al collect+census
IN-REQUEST — ma per attribuire il PICCO, non un residuo che fisicamente
non esiste (~4 MB).**

## Q2 — P-RET0: refutazione valida ma il candidato era mezzo-morto in partenza

page.c:749: `page_full_retain = (block_size > MI_SMALL_MAX_OBJ_SIZE ? 0
: theap->page_full_retain)` — **solo small**; per pagine large il retain
è 0 incondizionato in ENTRAMBI i bracci ⇒ l'arm non poteva discriminare
nulla sul lato large. **Nessun ordinale gemello per large esiste**
(options.c:166, ord36 default 2, clamp [-1,32] in init.c:310; retain=0
mantiene allow_page_abandon=true, solo <0 lo spegne, init.c:309).
Con retain=0 restano committed: pagine full con blocchi vivi
(mi_page_to_full, page.c:775), abandoned con blocchi vivi, pagina
candidate per coda — ma a win=0 il census le limita a <1 MB totali: il
vero detentore è il contatore mai decrementato (Q1).

## Q3 — iterazione 2, ordinati per potere discriminante

1. **Census+collect IN-REQUEST al picco (worker VIVI)** — read-back:
   heaps_total==W+1 pinnato, coppia phys/phys_peak nella finestra.
   Unico braccio che decompone ciò che b misura davvero.
2. **Δcommitted per finestra disgiunta** (contatore monotono ⇒ Δ esatto,
   lezione WP-88): attribuisce il picco per FASE senza census.
3. Pagine parzialmente usate per bin: dentro il braccio 1, non
   standalone (a win=0 già misurate <1 MB).
4. Abandoned con blocchi vivi: 10 pagine a win=0 — non spiega 19,5
   MB/worker; solo come voce del braccio 1.
5. ord 44 minimal_purge_size: agisce sul residuo FISICO, non sul commit
   ⇒ potere NULLO su b come definita: ritirarlo da questa lane.

## Q4 — clamped dt 1-5 ms: coerente

purge_delay=0 ⇒ mi_arena_schedule_purge purga SINCRONA nel teardown
(arena.c:2059-2067): la deflazione fisica del lato primo cade dentro la
finestra 1-5 ms del secondo; a dt≥10 il teardown è già concluso. Nota:
può deflazionare solo un canale FISICO (il commit è monotono). Riga
probante: **Δpurged/Δpurge_calls per finestra** (key=purged già
in-band); pin: clamp ⇔ Δpurged>0 nella finestra del lato clamped.

## Emendamenti

- **A-DL48**: dichiarare commit_current≡commit_peak su macOS (catena
  prim.c:504→os.c:591) e riqualificare committed_postcollect_win0 come
  **PEAK-metric**; il giudice verifica commit==peak_commit su ogni raw.
- **A-DL49**: braccio census in-request al picco (worker vivi), pin
  heaps_total==W+1; attribuzione di b per bin/heap al PICCO.
- **A-DL50**: emettere Δpurged/Δpurge_calls per finestra nello sweep col
  pin di Q4.
- **A-DL51**: ord44 ritirato dalla lane commit; eventuale lane phys.

## Kill-switch

- **KL-91-1**: se in un raw win=0 commit≠peak_commit, ogni cifra fondata
  su A-DL48 è VOID.
- **KL-91-2**: finché b è definita su committed_postcollect_win0, ogni
  VATTR basato su soli knob post-picco (retain/abandon/purge) è VOID.
- **KL-91-3**: census a worker vivi verdict-grade solo se copre ≥90% del
  commit al picco; sotto = ADVISORY (mai più un census muto benedetto).

— Daan Leijen, sedia 7

---

# VERBALE — Dmitry Stogov, sedia 8, Concilio WP-91

**VERDETTO: CON EMENDAMENTI** — le 18 fixture sono un nucleo sano ma il
contratto ha una lettera REFUTATA dall'oracle (by-ref), il pin .out è un
bersaglio byte-impossibile per costruzione, e la sede lowering-only
diverge sui condizionali. Tutte le prove rieseguite dal vivo su 8.5.7.

## Q1 — Copertura: 11 BUCHI, pinnati dal vivo

Coperte: r1 (n1/p1), r3 (n3/p3), r6/r7/r9 (n6/n7/n9), tentative/RTWC/never
(p5/p6/p7), esenzioni dirette (p8/p9). Buchi (oracle rieseguito, stdout+rc):

1. **r2 by-ref — LETTERA DEL CONTRATTO REFUTATA**: «by-ref tipo ESATTO» è
   FALSA. `int &$x`→`int|string &$x` = **alive/0** (widening LEGALE);
   narrowing fatal («…must be compatible with P::m(string|int &$x)» —
   nota l'ordine canonico Zend dell'union nel messaggio); &-mismatch fatal
   in ENTRAMBE le direzioni (togliere E aggiungere &). Esatto = la
   REF-NESS, il tipo resta contravariante.
2. r2 required-grows: `m($a)`→`m($a,$b)` fatal «Declaration of C::m($a,
   $b) must be compatible with P::m($a)» — nessuna fixture.
3. r2 variadic assorbe: `m($a,$b,$c)`→`m(...$rest)` alive/0 — nessuna.
4. r4 grafia `?int` vs `int|null`: rc=0 — p4 testa il subset, non
   l'equivalenza di grafia.
5. r1 mixed=TOP narrowing (`mixed`→`int` alive/0) e **void esatto**
   (`void`→`int` fatal) — nessuna fixture.
6. r5 direzione inversa: static→instance fatal con messaggio DIVERSO da
   n5: «Cannot make static method P::m() non static in class C».
7. r8 readonly: fatal «Cannot redeclare readonly property P::$x as
   non-readonly C::$x» — nessuna fixture.
8. r8 tipo omesso/aggiunto: figlio untyped su parent tipato = messaggio
   di n8; figlio tipato su parent untyped = messaggio DISTINTO «Type of
   C::$x must be omitted to match the parent definition in class P».
9. **Esenzione INVERSA ctor**: interface-ctor e abstract-ctor VENGONO
   controllati (fatal «Declaration of C::__construct(string $y) must be
   compatible with I::__construct(int $x)», idem vs abstract). p8 da sola
   induce a esentare TUTTI i ctor — trappola viva.
10. Timing assente dal set: tutte le 18 sono parent-first incondizionate
    (v. Q3, t1-t4).
11. Nome irrisolvibile: l'oracle FATALA («…C::m(): B must be compatible
    with P::m(): A» con B mai dichiarata, nome stampato com'è scritto) —
    lo «skip conservativo» dell'appendice è divergenza da dichiarare per
    NOME, non fedeltà.

## Q2 — Pin: braccio ancorato, bersaglio NO

Binchk: la lane probe è onesta su ciò che prova (binario+ricetta
persistono) ma NON ancora il comando della fixture: probe e braccio
usano flag DUPLICATE a literal (ds35-verify.sh:47 vs :65) — una flag
caduta alla :47 (la recidiva WP-89) passerebbe «ok-via-probe». Header
.out riga 2 nomina 2 flag su 3. **Il pin troncato NON basta ed è
byte-impossibile**: l'oracle emette stdout=4 righe (display «\nFatal
error:…») + stderr=3 (log «PHP Fatal error:  …», doppio spazio); `2>&1|
head -3` tiene SOLO il blocco stderr, ma phpr emette SOLO il blocco
display (n7: stdout byte-identico via od, stderr VUOTO). Un'
implementazione perfetta non matcherebbe MAI il .out attuale. Serve pin
integrale a canali separati.

## Q3 — Sede: DUE posti, non uno

Pinnato vivo: t1 (parent DOPO) e t4: persist fatala PRE-output (niente
«pre|»), plain post — lowering-time è GIUSTO per il set hoisted
(top-level incondizionate, stesso perimetro §3.3-ter). MA t2
(condizionale eseguita): fatal DOPO «pre|» su ENTRAMBI i bracci; t3
(condizionale non eseguita): exit 0 ovunque. Lowering-only fatala dove
l'oracle esce 0 — il caso peggiore. Sede = lowering per le hoisted +
bind-in-registry per condizionali/dinamiche (incluse include/eval).

## Q4 — A-DS47/KS-DS-90-2

Sweep del catalogo eseguito: tutte le citazioni persist (§3.3/ter/quater)
ancorano a ds40-verify o ds35-verify, entrambi con binchk fail-closed —
nessun orfano. Residui: flag-vector condiviso probe/braccio e header a
3 flag (v. Q2).

## Emendamenti

- **A-DS48**: contratto r2 emendato — by-ref: ref-ness ESATTA, tipo
  contravariante (widening legale); messaggi con union in ordine
  canonico Zend.
- **A-DS49**: fixture v2 PRIMA del codice — 11 nuove per NOME (buchi
  Q1: byref×3, required-grows, variadic, grafia nullable, mixed, void,
  static-inversa, readonly, prop-omitted, iface/abstract-ctor,
  t1-t4 timing, nome irrisolvibile).
- **A-DS50**: pin v2 = stdout integrale + stderr SEPARATI; decidere per
  NOME se phpr modella la log-copy stderr o la dichiara a catalogo.
- **A-DS51**: sede a due posti (lowering hoisted + bind registry);
  irrisolvibili = fatal fedele t4 oppure divergenza a catalogo per NOME.
- **A-DS52**: ds35-verify v2 — PERSIST_FLAGS unico condiviso
  probe/braccio; header .out con le tre flag per NOME.

## Kill-switch

- **KS-DS-91-1**: implementazione che fatala su t3 (condizionale non
  eseguita) o anticipa t2 pre-output ⇒ REJECT.
- **KS-DS-91-2**: implementazione che esenta il ctor vs
  interface/abstract-ctor o fatala il by-ref widening ⇒ REJECT; gate di
  merge = fixture v2 per NOME, non solo le 18.
- **KS-DS-91-3**: pin fatal consumato come bersaglio byte-fedele se
  catturato `2>&1`+troncato ⇒ UNANCHORED.

Firmato: **Dmitry Stogov**, sedia 8, Concilio WP-91 — 2026-08-03.

---

# VERBALE — Brendan Gregg, sedia 9, Concilio WP-91

VERDETTO: **CONCORDO CON EMENDAMENTI**

## Q1 — storia g1→g2→g3 nel ledger

Ledgerata per generazione: righe 61-63 di `m89.campaign.ledger` portano
`judge_sha` + `generation` + `esito` + `fails=` + `verdict_file`. Verificato a
macchina: g1 `ae4f528f93095979` = blob committato **1021162**; g3
`294898431ef72d16` = blob **778d96f** = file attuale. **BUCO 1 (capitale di
catena): g2 `24cd290ae0a9fc2b` NON risolve a NESSUN blob committato** — stato
working-tree transiente (tutte e tre le generazioni "judged at HEAD=418def8",
HEAD mai mosso). Il pin ==1 refutato di g2 è ricostruibile solo dall'OUTPUT
(g2.out r.15-54), non dal codice. **BUCO 2**: nessun campo `reason=` /
`supersede_of=` — il PERCHÉ del supersede (g1: regime clamped nuovo da
DICHIARARE; g2: pin di forma-canale sbagliato) vive nei verdict file e nel
doc, non nel ledger. **BUCO 3**: la qualifica "FAIL del GIUDICE, non VOID
della campagna" sta SOLO in MEASURE89_RESULTS.md — dal solo ledger un FAIL di
giudice e un FAIL di dati sono indistinguibili.

## Q2 — pin presence-guard ==2

Verificato su **TUTTI i 57 raw m89** (conteggio `tag=mi_proc win=0 `): 2
esatto, zero devianti. `memcensus.rs` `dump_exit()` (r.196-218): dump #1 =
checkpoint `exit_mi` PRE-collect; poi `mi_collect(true)` sotto
`PHPR_MI_COLLECT_EXIT=1` (armato da run_arm) e dump #2 = `exit_collect_mi`
POST. **NON è doppia emissione: sono due checkpoint DISTINTI by design — il
delta pre/post È la misura di retention (commento r.202-206). Una emissione
per win distruggerebbe quell'informazione. La riqualificazione REGGE.**
Estrattore: `verdict89.sh:301` (awk, ultima occorrenza vince in END) e :303
(slack resettato a ogni mi_proc win=0) ⇒ consuma la POST ✓. Fragilità vera:
la riga mi_proc NON porta il nome del checkpoint — selezione POSIZIONALE su
canale append non-atomico (nota A-DL40); e nei campioni pre==post su
`commit=` SEMPRE ⇒ il selettore non è mai stato esercitato da dati che
discriminano. → A-BG54.

## Q3 — A-BG51 consumato

boot_epoch: giudicato su ogni raw (`verdict89.sh:138-148`: presenza +
boot≤epoch, else raw VOID); verificato su raw veri (identity count==1,
`srv_boot_epoch=` presente, coerente). **MA** `BOOT_EPOCH=$(date +%s)` del
HARNESS post-assert (campaign ~r.246), non tempo di boot del PROCESSO:
giudizio quasi-vacuo (morde solo su campo assente/orologio). Pid-echo:
`assert_http_pid` invocato in TUTTI e tre i runner (r.247/284/316) ⇒ **ogni
fase**, prima di ogni request misurata. Però NON è letteralmente "la prima
request": `wait_up` spara fino a 100 probe `/__reqns` NON asseriti prima —
la dicitura del doc va precisata in "prima request GIUDICATA". Lane
preflight: dichiarata (attempt=0 `phase=preflight`, r.84-88) ma ledgerata
SOLO su VOID: ledger pulito non distingue "preflight ok" da "mai girato".

## Q4 — doc contro g3

Ogni cifra verificata nel raw/verdict committato: b/se/2σ/a, 4 modi
dominanti, marginali BASE 3/3 IN, conversioni MiB rifatte ad aritmetica, cal
7.801.102×2, floor_inc, P-DT20 4/4, set clamped {dt1×2, dt2×2, dt5.afirst},
UNSTABLE 2/3 + `regime=OVERLAP-only (A-BG52)` ✓. Regime clamped nel doc con
pari forza del verdict ✓ (DICHIARATO + escluso dal tally + KB-88-1).
**REFUTAZIONE: "un marginale fuori fascia" (doc r.50-52) è FALSO — g3 marca
DUE marginali RET0 [OUT]: 24.395.776 (W4→8, r.112) E 16.400.384 (W12→16,
r.114)**; il riassunto del giudice stesso (r.115) sotto-riporta al singolare.
Inoltre WP_SESSION_89.md (r.44-48) omette il grade ADVISORY di b_ret0 nel
bullet VATTR; il verdict r.117 è ambiguo ("verdict-grade… grade inherits the
slope grades above"). Grade e conclusioni invariati — ma il gate è per NOME,
mai per conteggio.

## Emendamenti

- **A-BG53 (judge tether)**: il giudice di OGNI generazione va committato
  PRIMA del run (come il binario col matrix tether A-SK59); riga
  `phase=verdict` con `reason=` sintetico e `supersede_of=`.
- **A-BG54 (checkpoint nominato)**: `tag=mi_proc win=0` porti
  `ckpt=exit_mi|exit_collect_mi`; estrattori selezionano per NOME; pin ⇒
  "==1 per ckpt" (==2 totale ne discende).
- **A-BG55 (boot reale + preflight-ok)**: srv_boot_epoch dal processo
  (`ps -o lstart=` o header in-band), non wall-clock harness; riga
  `preflight=ok` anche sul cammino pulito.
- **A-BG56 (sanatoria)**: MEASURE89_RESULTS.md corregga "un marginale" in
  DUE per NOME; WP_SESSION_89 citi il grade ADVISORY di b_ret0; il giudice
  stampi conteggio+nomi dei marginali OUT.

## Kill-switch

- **KG-91-1**: judge_sha in ledger senza blob committato risolvibile ⇒
  generazione VOID-of-record: non supersede, non citabile dai doc.
- **KG-91-2**: (post A-BG54) estrattore win=0 posizionale anziché per ckpt
  nominato ⇒ cifra non verdict-grade.
- **KG-91-3**: doc che riporta il CONTEGGIO delle deviazioni difforme dal
  SET per NOME nel verdict ⇒ rigetto al gate cifre.

— Brendan Gregg, sedia 9

---

## NOTE DI TEAM (fase 2)

---

# TEAM-CIFRE — Concilio WP-91 (relatore: sedie 3, 4, 9)

Verbali fonte (vincolanti): verbale-3-klabnik.md, verbale-4-hejlsberg.md, verbale-9-gregg.md.

## CONVERGENZE (emendamenti unificati)

1. **Provenienza committata per OGNI anello della catena** — A-SK60+A-SK63+A-SK64 (Klabnik: operandi [derivata] risolti per provenienza `file:riga` a HEAD; allowlist/perimetro da MANIFEST committato, non regex di nome) ↔ A-AH54 (Hejlsberg: triangolo attempts↔stamp↔OUT con sha256==DSHA) ↔ A-BG53 (Gregg: giudice committato PRIMA del run). Stesso principio: nulla è verdict-grade se non risolve a blob committato.
2. **judge_sha g2 dangling** — A-AH56+KS-AH-91-2 (Hejlsberg) ≡ BUCO 1+KG-91-1 (Gregg): g2 `24cd290ae0a9fc2b` non risolve ad alcun blob; entrambi esigono generazione dichiarata "judge-unrecoverable"/VOID-of-record e non citabile. Verificato indipendentemente da entrambe le sedie.
3. **Dente generazioni / supersede leggibile in-banda** — A-SK66+KS-SK-91-4 (Klabnik: doc deve citare la G massima; .out superseded fuori dal corpus) ↔ A-BG53 (`reason=`, `supersede_of=` nel ledger) ↔ A-AH57 (Hejlsberg: consumazione --same-rev ledgerata). Unificare: il ledger porta supersede e consumazione; il gate morde sulla citazione.
4. **Per NOME, mai per conteggio** — KG-91-3+A-BG56 (Gregg: doc dice "un marginale", g3 ne marca DUE) rima con A-SK62 (Klabnik: giudicare OGNI token ≥3 cifre su riga [derivata]): il gate deve enumerare, non contare.
5. **Recorder/estrattori fail-closed simmetrici** — A-AH55 (cargo=unknown senza dente) ↔ A-BG54 (estrattore win=0 posizionale, ckpt non nominato): stessa classe, canale non autenticato dal nome.

## CONFLITTI

Nessun conflitto frontale: le tre sedie si rafforzano. Una **tensione di collocazione** su A-SK60: Klabnik preferisce che la cifra derivata la emetta l'EMITTER in un .out ("meglio ancora"), mentre la variante minima (gate che risolve e stampa la provenienza) basterebbe al dente di Hejlsberg. Da sciogliere in sede di attuazione; non tocca i kill-switch. Nota: Gregg CONFERMA la riqualificazione ==2 (due checkpoint by design) — nessuno la contesta.

## PRIORITÀ PROPOSTE per l'ordine S-90.0

1. **A-SK60+A-SK62 (+A-SK61 igiene corpus)** — primo assoluto: il forge A-SK56 è passato DAL VIVO (bound fabbricato PASS con tag [derivata]; 23,5% del corpus è cifre di indirizzi vmmap, amplificazione ~2×10⁴). Finché non atterrano, KS-SK-91-1 rende non-verdict-grade ogni PASS su doc con [derivata]: blocca tutto il resto.
2. **A-SK65** — cache avvelenabile E esfiltratrice (usata per costruire il forge): argv+nonce, env ignorato. KS-SK-91-3 già attivo.
3. **Catena judge_sha: A-BG53+A-AH56** — giudice committato pre-run, g2 annotata dangling nel doc; sana KG-91-1/KS-AH-91-2 prima della prossima campagna.
4. **A-AH54** — prefix-check esteso a battery-attempts.ledger + sha256==DSHA: chiude la riscrittura di storia in-window (KS-AH-91-1).
5. **Sanatorie e manifest**: A-BG56 (doc: DUE marginali per NOME), A-SK63/A-SK64 (manifest prima di WP-100, KS-SK-91-2), A-AH57, A-BG54/A-BG55, A-AH55, A-SK66.

Razionale d'ordine: prima i denti che oggi NON mordono su forge dimostrati (1-2), poi l'integrità di catena retroattiva (3-4), infine sanatorie documentali e scadenze (5).

---

# TEAM-CONFINE-ENGINE — Concilio WP-91 (relatore su verbali 6-Pedersen, 8-Stogov)

Fonti vincolanti: `verbale-6-pedersen.md`, `verbale-8-stogov.md`. Questo verbale è sintesi, non supersede.

## CONVERGENZE

1. **Fail-closed con riga ledgerata, mai abbandono cieco.** Pedersen (wait_up senza `server_gone=`, A-PP50/51) e Stogov (skip conservativo sul nome irrisolvibile = divergenza da dichiarare per NOME, non fedeltà) dicono la stessa cosa: ogni ramo di uscita o deviazione dall'oracle va nominato, mai lasciato implicito.
2. **Il pin/censimento in-band non basta: serve il contatore/oracle ESTERNO.** Pedersen: `grep -c` sulle sigle non morde il gutting comment-preserving → A-PP52 (contatore esterno `main_probe_fail==2`). Stogov: il .out troncato `2>&1|head -3` è byte-impossibile per costruzione → A-DS50 (pin a canali separati). Stesso principio: il bersaglio deve essere ancorato a ciò che l'oracle emette davvero, non a un artefatto del harness.
3. **Verificare contro l'oracle PRIMA di eseguire/implementare** (premessa fattuale WP-88 confermata): entrambe le refutazioni sono state prodotte a macchina, dal vivo, prima del codice.
4. **Recidiva flag-drift WP-89**: probe e braccio con flag duplicate a literal (ds35-verify:47 vs :65) → A-DS52 (PERSIST_FLAGS unico) è la stessa classe di A-PP53 (dichiarare per NOME esclusioni e vincoli del harness).

## CONFLITTI

Nessun conflitto diretto (domini disgiunti: Pedersen lifecycle/publish, Stogov LSP). Una tensione di sequenza: i kill-switch di Pedersen (KS-PP-91-1: nessuna campagna m90+ consumabile senza A-PP50) gate-ano la MISURA, quelli di Stogov (KS-DS-91-2: fixture v2 per NOME = gate di merge) gate-ano il CODICE A-DS35. Vanno onorati entrambi ma su lane indipendenti: A-PP50/51 non blocca A-DS35 e viceversa.

## PRIORITÀ per S-90.0

**La refutazione Stogov ridefinisce il primo item.** «A-DS35 fase 1 implementazione» com'era scritto è REFUTATO in tre punti: (a) lettera r2 by-ref falsa (esatta è la ref-ness, il tipo resta contravariante — A-DS48); (b) pin .out byte-impossibile (canali mescolati — A-DS50); (c) fixture v2 con 11 buchi (A-DS49) e sede singola divergente sui condizionali (A-DS51). Implementare ora significherebbe codificare un contratto falso contro un bersaglio irraggiungibile.

**Sequenza onesta proposta:**
1. **A-DS48 prima di tutto**: emendare il contratto r2 (ref-ness esatta, tipo contravariante, union in ordine canonico Zend).
2. **A-DS49 + A-DS50 PRIMA del codice**: fixture v2 (le 11 per NOME, incluse t1-t4 timing) con pin integrale stdout/stderr SEPARATI, generate dall'oracle 8.5.7; decidere per NOME se phpr modella la log-copy stderr o la cataloga.
3. **A-DS52**: ds35-verify v2 (PERSIST_FLAGS unico, header a 3 flag) — chiude la recidiva WP-89.
4. **Solo poi il codice, in sede DUALE** (A-DS51): lowering per le hoisted top-level + bind-in-registry per condizionali/dinamiche (include/eval). KS-DS-91-1 (t3 non deve fatalare, t2 non pre-output) e KS-DS-91-2 (no esenzione ctor vs iface/abstract, no fatal su widening by-ref) come gate di merge.
5. **In parallelo, lane lifecycle**: A-PP50/51 (wait_up ledgerato + escalation KILL) come gate delle campagne m90+ (KS-PP-91-1); A-PP52/53 subito dopo.

I verbali individuali restano vincolanti; KS-DS-91-3 declassa a UNANCHORED ogni consumo del pin troncato attuale.

---

# TEAM-MISURA — Concilio WP-91 (relatore; fonti: verbale-5-bak.md, verbale-7-leijen.md — i verbali individuali restano vincolanti)

## CONVERGENZE
1. **Cifre riprodotte**: Bak ricomputa dai 40 raw al byte (b_base=19.575.603,2, b_ret0=19.329.843,2, OLS dof=2 corretto); Leijen non contesta i numeri ma la loro semantica.
2. **Esito P-RET0**: entrambi confermano che la refutazione REGGE (b_ret0≈b_base, ratio 0,987, insensibile alla soglia in (0,5; 0,987]).
3. **Evidenza a macchina, mai fuori banda**: Bak esige robustezza emessa in-band (A-BB61/62, KB-91-1/2); Leijen esige read-back e pin (heaps_total==W+1, commit==peak su ogni raw). Stessa dottrina.
4. **Clamped dt 1-5**: stessa lettura causale — decommit sincrono al free (purge_delay=0) di un lato dentro la finestra dell'altro; dt5.bfirst (Bak) e arena.c:2059-2067 (Leijen) puntano allo stesso meccanismo. I due esperimenti proposti sono UNO: rerun con purge_delay≥finestra (A-BB63) + riga Δpurged/Δpurge_calls con pin «clamp ⇔ Δpurged>0» (A-DL50).

## CONFLITTI
**La riqualifica di Leijen cambia la lettura del VATTR/P-RET0 di Bak? SÌ, nella causa; NO, nell'esito.** Leijen dimostra (prim.c:504→os.c:591) che su macOS il contatore committed è monotono: b è la pendenza del **PICCO** di commit, non del trattenuto; i knob retain/abandon/purge agiscono POST-picco. Quindi b_ret0≈b_base non è una scoperta discriminante ma una **conseguenza quasi tautologica della metrica**: la robustezza statistica di Bak (2σ-floor 16.691.057 ≥ 0,8·b_base, anti-moda, tie-alt) è aritmeticamente valida ma misura la solidità di un negativo che la metrica garantiva in partenza. Tensione formale: **KB-91-1** (verdict-grade ammesso con robustezza in-band) vs **KL-91-2** (ogni VATTR su knob post-picco è VOID finché b è definita su committed_postcollect_win0). KL-91-2 è più restrittivo e prevale finché la metrica non è riqualificata: la citazione corretta diventa «P-RET0 NOT-attributed; robustezza mostrata (Bak); braccio strutturalmente non discriminante per b-come-PEAK (Leijen)». Le due sedie sono **compatibili** — Bak giudica lo stimatore, Leijen la metrica — ma la composizione declassa la classe VATTR-post-picco, non solo l'etichetta g3 (che per Bak era già ADVISORY-inherited). Rafforzo incrociato: Q2 di Leijen (retain solo small, page.c:749) spiega perché il braccio era mezzo-morto anche a prescindere dal picco.

## PRIORITÀ ITERAZIONE 2 (attribuzione di b)
1. **A-DL48 subito** (pre-condizione di tutto): dichiarare commit≡peak su macOS; giudice verifica commit==peak_commit su ogni raw (KL-91-1). Costo nullo, riorienta la lane.
2. **Census+collect IN-REQUEST al picco** (A-DL49, worker vivi, pin heaps_total==W+1; gate copertura ≥90% del commit o ADVISORY, KL-91-3). Unico braccio che decompone ciò che b misura.
3. **Δcommitted per finestra disgiunta** (contatore monotono ⇒ Δ esatto, lezione WP-88): attribuzione per FASE, complementare al census.
4. **Esperimento clamped unificato** (A-BB63+A-DL50): purge_delay≥finestra con read-back, predizione ex-ante nel header, riga Δpurged/Δpurge_calls.
5. **Robustezza VATTR in-band** (A-BB62, +A-BB61 tie, +A-BB64 sanatoria doc): necessaria per ogni FUTURO negativo, ma da sola non attribuisce nulla — dopo i bracci 1-3.
6. **Ritiri**: ord44 fuori dalla lane commit (A-DL51); banda VATTR post-picco congelata da KL-91-2.

---

# TEAM-SIGILLI — Concilio WP-91 (relatore su verbali 1-Hoare, 2-Matsakis)

Fonti vincolanti: `verbale-1-hoare.md`, `verbale-2-matsakis.md`.

## CONVERGENZE
1. **Single-source dei rilevatori.** Stesso vizio da due lati: Hoare (A-TH52) — TH49RE/TH44RE ri-digitate tra self-test e sweep ⇒ positivo che non esercita il rilevatore di produzione (classe A-PP48); Matsakis (A-MS48) — belt con regex a binder letterale (`|f| f.set(`) che un binder diverso, `take/swap`, UFCS o split multilinea eludono. Regola comune: UNA variabile-regex condivisa self-test↔sweep (modello TH33RE), decoy che passano per la regex COMPOSTA di produzione, same-commit.
2. **Il limite lessicale va dichiarato e recintato.** Hoare: `paste!`/`concat_idents` fuori portata di ogni sigillo lessicale ⇒ bando per nome nei Cargo.toml (A-TH53) finché A-MS27 (rustc giudice) non chiude. Matsakis converge sul principio col sigillo di TIPO/visibilità: dove il compilatore può morire a compilazione, la regex è solo belt.
3. **Restringere il perimetro di privacy, non fidarsi del file.** Matsakis A-MS46: `mod probe` annidato ~40 righe con `pub(super)` — il perimetro attuale (`mod implementation`, ~1880 righe test inclusi) lascia ogni riga futura libera di scrivere il flag, giudicata solo dalla belt bucata. Coerente con la linea Hoare: il sigillo di forma (finestra awk chiusa, pin ==1) batte il pattern-match aperto (A-TH56).
4. **Gate igienici fail-closed.** A-MS49 (`rm -f .done` in testa + confronto git=HEAD imposto) e A-TH56 (pin `^probe_in()` ==1, finestra chiusa, tainted2 declassato a forward-guard): stato stantio e doppia definizione devono morire prima del verdetto.

## CONFLITTI
Nessun conflitto frontale. Tensione di metodo: Hoare estende la belt (branch nuovi per 6 grafie Q1); Matsakis preferisce spostare il giudizio sul compilatore (privacy/mod annidato) e tenere la belt come cintura. Composizione naturale: A-MS46 riduce la superficie che A-TH52/A-MS48 devono coprire — prima il recinto di tipo, poi la belt single-source sul residuo.

## PRIORITÀ (proposta del team; i verbali restano vincolanti)
**Sigilli v8 SUBITO (meccanici, chiudono kill-switch attivi):**
- A-TH52 + A-MS48 (single-source + grafie estese + decoy che mordono la regex composta) — sblocca KH91-2 e KS-MS-91-1.
- A-TH56 (noprobe: pin ==1, finestra chiusa, pin di forma) — sblocca KH91-3.
- A-MS49 (.done igiene) e A-MS47 (`let <ident> = ProbeWindow::arm()` ==1, `-D clippy::let_underscore_must_use`).
- A-TH54 (riscrittura A-TH50 in forma order-dependent; fix "a_ds36"→a_ds26) — doc, costo zero, KH91-1 già attivo sui claim.

**DESIGN (sessione dedicata):**
- A-MS46 (mod probe annidato) — refactor piccolo ma di forma, va col codice non con gli script.
- A-TH53 (bando paste/concat_idents) — decisione di policy da ratificare.
- A-TH55 (ordine canonico KIND intra-putord in a_ds26/a_ds38).
- A-MS50 — vedi sotto.

## RINVIO AL TEAM-MISURA (non deciso qui)
La refutazione capitale Matsakis Q4 — i visitor census ALLOCANO (Vec::push, BTreeMap::entry) dentro la visita mimalloc del subproc visitato, violando l'invariante dichiarato (righe 698-700) — tocca il CANALE DI MISURA: KS-MS-91-3 rende ADVISORY ogni riga mi_theap_* pre-A-MS50. Se e come declassare le figure m89 spetta al team-misura; A-MS50 (capacità pre-riservata, bins array-fisso classe BinTab, `acc` riderivato da arg) è il prerequisito per tornare verdict-grade.

Nessuna delle refutazioni tocca il PASS di VERDICT89 (dichiarato da entrambe le sedie nei rispettivi ambiti).

---

## ⚖️ SINTESI DI CONVERGENZA (compilata dalle ricevute fase 1+2; verbali = fonte vincolante)

**Verdetto complessivo: 9× CON EMENDAMENTI — nessuna opposizione; nessun
conflitto frontale nei 4 team (una tensione di collocazione su A-SK60 e
una belt-vs-tipo, entrambe composte in nota di team).** Cifre di
attempt=1 g3 confermate a ricomputo indipendente (Bak al byte). SEI
refutazioni capitali, tre delle quali morse A MACCHINA:

### Refutazioni capitali
1. **🔴 Il forge A-SK56 è PASSATO dal vivo (Klabnik)**: una cifra
   fabbricata (19.600.000 B) legalizzata con `[derivata: X−Y]` scegliendo
   operandi PRESENTI in corpus (indirizzi vmmap tokenizzati) — la
   verifica aritmetica libera è un canale di legalizzazione; serve
   derivata-per-PROVENIENZA (A-SK60/62) + igiene corpus (A-SK61) →
   KS-SK-91-1.
2. **🔴 b è la pendenza del PICCO di commit, non del residente
   (Leijen)**: commit==peak_commit 20/20 su macOS (decommit non abbassa
   il contatore letto), abandoned free = 0 B in tutti i raw, phys a
   win=0 ~4 MB ⇒ P-RET0 sul committed post-collect era quasi-tautologica
   (il retain agisce sul residente, non sul picco). b va RIQUALIFICATA
   PEAK-metric nei doc (A-DL48, KL-91-1/2); l'iterazione 2 si sposta sul
   census IN-REQUEST al picco (A-DL49, ≥90%/heaps_total==W+1) +
   Δcommitted/Δpurged per finestra (A-DL50); ord 44 ritirato dalla lane
   commit (A-DL51).
3. **🔴 La catena dei giudici non è ricostruibile a macchina
   (Hejlsberg+Gregg, convergenza)**: g2 judge_sha=24cd290ae0a9fc2b non
   risolve a NESSUN blob committato (il giudice emendato fra g2 e g3 non
   fu committato prima del run); e il battery-attempts.ledger, ora
   allowlistato in finestra, è RISCRIVIBILE in-window (manca il
   prefix-check delle stamps) → A-BG53/A-AH56 (judge committato PRE-run,
   sha risolvibile), A-AH54 (prefix-check attempts), A-AH57, A-SK66
   (dente citazione generazioni) — KS-AH-91-1/2, KG-91-1, KS-SK-91-4.
4. **I visitor del census per-theap ALLOCANO (Matsakis)**: Vec/BTreeMap
   dentro le callback violano l'invariante no-alloc dichiarato del
   canale census → A-MS50 (visitor senza allocazione), KS-MS-91-3.
5. **«by-ref tipo ESATTO» refutata dall'oracle (Stogov)**: il widening
   by-ref è LEGALE; e il pin ds35 con 2>&1 mescola stderr-oracle e
   stdout ⇒ bersaglio byte-IMPOSSIBILE → A-DS48 (contratto emendato),
   A-DS49 (fixture v2, 11 buchi nominati), A-DS50 (pin a canali
   separati integrali), A-DS51 (sede DUALE lowering+bind), A-DS52 —
   KS-DS-91-1..3.
6. **Correzioni documentali (Bak+Gregg, convergenza)**: MEASURE89 dice
   «UN marginale fuori fascia», il g3 ne marca DUE (24.395.776 e
   16.400.384); la soglia 0,8 di VATTR vive solo nel giudice (non
   ex-ante nel header); «verdict-grade inherits» autocontraddittorio →
   A-BB64/A-BG56 sanatorie; A-BB61/62 robustezza; KB-91-3.

### Ordine vincolante di apertura WP-90(sessione) (non rinegoziare)
1. **Sanatorie forma**: A-BB64≡A-BG56 (MEASURE89: DUE marginali; soglia
   0,8 dichiarata; «inherits» corretto) · A-DL48-forma (b riqualificata
   PEAK-metric in MEASURE89/session/NEXT) · A-TH54 (doc A-TH50 seconda
   metà + caveat os-TLS) · A-PP53 (tier0 dichiarato + dicitura
   post-wait_up) · correzione «7 siti»→8.
2. **Fix gate cifre (il forge vivo)**: A-SK60 (derivata per PROVENIENZA:
   operandi con sorgente nominata, mai X−Y libero) · A-SK62 (ogni token
   su riga derivata) · A-SK61 (igiene corpus: vmmap/nodot fuori) ·
   A-SK65 (cache argv+nonce) · A-SK63/64 (manifest bande+perimetro) —
   OGNI fix con bite-test stesso-commit (KS-SK-91-1..3).
3. **Catena giudici/attempts**: A-BG53+A-AH56 (giudice committato
   PRE-run, judge_sha risolvibile a blob; reason= nel ledger) · A-AH54
   (prefix-check + sha-tether su attempts.ledger) · A-AH57 (consumazione
   --same-rev ledgerata) · A-SK66 (dente gG a macchina) · A-AH55≡A-BG54
   (cargo= + ckpt nominato) — KS-AH-91-1..3, KG-91-1..3, KS-SK-91-4.
4. **Sigilli v8**: A-TH52≡A-MS48 (single-source regex self-test↔sweep +
   grafie residue) · A-TH56 (probe_in ==1 + finestra chiusa) · A-MS49
   (.done rm in testa + git=) · A-MS47 (pin sito let _w + clippy) ·
   A-PP50/51 (wait_up failpath + escalation KILL) · A-PP52 (contatore
   esterno npfail==2) — design: A-MS46, A-TH53, A-TH55, A-MS50.
5. **Misura (campagna iterazione 2, metrica RIQUALIFICATA)**: A-DL48
   (check commit==peak in-band, b dichiarata PEAK) · A-DL49 (census
   in-request al picco, pin heaps_total==W+1, ≥90% — KL-91-3) · A-MS50
   (visitor no-alloc PRIMA di riarmarlo) · A-DL50 (Δcommitted/Δpurged
   per finestra) · esperimento clamped unificato A-BB63+A-DL50
   (purge_delay≥finestra) · A-BB61/62 (tie in-band + robustezza VATTR a
   macchina) · A-DL51 (ritiro ord 44).
6. **ROADMAP — A-DS35 fase 1 RIDEFINITA**: A-DS48 (contratto by-ref
   emendato: ref-ness esatta, widening legale) → A-DS49+A-DS50 (fixture
   v2 + pin canali separati integrali PRIMA del codice) → A-DS52
   (PERSIST_FLAGS condiviso) → implementazione sede DUALE (A-DS51) —
   gate di merge KS-DS-88-3 + KS-DS-89-3 + KS-DS-91-1..3.
7. **Delibere/deferred**: A-BB60/A-PP49 restano design; A-MS27/A-PP18/
   A-PP27/A-AH38 invariati; KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)
KH91-1 (fail-fast deterministico dichiarato ma irraggiungibile ⇒ claim
VOID) · KH91-2 (sigilli-grafie senza single-source ⇒ ADVISORY) · KH91-3
(noprobe doppia-def/finestra aperta ⇒ VOID) · KS-MS-91-1 (terzo
scrittore flag ⇒ ADVISORY) · KS-MS-91-2 (theap senza quiescenza ⇒ VOID)
· KS-MS-91-3 (visitor census che alloca ⇒ ADVISORY) · KS-SK-91-1 (PASS
cifre su doc con derivata non-provenance ⇒ VOID) · KS-SK-91-2 (perimetro
per nome ⇒ VOID) · KS-SK-91-3 (cache ereditata ⇒ VOID) · KS-SK-91-4
(citazione gN con gM>N ⇒ dente a macchina) · KS-AH-91-1
(rewrite/sha-mismatch attempts ⇒ consumo VOID) · KS-AH-91-2 (judge_sha
max-gen non risolvibile ⇒ VERDICT VOID) · KS-AH-91-3 (cargo=
vuoto/unknown ⇒ matrix NULL) · KB-91-1 (verdict-grade ereditato da
ADVISORY senza robustezza ⇒ declassa) · KB-91-2 (soglia solo-giudice ⇒
insensibilità o VOID) · KB-91-3 (doc difforme dal g.out ⇒ VOID) ·
KS-PP-91-1 (campagna senza server_gone su ogni ramo) · KS-PP-91-2
(censimento sigle = ADVISORY senza contatore esterno) · KL-91-1
(commit≠peak_commit ⇒ VOID) · KL-91-2 (knob post-picco ⇒ VATTR VOID) ·
KL-91-3 (census <90% del picco = ADVISORY) · KS-DS-91-1 (fatal su
t3/t2-anticipato ⇒ REJECT) · KS-DS-91-2 (ctor-vs-interface esentato o
byref-widening fatale ⇒ REJECT) · KS-DS-91-3 (pin 2>&1 troncato ⇒
UNANCHORED) · KG-91-1 (judge_sha senza blob ⇒ VOID-of-record) · KG-91-2
(estrattore posizionale ⇒ non verdict-grade) · KG-91-3 (conteggio ≠ set
per NOME ⇒ rigetto). [27 KS]
