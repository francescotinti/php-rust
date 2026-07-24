# WP_SESSION_46 — Cycle collector esteso ai root container (Ref/Array/Closure)

> ⚡ **WP-46 (2026-07-24, `d4a02fa`→`e6af390`+)** — **La leva dominante della
> FOOTPRINT_CPU_ROADMAP eseguita: il cycle collector ora traccia i possible
> root NON-oggetto (array condivisi, inner delle Ref-cell, closure), modello
> zend_gc.c letto dall'oracle via Vexp. 18/18 probe oracle byte-identici,
> famiglia gc del corpus 36→14 fail (22 fixati, 0 regressioni), corpus intero
> 1447→1421 (0 nuovi fail, 26 fixati).**

## Il design (fedele a zend_gc.c, letto PRIMA di scrivere — regola Vexp)

1. **Chi diventa root** (`gc_possible_root` ha `ZEND_ASSERT(IS_ARRAY ||
   IS_OBJECT)`; `gc_check_possible_root` per le reference): una REFERENCE
   non è MAI root — si guarda il suo inner e si bufferizza QUELLO se
   collectable. phpr: `gc_note` con `strong_count > 1` ora bufferizza
   l'array condiviso (`may_hold_containers` come narrowing del rule Zend),
   l'inner-collectable delle Ref, e le closure con capture container /
   bound_this (una closure è un oggetto in Zend).
2. **Buffer container = `Weak`** (`CtrWeak`), NON clone forte: un clone
   forte altera `strong_count` per tutta la vita bufferizzata e blocca i
   descend `count==1` a valle — la sentinella WP-39 l'ha beccato come
   ritardo dei dtor `[k1][k2]` al primo tentativo. Con `Weak`: zero
   perturbazione, e il candidato morto per refcount fallisce l'upgrade al
   drain = la rimozione-a-morte del buffer Zend, gratis. **Dedup O(1) =
   `Rc::weak_count == 0`** (nessun altro nel workspace downgrade questi Rc)
   — la lezione WP-40 del flag-nel-valore senza toccare alcun layout.
3. **Spezzare i cicli puro-container in safe Rust**: ogni ciclo che non
   contiene Object passa per una `RefCell` (gli array phpr sono COW —
   `$a[0]=$a` copia; le capture by-value sono fissate alla creazione) ⇒
   svuotare le REF-CELL white in place è l'unico punto di taglio necessario;
   array e closure del ciclo cadono per cascata Rc. `PhpArray` non si tocca.
4. **Conteggio Zend-esatto di `gc_collect_cycles()`** (decifrato da
   `zend_gc_collect_cycles` 1995-2194 + probe): conta solo il garbage
   distrutto VIA BUFFER; le reference mai; gli oggetti con dtor pendente
   sono esclusi dal round col loro sottoalbero white
   (`gc_remove_nested_data_from_buffer`) e ri-rootati purple; **peel
   refcount-dead**: si pelano iterativamente i white con zero in-edge
   white (= morti per refcount in Zend, mai contati — created li pinna,
   Zend no); **eccezione dtor-phase-dead**: il DELREF senza zero-check di
   `gc_call_destructors` (riga 1898) lascia in buffer l'oggetto il cui
   ultimo holder è sparito DENTRO la fase dtor → contato al rerun; il
   mirror phpr è esatto: `strong_count == 1` (solo created, nemmeno
   candidate-buffered) a fine fase dtor, perché il clone receiver di
   `call_method_sync` droppa senza nota come il DELREF muto.
5. **Ordine Zend nel round**: dtor PRIMA del destroy (zend_gc.c 2093→2113);
   il destroy tocca solo ciò che nessun dtor pendente può raggiungere ⇒
   resurrection-safe per costruzione; il fixpoint del loop phpr resta (il
   garbage da cascata non sopravvive alla chiamata).

## Under-note chiusi (siti di drop che non notavano — dtor ritardati a shutdown)

- `Op::PropUnset` / typed-unset (`Props::remove` → `Option<Zval>`,
  `Props::replace` per il typed) — gc_029 passava da qui.
- `Op::BindRef` rebind (`$a =& $b` scarta il vecchio valore dello slot) —
  gc_019/gc_021.
