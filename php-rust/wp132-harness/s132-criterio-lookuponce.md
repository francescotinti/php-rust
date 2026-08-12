# Criterio S-132 p.2 — LEVA L-LO1 «lookup-once» sulla props-map in field_write_prop_step (forma NOMINATA dal modello propstep S-131; commit PRIMA di ogni run e di ogni edit .rs)

1. **Forma**: nel ramo NON-leaf `!rest.is_empty() && !is_enum && !is_lazy` di
   `field_write_prop_step`, le ~3-4 operazioni sulla props-map della STESSA
   chiave (`props.get` nella container-guard + `contains` nel defer-check +
   `contains`/`get_mut` nel container_op) collassano in **UN accesso**:
   `get_slot_mut(si)` quando `ra = Slot{slot: Some(si)}` (indice WP-29 stampato
   per la layout della classe dell'OGGETTO — `cid = obj.class_id` — quindi
   `get_slot*`/`replace_slot` ≡ `get`/`contains`/`get_mut` by-name PER
   COSTRUZIONE: stesso slot che `slot_of(key0)` risolverebbe), altrimenti UN
   `get_mut(key0.as_ref())`. Lo stato per `prop_indirect_guard` deriva dallo
   STESSO accesso (`prop_slot_state` su reborrow condiviso); ordine
   guardia→defer(denied0/absent)→descend INVARIATO; enum, lazy e ramo leaf
   INVARIATI (fuori perimetro). Nessun cambio alloc previsto (nessuna Cow nuova).
2. **Giudice** `objdatains`; A = stash pin s131 (`phpr-s131`, ff66cb84), B =
   build canonica post-edit (hash a verbale); segno: B CALA; smoke R=2 con
   early-stop a segno opposto → conferma R=5. D = medianaA − medianaB (serie
   piene, net floor med3 per binario).
3. **Soglia giudice (az.rev. S-131 #1)**: max(4 ns/iter, rumore drop-1 corrente,
   **spread storico TRA mediane di batch sul pin s131 = 10,0 ns/iter** — batch
   committati 2026-08-11 del binario ff66cb84: smoke_B 1223,3 · R5_B 1230,0 ·
   submicro 1220,0 —, SL prop 0,80). `trange` con tie-break DETERMINISTICO
   (az.rev. S-131 #4): chiave di scarto (|x−mediana|, x) — a parità di distanza
   si scarta il valore MAGGIORE; funzione del multiset, non dell'ordine di arrivo.
4. **Bande guardie** SOLO-REGRESSIONE fondate sul pin s131 (range drop-1 delle
   7 gambe B committate S-131 — smoke R=2 + R=5, stesso binario ff66cb84 che qui
   è A, raw datati 2026-08-11): **objalloc 13,3 · objchurn 6,7 · objmap 10,0**
   ns/iter; le sei micro su SL storiche committate (max s123/s125).
   Soglia_reg = −max(4, banda).
5. **Riconciliazione (az.rev. S-131 #2)**: il verdetto R=5 DICHIARA
   |D_smoke−D_R5| (FUORI BANDA se > soglia corrente) e D vs UB del modello
   (**UB ≈ 30 ns/statement** = ~3 lookup × 10,4 dentro gli 81,9 non-resolve,
   s131-propstep-lettura p.4; FUORI BANDA se D > UB) — dichiarazioni a verbale,
   non gate: la promozione resta su soglia+guardie.
6. **Invarianti**: arbitro `s132-ab.sh` COPIA DICHIARATA di s131-ab.sh coi SOLI
   adattamenti p.3-5 + **verdetto = FILE NUOVO per tentativo** (az.rev. S-131
   #5: rifiuto se il file esiste; tentativo nuovo = TAG nuovo); quiescenza gate
   SEPARATO in testa (file proprio, header con file+valore); path B NEUTRO in
   scratchpad (argv del lancio SENZA pattern del gate — lezione S-131); disasm
   bl-count n/a dichiarato (la leva non tocca run_loop; parità stdout bilaterale
   per categoria nell'arbitro); rc autoritativo = SOLO `ab-out/<TAG>.rc`.
7. **Promozione** SOLO se giudice sopra soglia e guardie ok: catena
   `s132-promozione.sh` copia dichiarata di s131-promozione.sh → pin s132 via
   `scripts/pin-phpr.sh` + `pin-server.sh` (dopo la build canonica ricontrollare
   l'hash del server). Esito avverso ⇒ verdetto committato PRIMA di ogni
   lettura/emenda; revert che DEVE riprodurre ff66cb84 al byte.
