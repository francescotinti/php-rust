# Team «ricevitore» — Fase 2 Concilio WP-103

**Relatore**: sintesi dei verbali sedie 1 (Hoare), 2 (Matsakis), 8 (Stogov).
**Perimetro**: famiglia MOVE-dell'handle (H-C1b spedita) e sue guardie; H-C1a
(split gc_note); bozza §S-102 punto 3. I verbali individuali restano VINCOLANTI;
questo file coordina, non sostituisce.

---

## 1. CONVERGENZE (tutte e tre le sedie)

### 1.1 H-C1b (MOVE) è sound; la promozione NON si riapre
- **Hoare**: verificato sul codice — handle owned dal `pop`, i sentieri
  magic/hook clonano `target` nel frame PRIMA del `continue`, l'ultimo drop non
  si sposta. Nessuna refutazione capitale.
- **Matsakis**: «la leva è sana: l'handle poppato è owned, il move non crea
  alias, il braccio Ref conserva `deref_clone`». La refutazione RC-MA-103-1
  colpisce l'ARGOMENTO del criterio, non la leva.
- **Stogov**: «le due leve H-C1a/b stanno in piedi sui loro A/B»; census
  conferma al 100% la sua predizione. La riserva sul gate «necessario ma non
  sufficiente» si sana retro-attivamente, non riapre la promozione.

### 1.2 Le fixture esistenti NON coprono i casi taglienti per un handle mosso
Convergenza piena sul PRINCIPIO (buchi per NOME, attese scritte PRIMA, 2 modi),
con liste complementari e non in conflitto:
- **Stogov**: fixture 14 destruct-reenter-PropSet (A-ST-103-1: `__destruct`
  del vecchio valore che rientra nel VM e rilegge/riscrive la stessa proprietà
  — semantica Zend `garbage`-poi-dtor, il dtor vede il valore NUOVO);
  fixture 15 typed-write-coercion (A-ST-103-2: coercion riuscita E fallita,
  aggancio §3.12); fixture 16 clone (A-ST-103-3: addref tabella, CoW array,
  `__clone` che tocca le proprietà — l'handle mosso non deve aliasare la
  tabella del clone).
- **Matsakis**: fixture 14 lazy-init sincrono che droppa l'ULTIMA ref esterna
  + `gc_collect_cycles()` + ordine `__destruct` (A-MA-103-3), più variante
  PropSet con soglia gc adattiva attraversata MID-ARM con old-value ciclico.
  Motivazione: 04/12/13 misurano finestre in cui i conteggi old/new sono
  IDENTICI; la finestra −1 è la re-entrancy SINCRONA intra-arm.
- **Hoare**: la fixture Generator (§2.3 sotto) nella stessa disciplina.
Nota di numerazione: Stogov e Matsakis chiamano entrambi «fixture 14» casi
diversi — il relatore propone la sequenza 14=destruct-reenter-PropSet (Stogov),
15=typed-write-coercion, 16=clone, 17=lazy-init-drop-ultima-ref (Matsakis),
18=PropSet-soglia-gc-mid-arm, 19=generator (Hoare, se non arriva evidenza
birth-track). I NOMI dei verbali restano la fonte, i numeri sono di comodo.

### 1.3 La guardia/predicato contenitore va resa non-decadibile
- **Hoare A-HO-103-1**: la lista Object|Ref|Array|Closure è triplicata
  (guardia mod.rs:3909, `gc_note_slow` con `_ => {}` a mod.rs:4011, scansione
  capture mod.rs:3999-4002); una variante Zval nuova compila in silenzio.
  Rimedio: `Zval::is_gc_container()` come match SENZA wildcard sui 14, usato
  dai tre siti.
- **Stogov A-ST-103-5**: la forma Zend-fedele è a DUE livelli —
  `Z_REFCOUNTED` (bit-test) + collectable (`GC_MAY_LEAK`): stringhe e
  resource sono refcounted ma NON collectable, mai nel root buffer; le Ref si
  scartocciano al valore interno. Verificare che phpr escluda
  stringhe/resource al secondo livello.
- Le due proposte sono COMPONIBILI, non alternative (vedi §2.1).
- **Matsakis** converge sul lato estensioni: KS-MA-103-2 vieta l'estensione
  del move a Silent/Dynamic/OpSet/IncDec senza guardia
  `matches!(obj, Zval::Ref(_))` e senza occorrenze contate per-forma.

