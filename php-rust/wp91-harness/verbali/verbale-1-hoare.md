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
