# s128-admission-lettura.md — L-OL1 seg.2: ammissione ESEGUITA, forma nominata

Ordine rispettato (criterio f14a365 PRIMA di tutto): census (`s128-admission-verdetto.out`,
ACQUISITA, F1 6/6 OK) → disasm (`s128-disasm-verdetto.out`) → profilo campionato →
sonde differenziali (`s128-probes-verdetto.out`, ACQUISITE) → QUESTA lettura → forma.

## Census pin s127b (R=2 identici) + sonde differenziali
objalloc 7 · objdatains 12 (**Δins_alloc = 5**) · objdropdef 8 · objchurn 13 · objmap 1.
Sonde (Δ vs objalloc): `[]=` **+4** · `[0]=` **+5** · `['k']=` **+5** · locale `$a=[];$a['k']=` **+3**
· 2 chiavi diverse **+8** (5+3) · **overwrite stesso k +9 (5+4)**.

## Lettura (il fatto nuovo)
1. **La struttura non spiega i conti**: KeyIndex sotto soglia è GIÀ scan-mode senza alloc
   (array.rs: `build`/`insert_new` con `SCAN_MAX`); `to_hashed` su vuoto non alloca; eppure
   l'overwrite in place — zero lavoro strutturale — costa **+4 alloc**.
2. **Costo FISSO per-statement dei path-write**: `pop_field_keys` (vm/mod.rs:13511) alloca un
   `Vec<Zval>` per OGNI statement con chiave (`[]=` senza chiave non lo paga: ecco il −1 di p2);
   `pop_keys` (mod.rs:8193, `split_off`) idem per la famiglia AssignPath locale (ecco il +3 di
   p3-local = init 1 + keys 1 + push 1, e l'**unica** alloc di objmap).
3. Il resto del Δins: COW `make_mut` del default condiviso (+1 alla prima scrittura, F1 nota),
   push entries (+1), e ~2/statement residui nella filiera `field_write` (1297 ins, 92 bl;
   `dropped` Vec quando sposta un vecchio valore) — NOMINATI per seg.3, non oggetto di questa forma.
4. Profilo objdatains: nessun dominatore; famiglia lookup-per-NOME in testa (HashMap::get 1366 +
   memcmp 975 + slot_of 520 + hash 217+181) — coerente con S-127; indizio, nessuna cifra.

## FORMA NOMINATA: L-OL1-F2 «keys-scratch» (famiglia Field* write)
`Vm.field_keys_scratch: Vec<Zval>` riusato: i handler di scrittura (`FieldAssign`/`FieldAssignOp`/
`FieldIncDec`) prendono le chiavi nello scratch (mem::take → fill → …); `field_write`/`field_write_walk`/
`field_write_prop_step` passano da `&mut vec::IntoIter<Zval>` a iteratore GENERICO; i consumatori
terminali (`field_set_mode`/`field_set_in_root`) drenano con `drain(..)` e RESTITUISCONO il buffer
(capacità conservata) prima del drain AA. Rientranza (AA/magic → user code → nested write): lo
scratch preso è già fuori ⇒ mem::take dà Vec fresco = comportamento attuale, solo senza riuso. La
famiglia AssignPath locale (`pop_keys`/`path_op`) NON è in questa forma (apertura seg.3 per NOME).

**Predizioni census della forma (arbitro census, da verificare PRIMA dell'A/B)**:
objdatains 12→**11** · objchurn 13→**12** · p4-int 12→**11** · p5-two 15→**13** · p6-overwrite
16→**14** · INVARIATI: objalloc 7, objallocni 5, objdropdef 8, objmap 1, p2-append 11, p3-local 10.

## Criterio A/B della forma (vincoli fissati in criterio p.7, qui istanziati; commit PRIMA del codice)
- Giudice: `wp127-harness/micro-orm/objdatains.php` (N=3e6 dal sorgente), R=5 ABAB, user CPU
  netto-pavimento per-binario; pavimento = med3 `empty.php` PER binario.
- Segno: objdatains phpr CALA. Soglia: max(4 ns/iter, rumore R=5, banda-layout prop = SL max
  s123/s125 della categoria prop). Smoke R=2 early-stop a segno opposto.
- Guardie SOLO-REGRESSIONE: sei micro storiche + objalloc + objchurn + objmap.
  **Bande objchurn/objmap: NESSUNA banda storica ⇒ dichiarato «default 4 ns» + spread del run**
  (az.rev. S-127 #4); nessun commento ereditato (wp97) nell'arbitro.
- Gate promozione (solo se soglia superata): batteria · corpus congelato 1414 per NOME ×2 modi ·
  fixture bilaterali · ORM 16 nomi == baseline · hk 0E/0F · pin via scripts/pin-phpr.sh.
- rc di ogni script = SOLO il file rc proprio, nominato nel verdetto (az.rev. #2); verdetto avverso
  committato PRIMA di ogni emenda (az.rev. #3).