### 1.4 S-102 punto 3 (ricevitore da slot) com'è scritto NON è ammissibile
- **Hoare A-HO-103-4**: non può essere un MOVE (lo slot conserva la
  proprietà, non c'è pop); le due forme possibili vanno NOMINATE nel
  criterio: clone-dallo-slot (3 clone → 1) o borrow col sigillo di tipo
  A-MA-102-3 — mai un borrow nudo.
- **Matsakis RC-MA-103-2** (capitale): il controfattuale «3 coppie × 2 ns» è
  DOPPIO-CONTATO — i 2 ns misurati da H-C1b sono la coppia clone+drop, che
  clone-from-slot NON elimina (risparmia solo il round-trip di pila, già nel
  ledger del punto 2); borrow è VIETATO (unanimità WP-102, e sul fallback la
  prova «nessun PHP nel borrow-window» fallisce per costruzione: lazy-init è
  sincrono); TAKE lascia slot vuoto visibile alla re-entrancy (A-ZV2 sospesa).
- Convergenza operativa: il punto 3 va RISCRITTO dichiarando il meccanismo
  (A-MA-103-4) e con controfattuale ring-fenced dal punto 2; se la forma
  scelta borrowa senza sigillo ⇒ reject (KS-HO-103-2 ≡ KS-MA-103-1).

### 1.5 Il criterio registrato di H-C1b va corretto (non la leva)
- **Matsakis RC-MA-103-1** (capitale): «strong_count non osservabile da PHP»
  è FALSO — il motore stesso osserva conteggi assoluti in ≥6 siti
  (oop.rs:1084 `==2+extra`; mod.rs:4126 e 4458 `==2`; mod.rs:4913-4918
  collector `−2 > in_edges`; mod.rs:5251 `==1`; run.rs:742 `==1`). Ciò che
  salva la leva è **INV-RECV-1**: in ogni punto dell'arm raggiungibile da PHP
  sincrono o da un observer di conteggio vive ≥1 handle owned del ricevitore.
  Emendamento A-MA-103-1: nominare INV-RECV-1 nel codice + tabella di audit
  dei ≥6 observer × raggiungibilità mid-arm + correzione della motivazione.
- **Hoare e Stogov** non contestano: nessun conflitto — l'audit INV-RECV-1 è
  anche la premessa sotto cui l'asimmetria clone/move del perimetro resta
  perf-only (Matsakis §2).

---

## 2. CONFLITTI (per NOME, posizione di ciascuna sedia)

### 2.1 Forma del predicato contenitore: unico esaustivo vs due livelli
- **Hoare (A-HO-103-1)**: UN predicato `is_gc_container`, match senza
  wildcard su tutte le 14 varianti, tre siti convergenti — l'obiettivo è
  l'ESAUSTIVITÀ a compile-time (variante nuova ⇒ errore di compilazione).
- **Stogov (A-ST-103-5)**: DUE livelli come Zend (refcounted ≠ collectable)
  — l'obiettivo è la FEDELTÀ + niente churn (stringhe/resource mai
  bufferizzate).
- **Lettura del relatore**: conflitto di FORMA, non di sostanza — sono
  ortogonali e componibili: `is_gc_container` esaustivo (senza wildcard) che
  al suo interno codifica i due livelli Zend (refcounted → collectable,
  Ref scartocciata al valore interno). Una sola implementazione soddisfa
  entrambi gli emendamenti; se le sedie non concordano sulla composizione,
  decide il Concilio in plenaria.

### 2.2 Severità del gating sull'apparato batteria/feature
- **Hoare (A-HO-103-3, KS-HO-103-3)**: il feature-check è monco (una feature
  su ≥2, check ≠ run) e si promuove solo dopo un MORSO dimostrato sulla
  matrice completa. KS-HO-103-1: NESSUNA leva S-102 su Zval/pila entra in
  misura prima di `is_gc_container` in tree e batteria verde.
- **Stogov (KS-ST-103-1)**: il blocco è sulle FIXTURE 14-16, non
  sull'apparato predicato; la promozione S-101 non si riapre.
- **Matsakis**: nessun gate sull'apparato feature; i suoi gate sono su
  borrow/estensioni/verdetti observer.
- Conflitto REALE ma di sequenza, risolto in §3: le guardie di Hoare e le
  fixture di Stogov/Matsakis bloccano entrambe le leve nuove; nessuna blocca
  il ri-baseline in sola misura.

