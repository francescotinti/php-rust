# s154-criterio-ce1 — leva L-CE1 «class_exists lookup no-alloc via LcKey» (fetta ALLOC per NOME, coda S-152/S-154; PRE-REGISTRATO prima di edit/misura)

1. **Edit** (solo mod.rs, fuori run_loop — `resolve_class_autoload` è metodo
   NON-inline: disasm non dovuto, dichiarato): (a)
   `resolve_named_class_with_autoload` perde il `to_vec` (passa la slice);
   (b) `resolve_class_autoload` sostituisce `bare.to_ascii_lowercase()` con
   `LcKey::new(bare)` (SSO stack 64 B GIÀ ESISTENTE, heap fallback oltre —
   nessun tipo nuovo). Semantica INVARIATA (stessa chiave, stesso ordine di
   lookup/trait/autoload); miss-path e nomi >64 B allocano come oggi
   (dichiarato).
2. **Attesa conteggio** (da k=2 ESATTE, s152 ce-count; census s154:
   class_exists 9,74M alloc ≈ 4,87M chiamate): hit-path con nome ≤64 B
   k 2→0.
3. **Giudice**: `m-classexists.php` (wp154-harness), N=10.000.000 letterale
   (tick ≈ 1 ns/iter). Segno atteso D=A−B POSITIVO.
4. **Soglia**: max(4 ns/iter; rumore drop-1). **UB-alloc falsificabile**:
   2 × miheap 6,9 = **13,8 ns/iter**; D > 13,8+rumore = canale non-alloc
   (malloc/free evitati ≠ solo prezzo pair; copia to_vec eliminata) —
   POSSIBILE e ATTESO alla lezione S-154-sonda: va a verbale con sonda
   ce-count k post-cura DOVUTA al probe ricostruito (non blocca il verdetto).
5. **R**: smoke R=2 early-stop a segno opposto → R=5 ABAB. **A = gemello
   2023cbb981a19d5f** (build FREDDA del tree corrente, ricetta canonica,
   contenuto==pin verificato in s154-sonda S2; emenda §7-bis rispettata);
   B = stessa ricetta fredda dal tree + SOLI edit L-CE1, hash dichiarato al
   run.
6. **Guardie SOLO-REGRESSIONE**: backtrace (giudice BT2, N=150000) + obj*
   (bande fondate 13,3/6,7/10,0/3,3, altrove max(4;drop-1)) + le sei (SL
   storiche). L-CE1 non tocca alcun loro cammino (resolve_class_autoload non
   è sul cammino di quei micro: nessuno fa lookup di classi per nome in
   loop).
6-bis. **EMENDA (dichiarata nell'atto, dopo il morso ab-ce1b; rev. S-112)**:
   la guardia backtrace a N=150000 ha tick di quantizzazione 66,7 ns/iter
   (timer 10 ms — nota revisore S-153) e ha morso con D=−66,7 = ESATTAMENTE
   1 tick, rumore drop-1 0,0/0,0 (collasso da quantizzazione), con B
   IDENTICO allo smoke (466,7) e A flippato di un tick (500→400): guardia
   SOTTO-RISOLUTA per una soglia a 4 ns (veto trasversale nominato).
   ARBITRATO: la guardia si riesegue RI-RISOLUTA con `m-backtrace-hi.php`
   (derivato dichiarato: solo 150000→600000, tick 16,7), R=5 ABAB coi due
   bracci del record, pavimenti med3, soglia INVARIATA max(4; rumore
   drop-1): D ≤ −soglia ⇒ REGRESSIONE REALE (leva in istruttoria, niente
   promo); |D| < soglia ⇒ tick-flip REFUTATO, guardia dichiarata ok e il
   verdetto ab-ce1b si legge col giudice VINTO. Il morso resta agli atti.
7. **Parità**: output A==B su OGNI categoria pena STOP (m-classexists stampa
   10000000). Fedeltà: fx-ce bilaterale (forme case-insensitive, leading-\,
   interface, trait, autoload=false, nome >64 B) oracle==phpr==A==B al promo.
8. **Igiene**: lock di sessione presente, quiescenza rc=0, attesi smoke BLIND
   (`s154-smoke-atteso-ce1.md`) verificati da SECONDO attore PRIMA del run;
   rc autoritativi da file.
