# Verbale sedia 7 — Leijen (allocatore mimalloc, footprint fisico) — WP-98

Oggetto: S-96.0 (apparato + fix di soundness + confronto piano B) e §WP-97.

## VERDETTO

**NON REFUTATO nel merito; DUE refutazioni capitali sulle premesse con cui il
mio perimetro è stato interrogato.** La chiusura di A-ZV2 per verdetto invece
che per tempo è disciplina, non rinuncia. Ma l'atto d'accusa che mi viene
rivolto («il contatore `would_take_safe_str` è un numero di ALLOCAZIONI
evitate») è **falso a macchina**: `Zval::Str(Rc<PhpStr>)`
(`crates/php-types/src/zval.rs:22`) — un clone di stringa è un incremento di
refcount, **zero allocazioni**. I 9.989.963 sono elisioni di `Rc::clone`, cioè
CPU e linee di cache toccate, non byte chiesti all'allocatore. Chi legge quel
contatore «in chiave footprint» sbaglia di un canale intero, e sbaglierebbe in
buona fede: nessuno lo ha ancora fatto, ed è bene che nessuno cominci.

Il canale footprint di A-ZV2 esiste, ma è **un altro** e non è contato: un
valore che arriva a rc=1 fa sì che il write successivo passi per `Rc::make_mut`
**in place** invece di separare (CoW evitata) — quello sì alloca, ma solo se un
write segue, e il censimento conta SITI di lettura, non coppie lettura-scrittura.
`would_take_safe_str` è quindi un **maggiorante lasco di un canale il cui
contenuto in byte può benissimo essere zero**. Errore di perimetro: sì, ma non
quello contestato.

## Emendamenti

- **A-DL-98-1 (il falsificatore da 20 minuti che nessuno ha lanciato).** La leva
  arene per-file è ferma da tre rotazioni perché in coda c'è una COSTRUZIONE
  quando in coda dovrebbe esserci una MISURA che la può uccidere: `N = 25.795.552
  − T_max`, e **T_max non è mai stato misurato** (39.534.144 è capacità, non
  touched — sanatoria A12/M2). Un run strumentato per-unità dà T_max; se T_max è
  grande, N è piccolo e la leva muore **senza scriverla**. Questo, non la leva,
  va in testa alla coda.
- **A-DL-98-2 (il picco viaggia con la coppia, sempre).** Qualunque leva CPU del
  §WP-97 si giudichi con una coppia, la coppia registra ANCHE il peak fisico:
  costa zero (`/usr/bin/time -l` già gira) e senza di esso il footprint resta
  non misurato da m90 per *scelta implicita*. Predizione ex-ante firmata Δ≈0.
- **A-DL-98-3 (α resta da RI-DERIVARE, A-DL-72 invariato).** Sotto
  `MIMALLOC_PURGE_DELAY=0` mimalloc decommitta subito: l'argomento «pagine
  committed riusabili» è falso. 15 arene per-file = decommit→recommit→re-fault;
  con T_max si firma anche una predizione di `page reclaims`, o la leva paga in
  CPU ciò che incassa in footprint.
- **A-DL-98-4 (O1: predizione sul metro GIUSTO).** O1 va predetta, ma non dove
  mi si chiede: le pagine di testo sono file-backed e pulite, e **non sono
  addebitate a `phys_footprint`**. La predizione ex-ante è Δpeak ≈ 0; e il
  `.text` TOTALE da `nm -S` può **crescere** (prologhi/epiloghi non più fusi,
  sequenze di call) mentre il testo caldo residente cala — crescita che NON è un
  fallimento della leva.

## Kill-switch

- **KS-DL-98-1**: qualunque ricevuta che converta `would_take_safe_str` (o
  `_safe`, o `would_take`) in byte, allocazioni o footprint ⇒ **NULLA**.
- **KS-DL-98-2**: qualunque claim footprint di O1 letto su `phys_footprint` ⇒
  **VOID**; e `nm -S` e `max_rss` non si sommano né si confrontano fra loro.
- **KS-DL-98-3 (decadenza)**: se T_max non è misurato entro la prossima
  sessione, la leva arene per-file si dichiara **CHIUSA**, non rinviata. Una
  voce che nessuno esegue per quattro rotazioni non è una voce: è un alibi.

## Refutazioni capitali

**SÌ, due.**

1. **`would_take_safe_str` non è un numero di allocazioni** (Rc). L'accusa di
   «errore di perimetro» poggia su una premessa falsa; l'errore vero è
   simmetrico e opposto — il canale CoW, che il contatore non vede.
2. **La leva arene per-file non è una leva sull'OGGETTO del roadmap.** A-DL-71:
   vale 47% sul picco CLI hello, **1,8% sul media e 1,06% sul full**. Nessuna
   cifra del trend può riceverla né falsificarla. Ecco perché tutte le sedie la
   approvano nel merito e nessuno la esegue: paga su un oggetto che non è
   giudicato. O il picco CLI diventa un oggetto DICHIARATO con una sua colonna,
   o la leva va chiusa. Continuare a chiamarla «la leva del footprint» è un
   errore di categoria, e dura da tre rotazioni.

Sul §WP-97: **tre candidate CPU è corretto**, e va scritto perché. Dopo la
sanatoria WP-96 il regresso media footprint 3,381× è RITIRATO (era la gamba
oracle): **il footprint oggi non ha un difetto nominato**, quindi non ha una
candidata migliore. Non è una lacuna da coprire — è uno stato da verbalizzare.
