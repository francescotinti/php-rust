# WP_SESSION_28 — archivio storico della sessione WP-28

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> 🏁 **WP-28 (2026-07-20, gated `b72d14f` + `29bbb4e`)**: chiusura dei gap
> estensioni del handoff WP-27. **(1) asymmetric visibility 29→38/39**:
> `prop_indirect_guard` (container-fetch W/RW/UNSET di prop readonly/set-denied
> — Zend get_property_ptr_ptr+read_property: valore oggetto passa, RW su
> typed-uninit = uninit fatal, unset su uninit no-op, altrimenti "Cannot
> indirectly modify") cablato in field_write/field_unset/field_cell/
> asym_set_ref_copy; assign-on-null Warning→**Error** con verbo assign/modify;
> promotion porta set_visibility (cpp_*); ridichiarazione PLAIN di prop hooked
> **EREDITA gli hook** (GH-19044); msg readonly esplicito protected(set).
> **(2) ext/xsl 57→63/64**: trace-shaping (frame prelude→prelude SPARISCONO dal
> backtrace, call-site prelude = "[internal function]" — bug49634 + 3 corpus);
> registerPHPFunctionNS; sezione xsl in phpinfo; **input-callback libxml FFI per
> compress.zlib://** (xslt008/-mb/009 — ⚠️ xslt009 passa con CWD = root di
> php-8.5.7, convenzione make test: misurare la suite xsl dalla root).
> **(3) GC free-order Zend-fedele**: gc_queue max-heap→**FIFO** (ordine di nota
> = ordine di release = ordine free/destructor Zend) + **gc_birth** (le entry di
> gc_track/re-seed sono seed interni phpr: la cascata del padre le CONSUMA) +
> **gc_release_cascade** (untrack dei discendenti esclusivi senza distruttore ⇒
> Object::drop postorder replica la cascata, id del PADRE in cima al free-list)
> + purge var_dump_debug/stringify_args al release. Probe id unset/temp/
> multi-unset ESATTI vs oracle; tidy resta 44/45 (010: solo il caso
> var_dump-albero, inquinato dalle over-note del dump — residuo).
> **Gate22 tutto verde** (nessuna regressione da FIFO/trace su ORM/hk/option/
> restapi) · corpus 1476→**1455** (21 rimossi, 0 nuovi) · **run19 = run17 per
> nome** · **multisite riconfermata: 1 diff (wp_is_stream #2) = minimo teorico**.

## Lezioni operative della sessione

- ⭐⭐ **Ordine free/destructor Zend = ordine delle RELEASE**: la coda dei
  candidati GC deve essere FIFO in ordine di nota; le entry di gc_track alla
  nascita (e i re-seed light-demoted) NON sono release — vanno marcate
  (gc_birth) e consumate dalla cascata del padre, altrimenti bloccano il
  riuso id children-first di Zend.
- ⭐ **Cache per-id (var_dump_debug/stringify_args) vanno purgate al FREE**,
  non solo al riuso in next_id: un debugInfo memoizzato tiene vivi i
  contenitori dell'oggetto e falsa i conteggi di esclusività della cascata.
- ⭐ **zend_std_read_property W/RW/UNSET** su prop readonly/set-denied:
  oggetto→copia (l'indirezione via handle non scrive lo slot), UNDEF+UNSET→
  no-op, altrimenti "Cannot indirectly modify"; ptr_ptr RW+UNDEF+typed →
  uninit fatal PRIMA di readonly/aviz (vale anche per prop pubbliche!).
- ⭐ **I prelude sono gli internals C di Zend anche nei BACKTRACE**: frame
  prelude→prelude si elidono; un frame user chiamato dal prelude rende
  "[internal function]" senza chiavi file/line nell'array.
- ⭐ **Suite phpt e CWD**: run-tests gira dalla ROOT di php-src — i test che
  usano path relativi (xslt009: document('compress.zlib://ext/...')) passano
  solo da lì. Il runner eredita la cwd: misurare le suite ext dalla root.
- ⚠️ Il timeout dei task background è 10 min: run >10' vanno lanciate con un
  daemonizer perl (double-fork + setpgrp + exec-array per i path con spazi) e
  monitorate sul marker .done.
