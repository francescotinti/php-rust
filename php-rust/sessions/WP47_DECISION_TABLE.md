# WP-47 — Tabella decisionale PRE-REGISTRATA (scritta PRIMA di leggere i numeri del walk esteso)

> Regola (mandato WP-47): l'attribuzione owner-level deve riconciliare
> 1,9G arr + 1,29G str entro ±15%. Le leve si scelgono da QUESTA tabella,
> compilata prima di conoscere il verdetto — per canale/holder dominante.
> Data: 2026-07-24, prima del run census esteso.

| # | Holder dominante (se ≥25% del canale arr o str) | Leva pre-registrata | Note |
|---|---|---|---|
| 1 | **FramePool: slot di frame ritirati NON azzerati** (Zval residui nei 64×512 slot riciclati) | **Drain/clear degli slot al retire** (`Frame::reset` azzera slots+operand stack al rientro nel pool) | Leva a costo ~zero CPU (il retire è già un evento); guardia CPU ≤+0,5% |
| 2 | **Frames stack vivi (ricorsione PHPUnit)**: operand stack / slot di frame ANCORA in stack ma logicamente morti | Disciplina di confine per-test (collect+drain al boundary) — NON si possono azzerare frame vivi | Modello Pedersen request-bound, Fase 1.4 della roadmap |
| 3 | **`created` registry (BTreeMap<u32, Rc<Object>>)**: pinna transitivamente arr/str via props | **created→Weak (o eviction a rc==1)** — Fase 1.2 già pre-approvata dalla roadmap | Il buco già indiziato dal review; gate dtor-order con sentinelle pinnate |
| 4 | **ob / output buffers** | Truncate/shrink al flush + confine per-test | Improbabile dominante (stringhe, non 3,1M array) |
| 5 | **resources / iteratori / session / typed_refs** | Release eager al close/fine-foreach (gc_047 è già in coda per l'iteratore al break) | Se iteratori: il fix gc_047 diventa prioritario |
| 6 | **IC / tabelle VM (PropIc/MethodIc/unit tables)** | Cap + eviction o Weak nelle IC; unit: shrink Fase 1.1 | Le IC tengono Rc di classi/funzioni, non dovrebbero tenere 1,9G di array |
| 7 | **Unit compilate / const-pool dei Module leakati** (array literal materializzati per-valore) | Interning const-array (già in coda, divergenza conteggio documentata) + shrink unit | Se il const-pool tiene M di array duplicati → interning sale di priorità |
| 8 | **Globals PHP-visibili sottostimati dal walk vecchio** (walk per-categoria bucato, es. array annidati non discesi) | Correggere il WALK, non il runtime — l'attribuzione WP-45 era sbagliata di misura, non di sostanza | Prima di ogni leva: ri-validare il deep_size |
| 9 | **Nessun holder singolo ≥25% (long tail)** | Owner-tracer campionario 1-su-N sui 3,1M array per istogramma fine; poi confine per-test come leva generale | Il tracer decide, non si indovina |

**Vincolo trasversale pre-registrato**: qualunque leva scelta va gated con
sentinelle dtor-order pinnate + corpus per nome; la soglia GC base 50k non
si tocca; census SOLO nel binario `phpr-mem-target/`.

**Recupero CPU (Obiettivo 2) — pre-registrato indipendentemente dall'esito
attribuzione**: classify a 2 passate (children-map solo se dtorW non vuoto),
mark intrusivi al posto delle HashMap, collect ancorato al confine test,
isteresi della soglia (i collect efficaci non riabbassano sotto il floor
dell'ultimo max se freed/roots < 1%). A/B 6 round old=e6af390.
