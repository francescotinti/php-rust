# Verbale sedia 1 — Tony Hoare (Concilio WP-92)

Perimetro: sigilli lessicali v8, pin A-MS47/A-MS48, finestra noprobe A-TH56, doc TLS A-TH54, fail-fast.

## VERDETTO: CONCORDO CON EMENDAMENTI

## Q1 — Grafie che ancora sfuggono a TH49RE_V8/TH49ML_PROG
Verificato contro il testo di produzione (gate-lever-pins.sh 1239-1253):
1. **`use crate::vm::CachedUnit as CU;` MONORIGA senza graffe** sfugge a TUTTO: il ramo regex use richiede `[{]`; l'awk fa `next` esplicito sulle use monoriga; la rete alias A-TH42 richiede `=`. Ironia: la grafia più naturale è l'unica scoperta (la variante multiriga e quella a gruppo sono coperte).
2. **Percorso dot-EOL senza tolleranza commenti**: lo skip commento/blank esiste solo sul ramo pend (`.nome` a EOL); `u.` ⏎ `// c` ⏎ `vm_gate(x)` azzera `dot` sulla riga commento e passa.
3. **Split a TRE righe** `u.` ⏎ `vm_gate` ⏎ `(x)`: nessuna riga contiene una grafia riconosciuta da regex o awk (la regola dot esige nome+paren sulla stessa riga).
4. **Commento `//` in coda al nome**: `u.production_gate // c` ⏎ `(x)` — la regola pend esige il nome a fine riga; nessun `(` sulla riga ⇒ sfugge anche ai pin `.production_gate(`.
5. **`::` spaziato in posizione di TIPO** (`fn f(x: vm :: CachedUnit)`, return type, turbofish): tutti i rami `::`-spaziati sono ancorati a `=`. Severità minore (non è mint diretto), ma il canale d'alias per firma esiste.

## Q2 — Buchi nei pin A-MS47 (==2) e A-MS48
Confermati i 2 siti legali (worker_pool.rs 385, 520). Ma:
1. **A-MS47 non è un census**: `narm` conta solo `let <nome> = ProbeWindow::arm()`. `let mut w =`, `let w: ProbeWindow =`, `let (w,_) =`, `drop(ProbeWindow::arm())`, arm in posizione d'espressione — tutte invisibili a `narm` E a `nsil`. Un TERZO sito così scritto passa con narm==2/nsil==0. Il pin morde sulla conversione dei 2 siti noti (2→1 FAIL), non sull'aggiunta.
2. **A-MS48, quinta grafia**: `CENSUS_PROBE_ACTIVE.with(Cell::set)` / `with(|f| Cell::set(f, true))` (point-free/UFCS-Cell) sfugge alle QUATTRO reti (anybind esige `|b| b.set`; UFCS esige `LocalKey::`). Anche il corpo chiuso a graffe `|f| { f.set(true) }` sfugge ad anybind: una scrittura NUOVA così spellata è invisibile.

## Q3 — Elusioni della finestra noprobe (A-TH56)
1. `npdef` conta solo `^probe_in()` a colonna 0: una **ridefinizione indentata** ` probe_in() { return 1; }` o in forma **`function probe_in {`** dopo il decoy passa npdef==1, l'awk estrae il decoy (form-pin OK), bash usa l'ultima definizione svuotata.
2. Il form-pin è di FORMA, non di raggiungibilità: `false && strings -- "$1" | grep -q '<payload>'` soddisfa nform==1 a rilevazione svuotata. Residuo dichiarabile (oltre la portata lessicale), ma va DETTO.

## Q4 — Doc TLS A-TH54 (vm/mod.rs 16534-16555)
La dicotomia "destructor già eseguito / chiave ancora viva" **omette il terzo stato**: chiave RefCell MAI inizializzata su quel thread ⇒ primo accesso durante il teardown ⇒ std NON garantisce (init pigra con destructor possibilmente mai eseguito, o panic, per piattaforma). KH91-1 sopravvive, ma l'enumerazione è incompleta. Inoltre il percorso silente non è solo "senza fail-fast": l'emissione uc_log finisce in un buffer che muore senza flush — è un canale di **scrittura persa**, non solo di mancato panic.

## Emendamenti
- **A-TH-57** "Census totale arm": aggiungere `ntot = grep -c 'ProbeWindow::arm'` su righe di codice, pin ==2 (chiude Q2.1).
- **A-TH-58** "Use monoriga con as": ramo regex `use [^;{]*(CachedUnit|VmGate) as ` (chiude Q1.1).
- **A-TH-59** "Dot-path tollerante + tre righe": estendere TH49ML_PROG con skip commenti sul ramo dot e stato nome-pendente (Q1.2-4).
- **A-MS-emend** (per il team sigilli): quinta rete point-free/graffe su CENSUS_PROBE_ACTIVE (Q2.2).
- **A-TH-60** "Definizioni probe_in a census": `grep -cE '^[[:space:]]*(function[[:space:]]+)?probe_in'` ==1 (Q3.1).
- **A-TH-61** "Doc A-TH54 terzo stato": aggiungere il caso never-initialized-at-teardown e la scrittura persa (Q4).

## Kill-switch
- **KS-TH-92-1**: se il census totale `ProbeWindow::arm` (righe codice) ≠ narm, ogni figura di campagna con probe è VOID.
- **KS-TH-92-2**: se noprobe contiene una seconda definizione di `probe_in` in QUALSIASI grafia (indentata/function-form), ogni VERDICT che vi si appoggia è VOID.

## Refutazioni capitali
Nessuna: i buchi trovati sono nelle guardie in avanti, non nelle cifre di S-90.0 (i 2 siti reali sono conformi alle grafie contate; nessuna figura emessa attraversa i buchi). Le cifre restano in piedi; i denti vanno affilati PRIMA di S-91.0.

— T. Hoare, sedia 1, WP-92
