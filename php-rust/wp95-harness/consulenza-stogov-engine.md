# Parere tecnico — Dmitry Stogov (consulente esterno) — WP-95

Ho letto `run_loop` (`vm/run.rs:466-525`), `gc_note` e `gc_note_frame`
(`vm/mod.rs:3880`, `:4495`). Cifre di Zend a memoria, da verificare: usatele
come ordini di grandezza, non come pin.

## 1. Dispatch: che cosa ha reso davvero in Zend

Ordine dei guadagni misurati in PHP 7: SWITCH → GOTO (threading) ≈ 5%;
GOTO → HYBRID (handler come funzioni vere + dispatch replicato in coda a
ciascuno) ≈ altri 5-8%. **Tutto il resto del 2× di PHP 7 non veniva dal
dispatch**: zval a 16 byte senza allocazione, hashtable packed, stringhe
internate con hash precalcolato, eliminazione del refcount sugli immutabili.

Perché HYBRID rende: ogni handler termina con il *proprio* sito di branch
indiretto, così il BTB impara la correlazione opcode→successore. Il vostro
`br` è **unico per tutti gli opcode**: è il caso peggiore per il predittore, e
i 6,66% sull'`ldrh` sono il carico dipendente sulla tabella degli offset.

In Rust safe il computed goto non c'è. Non tentate di emularlo: **riducete il
numero di dispatch invece di renderli più veloci**. È l'unica leva equivalente
disponibile, e Zend la usa comunque (IS_SMART_BRANCH: i comparison si fondono
col JMPZ/JMPNZ seguente). Corollario dalla vostra stessa lezione WP-33 («un
branch mai preso in testa al loop costa ~3%»): una catena `if` pre-match sui
top-8 opcode va misurata A/B, mai assunta.

Secondo punto, gratis: **241,7 KiB in una funzione sola**. Su un P-core Apple
la L1i è ~192 KiB: il vostro loop non ci sta. Gli arm freddi vanno estratti in
`#[inline(never)]` (avete già il pattern: `concat_n_join`, `concat_assign_slot`).
Non cambia semantica, non cambia un byte di output, e libera anche il register
allocator. È il lavoro a rapporto resa/rischio più alto del lotto.

## 2. Dove metterei lo sforzo per primo

**Zval clone+drop 8,2% — prima di tutto.** In Zend la specializzazione che
conta non è per *tipo* (ADD_LONG_LONG è roba del JIT) ma per **locazione
dell'operando**: CONST/TMP/VAR/CV, generata da `zend_vm_gen.php`. Un handler
che consuma un TMP lo *prende*; uno che legge un CV lo *guarda*. Voi clonate
perché l'HIR non dice mai «questo è l'ultimo uso».

Leva: una passata di **liveness/last-use** in compilazione che marchi ogni
operando `Take | Borrow | Copy`. Predizione da scrivere prima: metà dell'8,2%
più una fetta dei 3,0% dell'allocatore e parte del 5,2% di GC (meno temporanei
= meno note). La stessa passata vi dà il prerequisito della fusione del punto 1
(fondere solo quando il temp intermedio ha un unico consumatore): **un'analisi,
due leve**. Questo è l'ordine giusto: dati prima, dispatch dopo.

## 3. `gc_note_frame` (1,48%)

Zend non paga il GC per frame: `zend_free_compiled_variables` chiama il dtor
per CV, e `gc_possible_root` scatta **solo** quando un decremento lascia
refcount>0 su un tipo collezionabile — controllo O(1) inline su GC_TYPE_INFO,
**senza discesa** e **senza addref**. Voi fate due cose che Zend non fa: (a)
`gc_note` *discende* ricorsivamente in array/closure quando `strong_count==1`
— cioè una visita ad albero a ogni teardown; (b) `gc_buf.push(Rc::clone(rc))`
— traffico di refcount su ogni buffering (Zend bufferizza il puntatore nudo).

Leva a rischio zero: **bitmask per-funzione degli slot che possono contenere un
container**, calcolata in compilazione dall'HIR; `gc_note_frame` itera solo
quelli, e salta del tutto le funzioni tutte-scalari (in WordPress sono tante).
Più il fast-path a stack vuoto. Il punto (b) toccatelo dopo, e con cautela:
cambia la vita dei temporanei, quindi l'ordine dei distruttori.

## 4. `SipHasher::write` (1,14%)

Vale molto e costa poco. In Zend ogni `zend_string` porta `h` cachato; le
interned hanno l'hash a compile-time; il confronto è `h` → len → memcmp. Qui:
hash cachato in `PhpStr` (una `Cell<u64>`) + eliminazione degli ultimi
`HashMap` con hasher di default. Bersaglio aggregato: 1,14 + `KeyIndex::lookup`
1,51 + parte di `PropsLayout::slot_of` 1,18 ≈ **3,8%**.
⚠️ Trappola: cambiare hasher cambia l'ordine di iterazione. Se un solo
`HashMap` finisce in output (var_dump, get_object_vars, sort stabile) l'output
cambia. Il gate per NOME è obbligatorio qui.

## 5. Trappole di semantica

- **Eccezioni/backtrace**: fondere due op cambia `ip`. La exc_table, `getLine`,
  `getTrace` e la risoluzione di `finally` sono indicizzate sull'indice
  originale. Fondete solo entro un basic block e mantenete la mappa indici.
- **`set_error_handler`**: *qualunque* op che può emettere un diagnostico
  (undefined var/index, conversione non numerica, div-by-zero) rientra nella
  VM con PHP arbitrario. Nessun fast-path può assumere «niente accade fra
  queste due op» se la prima può avvisare.
- **Generatori**: `yield` esce da `run_loop` e la ripresa vuole `ip` esatto.
  Ogni stato issato in locali (`top`, `func`, ip cachato) va rimaterializzato
  dopo ogni chiamata che può rientrare o sospendere. È il bug numero uno.
- **`goto`/label**: mai fondere attraverso un target di salto.
- **Distruttori**: un drop dentro un arm esegue `__destruct`, che può far
  crescere `self.frames` e rientrare. Ogni ottimizzazione «evita la clone»
  sposta *quando* il temporaneo muore — cioè l'ordine dei distruttori, che è
  osservabile (avete già pagato questo prezzo in WP-23..28).

**Ordine consigliato**: (1) estrazione arm freddi, (2) liveness/Take-Borrow,
(3) bitmask GC per frame, (4) hash cachato, (5) fusione guidata dai bigrammi
dell'op-census. Il dispatch per ultimo, e solo come conteggio ridotto.
