# s153-criterio-bt2 — leva L-BT2 «debug_backtrace: chiavi statiche + ZStr condivisi» (fetta micro per NOME dalla pesca S-152; PRE-REGISTRATO prima di edit/misura)

1. **Edit**: host.rs `ho_debug_backtrace` (chiavi risultato da pool thread-local
   STATICO — Key::Str clonato, zero alloc/hash per insert; `type` da ZStr
   statici `::`/`->`; niente clone del file) + mod.rs `BtFrame`
   {function,file,class} → ZStr costruiti UNA volta in `collect_backtrace_opt`
   + `ho_debug_print_backtrace` adeguato (`as_bytes`). **NIENTE presize**
   (PhpArray senza with_capacity: fuori fetta, dichiarato). Host builtin fuori
   run_loop ⇒ disasm non dovuto (lezione FR1 non applicabile, dichiarato).
2. **Attesa conteggio** (da k=45 ESATTE, pesca s152): per frame-con-classe
   spariscono 5 chiavi (5) + type (2) + 1 dei 2 alloc di file (clone+new→move)
   + 1 di function + 1 di class = −10/frame; ×2 frame = **−20±3 alloc/call**
   (le alloc di crescita PhpArray e gli Rc restano) ⇒ k atteso 22–28.
3. **Giudice**: m-backtrace (`wp149-harness/m-backtrace.php`), **N=150.000
   DICHIARATO nel runner** (il loop del sorgente usa `$n` variabile: l'awk
   generico non lo estrae). Segno atteso D=A−B **POSITIVO**.
4. **Soglia**: max(4 ns/iter; rumore drop-1 del run). **UB falsificabile**:
   23 alloc/call × miheap 6,9 ns (alloc+free coppia, verdetto s152-sonda) ≈
   **160 ns/iter**; D > 160+rumore ⇒ fuori banda a verbale, sonda dovuta.
5. **R**: smoke R=2 early-stop a segno opposto → R=5 ABAB; **A = gemello
   f95a1067** (emenda §7-bis TD1 recepita: braccio A dal tree corrente, mai
   dallo stash del pin a tree avanzato); B dichiarato per hash al run.
6. **Guardie SOLO-REGRESSIONE** (nessun companion): objdropdef objchurn
   objdatains objalloc objallocni objmap (bande fondate dove esistono:
   13,3/6,7/10,0/3,3; altrove max(4; drop-1)) + arith prop calls str arr re
   (SL storiche). BT2 non tocca alcun loro cammino.
7. **Parità**: output A==B su OGNI categoria pena STOP (m-backtrace stampa
   300000). Fedeltà: fx-backtrace bilaterale byte-id AL PROMO; il perimetro
   semantico (chiavi = ZStr regolari, hash per contenuto) non cambia forma.
8. **Igiene**: lock mio, quiescenza rc=0, attesi smoke BLIND
   (`s153-smoke-atteso-bt2.md`) verificati da SECONDO attore prima del run di
   record; rc autoritativi da file `.rc`.
