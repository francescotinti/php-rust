# s157-criterio-al1 — leva «L-AL1: miss/autoload class_exists a plumbing 0-alloc» (fetta NUOVA per NOME, E ∈ [4,82;6,05]M, istruttoria census S-156; PRE-REGISTRATO prima di edit/misura)

1. **Istruttoria di forma (fatta, al sorgente)**: il miss ripetuto di
   class_exists con autoloader Composer è CACHEATO lato user
   (`missingClasses`, ClassLoader.php:448) ⇒ il costo per-miss è dominato dal
   plumbing engine di `try_autoload` (mod.rs): (1) `autoloading.insert(key.
   to_vec())` guard re-entrante; (2) `Zval::Str(PhpStr::new(name.to_vec()))`
   arg del loader (2 alloc: to_vec + copia ZStr); (3) `vec![arg.clone()]`
   per iterazione loader. ≈4 alloc/miss sul cammino comune (1 loader).
2. **Edit (SOLO vm/mod.rs)**: (a) campo `autoload_key_pool: Vec<Vec<u8>>` —
   guard key dal pool (`HashSet::take` la restituisce), steady-state 0-alloc;
   (b) `try_autoload(name, key, name_zs: Option<&PhpStr>)`: arg = rc-clone
   dello ZStr originale quando `z.as_bytes()==name` (cammino class_exists
   senza prefisso `\`), altrimenti `PhpStr::new(name)` DIRETTO (niente
   to_vec); (c) `resolve_class_autoload` delega a
   `resolve_class_autoload_with(name, zs)`; `resolve_named_class_with_autoload`
   passa `Some(&raw)`; gli altri 10 chiamanti INVARIATI (None).
   EMENDA DICHIARATA (pre-misura, alla scrittura degli edit): `try_autoload`
   ha DUE chiamanti diretti in più oltre ai previsti (unserialize mod.rs e
   un sito host.rs) — entrambi a `None`, comportamento INVARIATO (diagnostica
   rust-analyzer 0 errori sui due file; RA attivo DICHIARATO durante gambe
   pair t7 on-1..on-2: la firma ictx per-gamba arbitra).
   RESIDUO DICHIARATO: `vec![arg.clone()]` per loader resta (1 alloc/iter
   loader). Semantica INVARIATA: stessa stringa (case originale) al loader —
   stringhe PHP immutabili, condivisione rc sicura; guard case-insensitive
   su chiave lowercased identica; ordine walk/cursori INTATTI.
3. **Attesa fondata**: 3 alloc/miss rimossi sul cammino comune ⇒ UB-alloc
   falsificabile = 3 × miheap 6,9 = **20,7 ns/iter** (giudice a 1 miss/iter).
   Scala suite: E è ALLOC non chiamate ⇒ leva MICRO-JUDGED (attesa ORM
   0,01–0,05 s < risoluzione 0,293; veto S-155 rispettato).
4. **Giudice**: `m-missload.php` (wp157-harness), N=10.000.000 letterale
   (tick 1,0 = soglia/4), 1 class_exists MISS/iter con autoloader closure
   no-op registrato (modello Composer miss-cacheato). Segno atteso D=A−B
   POSITIVO. Parità: stampa `ML-OK 10000000`.
5. **Soglia**: max(4 ns/iter; rumore drop-1). D > UB 20,7+rumore ⇒ canale
   non-alloc, reperto a verbale + sonda conteggi DOVUTA (non blocca).
   **Riconciliazione smoke↔R5** (az.rev. S-156 #2): banda max(4, rumore);
   fuori banda ⇒ reperto ARBITRATO nel verdetto di sessione PRIMA di usare
   D come cifra di leva.
6. **R**: smoke R=2 early-stop a segno opposto → R=5 ABAB. **A = GEMELLO
   RICOSTRUITO sul tree s156** (ricetta canonica; atteso 42efea3e34feb390 —
   byte-id al pin; se diverge dal byte si arbitra a CONTENUTO 48 B
   LC_UUID+firma, emenda S-154, e si DICHIARA); B = tree s156 + SOLI edit
   p.2 (edits in tree NON committati; A si costruisce con `git stash`,
   B dopo `git stash pop`), hash dichiarato al run, stash SOLO via
   `pin-phpr.sh --braccio`. Build SOLO a coppia p.1 conclusa (macchina
   quieta; nessun edit coi build in volo).
7. **Guardie SOLO-REGRESSIONE** con COMPARATORE PRE-REGISTRATO (az.rev.
   S-156 #1): morde sse **D < −soglia (disuguaglianza STRETTA)**; il bordo
   esatto NON morde e si dichiara reperto. Set: **hostargs (m-hostargs
   N=10M — hit-path class_exists, NUOVA guardia dichiarata)** + backtrace24
   (N=2,4M) + obj* (bande fondate 13,3/6,7/10,0/3,3) + le sei (SL storiche).
8. **Disasm**: la leva NON tocca run.rs; bl-count `run_loop` A vs B
   registrato comunque (atteso Δ=0; Δ≠0 = reperto, non gate).
9. **Parità/igiene**: output A==B su OGNI categoria pena STOP; attesi smoke
   BLIND (`s157-smoke-atteso-al1.md`) verificati da SECONDO attore PRIMA del
   run; lock di sessione presente; quiescenza rc=0; rc autoritativi da file;
   dente loc mod.rs: se il cap 25742 morde al promo ⇒ salita DICHIARATA sul
   file di test (lezione S-156).