- `gc_active` latch (`gc_collecting`): collect annidato da dentro un dtor
  ritorna 0 (gc_016); il latch si pulisce anche sull'error-path.
- `gc_enable`/`gc_disable`/`gc_enabled`/`gc_status` diventati HOST builtins
  con stato reale (`gc_enabled` gate del solo auto-collect; INI
  `zend.enable_gc` registrata e mirrorata; `running` = latch — gc_049 lo
  usa nei dtor; `runs`/`collected`/`roots` contatori veri).
- `var_dump`: `*RECURSION*` senza `&` (Zend lo emette con PUTS, senza il
  prefisso COMMON) — gc_004/006/007/009, bug72530, bug35163×2, bug35239,
  foreach_002.
- **Fast-path `Op::Sweep` (WP-39) esteso alla pressione container**: senza,
  un workload di soli cicli container non svegliava mai la sweep (sintetico
  60k: collects 0) — ora `collects 1 (roots 50000 freed 50000)` con UNA
  sweep svegliata.

## Sentinelle (RULEBOOK §3)

- `gc_object_cycle_collect_sentinel` (commit `d4a02fa`, PRIMA di toccare
  codice): pin O1-O7b dei cicli oggetto già oracle-identici — il restructure
  non ne ha cambiato un byte.
- `gc_container_cycle_collect_acceptance`: probe oracle-derived 1-13
  (conteggi + timing dtor). ⭐ `vm_stdout` test = Registry::default(): niente
  `mt_rand` nei fixture (usare variabili per sfuggire al const-interning).

## Gate

- Famiglia gc: 36 → **14** fail per nome, 0 regressioni. Residui catalogati
  (PHPR_DIVERGENCES 2026-07-24): meccanica soglia Zend 10001 (bug70805×3,
  gc_023, gc_045 — soglia phpr 50k INTOCCABILE per direttiva), divergenze
  pre-esistenti non-GC (bug64960/65372/67314, gc_046), temp container morti
  a metà statement (gc_022 — nessun sito di nota; gli oggetti hanno il
  re-check light-demoted, i container no), forma stack-trace dtor-da-collect
  (gc_030), release iteratore foreach al break (gc_047), rerun-shape gc_049.
- Corpus standalone (binario `0c76bb9`): **1447 → 1421**, 0 nuovi, 26 fixati.
- cargo test --release: **1639/0** (1637 + 2 sentinelle).
- **Gate22 completo VERDE (binario `e6af390`, 02:57→03:21, archivio
  `gate-out-wp46-archived/`)**: corpus 1421 (0 nuovi-fail, solo rimozioni —
  la policy del gate le ammette) · sess 28 / date 351 / refl 290 IDENTICI ·
  ORM 3E/13F identico per nome · hk 0E/0F · cargo 1639/0 · gd/mysqli/media
  probe BYTE-ID · http DIFF-set 16 byte-identico al WP-44 · **option 413
  IDENTICO per nome** · **restapi 3508 IDENTICO per nome (1E/1F = oracle)**.
- Divergenza documentata: conteggio su array annidati da LITERAL const in
  ciclo (Zend: immutable, conta 1; phpr: materializzati per-valore, conta 3
  — memoria comunque liberata). Interning const-array = candidato Fase 1.5.

## Mechanism check (mem-census + gc-census, gruppo media) — ⭐⭐ VERDETTO PESANTE

**Il collector ora GIRA ma il garbage non c'è (o non è suo)**: peak esterno
4,89G (census-run; baseline census WP-45 4,77G) · arr.live 1,905G/3,113M
vive · str.live 1,288G/23,82M — **IDENTICI alla cifra a WP-45**. gc-census:
**collects 11, roots processati 726.439, freed 543**, threshold adattivo
esploso a 450k (collect inefficaci ⇒ i root notati sono dati VIVI veri).

Due letture possibili, da discriminare in WP-47:
- (A) i cicli morti non transitano MAI da un sito di nota — ma i probe
  costruiti per trovare il drop silente falliscono tutti (l'element-write
  su elemento ref fa write-through e SPEZZA il ciclo; slot/prop/unset/
  BindRef/teardown ora notano tutti);
