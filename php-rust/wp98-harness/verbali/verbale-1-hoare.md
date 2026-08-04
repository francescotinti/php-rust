# Verbale sedia 1 — Hoare (WP-98, su S-96.0 e programma §WP-97)

Perimetro: design linguaggio/runtime Rust, safe-only. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI (con due refutazioni capitali)

**Stato dei miei emendamenti WP-97.** A-TH-97-1 applicato, e nella variante
MIGLIORE delle due che avevo offerto (`inb` clonato da `out` PRIMA del merge
exc, kill delle def, poi exc rifuso in entrambi): `out` — giudice della
movibilità — porta il contributo dell'handler senza il kill, `inb` lo riceve
dopo. **È sound.** A-TH-97-2 applicato ma solo a metà (vedi A-TH-98-2).
A-TH-97-3 eseguito (design96). **A-TH-97-4 e A-TH-97-5 sono EVAPORATI**: non
sono nel backlog per NOME né nei NON-riproporre. KS-TH-97-1/2/3 nessuna scatta.

## Emendamenti

**A-TH-98-1 — L'invariante che rende sound il fix non è presidiata da nulla.**
Oggi nessun op ha insieme `defs` ed `edges`: se ne avesse uno, il kill di
`e.defs` colpirebbe TUTTI gli archi normali, ricreando la stessa classe di
errore su un arco dove la def non è avvenuta. Serve `debug_assert!(e.defs
.is_empty() || (e.edges.is_empty() && e.fall_defs.is_empty()))`. In più: il
loop che allarga `nbits` incatena `uses`/`defs`/`fall_defs` ma **non gli edefs
per-arco** (`CatchMatch::var`) — e `Bits::clear` indicizza senza difesa, quindi
lì si panica invece di degradare. Un canale su quattro dimenticato.

**A-TH-98-2 — L'esaustività copre le VARIANTI, non i CAMPI.** Ogni arm
no-effect è scritto `Op::X { .. }`: aggiungere un campo `Slot` a `Sweep`,
`FillDefault` o `HookCall` compila in silenzio. Campionate tre sospette —
`CallBuiltinRefCell { name, argc }`, `StaticStore { id }`, `MakeFcc { name }`:
**nessuna porta uno Slot** (verificato sull'enum: l'insieme delle varianti che
portano `Slot`/`DimBase`/`FieldBase` è interamente classificato FUORI
dall'elenco no-effect). L'elenco di oggi regge; la cura no.

**A-TH-98-3 — `renounce()` scarta le rinunce in silenzio.** La sua `nbits` si
allarga solo su `LoadSlot`/`LoadVar`, mentre quella di `analyze` si allarga su
tutti gli usi; `mark` scarta l'indice fuori larghezza e `Bits::get` risponde
«non rinunciato». Doppio fallimento silenzioso, e nella direzione che rende F2
MENO conservativa. Stessa larghezza di `analyze` + assert.

**A-TH-98-4 — L'arco di ri-lancio di `EndFinally` non è modellato.** `edges` =
`after` + i soli bersagli di `ParkJump`. Un'eccezione parcheggiata e ri-lanciata
all'`EndFinally` raggiunge un handler ESTERNO: quell'arco esiste solo se l'op
cade dentro la regione esterna di `exc_table`, cosa che nessuna fixture prova.
Stessa classe di A-TH-97-1. Fixture obbligatoria prima di ogni emissione.

**A-TH-98-5 — Ri-registrare A-TH-97-4 (contabilità `gc_note` del valore
spostato) e A-TH-97-5.** Un emendamento il cui oggetto è rinviato non decade da
solo.

**A-TH-98-6 — design96 arbitra con un numero che non trascrive** (+2,9%,
`WP_SESSION_41`). Un documento di decisione non auditabile non decide.

## Kill-switch

**KS-TH-98-1**: un campo `Slot` nuovo su una variante dell'elenco no-effect →
tutti i conteggi F1/F2 invalidi.
**KS-TH-98-2**: un riconteggio a parità di binario che muove `slot_reads_rc` →
il raw scende da VERDICT a SCREEN e nessun delta sotto il rumore è attribuibile.
**KS-TH-98-3**: se il tetto A-LB-97-1 resta insoddisfacibile prima di O1, la
voce 1 del §WP-97 non può concludere e va eseguita SOLO insieme a O1.

## Refutazioni capitali — SÌ, due

**R1 — Il pavimento di rumore è dentro il raw stesso.**
`delta_slot_reads_rc_vs_f2 = -14` su un contatore che NESSUNA modifica del
changeset può toccare (`note_slot_read` conta al sito di lettura, indipendente
da `analyze` e da `observes_scope`). Quindi le due esecuzioni non sono
identiche. Ne segue: i delta −21/−18/−6 **non sono attribuibili** ad A-SK-97-1
né a nulla; `grade=VERDICT` è sbagliato (contare esattamente un'esecuzione non
deterministica dà un verdetto sul run, non sul programma); sopravvive solo
`delta_would_take = 0`, che è esatto.

**R2 — Il verdetto del passo 2 è insieme troppo debole e troppo largo.** Coi
numeri del documento: lordo 0,84–1,21% contro un pedaggio in casa di +2,9%. Non
è «non distinguibile da zero»: **in quella forma è nettamente negativo**. Nella
forma per-braccio, che §5.1 dichiara mai valutata, è **ignoto**. «Non
distinguibile da zero» non descrive nessuna delle due. E la rinuncia a `nm -S`
è circolare: non si misura perché si è deciso di non costruire, e si è deciso
per un pedaggio non misurato. Riscrivere: «la forma a braccio nuovo perde con
margine; la forma per-sito è indecisa e costa una misura, non una leva».
