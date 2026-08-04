# Verbale sedia 2 — Niko Matsakis (Concilio WP-96)

Perimetro: ownership, aliasing, borrow checking, soundness. Mandato: refutare.

## VERDETTO

**CON EMENDAMENTI — l'obiettivo dichiarato di A4 NON è raggiunto.**

Ciò che A4 ha fatto è corretto: `IN_TRACE`/`THR_ID` sono `Cell` con
`const {}` init, quindi il thread_local **non ha distruttore e non alloca
in init pigra** — `try_with` non può fallire e, soprattutto, non c'è il
percorso di allocazione ricorsiva che una TLS lazy avrebbe introdotto.
`galloc_note`/`gfree_note` sono `fetch_add` puri: nessun lock, nessuna
alloc, nessun panic. `HUGE_TRACE` fail-closed a 0. Fin qui, sound.

**Ma la funzione riscritta per non panicare contiene ancora un `eprintln!`**
(main.rs:121), documentato dalla std come *«panics if writing to
io::stderr fails»*. Stderr chiuso, EPIPE, o disco pieno ⇒ panic che
**srotola fuori da `GlobalAlloc::alloc`**: esattamente l'UB-da-contratto
che A-TH-73/74 volevano chiudere. `thread::current()` è stato tolto e il
gemello è rimasto in piedi due righe sotto. Aggravante di secondo ordine:
il panic srotolante salta `f.set(false)`, e il flag di rientranza resta
`true` per sempre su quel thread — il canale si spegne **in silenzio**
(lezione WP-94: «un dente che smette di mordere non lo annuncia»).

## Emendamenti

- **A-MS-65** — `huge_note`: output panic-free (`io::stderr().write_all`
  con `let _ =`, o `libc::write`), MAI `eprintln!`. Nessuna macro che
  possa unwindare dentro un `GlobalAlloc`.
- **A-MS-66** — il flag di rientranza si rilascia con un **drop-guard**
  (`struct Reset<'a>(&'a Cell<bool>)`), non con un `set(false)` finale:
  correttezza per costruzione, non per raggiungibilità.
- **A-MS-67** — `huge_note` su `realloc` **asimmetrico**: si nota solo `n`.
  Un blocco huge che si contrae (40 MB → 1 KB) **non emette nulla**: la sua
  morte è invisibile. Questa è la stessa classe di errore che generò il
  «mai liberata» di WP-93. Notare `l.size()` come `dealloc` e `n` come
  `alloc`, sempre (chiude anche §WP-95 punto 2).
- **A-MS-68** — `thr=` è un seriale assegnato **all'ordine della prima
  huge**, non al worker: non è congiungibile per NOME. Registrazione
  esplicita dell'id all'entrata del worker (fuori dall'allocatore) o il
  campo si chiami `serial=`, mai `thr=`.
- **A-MS-69** (leva §WP-95) — nel commit della leva **vietata ogni nuova
  `unsafe`/`transmute`/`Box::leak`/`&'static` in `lower/mod.rs`**. Con arene
  per-file il borrow checker è il gate: se il codice compila, i nodi non
  sopravvivono all'arena. L'unico modo di sbagliare è **disattivare il
  gate** per tenere in vita l'AST — e quella è la leva WP-78 che divenne
  leak.
- **A-MS-70** (leva) — `low` è costruito su `&file` (il PRIMO unit) e poi
  riceve statement di `file_ns/bc/gmp/mysqli/gd/fi`: **gli span sono
  risolti contro la sorgente sbagliata**. Oggi è latente; spezzando le
  arene diventa attivo. Un `Lowerer` per unit, o prova esplicita che gli
  span non indicizzano mai `file`.

## Kill-switch

- **KS-MS-96-1** — se `grep` trova una macro panicante (`eprintln!`,
  `println!`, `unwrap`, `expect`, `panic!`, `assert*`) nel corpo
  raggiungibile da `MemCountingMi::{alloc,dealloc,alloc_zeroed,realloc}`:
  **build della campagna INVALIDA**, cifre VOID.
- **KS-MS-96-2** — se il digest di `(class_index, fn_index, static_count)`
  del preludio cambia dopo la leva: **revert della leva**. Le arene
  cambiano l'ownership, non l'ordine di hoist.

## Refutazioni

1. **CAPITALE**: «A4 ha tolto i percorsi che panicano dal `GlobalAlloc`» —
   **FALSO**, `eprintln!` panica su stderr in errore ed è nella stessa
   funzione (A-MS-65).
2. «Σ `allocated_bytes` per-arena è il numeratore della leva» — **falso**:
   N arene ripagano ciascuna il raddoppio iniziale, Σ può **crescere**
   mentre il **peak concorrente** cala. Il numeratore è il peak simultaneo;
   il controllo positivo `25795552` va riletto in quella metrica.
3. «`try_with` protegge da TLS distrutto» — vero ma **vacuo** qui: `Cell`
   senza Drop non ha distruttore. La vera protezione è il `const {}` init
   (nessuna alloc in init pigra): se un domani `THR_ID` diventa un
   `String`, la protezione sparisce senza che una riga la nomini.
4. Nota di verifica: `init_huge_trace()` è chiamata **dopo**
   `logging::init()`; «prima di ogni spawn» va provato, non asserito.