- (B) **la diagnosi WP-45 era in parte una MIS-ATTRIBUZIONE**:
  "irraggiungibile dal root-walk" ≠ "ciclo morto" — il root-walk non
  cammina FramePool (frame ritirati con slot?), operand stack, tabelle VM
  (ob, resources, iteratori, IC), e ciò che tiene 1,9G di array può essere
  VIVO ma fuori dalle 11 categorie camminate. Il collector che processa
  726k root e ne libera 543 pesa a favore di (B).

⇒ Il deliverable WP-46 (root-tracking modello Zend) è costruito, corretto e
gate-proven; il bersaglio ~3G NON è caduto. WP-47 = attribuzione di SECONDA
generazione: owner-tracer (chi tiene i 3,1M array vivi), root-walk esteso a
FramePool/tabelle mancanti, PRIMA di qualunque altra leva footprint.

## A/B 6 round (old=dd148a0, new=e6af390; oracle 20,79-20,85 stabilissimo)

**REGRESSIONE CPU +7,0% TENUTA (direttiva no-revert)**: old 55,50s medio
(55,24-55,74) = 2,67× · new 59,39s medio (59,31-59,58) = **2,85×** — 6/6
round new>old, delta stabilissimo. Footprint: old 4,41-4,46G → new
**4,55-4,56G** (+2,5%) = 12,3×. Parità A/B perfetta (762/1912/52 su tutte
le 14 run). Attribuzione del costo: gli 11 collect (classify = walk del
grafo raggiungibile da ~726k root VIVI) + il rooting nel note-path (Weak
downgrade/probe sugli array condivisi) + il termine ctr nel fast-path
Sweep — pagati senza recupero perché freed=543. ⭐ Se WP-47 conferma la
lettura (B) (root vivi strutturali), le leve di recupero sono: rooting più
selettivo, cap più aggressivo dell'escalation, collect ancorati al confine
test — ma PRIMA l'attribuzione.

## ⭐⭐ Lezioni

- ⭐⭐ **Un buffer GC che pinna coi cloni forti è un bug di fisica**: ogni
  `strong_count == 1` a valle smette di essere vero. `Weak` + upgrade al
  drain riproduce ESATTAMENTE la semantica del buffer Zend (entry rimossa
  alla morte per refcount) senza perturbare nulla. Il dedup `weak_count==0`
  è gratis se nessun altro downgrade il tipo.
- ⭐⭐ **Il conteggio di gc_collect_cycles() non è |whites|**: è "distrutto
  via buffer", con esclusione dtor-subtree, peel refcount-dead e l'eccezione
  DELREF-muto — 18 probe oracle sono serviti a falsificare due modelli
  sbagliati prima di quello giusto. Ogni scorciatoia ("conta tutti i white")
  regala +1/-1 su metà famiglia gc.
- ⭐⭐ **Ogni sito di drop che non nota è un dtor che slitta a shutdown**:
  PropUnset e BindRef erano under-note storici, invisibili finché il
  collector non vedeva i container. La sweep-verify non li beccava perché
  gli oggetti restavano `strong>2` (pinnati dal ciclo container invisibile).
- ⭐ Un fast-path di skip (Op::Sweep WP-39) è un CONTRATTO col resto del
  sistema: nuova pressione ⇒ nuovo termine nella condizione, o il
  sottosistema muore in silenzio (sintetico: 300k note, 0 sweep, 0 collect).
- ⭐ Vexp sul C dell'oracle PRIMA del design: `gc_check_possible_root` (le
  ref non si bufferizzano) e il DELREF-senza-check hanno dettato due scelte
  architetturali che nessun probe avrebbe suggerito da solo. (Indice vexp
  corrotto: `vexp daemon-cmd stop` + rm index.db* + `._*` AppleDouble +
  reindex 83s.)

## Prossimo

[DA COMPILARE dopo census/AB: se footprint media crolla → Fase 1 residua
(shrink unit ~0,3G, created→Weak, cold-box Object) o Fase 2 CPU; residui gc
famiglia: gc_047 (nota release iteratore al break), gc_030 (trace shape),
interning const-array.]
