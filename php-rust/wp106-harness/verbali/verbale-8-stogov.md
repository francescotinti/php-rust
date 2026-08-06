# Verbale sedia 8 — STOGOV (Zend engine / opcache, semantica PHP) — Concilio WP-106 su S-104

**VERDETTO: CON EMENDAMENTI** (nessun MI OPPONGO; nessuna refutazione
capitale). La caduta H-C2 con meccanismo icache è coerente con la storia
della VM Zend: la valuta del dispatch loop è la I-cache, e Zend vince
riducendo i byte TOCCATI per iterazione, non i costi di chiamata.

## (a) memory_get_usage stub costante — R-ST-106-1/2, A-ST-106-1

**R-ST-106-1**: lo stub 2.000.000 è PEGGIO di un'assenza — viola
correct-or-absent. In Zend è il contatore vivo di AG(mm_heap), e i
programmi reali lo usano per CONTROLLO DI FLUSSO (chunking di batch,
check di headroom in WP/WP-CLI, debug bar): un costante sotto
memory_limit disattiva PER SEMPRE ogni salvaguardia. Peggio: **compone
col leak generator-in-cycle** — il leak che il GC non raccoglie è anche
invisibile al programma, perché il contatore mente. fx20 l'ha già
provato in casa (verdetto in-script VACUO). Catalogo: voce 🔴 NUOVA,
subito, insieme a memory_get_peak_usage (alias dello stesso stub) e
memory_reset_peak_usage (no-op).

**R-ST-106-2 — refuto la cura come formulata**: «contatore ZMM-like sul
global allocator promosso a release» ha due difetti. (i) galloc−gfree
conta TUTTO il processo Rust (unit-cache, compile, buffer interni) che
Zend NON conta (AG contabilizza solo emalloc per-richiesta; il malloc
persistente è escluso) — byte-parity IMPOSSIBILE per costruzione, e in
php-server un atomico process-global conflaziona i worker mentre AG è
per-thread. (ii) La promozione aggiunge 2 atomiche per alloc/free
esattamente sul canale calls appena inchiodato (1 alloc + 1 free per
chiamata): non è gratis dove H-D sta misurando.

**A-ST-106-1**: cura a due gradini. (i) Release: contatore NETTO
per-thread (TLS, niente atomics nel CLI mono-thread; nel server design
per-richiesta esplicito, non gratis) alimentato dal wrapper, oppure
query mi_* on-demand (costo zero sul percorso caldo); semantica
DICHIARATA a catalogo: functional-parity (monotono, ordine di grandezza,
peak vero, reset funzionante), MAI byte-parity con l'oracle. (ii)
**KS-ST-106-1**: la promozione passa SOLO con A/B sui sei giudici (calls
in testa) + disasm bl-count, criterio pre-registrato; senza, resta
census-only e la voce resta 🔴.

## (b) generator-in-cycle e maturità §3.12/§3.13 — R-ST-106-3, A-ST-106-2

**R-ST-106-3**: sì, resta la divergenza GC più grave — l'unica con massa
NON limitata (leak per l'intera richiesta) e timing dei distruttori
osservabile. Zend la chiuse esattamente con zend_generator_get_gc
(slot di execute_data + sent value + retval + this/closure). §3.12 e
§3.13 sono limitate (stato post-errore; unit del warning).

**A-ST-106-2**: la decisione è MATURA per tutte e tre, e la risposta è
**FEDELTÀ, non assenza**: non si può «assentare» una transizione di
stato di feature già implementate (typed props, generator, warning).
Ordine: generator (get_gc omologo: container + descend delle catture;
arbitro = fixture rossa esistente) > §3.13 marca (unit,line) con fixture
include+eval > §3.12 SOLO regime (i) weak+op-throws (catena
UNDEF→verify PER TIPO; bracci strict e `.=` nel gate — KS-ST-105-1
resta). Nessun altro censimento serve; serve solo la finestra.

## (c) superistruzioni — R-ST-106-4, A-ST-106-3

**R-ST-106-4**: icache-bound conferma la strada Zend, con un distinguo
che refuta la scorciatoia: Zend SPECIALIZZA moltiplicando handler e
vince perché contano i byte toccati per iter, non il testo totale; ma
run_loop è UN match monolitico — varianti aggiunte allargano il dispatch
stesso. Quindi FUSIONE che SOSTITUISCE sequenze, non specializzazione
che aggiunge varianti. Budget nel criterio: taglia di run_loop (pin
257.632 B).

**A-ST-106-3 — le due fusioni che rendono di più**: (1) **prop → op RMW
fuso su proprietà** (omologo ZEND_ASSIGN_OBJ_OP / PRE_INC_OBJ): `$o->x
op= k` in UN op, niente transiti di pila, gran parte degli 11 DropS/iter
eliminati alla fonte — ed è lo STESSO sito dove vivono §3.11/§3.12: la
fusione può portare la fedeltà nel medesimo op. (2) **arith → riesumare
le forme registro Add** (ARCO REGISTRI, sospese «salvo misura ≥
pavimento»): tre-indirizzi senza transiti (23/iter censiti S-102),
omologo delle SPEC(CV,CV) su TMP. **KS-ST-106-2**: ogni fusione col
protocollo S-104 — criterio prima, disasm bl/byte prima-dopo, A/B da
sola.

## Priorità S-105 (dal mio perimetro)

1. Fusione prop RMW (leva perf dovuta per ritmo + veicolo di fedeltà
   §3.11/§3.12-regime-i). 2. Catalogo memory_get_usage 🔴 + gradino (i)
   della cura. 3. Fix generator get_gc (fixture rossa = arbitro).
4. §3.13 (unit,line). Refutazioni capitali: nessuna.
