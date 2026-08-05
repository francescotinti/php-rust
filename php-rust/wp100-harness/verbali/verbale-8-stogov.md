# Verbale Sedia 8 — Stogov (Zend engine, semantica) — Concilio WP-100

## VERDETTO
S-98.0: la FORMA del handler `Op::BinaryAdd` REGGE i miei attacchi semantici.
Il programma S-99.0 NO: il punto 3 (rollout nelle forme registro) porta un
criterio derivato dal percorso SBAGLIATO, e la coda AssignOp è ambigua tra
verdetto `.out` e ordine. **Approvo il report CON EMENDAMENTI; refuto il
criterio del punto 3 dell'ordine.**

## Attacchi condotti (esiti)
(a) **Ref sulla pila**: `Op::LoadVar` (run.rs:636-653) e `Op::LoadGlobal`
(:755-758) passano da `read_slot`, che DEREF-CLONA i `Ref`: nessun `Ref`
raggiunge `BinaryAdd` da questi emettitori. Se mai arrivasse (nessun sito
nell'emissione lo produce), la guardia `(Long,Long)` MISS → pop lhs →
`binary_value_ab` — lo STESSO funnel del vecchio `Binary(Add)` (che pure
non fast-pathava i Ref in `binary_fast`). Identico per costruzione. TIENE.

(b) **Overflow**: run.rs:959-962 = `checked_add`, MISS → `l as f64 + r as
f64` sugli operandi ORIGINALI — verbatim `binary_fast` (:118-121) e
identico a Zend `fast_add_function` (`(double)op1 + (double)op2`). −0.0
non producibile da Long+Long. TIENE.

(c) **GMP/BcMath/__toString**: oggetto ⇒ guardia MISS ⇒ `binary_value_ab`
⇒ `binary_fast` MISS ⇒ `apply_binop_ovl` — braccio identico al vecchio.
Add non è `cmp_op`: il ramo `__toString` (run.rs:335) non è mai toccato,
com'era prima. TIENE.

(d) **Diagnostica**: il fallback passa `(lhs, rhs)` in ORDINE (run.rs:
969-971): "Unsupported operand types: int + array" conserva i nomi. Il
warning undef è emesso da LoadVar PRIMA dell'op (queued, :640-646),
invariato; e con un operando Undef la pila porta Null ⇒ MISS ⇒ funnel
vecchio. Né il vecchio hit di `binary_fast` né il nuovo hit in-place
flushano i diags: parità anche lì. TIENE.

(e) **Siti op=**: expr.rs:90-106 (slot), assign.rs:956-962 (prop, RHS
prima di PropGet come ASSIGN_OBJ_OP), :971-992 (global/superglobal):
`emit_binary` sostituisce 1:1 nella STESSA posizione di sequenza; timing
di __get/__set e degli effetti del RHS invariato. `FieldAssignOp`/
`AssignOpPath` portano il payload `op` e NON passano da `emit_binary`:
fuori perimetro, dichiararlo nel rollout. TIENE (con nota).

## Refutazioni capitali
**R1 — Il criterio del punto 3 è derivato dal percorso sbagliato.**
D=6,07 ns/occ misura il plumbing dello STACK-path (call con due Zval per
valore + pop/push). Ma `BinarySS` (run.rs:1011-1019) già BORROW-a i due
slot dentro `binary_fast` senza marshalling né pop/push: il rimovibile lì
è solo match esterno+Option — classe di costo DIVERSA e plausibilmente
≪6 ns. Un criterio «in ns/occorrenza derivato da D=6,07» ripete l'errore
che S-98.0 stessa ha messo nei NON-riproporre (criterio dal conteggio
senza cammino critico). Va derivato da un controfattuale del percorso
REGISTRO, con la sua soglia.

**R2 — La coda AssignOp è ambigua.** Il verdetto di `hb2-addspec.out`
(:134-136) la elenca fra le «prossime occorrenze della stessa famiglia»;
l'ordine S-99 la tiene in BACKLOG dietro le sette trappole A-ST-99-3 (non
ancora SCRITTE). Due letture possibili = un buco: senza dente, il rollout
punto 3 può inghiottire il fold.

## Emendamenti
- **A-ST-100-1**: prima del punto 3, controfattuale statico del percorso
  registro (che cosa rimuove la specializzazione DENTRO BinarySS/SC/Dst)
  e soglia pre-registrata da QUELLO, non da D=6,07.
- **A-ST-100-2**: le sette fixture-trappola A-ST-99-3 diventano
  PRECONDIZIONE nominata nell'ordine (non prosa di backlog).
- **A-ST-100-3**: fixture in-tree (non smoke una-tantum): `PHP_INT_MIN +
  (-1)`, ordine dei nomi con const-lhs (`3 + $arr`), alias by-ref
  (`$r=&$a; $r+1`), confrontate all'oracle.

## Kill-switch
- **KS-ST-100-1**: ogni braccio specializzato dentro le forme registro
  tiene il MISS = funnel owned ATTUALE (warning Undef, deref Ref,
  `binary_value_ab`), ordine dei diags byte-identico; un braccio che
  tocca i tag Undef/Ref da sé è VOID.
- **KS-ST-100-2**: nessun fold/fusione AssignOp atterra prima che le
  sette trappole passino per NOME su ENTRAMBI i motori.
