# s156-criterio-hd2host — leva «estensione L-HD2 forma-2 a CallHostBuiltin» (fetta ALLOC canale H-D, sito nominato S-155; PRE-REGISTRATO prima di edit/misura)

1. **Edit** (run.rs braccio `Op::CallHostBuiltin` + mod.rs macro + host.rs firme):
   (a) macro `host_builtins!` a DUE sezioni (slice/vec) che genera
   `dispatch_host_builtin_slice(&mut [Zval]) -> Option<…>` accanto al
   dispatcher Vec (che vi DELEGA per i nomi convertiti: nessuna doppia lista,
   match esaustivo conservato); (b) nel braccio op, arità ≤4 → POP DIRETTI in
   array nativo `[Zval; 4]` (ordine sorgente == split_off di pop_keys) e
   dispatch slice; nome NON convertito → ricostruzione Vec dai medesimi
   valori (1 alloc, identico a oggi + un match perso, dichiarato); arità >4 →
   cammino Vec invariato; (c) prima tranche convertita (corpi verificati
   SOLO-LETTURA, 1 solo chiamante ciascuno): class_exists, function_exists,
   interface_exists, trait_exists, method_exists, debug_backtrace — firme
   `Vec<Zval>` → `&[Zval]`. Semantica INVARIATA (stessi valori, stesso ordine,
   stesso contratto di visibilità storico HD2: args fuori dal frame durante
   la chiamata, come pop_keys oggi).
2. **Attesa fondata** (denominatore: census p.1 dà ALLOC non chiamate ⇒ scala
   suite DICHIARATA sotto-risoluzione: ~7 ns × N chiamate ≈ 0,01–0,05 s ORM
   < 0,293 — leva MICRO-JUDGED, veto S-155 rispettato): prezzo HD2 misurato
   ~7 ns/chiamata (S-125); driver 2 chiamate/iter ⇒ attesa ≈ +14 ns/iter.
3. **Giudice**: `m-hostargs.php` (wp156-harness), N=10.000.000 letterale,
   2 hostcall/iter a corpo 0-alloc provato (sonde fe/ce-true S-155).
   Segno atteso D=A−B POSITIVO.
4. **Soglia**: max(4 ns/iter; rumore drop-1). **UB-alloc falsificabile**:
   2 × miheap 6,9 = **13,8 ns/iter**; D > 13,8+rumore = canale non-alloc —
   a verbale con sonda conteggi post-cura DOVUTA (non blocca il verdetto).
5. **R**: smoke R=2 early-stop a segno opposto → R=5 ABAB. **A = gemello
   2023cbb981a19d5f** (pin INVARIATO s154: il gemello §7-bis resta valido);
   B = ricetta canonica fredda dal tree + SOLI edit p.1, hash dichiarato al
   run, stash SOLO via `pin-phpr.sh --braccio`.
6. **Guardie SOLO-REGRESSIONE**: backtrace RI-RISOLUTA a N=2.400.000
   (`m-backtrace24.php`, derivato dichiarato: solo N; REGOLE §3 az.rev.
   S-154 — tick ≈4,2 ≤ soglia/4) + obj* (bande fondate 13,3/6,7/10,0/3,3) +
   le sei (SL storiche). NB: debug_backtrace è nome CONVERTITO ⇒ la guardia
   backtrace copre anche il cammino slice (regressione = morde).
7. **Disasm DOVUTO** (protocollo S-104, leva su run_loop): bl-count di
   `run_loop` A vs B registrato PRIMA del giudizio; delta dichiarato a
   verbale (nessuna soglia pre-fissata: reperto, non gate).
8. **Parità**: output A==B su OGNI categoria pena STOP (m-hostargs stampa
   HA-OK 20000000). Fedeltà al promo: fx-ce bilaterale s154 (riusato) +
   forme method_exists/trait_exists/interface_exists nel corpus congelato.
9. **Igiene**: lock di sessione presente, quiescenza rc=0, attesi smoke
   BLIND (`s156-smoke-atteso-hd2host.md`) verificati da SECONDO attore PRIMA
   del run; rc autoritativi da file; CI in coda dichiarata se attiva
   (quiescenza la vede).
