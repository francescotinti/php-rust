# S-152 p.2 — pesca outlier per NOME (mandato concilio, dentro A1) — lettura

Metodo: micro-script a DUE N col probe census s151 (ab02faec0abfab67),
k = Δconteggio/ΔN (le costanti di setup si elidono); CONTEGGI, mai tempo.
Esperimenti agli atti: `bt-count.php` / `ce-count.php` + `pesca-out/`.

## debug_backtrace — istruttoria «perché 21,3M con limit=2?» CHIUSA
- k = **45 alloc/chiamata ESATTE** (Δ 9.000.000/200.000; b=2.282 B/chiamata)
  per la forma Doctrine `debug_backtrace(IGNORE_ARGS, 2)` a stack ≥4.
- ⇒ ORM ≈ **473k chiamate reali** (21,3M/45). Il 21,3M NON è un residuo di
  BT1 (limit onorato: 2 frame): è il PREZZO in alloc della costruzione del
  risultato — ~20 alloc/frame: 5 chiavi array ricostruite per frame
  (`Key::from_bytes`), PhpStr nuovi per file/function/class/type
  (`bt.file.clone()` + `PhpStr::new`, `ty.to_vec()`), storage PhpArray +
  `Rc::new` per frame e per outer (host.rs `ho_debug_backtrace`).
- Zend: le zend_string dei nomi sono INTERNED (refcount, non alloc) ⇒ il
  divario per-chiamata resta (bilaterale m-backtrace 5,50× netto @ s150).
- Scala leva (regola s149 p.5): anche riducendo 45→15 alloc/chiamata,
  30×473k×~8–12 ns ≈ 0,11–0,17 s < risoluzione ORM 0,26–0,30 s ⇒ ZERO
  codice a scala suite; resta FETTA micro-judged su m-backtrace (in coda
  per NOME, con BT2-alloc: chiavi statiche interned + ZStr condivisi nel
  BtFrame + presize array).

## class_exists — anomalia SPIEGATA
- k = **2 alloc/chiamata ESATTE** (Δ 800.000/400.000 chiamate; 41,5 B/med) sui
  due rami (esiste/non esiste, autoload off).
- ⇒ ORM ≈ **4,85M chiamate** (9,7M/2): il volume è di CHIAMATE Doctrine, non
  grasso interno; le 2 alloc ≈ lowercase del nome (+ probe arg). Leva
  possibile lookup case-insensitive senza alloc: 9,7M×~7–9 ns ≈ 0,08 s ⇒
  sotto scala suite; micro-fetta in coda per NOME.

## Quadro (testa hostcall post-BT1 82,2M alloc/replica)
A prezzi pair s149 (6,9–11,7 ns/coppia 16–48 B) l'INTERA testa vale
~0,6–1,0 s su D_gap 30,5 s: nessun singolo nome può muovere l'ORM oltre la
risoluzione ⇒ la pesca-outlier ha esaurito i bersagli a scala suite
(coerente col mandato: BT1 era l'outlier ALGORITMICO; i residui sono prezzo
per-alloc, bilaterale o volume di chiamate). Restano da contare (in coda,
non bloccanti): get_declared_classes 4,6M (ipotesi: array intero delle
~2.393 classi ricostruito per chiamata ⇒ ~1–2k chiamate), array_map 7,7M,
__reflect_* ≈14M (famiglia sotto soglia 0,50× per FAMIGLIA già in s149).

## Istruttoria +3,2% (s148 325,4M ↔ s149 335,8M) — lato SORGENTE chiusa
Diff d59b2b5→5294ec1: la partizione per-NOME è tabella STATICA pre-allocata
(«il censimento stesso non alloca mai»), scope by-ref, siti di innesto 8
righe cfg-gated in run.rs, perimetro del TAG invariato ⇒ **l'auto-conteggio
del probe s149 è REFUTATO come meccanismo**. Lo scarto resta di RUN/ambiente
(path già refutato in S-150) e con la testa nuova post-BT1 (82,2M) il
confronto storico non è più decisionale: CHIUSO come non-decisivo,
meccanismo-sorgente escluso. Il Δ313 tra repliche → rerun quiet
(`s152-criterio-quiet.md`).
