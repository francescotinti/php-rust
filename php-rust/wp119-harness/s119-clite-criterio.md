# s119-clite-criterio.md — C-lite ESECUZIONE (istruttoria s118, vincolo concilio; criterio PRIMA della run)

**Oggetto**: tabella 6×4 — (rc-clone/iter, alloc/iter) × (phpr, Zend) sulle sei categorie
(`wp97-harness/micro/`); SOLO CONTEGGI, nessuna cifra di tempo (build census ≠ pin, REGOLE §3/§6).
Netting: evento_categoria − evento_empty.php, / N_iter emesso dal sorgente (stessa awk del giudice).
R=2 per lato: conteggi attesi identici; se divergono si pubblica lo spread. Verdetto:
`s119-clite-verdetto.out` (solo esiti/numeri) + classifica delta (phpr−Zend)/iter che ordina i vagoni treno-2.

**Gamba phpr**: build strumentata `--features zval-census,mem-census` in target separato
(`phpr-census-target`), MAI il pin; alloc/iter = `galloc_n` netto; rc/iter = contatore NUOVO
`zvalclone_rc` (impl Clone manuale di Zval sotto feature, conta i cloni delle varianti Rc).

**DEROGHE NOMINATE** (az. rev. S-118 §4 — trappola citata):
1. «rc-op = incref+decref» (istruttoria) → si misura il CLONE Zval-level (incref lato clone,
   lower bound: i cloni diretti di Rc interni sfuggono); decref per CONSERVAZIONE sul ciclo
   stazionario (convenzione zvalcensus.rs:155). Trappola evitata: nessun choke point incref/decref
   esiste in phpr (Rc std); il conteggio pieno richiederebbe newtype invasivo — fuori timebox.
2. Gamba Zend rc-op = mappa STATICA (opcode/iter dal dump opcache × rc-op per handler, verificati
   su php-8.5.7 via Vexp), non dinamica: incref/decref Zend sono macro inline, invisibili a
   DTrace/pid-probe. Dichiarata: stima analitica, non misura.
3. Gamba Zend alloc = dinamica con `USE_ZEND_ALLOC=0` + dylib interposta che CONTA
   malloc/calloc/realloc/free (realloc disaggregato come phpr). La deroga: ZendMM spento cambia
   il TEMPO ma non il NUMERO di emalloc-eventi; il tempo qui non si misura. Oracle MAI ricompilato.
4. La build census EMENDA il sorgente (zval.rs cfg_attr + memcensus): fuori feature l'espansione è
   identica ⇒ la ricetta A′ DEVE riprodurre il pin 15dfb6b3 al byte dopo l'edit, pena STOP.

**Timebox**: ½ sessione (vincolo concilio). Se la gamba Zend-statica sfora, si pubblica la tabella
con la colonna dichiarata parziale (le categorie analizzate per NOME) — mai cifre implicite.
