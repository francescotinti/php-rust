# S-153 p.1 — sonda conteggi per SITO: famiglia borrow-teardown C2 (A3a)

Mandato (NEXT §S-153 p.1): i 3 siti dominanti c2 del census s151
(frame_teardown.borrow 61,0M · PropSetPop.borrow 57,4M · Sweep.borrow 50,0M)
valgono ~170M borrow ≈ 0,55 s. Prima della leva: decomporre OGNI sito nei
suoi sub-siti di borrow con conteggi ESATTI. Metodo pesca S-152: micro a DUE
N col probe census s151 `ab02faec0abfab67` (conservato ×2, hash verificato in
pre-flight), k = Δconteggio/ΔN — le costanti di setup si elidono. CONTEGGI,
mai tempo (immuni da contesa; lock di finestra comunque preso, CI in coda).

## Sub-siti enumerati dal sorgente (pin s150, lettura Serena 2026-08-18)

- `gc_note_frame` (fase frame_teardown, mod.rs:4578): per frame con `$this`
  Object il cammino paga **2 borrow dello stesso handle**: `o.borrow().id`
  (probe destructed/created, r.4610) + il borrow unificato di `gc_note_slow`
  (r.3952, H-C1a). Ogni Object nei `slots`/`stack`/`extra_args` paga 1
  borrow (note_slow). Sub-siti: ft_this_id · ft_this_note · ft_slot_note.
- `gc_sweep_impl` (fase Sweep, mod.rs:4129): candidato DEMOTED (vivo) = 1
  borrow (r.4187); candidato LIBERATO = cand_id (r.4205) + gc_unbuffer
  (r.4372) + class_id (r.4376) + lazy check (r.4391) + cascade ≈ **4–5
  borrow dello stesso handle**; drain finale 1 borrow per slot residuo.
  Sub-siti: sw_demote · sw_free_seq.
- `prop_set_entry` IC-hit non-TY (run.rs:698): hit-check 1 borrow (r.702) +
  `write_property_at` (borrow/borrow_mut interni). Rapporto census suite
  borrow:borrow_mut = 57,4:11,8 ≈ 4,9:1 ⇒ nel run reale dominano cammini
  NON IC-hit; il micro dà il k del solo cammino IC-hit.

## Micro (DUE N: N1=100000, N2=300000, ΔN=200000)

- `td-this.php`: N chiamate a metodo VUOTO su un oggetto stabile.
- `td-local.php`: N chiamate a funzione VUOTA con 1 argomento oggetto.
- `td-die.php`: N iterazioni `new C;` scartata (free via sweep per statement).
- `ps-set.php`: N assegnamenti `$o->x = $i` (classe plain, IC-hit steady).

## Attese BLIND (falsificabili, dichiarate PRIMA del run)

| micro | sito.canale | k atteso | perché |
|---|---|---|---|
| td-this | frame_teardown.borrow | **2** | id-probe + note_slow sullo stesso handle |
| td-local | frame_teardown.borrow | **1** | solo note_slow (niente $this) |
| td-this/td-local | Sweep.borrow | **1** | demote del vivo (mirror flags già settati) |
| td-die | Sweep.borrow | **4–6** | sequenza free: cand_id+unbuffer+class_id+lazy+cascade |
| ps-set | PropSetPop.borrow | **1–2** | hit-check + eventuale borrow in write_property_at |
| ps-set | PropSetPop.borrow_mut | **1** | la scrittura |

k fuori attesa = sub-sito NON enumerato ⇒ si torna al sorgente PRIMA di ogni
criterio. Esiti in `sonda-out/` + verdetto `s153-sonda-conteggi-verdetto.out`.
La leva (forma candidata: borrow hoistato/unificato, store INTATTO — A3c
chiusa) si pre-registra SOLO dopo questa tabella riempita.
