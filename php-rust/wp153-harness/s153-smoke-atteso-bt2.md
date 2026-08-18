# s153-smoke-atteso-bt2 — attesi BLIND dello smoke L-BT2 (dichiarati PRIMA del run)

Smoke = `s153-ab3.sh <B> <hash8> smoke-bt2 2` (R=2, early-stop a segno opposto
prima del record R=5).

## Attesi ESATTI

1. **Parità output** su TUTTE le categorie: diff A vs B VUOTO. In particolare:
   m-backtrace stampa ESATTAMENTE `300000` + newline (bt_recurse: 150.000
   iterazioni × count(bt)=2 frame con limit=2 — derivato dal sorgente del
   giudice); objdropdef/objchurn/objdatains stampano `1500000` (già derivato
   e promosso in s153-smoke-atteso.md).
2. **Guardie d'ingresso**: A = gemello `f95a1067` (emenda §7-bis recepita);
   B hash8 = dichiarato ≠ f95a1067; quiescenza rc=0; lock presente e mio.
3. **Giudice backtrace, R=2**: D=A−B segno **POSITIVO 2/2**; un segno opposto
   = early-stop e istruttoria. Magnitudine attesa (solo orientativa allo
   smoke, MAI cifra): D ∈ [60; 220] ns/iter (centro modello ~140–160 =
   −20±3 alloc/call × miheap 6,9; UB 160). N=150000 DICHIARATO nel runner
   (il sorgente usa `$n`: l'awk generico non lo estrae — punto d'audit).
4. **Guardie**: nessuna regressione oltre soglia; BT2 non tocca i cammini
   obj*/sei (host builtin + BtFrame soltanto).
5. **File attesi**: `ab-out/smoke-bt2.rc` autoritativo (0 o 4: con R=2 la
   soglia può non essere superata senza che lo smoke fallisca — arbitra il
   SEGNO 2/2), `s153-smoke-bt2-verdetto.out`, `ab-out/smoke-bt2-runs.tsv`
   con 2 righe per categoria (13 categorie).

## Punti d'audit per il secondo attore

- il valore 300000 va ri-derivato dal sorgente di m-backtrace.php (non dal
  binario); attenzione: `count($bt)` con limit=2 su pila profonda = 2;
- s153-ab3.sh: N=150000 per backtrace nel runner; php_of mappa backtrace →
  m-backtrace.php; floor su wp149-harness/empty.php (appena copiato: va
  verificato uguale a wp127-harness/micro-orm/empty.php);
- il diff sorgente BT2 (git diff HEAD: mod.rs BtFrame/collect + host.rs
  statics/ho_debug_backtrace/ho_debug_print_backtrace) deve preservare:
  ordine di inserimento chiavi (file,line,function,[class,object?,type],
  [args]), semantica IGNORE_ARGS/limit/PROVIDE_OBJECT, composito eval,
  `{closure}`/`eval` come nomi funzione, e NON toccare run_loop;
- le chiavi statiche sono ZStr regolari (hash per contenuto): nessun cambio
  di forma osservabile dall'utente PHP.
