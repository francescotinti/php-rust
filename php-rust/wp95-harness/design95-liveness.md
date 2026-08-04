# design95-liveness — A-ZV2: l'ultimo uso, deciso a compilazione

**Decisione dell'utente (2026-08-04)**: si va con la *strada lunga e giusta*.
La superistruzione `LoadSlot+Binary` resta **PIANO B**, documentata in
`design95-leva-zval.md` §Correzione — da riprendere se questa si dimostra
impraticabile o se serve un risultato rapido.

## Il fatto da aggredire (misurato, non stimato)

`wp95-harness/zvalcensus-before.out`, gruppo media, suite identica al
riferimento:

```
slot_reads=60598149  slot_reads_rc=53561241  (88,4%)
```

53,5 milioni di `refcount++` seguiti da `refcount--` in una sola esecuzione.
Nel profilo questo si vede come `Zval::drop` 7,20% + `Zval::clone` 2,85%
della CPU userland.

## L'idea

`LoadSlot`/`LoadVar` clonano **sempre** perché lo slot potrebbe servire
ancora. Nella maggioranza dei casi non serve: è l'**ultimo uso** prima che lo
slot venga riscritto o che la funzione finisca. In quei casi il valore si può
**spostare** invece di copiarlo, lasciando `Undef` nello slot.

È la stessa mossa che Stogov descrive per Zend (operandi `Take|Borrow|Copy`)
e vale su tutti i siti di lettura, non su un opcode alla volta: per questo è
la strada *giusta*, e per questo costa più di una superistruzione.

## Perché in PHP è più difficile che in un linguaggio qualunque

Uno slot locale **non è privato**: PHP offre molti modi per osservarlo dopo
che il compilatore ha creduto che fosse morto. L'analisi deve arrendersi (e
tenere il clone) in presenza di uno qualunque di questi, per funzione:

- `compact()` e `extract()` — leggono/scrivono i locali per nome;
- `get_defined_vars()` — materializza tutti i locali vivi;
- variabili variabili `$$x` e `${expr}` — il nome è dinamico;
- `eval()` e `include`/`require` dentro la funzione — codice arbitrario che
  vede lo scope;
- `debug_backtrace()` con gli argomenti, `debug_zval_refcount`;
- le **closure** che catturano per riferimento (`use (&$x)`) e `$this`;
- i **generatori**: lo stato dei locali sopravvive alla sospensione;
- i **riferimenti** (`Zval::Ref`): lo slot è condiviso e lo spostamento
  sarebbe osservabile altrove;
- i blocchi `try`/`catch`/`finally`: un salto non locale può rendere vivo uno
  slot che il flusso lineare dava per morto;
- i **distruttori**: svuotare uno slot **anticipa** un `__destruct`. Questo è
  il rischio più insidioso perché è osservabile (ordine di stampa) e non fa
  fallire nulla in modo rumoroso.

Il punto (10) merita una regola esplicita: **lo spostamento non deve
anticipare la morte di un valore**. Se lo slot è l'ultimo riferimento, il
`Drop` avviene al termine dell'operazione invece che alla riscrittura dello
slot; l'ordine dei distruttori è semantica PHP osservabile.

## Le fasi (ognuna con il suo giudice, nessuna che dipende dalla successiva)

**F1 — L'analisi, senza usarla.** Calcolare l'ultimo uso per slot su ogni
funzione compilata e *contare* quante letture sarebbero spostabili, senza
cambiare una sola emissione. Giudice: il contatore `would_take` confrontato
con `slot_reads_rc=53561241`. **Criterio di prosecuzione: se le letture
spostabili sono meno del 20% delle `slot_reads_rc`, la leva non vale la sua
complessità e si passa al piano B.** Questa fase è a rischio ZERO: nessun
bit del binario di parità cambia (feature di sola misura).

**F2 — Il perimetro conservativo.** Implementare i predicati di rinuncia
(l'elenco sopra) e ri-contare. La differenza fra F1 e F2 dice **quanto costa
la prudenza**: se il perimetro conservativo azzera il guadagno, lo sappiamo
prima di scrivere l'opcode.

**F3 — L'opcode.** `TakeSlot(s)`: sposta il valore e lascia `Undef`. Emesso
solo dove F2 lo consente. Gate di parità COMPLETI nello stesso commit
(corpus 1418 + refl 290 + ORM + hk + battery61), più i test scritti apposta
per le trappole: `$a .= $a`, distruttore che osserva l'ordine, generatore
sospeso, `compact()` dopo l'ultimo uso apparente.

**F4 — La misura.** Coppia oracle-vs-phpr della stessa sera, con l'oracle
rimisurato (mai il denominatore congelato — sanatoria WP-96), più coppia A/A
per lo spread inter-build.

## PREDIZIONE EX-ANTE (firmata prima di F1)

- **P1 (meccanismo, F1)**: `would_take` ≥ **20%** di `slot_reads_rc`. Sotto
  il 20% → si abbandona per il piano B, e lo si scrive.
- **P2 (costo della prudenza, F2)**: `would_take_safe` ≥ **60%** di
  `would_take`. Se il perimetro conservativo taglia più del 40%, la leva
  vale meno della sua complessità.
- **P3 (tempo, F4)**: user CPU **−2,0…−4,0%** sul gruppo media (banda più
  alta della superistruzione perché agisce su TUTTI i siti di lettura, non
  su una coppia di opcode). Falsificata se peggiora o se supera −8%.
- **P4 (parità)**: corpus per NOME invariato; `battery61` rc=0 con gli stessi
  sei esiti; media group 762/1912/52 identici.

## Quello che questa leva NON è

Non è un'ottimizzazione del `run_loop`: il tetto del dispatch resta il 4,33%
della CPU e non lo tocchiamo (consulenza Bak). Non è la superistruzione:
quella aggiunge un corpo caldo, questa **non aggiunge opcode al percorso
caldo**, ne cambia uno esistente.
