# Verbale sedia 1 — Hoare (Concilio WP-95, perimetro: design linguaggio/runtime Rust, safe-only)

**VERDETTO: CON EMENDAMENTI**

## Q1 — huge_note nel GlobalAlloc: sana nell'insieme, due hazard latenti nominati

Struttura corretta: soglia prima di tutto (main.rs:77), guardia TLS `IN_TRACE`
const-init senza Drop (main.rs:72-74) — le alloc annidate di `force_capture`
sotto soglia escono al check size, quelle huge al check guard (main.rs:92);
`with` su TLS non-Drop non panica; la corsa a `mode==2` su AtomicU8
(main.rs:80-83) è idempotente-benigna.

**Hazard 1 (deadlock latente)**: `std::env::var_os` DENTRO l'alloc-path
(main.rs:82), lazy alla prima huge. Se la prima alloc ≥512KiB del processo
avviene mentre lo stesso thread tiene il lock env di std (putenv PHP →
`set_var` con valore ≥512KiB), il read-lock annidato blocca. Improbabile ma
gratuito da eliminare: leggere la env in `main()` prima dello spawn.

**Hazard 2 (UB da contratto)**: un panic che sfugge da `GlobalAlloc` è UB.
Unica via nominabile: `thread::current()` (main.rs:99) durante la distruzione
TLS — una dealloc huge di cache per-thread a fine thread col trace acceso può
cadere in quella fase; il comportamento su rustc 1.96 **non l'ho provato a
macchina**, lo dichiaro. Fix banale: niente `thread::current()` nel hook.

I contatori galloc/gfree contano anche le alloc del backtrace: accettabile,
il build è dichiarato ADVISORY (huge-sites.out:2).

## Q2 — PHPR_PRELUDE_STATS: commento REFUTATO per ARITMETICA

lower/mod.rs:1013-1015 afferma «allocated_bytes … è il touched fisico». Coi
numeri della sessione stessa: Σ primi cinque chunk = 19.300.272; se 39.534.144
fosse l'occupato, l'ultimo chunk conterrebbe 20.233.872 B su capacità
19.922.928 — **impossibile**. Quindi `allocated_bytes` ≈ Σ capacità chunk e
INCLUDE la coda mai toccata (13.738.592); il touched vero ≈ 25,8 MB.
huge-sites.out:80-83 lo legge correttamente come «capacità»; il commento nel
sorgente dice l'opposto. Il numeratore della leva (predizione-misurata WP-48)
va pinnato a ~25,8 MB, non 39,5. Non capitale: B1/B2 e il 4,42× CLI (misura
indipendente) restano in piedi.

## Q3 — leva per-file: cosa DEVE provare la sessione attuante

Fatti a macchina: il preludio è GIÀ multi-unit — 7 `File::ephemeral` TUTTI col
nome "prelude" (mod.rs:907,928,939,950,966,982,998) su UNA `Bump`
(mod.rs:906); dentro PRELUDE_SRC l'ordine è classi-di-TUTTO poi
funzioni-di-TUTTO (mod.rs:917-923; commento :915 «a prelude function may new a
prelude class»). Obblighi di prova (A-TH-76):
1. lo split di PRELUDE_SRC cambia l'interleaving in classi(f1),funzioni(f1),…:
   provare che `hoist_function` NON lega classi a hoist-time, oppure due
   passate senza tenere vivi tutti gli AST (= doppio parse, da misurare);
2. `get_declared_classes/functions` identiche per NOME e ordine;
3. numeri di riga: PRELUDE_SRC ha numerazione globale; lo split muove OGNI
   span (⭐⭐ WP-65: identità = SPAN) — trace, getStartLine/getFileName,
   `__LINE__`: gate refl 290 + corpus per NOME;
4. nome unit IDENTICO ("prelude", nessun suffisso) o salta la parità messaggi;
5. ogni frammento resta unità di compilazione con lo scoping odierno
   (declare/namespace — il file NS separato esiste per questo, mod.rs:924-928);
6. MAIN_CHAIN_FP: input ENUMERATI (mod.rs:856-865, A-DS7) — lo split muove la
   catena vergine; il falsifier (mod.rs:876) deve muoversi stesso-commit;
7. peak per-file previsto in BYTE d'arena PRIMA (il «~74KB reflection.php» è
   sorgente, non arena; rapporto misurato ~97× ⇒ ~7 MB da scrivere prima).

L'arm LEVER-2 (worker_pool.rs:522-523,631-634) è pulito: una branch per
richiesta, collect dopo la send. Non riproporre mi_collect: refutato con
misura (delta 0,12%).

## Q4 — priorità S-94.0 (FONDAMENTALI-first)

1) battery61 riproducibile modo nativo (criterio 5, mezza sessione, misura
sull'oggetto); 2) leva per-file con i 7 obblighi + gate completi + A-TH-75
(l'oggetto footprint più grande nominato); 3) attribuzione slope ~18,8
MB/worker al canale m91 con battery-91pre (criterio 1). Apparato: SOLO se
blocca (condizione 4, nessun gate nuovo).

## Emendamenti

- **A-TH-73** «env-read fuori dall'allocatore»: `HUGE_TRACE` inizializzato in
  `main()` prima dello spawn; nessun `var_os` nell'alloc-path (main.rs:82).
- **A-TH-74** «nessun panic-path nel GlobalAlloc»: rimuovere
  `thread::current()` da huge_note (main.rs:99); id numerico o niente.
- **A-TH-75** «semantica bumpalo pinnata a macchina»: unit test su
  allocated_bytes/chunk_capacity (bumpalo 3.20.3) + correzione commento
  mod.rs:1013-1015; numeratore leva = touched ≈25,8 MB.
- **A-TH-76** «leva per-file = i 7 obblighi di prova» (Q3), attuazione e gate
  nello stesso commit.

## Kill-switch

- **KS-TH-95-1**: campagna con PHPR_HUGE_TRACE/PHPR_PRELUDE_STATS attivi, o
  build strumentato usato per cifre footprint → run NULLO (KB-78-5).
- **KS-TH-95-2**: leva per-file — UNA divergenza per NOME (classe, funzione,
  riga) nei gate ⇒ leva RESPINTA; vietato adattare i gate.
- **KS-TH-95-3**: predizione WP-48 firmata col numeratore 39.534.144 senza il
  test A-TH-75 ⇒ predizione NULLA.

## Refutazioni capitali

**No.** Una refutazione aritmetica (commento/numeratore di Q2) e due hazard
latenti (Q1): correggibili, i verdetti B1/B2/B3 della sessione restano in
piedi.