### 2.3 Generator: fixture dovuta o evidenza alternativa
- **Hoare (A-HO-103-2)**: buco PRE-esistente (non imputa la leva) ma ora
  documentato come «same perimeter» — il commento traveste il buco da
  scelta. Alternativa secca: o evidenza per NOME che i generator sono notati
  da un ALTRO canale (birth-track), o fixture
  generator-tiene-l'ultimo-ref / generator-in-ciclo con ordine `__destruct`
  atteso PRIMA.
- **Matsakis e Stogov**: non trattano Generator. Nessuna opposizione: il
  relatore lo propone come voce ammessa (è un buco dell'OGGETTO — la rete
  gc attorno alla famiglia — non apparato), in coda alle fixture nominate
  dalle altre sedie, con la via d'uscita economica (evidenza birth-track)
  da tentare PRIMA della fixture.

### 2.4 H-C1c (specie per copia)
- **Stogov (A-ST-103-6, KS-ST-103-2)**: pinnare PRIMA le 5 specie (interned
  rc-free / string heap addref / array addref+CoW con separazione alla
  scrittura osservabile / IS_ARRAY_IMMUTABLE rc-free ed escluso da get_gc /
  object addref puro): un «copy+addref condizionale» uniforme sbaglia (a) e
  (d). H-C1c non si apre senza fixture per specie con attese oracle.
- Hoare e Matsakis non si esprimono su H-C1c. Nessun conflitto: adottare
  KS-ST-103-2 come gate della voce, SE e quando H-C1c viene iscritta.

---

## 3. PRIORITÀ PROPOSTE PER L'ORDINE S-102
*(regola di ammissione: apparato solo se blocca l'oggetto)*

1. **Audit INV-RECV-1** (A-MA-103-1): tabella dei ≥6 observer assoluti ×
   raggiungibilità mid-arm; correzione della motivazione registrata del
   criterio H-C1b; commento nel codice sui due siti. — È OGGETTO, non
   apparato: è l'argomento di correttezza della leva già spedita, e la
   refutazione capitale RC-MA-103-1 lo esige.
2. **Fixture per NOME della famiglia** (attese scritte PRIMA, verdi nei 2
   modi): destruct-reenter-PropSet (A-ST-103-1), lazy-init-drop-ultima-ref +
   PropSet-soglia-gc-mid-arm (A-MA-103-3), typed-write-coercion (A-ST-103-2),
   clone (A-ST-103-3). Gate comune: KS-ST-103-1 — nessuna NUOVA leva H-C sul
   percorso ricevitore/proprietà si promuove prima.
3. **`is_gc_container` esaustivo a due livelli** (A-HO-103-1 ⊕ A-ST-103-5,
   composizione §2.1) + debug_assert «nessuno Zval::Ref come target nei path
   prop» (A-MA-103-2). Ammesso perché KS-HO-103-1 lo rende BLOCCANTE per
   qualunque leva S-102 che tocca Zval o la pila operandi — blocca l'oggetto,
   quindi entra. Il dente feature-matrix (A-HO-103-3) si dichiara verde solo
   dopo morso provocato (KS-HO-103-3).
4. **Generator**: prima tentare l'evidenza per NOME del canale alternativo
   (birth-track); se assente, fixture generator (A-HO-103-2). Costo minimo
   prima, fixture solo se il buco è confermato.
5. **Riscrittura del punto 3 bozza S-102** (A-MA-103-4 + A-HO-103-4):
   meccanismo DICHIARATO; se clone-from-slot, controfattuale = SOLO traffico
   di pila, ring-fenced dal punto 2 (mai le stesse ns due volte); borrow solo
   col sigillo di tipo A-MA-102-3 e prova per-path (il fallback fallisce per
   costruzione: lazy-init è sincrono). Fino ad allora il punto 3 NON entra
   in misura.
6. **Fedeltà collaterale già istruita**: fix §3.13 riga-al-momento
   dell'accodamento (A-ST-103-4), nello STESSO commit che cancella il
   carve-out `09-unset-during-read.expected-divergence.diff` (KS-ST-103-3).
7. **H-C1c**: NON si apre in S-102 senza il pin delle 5 specie + fixture per
   specie con attese oracle (A-ST-103-6, KS-ST-103-2).

### Kill-switch cumulati del team (tutti attivi)
KS-HO-103-1/2/3 · KS-MA-103-1/2/3 · KS-ST-103-1/2/3. In particolare:
qualunque cambio che ribalti il verdetto di un sito exact-count
(`==1/==2/−2`) nelle fixture = reject senza appello (KS-MA-103-3); borrow di
slot senza sigillo di tipo = reject (KS-HO-103-2 ≡ KS-MA-103-1).
