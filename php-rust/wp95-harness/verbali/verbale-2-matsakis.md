# Verbale sedia 2 — Matsakis (ownership/aliasing/borrow, lifetime) — Concilio WP-95 su S-93.0

**VERDETTO: CON EMENDAMENTI** (una refutazione capitale sulla cifra della leva per-file).

## Q1 — La catena di proprietà è PROVATA, e a macchina

Sì, e con prova più forte del trace dinamico di S-93.0. La firma è
`fn lower_prelude_uncached() -> LoweredPrelude` (lower/mod.rs:904) con arena
`Bump::new()` variabile LOCALE (lower/mod.rs:905); `LoweredPrelude` è un alias
SENZA parametri di lifetime (lower/mod.rs:792-804: `Vec<Rc<ClassDecl>>`,
`HashMap<Vec<u8>,usize>`, `Vec<Rc<FnDecl>>`, `usize`). `ClassDecl` è
interamente owned (hir.rs:198-278: `Box<[u8]>`, `Vec`, indici `ClassId` — il
parent è un indice, non un puntatore: niente cicli Rc dal grafo dei decl).
In hir.rs gli unici riferimenti sono `&'static str` di costanti compile-time
(hir.rs:116, 174). In lower/mod.rs **zero `unsafe`, zero `Box::leak`,
`transmute`, `ManuallyDrop`** (verificato per pattern). Dunque il borrow
checker STESSO prova che nulla della tupla restituita borrows dall'arena: i
byte dell'AST sopravvivono per COPIA (in `Box<[u8]>`/`Vec` owned), mai per
leak. Se esistesse un leak intenzionale di byte interned vivrebbe come
`Box::leak`→`&'static` — non esiste su questo percorso; l'unico leak
dichiarato è a valle, in `UNIT_CACHE` (vm/mod.rs:15555-15560, thread_local,
«entries hold leaked modules»), estraneo alla `LoweredPrelude`. Le sei
dealloc del trace sono il Drop della `Bump` al ritorno: coerente e ora
ridondante rispetto alla prova per tipi. **Emendamento**: renderla un dente
(A-MS-64), perché oggi è «per firma», non un test che morde.

## Q2 — Il retained per-worker: cosa pesa la copia per-thread

`PRELUDE_CACHE` è `thread_local OnceCell<LoweredPrelude>`
(lower/mod.rs:806-811): UNA copia intera per worker, e il tipo la FORZA —
`Rc` è `!Send`/`!Sync`, quindi nessuna condivisione tra thread compila senza
decidere prima Rc→Arc (invasivo: `Rc<ClassDecl>` pervade la VM). Il peso VERO
della copia NON è misurato: 39.534.144 B è il touched dell'arena transiente,
non il live delle tabelle. Il canale m91 deve misurare il **delta live
on-thread attorno a `get_or_init`** (dopo il Drop dell'arena, che avviene
dentro la funzione), con `heap=<ptr>` in banda (≺ A-MS-53, lezione WP-92: Σ
per-thread senza heap moltiplica un heap solo), e decomporre lo slope 18,8
MB/worker per NOME in: (a) pagine committed-ma-libere dei sei chunk trattenute
dal theap (LEVER-2 refutata = niente decommit; touched stimabile
39,5−13,7 ≈ 25,8 MB), (b) live `PRELUDE_CACHE`, (c) live `UNIT_CACHE`/STUBS
(falsificati solo per i blocchi ≥512 KiB, NON per il retained sotto soglia).

## Q3 — Leva per-file: esprimibile SENZA unsafe

Sì. `hoist_classes(&mut self, stmts: &[Statement])` (lower/stmt.rs:538) ha il
lifetime degli statement INDIPENDENTE dal `'f` della `Lowerer<'f>`
(lower/mod.rs:1063-1064, che borrows solo la `File`), e l'output è owned:
la sequenza per unità `{ let arena = Bump::new(); parse; hoist; }` (Drop a
fine blocco) compila col borrow checker così com'è. Nessun ostacolo di
lifetime; l'ostacolo vero è semantico (ordine di hoist tra unità ⇒ gate
parità completi, come già dichiarato). Emendamento: `Bump::with_capacity`
per unità per uccidere la catena x2+16 e la coda morta (13.738.592 B).

## Q4 — Priorità S-94.0 (FONDAMENTALI-first)

1) A-MS-62 (contatore live-delta, un thread, mezz'ora) — nomina la metà
retained PRIMA di ogni leva; 2) leva per-file con predizione al denominatore
GIUSTO (vedi refutazione) + gate parità; 3) attribuzione slope su m91 con
heap=<ptr>; battery61 in parallelo come da criterio 5.

## Refutazioni capitali: SÌ (una)

**RC-MS-95-1**: la cifra della leva per-file in huge-sites.out:89-90 («peak
arena = max file ~74 KB reflection.php invece del cumulativo ~39.5 MB»)
CONFLA sorgente e arena: 406 KB di sorgente toccano 39,5 MB di arena (~97×);
74 KB di sorgente ⇒ picco atteso ~7 MB/thread (±2× di raddoppio), non 74 KB.
Riduzione ~5,5×, non ~500×. La predizione-misurata WP-48 esige il
denominatore corretto PRIMA dell'attuazione.

## Emendamenti

- **A-MS-62**: contatore live-delta on-thread attorno a `get_or_init` di
  PRELUDE_CACHE (peso della copia per-thread), in banda m91 con `heap=<ptr>`.
- **A-MS-63**: leva per-file con `Bump::with_capacity` dimensionata sul file
  (niente catena di raddoppio, niente coda morta).
- **A-MS-64**: dente `fn assert_static<T: 'static>()` su `LoweredPrelude`
  (+ assenza `unsafe` nel modulo): la prova per tipi diventa un test.

## Kill-switch

- **KS-MS-95-1**: vietato dichiarare «slope attribuito» senza heap=<ptr> in
  banda E senza separare committed-free da live per NOME (PRELUDE_CACHE /
  UNIT_CACHE / STUBS / residuo theap).
- **KS-MS-95-2**: vietata ogni attuazione della leva per-file che introduca
  `unsafe` o leak `'static` dell'arena: la sequenza parse→hoist→drop è
  esprimibile safe (Q3) — chi porta unsafe porta un errore di design.
- **KS-MS-95-3**: vietato proporre «condivisione del preludio tra worker»
  senza nominare il costo Rc→Arc: `Rc` è `!Send`, il tipo forza la copia
  per-thread; proposta di sharing senza questo costo = vacua.
