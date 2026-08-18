# S-153 p.1 — istruttoria famiglia borrow C2: conteggi per SITO (lettura)

Sonda `s153-sonda-conteggi.sh` rc=0 (probe ab02faec0abfab67, k=Δ/ΔN su
ΔN=200.000, tutti k INTERI ESATTI — verdetto `s153-sonda-conteggi-verdetto.out`).

## Tabella k (canale c2) + sub-siti nominati al sorgente (pin s150)

| micro | sito | k | sub-siti (mod.rs/run.rs/oop.rs) |
|---|---|---|---|
| td-this | frame_teardown.borrow | **2** | id-probe `$this` (mod.rs:4610) + nota `gc_note_slow` (3952) — ATTESA 2 ✓ |
| td-local | frame_teardown.borrow | **1** | sola nota (H-C1a già unificato) — ATTESA 1 ✓ |
| td-this/local | Sweep.borrow | **1** | demote del vivo (4187, già unificato) ✓ |
| td-die | Sweep.borrow | **5** | cand_id (4205) + unbuffer (4109) + class_id (4376) + lazy (4391) + cascade-oid (4462) — ATTESA 4–6 ✓ |
| ps-set | PropSetPop.borrow | **3** | **FUORI attesa 1–2** → terzo borrow NOMINATO: `lazy_prop_access` fast-path (mod.rs:12977); poi IC hit-check (run.rs:702) + Ref-check `write_property_at` (oop.rs:88) |
| ps-set | PropSetPop.borrow_mut | **1** | scrittura `replace_slot` (oop.rs:96) |
| td-this | MethodCall.borrow | **2** | fuori mandato, annotato per la coda |

Note: (a) il walk di `gc_release_cascade` (RelItem, Rc nudo nella copia census)
non è contato in C2 — borrow reale in più, fuori conteggio, dichiarato;
(b) il wrapper ObjRc conta SOLO i borrow del codice VM (interni census via
`raw()`): i k sono reali, non artefatti di probe.

## Conseguenze (fette)

1. **L-TD1 (questa sessione)**: teardown/sweep a borrow unico — `$this` 2→1
   (nota inline sotto il borrow dell'id-probe) + free-seq 5→2 (blocco letture
   unificato; id passato al cascade). −4 borrow/free+ctor-frame. SOLO mod.rs,
   fuori run_loop. Criterio: `s153-criterio-td1.md`.
2. **Fetta 2 in coda per NOME — PropSetPop 3+1→1+1**: riordino IC-first (l'IC
   hit-check prova già `lazy.is_none()` ⇒ sussume il probe di
   `lazy_prop_access`, invariante lazy⊇proxy da provare) + scrittura sotto il
   borrow del hit-check. `prop_set_entry` è `#[inline(always)]` nel run_loop
   ⇒ protocollo disasm size/bl PRIMA/DOPO obbligatorio (lezione H-C2/FR1).
   ~2 borrow × 57,4M ≈ 0,49 s lordi al prezzo mock (magnitudine da giudice).
3. Il canale zval-census `note_gcnote` e gc-census `note()` non vedono la
   nota inline di L-TD1 ($this): i census FUTURI su copia contano 1 borrow al
   posto di 2 — è l'EFFETTO della leva, dichiarato (il gate census-eco
   «conteggi identici» vale per le tranche A2 refactor, non per le leve A3).
