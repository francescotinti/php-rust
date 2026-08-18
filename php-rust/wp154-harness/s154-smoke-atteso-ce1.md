# s154-smoke-atteso-ce1 — attesi BLIND dello smoke L-CE1 (dichiarati PRIMA del run)

Smoke = `s154-ab-ce1.sh <B> <hash8> smoke-ce1 2` (R=2, early-stop a segno
opposto prima del record R=5).

## Attesi ESATTI

1. **Parità output** su TUTTE le categorie: diff A vs B VUOTO. In particolare:
   m-classexists stampa ESATTAMENTE `CE-OK 10000000` + newline (10.000.000
   iterazioni, class_exists true a ogni giro — la classe è definita nello
   stesso file, namespace `Doctrine\Tests\Models\CMS`, hit-path);
   m-backtrace (guardia) stampa `300000` (derivazione s153 promossa).
2. **Guardie d'ingresso**: A = gemello `2023cbb9` (build fredda del tree
   corrente, contenuto==pin per s154-sonda S2, stash verificato al run);
   B hash8 dichiarato ≠ 2023cbb9; quiescenza rc=0; lock presente (sessione).
3. **Giudice classexists, R=2**: D=A−B segno **POSITIVO 2/2**; un segno
   opposto = early-stop e istruttoria. Magnitudine SOLO orientativa allo
   smoke, MAI cifra: D ∈ [8; 45] ns/iter (modello alloc 2×6,9=13,8; il
   canale copy/malloc-free può eccedere l'UB-alloc — lezione S-154-sonda:
   sopra 13,8+rumore va a verbale, non invalida). N=10.000.000 letterale nel
   sorgente (l'awk generico DEVE estrarlo: punto d'audit).
4. **Guardie**: nessuna regressione oltre soglia; L-CE1 non tocca i cammini
   backtrace/obj*/sei (lookup di classe per nome assente in quei micro).
5. **File attesi**: `ab-out/smoke-ce1.rc` autoritativo (0 o 4: con R=2 la
   soglia può non essere superata senza che lo smoke fallisca — arbitra il
   SEGNO 2/2), `s154-smoke-ce1-verdetto.out`, `ab-out/smoke-ce1-runs.tsv`
   con 2 righe per categoria (14 categorie: classexists + backtrace + obj*6
   + sei).

## Punti d'audit per il secondo attore

- `CE-OK 10000000` va ri-derivato dal SORGENTE di m-classexists.php (loop
  letterale `$i < 10000000`, `$acc++` su ogni true; classe definita nel file
  ⇒ nessun autoload, hit-path);
- s154-ab-ce1.sh: src_of/php_of devono mappare classexists →
  wp154-harness/m-classexists.php; l'awk estrae N=10000000 dal letterale;
- il diff sorgente L-CE1 (mod.rs: resolve_named_class_with_autoload +
  resolve_class_autoload) deve preservare: stessa chiave lowercased (LcKey
  produce gli stessi byte di to_ascii_lowercase), stesso ORDINE
  index→trait→autoload→index, miss-path e >64 B invariati (heap), nessun
  edit dentro run_loop, firma dei metodi invariata;
- LcKey è il tipo SSO GIÀ esistente (mod.rs ~15420): nessun tipo nuovo.
