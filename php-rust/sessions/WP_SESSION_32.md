# WP_SESSION_32 — archivio storico della sessione WP-32

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-32 (2026-07-21/22, gated `43fc0c4`→`f020a33`, 7 commit)** — la leva
> "value-representation" RIDEFINITA dai dati (❌ **NaN-boxing BOCCIATO e da
> non riproporre**: Long(i64) a 64 bit pieni non entra nei 48 del payload
> NaN; ~9 impl unsafe su un value-core a zero unsafe; romperebbe la niche
> Option<Zval> degli array packed; ~5.000 siti; e il churn misurato NON
> viene dalla taglia — la zval di Zend è anch'essa 16B). Tre cluster:
> **(A) CmpJmp** — confronto+branch fusi a emit-time via cond_jump
> (root-match AST, mai peephole), handler = binary_value condiviso
> (refactor puro da Op::Binary ⇒ semantiche identiche per costruzione);
> cablati while/do-while/if/for/switch(Eq)/match(Identical)/&&/||/ternario;
> + peek zero-clone al posto dei 2 deref_clone per-confronto.
> **(B) path_apply** — PhpArray::slot_or_vivify e set_returning_displaced
> (UN lookup vs 2-4 + key clone per livello; semantica del composito ESATTA:
> no-revive WP-27, next_free, holds_containers — unit test di equivalenza a
> matrice); dropped Vec→Option e pop_keys via split_off (via 2 malloc/free
> per path op). **(C) Frame slimming ~400→≤176B** — FrameFlags(u8) per i 7
> bool + FrameExt boxed lazy per i campi freddi con ordine di drop
> conservato PER COSTRUZIONE (ext dopo iters; ogni campo Rc-bearing di
> FrameExt viveva già dopo iters; dyn_vars → Option<Box> IN PLACE; ret_cell
> resta inline) + 3 SENTINELLE drop-order committate PRIMA del layout
> (pinnano l'ordine phpr corrente — passate INVARIATE dopo).
> **Esito: microbench esteso −8,7%** (6,09 vs 6,67s, 5 coppie interleaved
> vs cb82691); **media group 72,4→69,0s (−4,7%), rapporto 3,3×**;
> full-suite 12:54 (−1% — ormai dominata da IO/mysql/C-libs); riprofilo
> (`wp30-harness/ab-out/new-wp32.sample`): **memmove 629→301 (−52%),
> path_apply SPARITO dalla top-20**. gate22 TUTTO verde; cargo 1600;
> **run23 = run22 nel fail-set per nome** (2F identici; 9 diff P-only da
> test-set upstream — vedi incidente wpdev sotto).
> ⚠️ **INCIDENTE WPDEV RISOLTO**: lo scratchpad della vecchia sessione
> (be003709) è stato RIPULITO a metà nottata — wpdev ha perso vendor/,
> composer.json e quasi tutto (65MB residui). Ricostruito PERMANENTE in
> **`~/Claude/wpdev`** = wordpress-develop **trunk@5e3fced** (2026-07-15,
> la revision del setup originale — il tag 7.0.1 NON basta: mancano le
> classi 7.1 e i data-provider differiscono) + composer install + 
> wp-tests-config.php con **DB_PASSWORD 'wp-secret-Pass1'** (recuperata dai
> probe mysqli wp8). Tutti gli script aggiornati (run-full-detached,
> gate22, media-pair, run-multisite, gate19). Validazione: option 413
> IDENTICO per nome al tree vecchio; full-suite 2F identici.

## Lezioni operative della sessione

- ⭐⭐ **Il timing di distruzione phpr (sweep-driven) diverge GIÀ da Zend**:
  le sentinelle drop-order NON possono essere oracle-diff — vanno pinnate
  sull'output phpr CORRENTE, committate PRIMA del cambio layout (metodo
  C2→C3: 3 sentinelle rosse-se-cambia, passate invariate).
- ⭐⭐ **Boxare campi freddi senza riordini osservabili**: mettere il Box
  DOPO l'ultimo campo hot Rc-bearing e ordinare i campi interni come nel
  layout pre-esistente; i campi che romperebbero l'ordine si boxano IN
  PLACE (dyn_vars → Option<Box> alla stessa posizione) o restano inline
  (ret_cell). MAI un Drop manuale su Frame (romperebbe mem::take del pool).
- ⭐ Fusione op a EMIT-TIME, mai peephole (rimuovere op sposta gli
  indirizzi); fondere solo quando la RADICE AST è il pattern (il bool
  interno consumato come valore non è mai fondibile per costruzione).
- ⭐ API composite di PhpArray: replicare il composito ESATTO (contains+
  insert+get_mut) con unit test di equivalenza a matrice su tutte le forme
  repr — mai "quasi uguale" (holds_containers/next_free/ordine sono parità).
- ⭐⭐ **Gli scratchpad delle vecchie sessioni in /private/tmp VENGONO
  RIPULITI**: wpdev ci ha vissuto per 9 sessioni ed è stato sventrato a
  metà run. Gli asset di lunga vita vanno in **~/Claude/** (ora:
  ~/Claude/wpdev). Se una suite dice "Could not open input file" dopo ore
  di sleep del Mac, è il reaper, non una regressione.
- ⭐ Ricostruire wpdev: trunk alla DATA del setup (il tag release non
  basta: test-set diverso), composer install, wp-tests-config con la
  password del probe wp8 ('wp-secret-Pass1'); validare con option 413 per
  nome + fail-set full identico; le differenze P-only upstream si
  documentano e si ribasa il confronto (run23 è la nuova base).
- Il Mac in sleep congela le run detached per ore: guardare i timestamp
  del .done prima di diagnosticare un hang.
