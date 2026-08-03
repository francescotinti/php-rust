# COUNCIL_WP95_REVIEWS.md — Concilio a 9 sedie su S-93.0 (direttiva utente A/B/C), protocollo a due fasi

Data: 2026-08-03. Verbali INTEGRALI (fase 1) = fonte VINCOLANTE; note di team (fase 2) e sintesi = compilate dalle ricevute + estrazioni mirate.
Indice: VERBALI INTEGRALI (fase 1) -> NOTE DI TEAM (fase 2) -> SINTESI DI CONVERGENZA (con §FONDAMENTALI in testa).

---

# VERBALI INTEGRALI (fase 1)

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


---

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


---

# Verbale sedia 3 — Klabnik (forge) — Concilio WP-95

## VERDETTO

**S-93.0 p.A è REFUTATO nel merito: A-SK-82 non chiude nulla.** Ho
fabbricato a HEAD, con la sola shell di sistema, **tre** vie che stampano
`PASS gate-measure-cifre --all … judge_sha=2f37f386d153d6ea` con rc=0 —
la firma del blob PRISTINO su codice che non è quello. Il gate cifre v3
+ A-SK-82 non è verdict-grade contro un avversario; è una porta con
tre finestre aperte. p.B/p.C restano in piedi (nessun forge li tocca).

## Forge eseguiti (comando · rc atteso · rc reale)

| # | canale | atteso | reale |
|---|---|---|---|
| F-K1 | `BASH_SOURCE=$SELF bash -c "$(cat $SELF)" $SELF <doc>` (testo pristino) | 1 REFUSE | **64 ADVISORY-PASS** — guardia passata |
| F-K2 | idem, testo patchato (`s/\$all_rc = 1;//` sul dente UNCOMMITTED), `--all`, doc-forge non committato in sessions/ | 1 | **0 + `PASS … --all` firmato**, zero righe A-SK-78/82 |
| F-K3 | symlink `L→dir` + `bash "$REPO/L/../wp81-harness/gate-measure-cifre.sh"` (testo patchato), advisory | 1/NOTE | **64, nessuna NOTE A-SK-78** |
| F-K4 | idem `--all` | 1 | **0 + `PASS … --all` firmato** |
| F-K5 | sourcing annidato (`. $SELF` da altro script) | 1 | 1 REFUSE — **gate tiene** |
| F-K6 | `exec -a <pristino> bash <patchato>` | 1 | bash riscrive `$0` col file: nessun guadagno — **respinto** |
| F-K7 | `BASH_ENV=evil bash $SELF --all`, `evil` definisce `perl(){ echo PASS…; }` — giudice pristino, path pristino | 1 | **0 + riga PASS fabbricata** |
| F-K8 | battery-equivalence: OUT con riga terminale `== BATTERY-88[0-9]PRE PASS …` + basename coerente | disciplina attempts attiva | **A-AH50 NON scatta** — esenzione pre-ledger comprata |
| F-K9 | idem con nome `88pre` coerente: consumo | 1 | 1 — lo **stamp a 5 campi committato** regge (l'identità A-AH68 no) |

Meccanica: `BASH_SOURCE` è iniettabile **dall'ambiente** sotto `bash -c`
(bash 3.2: diventa scalare esportato, `declare -x`, non array). `$HERE`
usa `cd`/`pwd` **logici**: `L/..` collassa lessicalmente sul giudice
pristino mentre il kernel legge il file patchato — `SELF_ABS` firma un
omonimo, esattamente il vizio che A-SK-78 doveva chiudere, un livello
più in là. `BASH_ENV` è il colpo di grazia: **nessun tether sul TESTO
può salvare un giudice il cui AMBIENTE è scelto dal chiamante**.

## Emendamenti

- **A-SK-88** — `declare -p BASH_SOURCE` deve iniziare con `declare -a`:
  uno scalare esportato è un'iniezione, REFUSE. (verificato)
- **A-SK-89** — `HERE`/`SELF_ABS` **fisici**: `cd -P … && pwd -P`; il
  forge F-K3/K4 muore lì (verificato: HERE fisico ≠ logico).
- **A-SK-90** — re-exec sanificante come primo atto: `exec env -u
  BASH_ENV -u ENV -u SHELLOPTS bash -p "$SELF_PHYS" "$@"` con marker
  anti-loop. `bash -p` neutralizza BASH_ENV **e** le funzioni esportate
  (verificato: HIJACK → REAL).
- **A-SK-91** — il tether va provato sul **comportamento**, non sul
  testo: dente che esegue il giudice con `perl`/`git` dirottati e pretende
  REFUSE.
- **A-AH-69** — esenzione pre-ledger ancorata: `battery-8[0-8]pre` (oggi
  `battery-8[0-8]*` regala l'esenzione a ogni batteria a tre cifre 8x0-8x8).
- **A-AH-70** — `--selftest-identity` estenda i casi al **consumo**
  (scope), non solo al predicato del nome.

## KS
- **KS-SK-95-1**: PASS verdict-grade prodotto per canale non sanificato ⇒ campagna VOID.
- **KS-SK-95-2**: path del giudice risolto logicamente ⇒ tether vacuo.
- **KS-SK-95-3**: dente che copre un canale e non la sua variante d'ambiente ⇒ dente sotto-portata.
- **KS-SK-95-4**: esenzione di scope da glob non ancorato ⇒ disciplina saltata in silenzio.

## Risposte

1. **T23 non è vacuo ma è sotto-portata da entrambi i lati**: arm-a prova
   il `-c` NUDO (la variante con env passa, F-K1); arm-b prova che il
   forge riproduce solo a rc=64 — non asserisce mai l'escalation a rc=0
   firmato, che è l'unica cosa che conta (F-K2/K4). Lo strip a marker
   fallisce rumorosamente se la guardia viene riscritta: quello va bene.
2. **Cifre di p.B**: B1/B2 (sei siti nominati dal backtrace, sei dealloc)
   sono **dimostrative**, non statistiche — consumabili come ADVISORY
   piene. B3 no: un run per braccio, due punti W, e il **rumore
   build-to-build dichiarato nello stesso .out è maggiore del delta
   rivendicato** — «delta nullo» va riscritto «indistinguibile dal
   rumore». Serve un grado **PROBE sotto ADVISORY** (rc=65, coerente con
   A-SK-79) per misure senza dispersione; il rapporto CLI phpr/oracle sta
   lì, magnitudine.
3. **S-94.0, FONDAMENTALI-first**: (a) **leva arene per-file del
   preludio** con gate parità completi — è l'oggetto, quantificato; (b)
   **battery61 riproducibile** (criterio 5); (c) apparato SOLO A-SK-88/89/90
   in un'ora, perché senza quelli ogni PASS futuro è firmabile da chiunque
   — poi congelato.

**Refutazioni capitali: SÌ (3).**


---

# Verbale sedia 4 — Hejlsberg (Concilio WP-95)

Perimetro: compilatori incrementali, interning/dedup, catene di evidenza e identità. Mandato: REFUTARE.

## VERDETTO
CON EMENDAMENTI. La leva per-file è la giusta prima mossa ma va emendata (pre-size, contatore per-unità PRIMA, vincolo d'ordine extends provato staticamente). L'identità dal contenuto (A-AH68) chiude il lato basename ma la catena stamp→attempts→OUT ha ancora TRE lati aperti nel `.done` e nell'autenticazione degli append — uno è capitale.

## Q1 — Leva per-file (nominata S-93.0 B3)
Fattibile e già semi-provata dai lifetimes: l'arena muore a fine `lower_prelude_uncached` (lower/mod.rs:904-1027) e i prodotti sopravvivono ⇒ nessun borrow evade, il Drop lo prova. I 6 unit separati (NS/BC/GMP/mysqli/gd/fileinfo) sono GIÀ hoist in chiamate distinte: splittare il concat di 9 file (mod.rs:747-757) è la stessa forma. DEVE PROVARE: (a) nessuna `extends`/`implements` in avanti TRA i 9 file — `parent` è risolto a `ClassId` AL lowering (hir.rs:207-210), quindi file k può riferire solo file ≤k: verificabile staticamente prima del codice; (b) contatore per-unità (A-AH-72) con predizione-misurata WP-48; (c) fp `main_chain_fp` INVARIATO (hasha i sorgenti, non l'arena — mod.rs:851-861): guardia d'identità gratis. Emenda tecnica: `Bump::with_capacity` per unit — la coda mai usata 13.738.592 B (huge-sites.out:80-83) è il costo della catena x2+16, evitabile senza cambiare altro.

## Q2 — Preludio PRE-COMPILATO (rkyv/bincode)
L'ostacolo `Rc<ClassDecl>` è più piccolo del temuto: dentro `ClassDecl` (hir.rs:198-278) e nei corpi NON ci sono `Rc` né `Cell` — gli `Rc` stanno solo alle tabelle (hir.rs:38, hir.rs:82; LoweredPrelude mod.rs:797-805), senza cicli né aliasing profondo: la dedup si ricostruisce al load (deserializza `Vec<ClassDecl>`, ri-avvolgi in `Rc`). Il vero costo: derive su TUTTA la foresta enum HIR + versioning. Legale SOLO come build-input embedded (`include_bytes!` da build-tool) dentro l'identità del binario — una cache su disco a runtime è una superficie di fabbricazione (KS-AH-95-2). DEVE PROVARE: dente che TIENE l'HIR plain-data (A-AH-73) e byte-determinismo dell'artefatto.

## Q3 — Preludio condiviso una volta per processo
Bloccato oggi da `Rc` !Send/!Sync a livello tabella: `Program.functions/classes` (hir.rs:38/82) e ogni consumatore. Serve Rc→Arc o `&'static ClassDecl` — ripple su tutta la VM, costo atomics sui path clone-caldi (unit cache WP-81). Aiuta SOLO il server (W−1 thread), zero sul CLI che è l'oggetto 4.42×. DEVE PROVARE PRIMA il numeratore: quota preludio-retained dei ~18,8 MB/worker (canale m91, A-MS-53) e l'immutabilità post-seed dei decl.

## Q4 — A-AH68: la catena regge?
Il lato basename è chiuso bene (bnc_judge unico predicato, battery-equivalence.sh:74-81, consumato da :214 e :83). Ma: (a) `DSHA` e `DMTX` sono estratti con `sed|head -1` da QUALUNQUE riga del `.done` (:229, :246) mentre `rev=$BREV` è cercato a parte (:224) — i campi possono nascere da DUE righe diverse: stessa classe del bug A-AH40; (b) il grep del triangolo A-AH54 (:374) è NON ancorato e la grammar ancora sha256 solo su FAIL/REFUSE/ABORT (:399) — le righe PASS sono esenti; (c) **capitale**: gli append in-window ai due ledger sono allowlisted (:430) e append-only (:450-470) ma MAI autenticati — `writer=script:<16hex>` è verificato solo in FORMA (:392), mai contro lo sha del battery script a HEAD: un consumo interamente fabbricato in-window (OUT+`.done`+stamp+riga PASS ben formate) non ha dente che lo morda.

## Q5 — Priorità S-94.0 (FONDAMENTALI-first: la leva È l'oggetto)
1) Leva per-file+pre-size col contatore per-unità stesso-commit, gate parità COMPLETI + battery-91pre alla prima ricompila; 2) misura CLI hello/refl post-leva vs oracle (il 4.42 deve muoversi); 3) battery61 nativo (criterio 5). Gli emendamenti al checker restano A VERBALE: apparato congelato (condizione 4), si attuano nella prossima finestra apparato.

## Emendamenti
- **A-AH-69**: `.done` parsato per-RIGA — i 4 campi dalla STESSA riga che porta `rev=$BREV`; più righe `rev=` ⇒ REFUSE (:229/:246).
- **A-AH-70**: ancora `sha256=[0-9a-f]{64}( |$)` anche sul grep triangolo (:374) e grammar-anchor esteso alle righe PASS (:399).
- **A-AH-71**: `writer=script:<h16>` deve eguagliare i primi 16 hex di sha256 dello script battery a HEAD — autenticazione, non forma (:392).
- **A-AH-72**: PHPR_PRELUDE_STATS v2 per-unità (`unit=<file> allocated=…`) PRIMA della leva.
- **A-AH-73**: test che `ClassDecl`/`FnDecl` restino plain-data (niente `Cell`/`Rc` interni) — precondizione della via precompilata.

## Kill-switch
- **KS-AH-95-1**: leva preludio attuata senza contatore per-unità pre-misurato ⇒ misura non consumabile.
- **KS-AH-95-2**: cache del preludio su DISCO a runtime ⇒ NEGATA; il precompilato è legale solo embedded nell'identità del binario.
- **KS-AH-95-3**: consumo battery con `.done` multi-riga a campi non accoppiati ⇒ VOID.

## Refutazioni capitali
**SÌ, una**: la catena stamp→attempts è autenticata in FORMA ma non in ORIGINE — append in-window allowlisted e mai legato allo script che li scrive (Q4c). I lati (a)/(b) sono aperti ma non capitali da soli.

## RANK leve
1. **Per-file + pre-size** (bassa superficie, il Drop già prova i lifetimes, serve CLI E worker).
2. **Pre-compilato embedded** (HIR è già serializzabile; solo se il residuo post-leva-1 resta ≥2× oracle — paga anche il parse CPU).
3. **Condiviso 'static/Arc** (solo server; prima il numeratore m91, poi la migrazione Rc→Arc misurata su full CPU).
4. **Lazy per-unità** (guadagno duplicato dalla 1; rompe l'invariante id-contigui mod.rs:793-796 e la parità reflection — superficie massima).


---

# Verbale sedia 5 — Bak (V8/HotSpot: alloc-rate, path caldi, startup/warmup) — Concilio WP-95

**VERDETTO: CON EMENDAMENTI.** B1/B2 (naming + natura transiente al livello malloc) reggono. La refutazione LEVER-2 come scritta NON è consumabile: verdetto NULLO su metrica cieca e senza pavimento di rumore. La dicotomia «riserva vs spike» è falsa al livello OS/provisioning.

## Q1 — Protocollo del probe slope: perché 21837 B/worker non è informativo
1. **Il pavimento di rumore è già nel file e supera l'effetto**: ctrl (build 8e966efd) slope 18814309 vs baseline build b048b697 slope 18852538 → 38229 B/worker di spread tra due run "identiche a meno di build" — MAGGIORE del delta lever 21837. Un delta sotto il pavimento osservato non refuta né conferma: è sotto-risoluzione. La varianza run-to-run non è mai stimata NEL probe (un run per W): chiamarla «refutata con misura» è troppo; è «non rilevata al di sotto della risoluzione».
2. **Metrica strutturalmente cieca**: `peak_footprint` è il massimo; `mi_collect` post-preludio agisce DOPO il picco (lo spike da 39,5MB è già avvenuto). Una leva post-picco non può, per costruzione, muovere il peak: il NULLO era atteso a priori. Serviva anche una metrica di residency post-warmup (checkpoint dopo l'ultima richiesta, purge già a 0).
3. **Due soli W**: slope da 2 punti assume linearità non testabile; N=25·W piccolo fa dominare il warmup nel peak.
**Per renderla consumabile**: R≥5 repliche per arm, INTERLEAVED (A,B,A,B…) sulla stessa binaria; W∈{1,2,4}; mediana±banda 2se pubblicata NEL probe; verdetto NULLO solo come test di EQUIVALENZA (|delta|+banda < soglia dichiarata ex-ante); doppia metrica peak+residency.

## Q2 — Alloc-rate e warmup per-worker (39,5MB churn alla prima richiesta)
Il preludio è `OnceCell` per-thread inizializzato NELLA prima richiesta: ogni worker paga parse+lower di ~406KB e 39,5MB di churn sul request path. Effetti: (a) le prime W richieste dopo ogni deploy/restart/autoscale portano una latenza da cold-start → il p99 di flotta è inquinato a ogni rollout, non «una volta sola»; (b) al restart il thundering herd sincronizza gli spike: picco transiente ~W×39,5MB che il provisioning DEVE coprire — uno spike sincrono su W thread è una riserva ai fini del capacity planning, anche se liberato; (c) freed ≠ decommitted: i raw m90 mostrano arena committed 151MB (W4) persistente anche a exit_collect_mi — al livello OS lo «spike» lascia residuo. Lezione V8: il warmup appartiene allo spawn (o al build), mai alla richiesta — spostare l'init all'avvio del worker toglie il p99-hit a costo ~zero, prima di ogni leva di memoria.

## Q3 — Rank V8-style
1. **Snapshot/precompilazione del preludio** (serializzare l'output del Lowerer a build-time, deserializzazione/mmap a startup): unica leva che elimina SIA i 39,5MB SIA la CPU di parse, per ogni thread E ogni processo CLI (4,42× → verso 1×). È l'endgame (V8: isolate da ~30ms a ~2ms), ma costo/gate massimi (versioning, parità completa).
2. **Condivisione cross-thread** (once-per-process invece di per-thread): spike ×1 invece di ×W e taglia la quota per-worker dello slope; bloccante probabile = `LoweredPrelude` non Send+Sync (Rc/Cell) — serve uno spike di fattibilità PRIMA di deciderne il costo.
3. **Arene per-file**: taglia il picco per-thread (una `Bump` oggi cumula le SETTE unità simultanee — visto nel corpo di `lower_prelude_uncached`), ma NON tocca la CPU di warmup né il ×W. Safe-only, giusta come prima mossa, non come destinazione.

## Q4 — Priorità S-94.0 (FONDAMENTALI-first)
1. battery61 riproducibile (criterio 5, invariato in cima). 2. Probe slope v2 (A-BB-74/75) — prerequisito per giudicare QUALSIASI leva futura. 3. Leva per-file arenas coi gate completi. 4. Cifra warmup: latenza prima-richiesta per worker (A-BB-76). 5. Spike fattibilità Send+Sync (A-BB-77).

## Emendamenti
- **A-BB-74**: protocollo slope v2 — R≥5 interleaved stessa binaria, W∈{1,2,4}, mediana+banda 2se NEL probe; NULLO solo come equivalenza con soglia ex-ante.
- **A-BB-75**: nessuna refutazione di leva post-picco su sola metrica peak — obbligatoria metrica residency post-warmup.
- **A-BB-76**: misurare la latenza della prima richiesta per worker (warmup p99) come cifra di canale prima di scegliere per-file/share/snapshot.
- **A-BB-77**: spike di fattibilità `LoweredPrelude: Send+Sync` (once-per-process) prima di investire nello snapshot.

## KS
- **KS-BB-95-1**: un peak non può refutare una leva che agisce dopo il picco — metrica cieca = refutazione vacua.
- **KS-BB-95-2**: uno spike sincrono su W thread è una riserva ai fini del provisioning; freed ≠ decommitted ≠ non-provisioned.
- **KS-BB-95-3**: il warmup appartiene allo spawn (o al build), mai alla richiesta.

## Refutazioni capitali
**SÌ, una**: la refutazione LEVER-2 «con misura» è inconsumabile come scritta — delta 21837 B/worker sotto il pavimento di rumore 38229 già presente nel file e mai dichiarato, su metrica (peak) strutturalmente insensibile alla leva. Va retrocessa a «non rilevata sotto risoluzione» finché il probe v2 non gira.


---

# Verbale sedia 6 — Pedersen (Concilio WP-95) — confine per-richiesta, lifecycle, igiene probe

**VERDETTO: PASS CON RISERVE — S-93.0 consumabile in ADVISORY; DUE refutazioni capitali sul lato B3/igiene.**
B1/B2 sono lavoro pulito: la coppia alloc/dealloc vista dal canale VIVO (huge-sites.out:13-18, 43-48) con falsificazione per NOME dei sospetti è il metodo giusto.

## Q1 — Igiene del probe
Le identità dei build strumentati sono dichiarate BENE (huge-sites.out:6-7: tre hash, parent 2859c81, feature mem-census) — non scambiabili per la release. **Ma il ripristino del pin è solo DICHIARATO** (huge-sites.out:8-9 «RIPRISTINATO d45b57843eeb1375», WP_SESSION_93.md:19): nessuna ricevuta in-band (shasum post-ripristino nel .out, né ri-run del gate lever-pins DOPO il probe). Il «PASS gate lever-pins v10» in NEXT_SESSION è pre-S-93.0. Porta 8199 e log grezzi del trace: smaltimento non dichiarato per NOME. **REFUTAZIONE CAPITALE n.1**: il criterio 3 («binari pinnati INVARIATI», GIUDIZIO_C_AXUM.md:62-63) poggia su una dichiarazione, non su una ricevuta.

## Q2 — Arm senza readback (A-DL41)
L'arm legge l'env (worker_pool.rs:521-523) e scatta in silenzio (worker_pool.rs:630-633): **nessuna eco in banda**. Un delta NULLO da arm muto è indistinguibile da un arm mai scattato (var mal digitata, feature spenta, worker senza richieste); su w=4 il conteggio atteso fired==4 è inverificabile. **REFUTAZIONE CAPITALE n.2**: «LEVER-2 REFUTATA CON MISURA» (WP_SESSION_93.md:31) è insostenibile come scritta dal SOLO probe. La conclusione sopravvive in ADVISORY sulla gamba m90 (arena committed invariata a exit_collect_mi, huge-sites.out:72-74) — gamba indipendente e già committata — ma non è promuovibile senza eco.

## Q3 — Punto di chiamata post-send
Nessun confine di richiesta violato (risposta già inviata, zero latenza aggiunta). **Però** il collect gira DOPO il dec Release di OUTSTANDING (worker_pool.rs:619-620) — è lavoro heap-mutante del worker FUORI dalla finestra del testimone: un censimento preso a outstanding==0 può sovrapporsi al collect in corso. Stessa classe del morso WP-94 «la coppia non prova la finestra». Fix a costo zero: ordine send → collect → dec (worker_pool.rs:607, 630-633 prima di 617-627).

## Q4 — huge_note e disciplina del canale census
Guardia di rientranza IN_TRACE corretta (main.rs:71-73, 100); l'early-return su size<HUGE_MIN (main.rs:76-78) protegge dalla ricorsione dell'env-read. Due vizi: (a) **realloc asimmetrico** (main.rs:120-124): nota `realloc` sul size nuovo ma NESSUN `dealloc` del size vecchio huge ⇒ il saldo alloc−dealloc del canale huge deriva sui path realloc (i sei chunk bumpalo passano da alloc/dealloc, quindi B1/B2 non ne soffrono — ma il canale resta storto); (b) le allocazioni del trace stesso (backtrace, format) passano dal GlobalAlloc e vengono CONTATE da galloc_note/gfree_note: con PHPR_HUGE_TRACE=1 i contatori census sono contaminati e vanno dichiarati VOID.

## Q5 — Priorità S-94.0 (FONDAMENTALI-first)
1. **battery61 riproducibile, modo nativo** (criterio 5, mezza sessione) — in cima, debito di 31 sessioni.
2. **Campagna m91 con battery-91pre** (MAI girata): certifica ANCHE le modifiche sorgente env-gated dormienti di S-93.0 — nessuna cifra nuova prima della battery.
3. **Attribuzione slope ~18,8 MB/worker per NOME** (criterio 1, canale m91) — con A-PP-75 attuato prima di fidarsi del testimone.
4. Leva per-file del preludio: DOPO, sessione dedicata, gate parità COMPLETI. Apparato solo se blocca (timebox).

## Emendamenti
- **A-PP-74**: ogni arm env-gated emette al fire una riga ascii-nuda `arm=<nome> fired=<n> thr=<id>`; raw senza fired==W ⇒ run VOID (legge A-DL41).
- **A-PP-75**: lavoro post-risposta DENTRO la finestra del testimone (send → lavoro → dec OUTSTANDING), o dichiarato fuori-testimone in banda.
- **A-PP-76**: huge_note simmetrico su realloc — emettere anche `dealloc` del layout vecchio (main.rs:120-124).
- **A-PP-77**: dopo OGNI probe con build strumentata: ricevuta di ripristino (ri-run gate lever-pins o shasum in-band nel .out) + smaltimento porta/log dichiarato per NOME.
- **A-PP-78**: PHPR_HUGE_TRACE=1 ⇒ contatori galloc/gfree del run dichiarati VOID (auto-contaminazione del trace).

## Kill-switch
- **KS-PP-95-1**: nessun esito di leva da arm senza eco in banda è promuovibile oltre ADVISORY.
- **KS-PP-95-2**: lavoro del worker schedulato dopo il dec di OUTSTANDING è fuori testimone — censimenti a outstanding==0 con tale lavoro pendente sono VOID.
- **KS-PP-95-3**: pin «ripristinato» senza ricevuta in-band = pin DICHIARATO — ogni claim di parità resta ADVISORY finché il gate non rimorde.

**Refutazioni capitali: SÌ (2)** — ripristino pin non provato; refutazione LEVER-2 muta come scritta (salva in ADVISORY via m90).


---

# Verbale sedia 7 — Leijen (allocatore mimalloc v3, footprint fisico) — Concilio WP-95

**VERDETTO**: la conclusione S-93.0 «la stat è di fatto cumulativa» è CONFERMATA, e ora ha il
meccanismo per NOME al livello di preprocessore. Ma DUE frasi agli atti sono refutate: la
nota-canale di huge-worker.out e l'inferenza «committed piatta ⇒ mi_collect non decommitta».

## Q1 — Perché il decremento non morde

Nessuna delle ipotesi elencate (theap NULL, pagina abandoned, theap diverso). Il call-site È
raggiunto: il free della huge singleton passa da `mi_free_ex` (free.c:186-204) →
`mi_free_block_local` con `track_stats=true` → chiamata a `mi_stat_free` (free.c:34). È il
CHIAMATO a essere vuoto: `mi_stat_free` è sotto `#if (MI_STAT>0)` (free.c:612); lo stub
release è free.c:635-637. E MI_STAT è 0 in OGNI build census: build.rs di libmimalloc-sys
0.1.49 definisce sempre `MI_DEBUG=0` senza feature `debug` (build.rs:108-115) ⇒ types.h:81-87
⇒ `MI_STAT 0` (types.h:85). L'INCREASE invece non è gated: `mi_theap_stat_increase(theap,
malloc_huge, …)` + `malloc_huge_count` girano incondizionati alla nascita della pagina
(page.c:935-936; macro internal.h:392 senza guardia). Asimmetria per costruzione: in release
malloc_huge conta le NASCITE di pagine huge — cumulativo, `current==total==peak` per sempre.
Asimmetria latente n.2 (per quando MI_STAT>0): il decrease va su `_mi_theap_default()` del
thread che libera (free.c:615), non sul theap che incrementò — un Σ per-theap può andare
negativo su free cross-thread.

## Q2 — Perché arena committed non scende

Semantica del contatore, non retention provata. In release `_mi_prim_decommit` su macOS fa
`madvise(MADV_FREE_REUSABLE)` ma pone `*needs_recommit=false` (prim.c:495-504, gate
`!MI_DEBUG && MI_SECURE<=2` a prim.c:503); `mi_os_decommit_ex` decrementa `committed` SOLO se
`needs_recommit` (os.c:590-591). Quindi committed non scende MAI su purge in release, per
disegno (la memoria resta riusabile senza recommit; il fisico torna all'OS via REUSABLE, con
accounting rss immediato — commento prim.c:496). Implicazione per lo slope 18814309 B/worker:
«committed piatta» NON lo attribuisce, e i sei huge (liberati e purgabili) NON possono
comporlo salvo prova fisica contraria; l'attribuzione è legittima solo sul canale fisico
on-thread (A-DL-55). La refutazione di LEVER-2 resta valida, ma sul SOLO braccio A/B fisico
(delta 0,12%), non sulla stat.

## Q3 — Strumento in-band per m91

(a) build census con `MI_STAT=1` esplicito (via CFLAGS del cc-crate; identità dichiarata nel
banner del dump insieme al livello MI_STAT); (b) coppia alloc/free IN-BAND nel GlobalAlloc
Rust (soglia ≥524288, chiave thread/theap) — è il canale che ha deciso S-93.0 in un colpo ed
è immune al gating C; (c) mai consumare una stat MI_STAT-gated senza dichiararne il livello.

## Q4 — Leva per-file, vista allocatore

Via libera: chunk ≤512 KiB evitano il path huge singleton (page.c:916-939) e il waste
os-good-size (39911424−39423200=488224 B sui sei chunk); le taglie normali finiscono nelle
page-queue e si riusano tra file. Frammentazione: rischio trascurabile sulle slice da 64 KiB;
il guadagno vero è eliminare lo spike touched 39,5 MB/thread e il 4,42× CLI.

## Q5 — Priorità S-94.0

1. Sanare il canale stat (A-DL-65/66/67) PRIMA della campagna m91; 2. leva per-file del
preludio con gate parità completi; 3. attribuzione slope solo su probe fisico on-thread.

## Emendamenti

- **A-DL-65**: census m91 = MI_STAT=1 dichiarato + coppia in-band Rust ≥524288 B; ogni cifra
  da stat gated senza livello MI_STAT nel banner è invalida.
- **A-DL-66**: vietato inferire retention/decommit da `committed` (os.c:591 ⊣ prim.c:504);
  retention si afferma solo dal probe fisico on-thread.
- **A-DL-67**: emendare huge-worker.out (nota-canale righe 6-8) e NEXT_SESSION §B con marcatura
  REFUTATA + puntatori file:riga.
- **A-DL-68**: leva per-file approvata dal perimetro allocatore (nessuna controindicazione di
  frammentazione; elimina il path huge).

## Kill-switch

- **KS-DL-95-1**: numero MI_STAT-gated in un verdetto senza livello MI_STAT dichiarato ⇒ il
  verdetto decade ad ADVISORY.
- **KS-DL-95-2**: divergenza coppia in-band vs stat mimalloc nel census ⇒ fede alla coppia
  in-band, stat solo testimone.

## Refutazioni capitali: SÌ (due)

1. huge-worker.out righe 6-8 («il decremento ESISTE … non è un contatore monco: è assenza di
   free») — FALSA: il decremento di free.c:631 è compilato VIA a MI_STAT=0, e il decremento di
   theap.c:362 vive nel blocco commentato `/*` theap.c:338 … `*/` theap.c:448 (dead code
   anche in debug). Il contatore È monco per costruzione in release.
2. «arena committed invariata a exit_collect_mi ⇒ mi_collect non decommitta quelle pagine» —
   NON SEGUE (prim.c:504 + os.c:590-591): committed non scende mai su purge in release;
   LEVER-2 resta refutata solo dalla misura fisica A/B.


---

# Verbale sedia 8 — Stogov (Zend/opcache, semantica engine) — Concilio WP-95

**VERDETTO: CON EMENDAMENTI.** La leva per-file è legittima e ha un omologo
Zend; ma il rischio nominato dal mandato (l'ordine globale di hoist) NON è il
rischio vero, e la cifra proiettata in huge-sites.out è sbagliata di due
ordini di grandezza.

## Q1 — Osservabili dipendenti dall'ordine di hoist

**Refuto il pericolo come inquadrato.** Con per-file l'ordine RELATIVO dentro
ciascuna tabella non cambia: classi f1,f2,… e funzioni f1,f2,… restano in
ordine di file (oggi il loop funzioni itera il concat nello stesso ordine).
`get_declared_classes()`/`get_defined_functions()` e i class-id sono stabili
PER COSTRUZIONE; cambia solo l'interleaving fra tabelle e gli slot statics
(non osservabili PHP). Verificato con audit vivo (oracle, token_get_all sui 9
file): **zero riferimenti extends/implements cross-file in avanti** (core 52
decl, spl 28, reflection 25, …, tidy 0); zero trait; enum solo in core.php.
Il rischio VERO è la sentinella `b"prelude"` (vedi Q3). Oracle verificato:
classi builtin → `getFileName`/`getStartLine`===false, `isInternal`===true;
ordine `get_declared_classes` = ordine di registrazione engine.

Fixture oracle-morse PRIMA del codice (giudice = baseline phpr d5ce86e3 per
F1-F3/F8; oracle vivo per F4/F5/F7):
- **F1-F3**: liste INTERE `get_declared_classes/interfaces/functions` + count, byte-id pre/post.
- **F4**: una classe campione PER file (Exception, ArrayObject, ReflectionClass, DateTime, PDO, SQLite3, DOMDocument, SessionHandler + funzione tidy): getFileName===false, getStartLine===false, isInternal===true.
- **F5**: eccezione lanciata DA codice preludio: `getFile()/getLine()/getTraceAsString()` — nessun `prelude:<riga>` affiora (oracle: call-site utente).
- **F6**: autoloader-logger: zero invocazioni per nomi del preludio.
- **F7**: "Cannot redeclare" su classe preludio: messaggio senza file/riga (vm/mod.rs:6598).
- **F8** (unit Rust): snapshot name→id di class_index/fn_index invariato; falsifier A-AH22 esteso: mutare UNO QUALSIASI dei 9 file muove main_chain_fp.

## Q2 — Precedente Zend

Zend NON ri-parsa mai gli stub per thread: classi interne registrate a MINIT
una volta per processo; opcache preload = classi `ZEND_ACC_IMMUTABLE` in SHM,
mutabile (statics, runtime cache) per-request via MAP_PTR. **Rank**: per-file
= palliativo del solo picco transiente, giusto PRIMO passo (safe-only);
l'omologo Zend è (b) tabelle immutabili condivise per-processo con statics
per-thread — attacca anche lo slope ~18,8 MB/worker (il clone per-thread di
LoweredPrelude); oltre, (c) preludio precompilato a build-time (≈ file cache)
elimina anche il parse CPU. Il fronte NON si chiude su per-file.

## Q3 — Numeri di riga / File ephemeral

La sentinella è UGUAGLIANZA AL BYTE `f == b"prelude"` in ~20 siti portanti:
host_reflect.rs:1511 (getFileName→false), vm/mod.rs:853 (**take_while sul
PREFISSO** delle funzioni condivise), :13221 (soppressione frame backtrace),
:16793 (relocation), :6598/12145 (redeclare), calls.rs:1324 (TypeError),
host.rs:3014 (internal in get_defined_functions), hir.rs:203, bytecode.rs:1769.
Le 6 unità extra usano GIÀ `b"prelude"` con righe che ripartono da 1: il
per-file non introduce una classe nuova di osservabile, PURCHÉ ogni unità
mantenga il nome esatto. Le righe non affiorano da Reflection (sentinella →
false); affiorano solo via F5. Gate: refl 290 + corpus 1418 per NOME + F4/F5.

## Q4 — Priorità S-94.0 (FONDAMENTALI-first)

1. Fixture F1-F8 + checker cross-file committati PRIMA del codice; 2. leva
per-file col contatore per-file (predizione-misurata WP-48); 3. gate parità
completi + battery-91pre (MAI girata — la ricompila la fa scattare) stessa
sessione; 4. misura hello CLI pre/post + slope W{1,4}. battery61 resta debito
separato (mezza sessione), non caricarlo sulla leva.

## Emendamenti

- **A-DS-66**: nome unità `b"prelude"` IDENTICO AL BYTE per ognuno dei 9 file + unit test che lo asserisce; in alternativa predicato unico migrato in TUTTI i ~20 siti — mai a metà.
- **A-DS-67**: fixture F1-F8 oracle-morse committate prima del codice (stile A-DS51).
- **A-DS-68**: checker cross-file forward-ref come gate pre-nascita; scope dichiarato (extends/implements; NON copre const-expr cross-file né attributi — estendere se l'hoist li risolve eager).
- **A-DS-69**: ri-quantificare il picco per-file col contatore PRIMA del claim: "peak arena = max file ~74 KB" confonde sorgente con arena (rapporto misurato ~97×, 39.534.144 B / ~406 KB): attesa ~5-8 MB, non 74 KB. Variante da misurare: una arena + `reset()` fra i file (bumpalo trattiene il chunk maggiore, niente ri-raddoppio).
- **A-DS-70**: rank leve a verbale: per-file → condiviso per-processo → precompilato build-time; nessuna chiusura di fronte su per-file (legge no-front-closure).

## KS

- **KS-DS-95-1**: unità rinominata (≠ `b"prelude"`) senza migrazione dei siti sentinella → STOP.
- **KS-DS-95-2**: codice della leva scritto prima delle fixture F1-F8 → STOP.
- **KS-DS-95-3**: claim di risparmio pubblicato senza contatore per-file predizione-vs-misurato → solo ADVISORY.

## Refutazioni capitali: SÌ

1. La cifra proiettata in `wp93-harness/huge-sites.out` ("peak arena = max
   file ~74 KB") scambia byte di SORGENTE per byte di ARENA: fattore ~97×.
2. Il rischio "cambia l'ordine globale" è REFUTATO come pericolo primario:
   l'ordine per-tabella è invariante per costruzione e l'audit cross-file è
   PASS; il pericolo reale è la sentinella `b"prelude"`.


---

# Verbale sedia 9 — Gregg (metodologia di misura + MANDATO INVERSO) — Concilio WP-95

**VERDETTO: APPROVATA CON EMENDAMENTI** — la migliore sessione d'oggetto da WP-90: due refutazioni vere sull'oggetto (malloc_huge cumulativo; LEVER-2 nulla) e un canale nuovo quantificato (preludio 39,5 MB). Ma una cifra è sopravvalutata di grado e una supersessione formale manca.

## §BILANCIO D'OGGETTO (cosa sappiamo OGGI che ieri non sapevamo)
1. I sei huge per worker hanno UN nome: sei chunk in raddoppio (x2+16, firma bumpalo) di UNA arena — il parse del PRELUDIO stdlib — e sono TUTTI liberati dentro la prima richiesta. Ieri erano «riserva fissa mai liberata»; oggi sono uno spike transiente.
2. malloc_huge nei dump m90 è un CUMULATIVO travestito da retained (current==total==peak su 20/20 raw, anche a exit_collect_mi). Il canale stat huge di m90 NON è consumabile come retained.
3. LEVER-2 (mi_collect post-preludio) non muove lo slope (delta 21837 B/worker, 0,12%), coerente con arena committed invariata nei raw m90.
4. Il preludio costa per-PROCESSO: arena 39534144 B (coda inutilizzata 13738592), e su hello il CLI pinnato sta a 44630520 vs oracle 10093048 = **4,42×**.
5. Corollario negativo: lo slope ~18,8 MB/worker resta SENZA nome — il maggior candidato nominato (39,9 MB huge) è stato esonerato.

**Il 4,42× ridisegna le priorità?** Sì, come candidato n.1, ma con cautela: GAP_TREND dice «riferimento resta WP-85» da 7 sessioni — la media ~3,0-3,1 non ha un «prima» fresco. Il delta hello (44,6−10,1 ≈ 34,5 MB) è dell'ordine dell'arena (39,5 MB): la leva per-file (peak ~74 KB) porterebbe hello-class verso ~1,1-1,5×, e poiché il corpus è dominato da test hello-sized la media può scendere sotto 3 in un colpo. Ma la priorità si DICHIARA solo dopo la coppia full stessa-sera: senza baseline aggiornata la predizione-misurata (WP-48) non ha giudice.

## Q1 — Il probe slope è una «misura»? NO, è uno SCREEN.
1 run per W, 2 punti W, zero varianza. La risoluzione dichiarabile è lo spread tra le due build strumentate: ctrl 18814309 vs baseline b048b697 18852538 = **38229 B > delta 21837 B**. Il verdetto legale è «effetto < ~0,2% alla risoluzione del probe», non «delta = 21837». Consumabile SOLO in coppia con la coerenza m90 (arena committed invariata). Cifre NON consumabili: slope 18814309 (build mem-census, W∈{1,4}, 1 run — mai nel ledger come slope); delta 21837 come pin. Il 4,42× = MAGNITUDINE (1 run per lato), non pin.

## Q2 — Supersessione huge-worker.out: SÌ, formale.
La conclusione «non viene MAI liberata … riserva FISSA» è refutata; anche la nota-canale (righe 6-8: «current==total NON è un contatore monco: è assenza di free») è refutata — il decremento esiste nel codice ma non morde su questo canale. Atto: UNA riga SUPERSEDED-IN-PART in testa al file (stesso commit della riga ledger, per NOME delle due clausole, puntatore a huge-sites.out). Il corpo NON si riscrive: è evidenza storica. I numeri per-worker (39911424, 6 blocchi) restano validi come CUMULATIVO.

## Q3 — Rischio d'oggetto più trascurato ORA
Ogni derivata di m90 che ha consumato malloc_huge come retained (repair90-estimators, VCOV 0,778, decomposizione b) è ora SOSPETTA e nessuno l'ha censita. La coverage marginale e l'«invisibile» 4,48 MB/worker vanno ricalcolati sapendo che 39,9 MB del censito erano transienti.

## Q4 — Ordine S-94.0 (FONDAMENTALI-first)
1. Supersessione huge-worker.out + audit derivate m90 contaminate (carta, breve).
2. **Coppia full/media stessa-sera** (contatore fermo a WP-85: baseline PRIMA della leva).
3. Campagna m91 (probe on-thread, heap=<ptr>): attribuzione slope 18,8 MB per NOME.
4. Leva arene per-file con predizione scritta (hello 44,6→~11 MB) + gate parità completi.
5. battery61 nativo (criterio 5).

## Emendamenti
- **A-BG-71**: riga SUPERSEDED-IN-PART in testa a huge-worker.out, due clausole per NOME, stesso commit della riga ledger; corpo intatto.
- **A-BG-72**: malloc_huge m90 declassato a CUMULATIVO ovunque; audit per NOME delle derivate che l'hanno trattato come retained, esito a ledger.
- **A-BG-73**: probe slope = grado ADVISORY-SCREEN; risoluzione = spread inter-build (38229 B); verdetto legale «effetto < 0,2%», pin del delta vietato.
- **A-BG-74**: 4,42× e 39534144 = MAGNITUDINE legata a identità build; pin solo dopo R≥3.
- **A-BG-75**: ⏱ FONDAMENTALI aggiornato (S-93.0 = prime misure nuove); obbligo coppia full in S-94.0 PRIMA della leva per-file.

## KS
- **KS-BG-95-1**: la risoluzione di un probe è lo spread tra le sue stesse baseline — se lo spread supera il delta, il probe è uno screen, non una misura.
- **KS-BG-95-2**: un rapporto su UN workload (4,42× su hello) è una magnitudine del canale, non una priorità di roadmap, finché la media non è rimisurata.

## Refutazioni capitali: SÌ
(a) huge-worker.out: conclusione «mai liberata» E nota-canale «assenza di free» refutate — supersessione obbligatoria. (b) La qualifica «REFUTATA CON MISURA» di LEVER-2 è refutata nel GRADO: è refutazione da screen+coerenza, non da misura.


---

# NOTE DI TEAM (fase 2)

# team-cifre — fase 2, Concilio WP-95 (relatore)

Sedie del team: Klabnik (3), Hejlsberg (4), Hoare (1, parte identità/gate).
Mandato: riconciliare o registrare i dissensi. Nessuna benedizione.

---

## 1. Convergenze

**C1 — La classe comune delle refutazioni di identità.** Le tre vie di
Klabnik (F-K1/K2 `BASH_SOURCE` da env; F-K3/K4 symlink + risoluzione
LOGICA; F-K7 `BASH_ENV`) e il capitale di Hejlsberg (Q4c, append
`writer=script:<h16>` verificato in FORMA) sono lo **stesso vizio**:

> **il giudice autentica STRINGHE che il chiamante sceglie, mai gli
> ARTEFATTI che il kernel legge ed esegue.**

Declinato:
- `$0` è un **nome fornito dal chiamante**, non il file aperto (F-K3/K4:
  `HERE="$(cd "$(dirname "$0")" && pwd)"` collassa `L/..` lessicalmente);
- `BASH_SOURCE[0]` sotto bash 3.2 è **dato d'ambiente**, non il registro
  di bash (verificato: `declare -x BASH_SOURCE="…"`, scalare esportato);
- l'**ambiente** che dà significato ai comandi del testo non è legato da
  nulla (F-K7: `perl` diventa una funzione del chiamante);
- lato ledger, `writer=script:<h16>` è una **forma di stringa**, non
  un'origine (A-AH-71).

Corollario condiviso: A-SK-78 → A-SK-82 hanno inseguito il vizio di un
livello alla volta (prima il nome, poi il registro del nome) senza mai
cambiare **classe** di ancoraggio. Ogni patch che resta sul TESTO o sul
NOME sarà aggirata dal livello successivo (KS-SK-95-2).

**C2 — FONDAMENTALI-first, l'oggetto è la leva.** Tutte e tre le sedie
mettono al primo posto la leva arene per-file del preludio con gate di
parità COMPLETI, e nessuna propone gate nuovi. Hoare 2), Hejlsberg 1),
Klabnik (a).

**C3 — battery61 riproducibile in modo nativo** (criterio 5): Hoare 1),
Klabnik (b), Hejlsberg 3). Concorde.

**C4 — vietato adattare i gate alla leva** (KS-TH-95-2): nessun dissenso.

**C5 — Un dente che prova il TESTO non prova il COMPORTAMENTO.**
A-SK-91 (giudice eseguito con `perl`/`git` dirottati) e A-AH-70/K
(`--selftest-identity` esteso al CONSUMO, non al predicato del nome)
sono la stessa forma su due oggetti diversi: nessuno dei due è ridondante.

---

## 2. Conflitti (posizione per sedia)

**K1 — L'apparato entra o no nell'ordine S-94.0?** *(conflitto reale)*
- **Klabnik**: SÌ, terzo posto, «SOLO A-SK-88/89/90 in un'ora, perché
  senza quelli ogni PASS futuro è firmabile da chiunque — poi congelato».
- **Hejlsberg**: NO — «apparato congelato (condizione 4), gli emendamenti
  restano A VERBALE, si attuano nella prossima finestra apparato».
- **Hoare**: «SOLO se blocca (condizione 4), **nessun gate nuovo**».
- **Composizione del relatore**: entra, per il criterio di Hoare, non
  malgrado esso. (i) Non è un gate NUOVO: è la riparazione di un gate
  esistente, misurata in poche righe. (ii) **Blocca l'oggetto**: le cifre
  che S-94.0 produrrà sulla leva sono consumate a rc=0 di
  `gate-measure-cifre --all`; se quel rc=0 è producibile da un canale
  scelto dal chiamante, la misura della leva **non ha autorità** —
  non è apparato per l'apparato, è l'autorità del numeratore. Il dissenso
  di Hejlsberg è registrato e resta valido come **vincolo di timebox**,
  non come veto.

**K2 — Grado PROBE (rc=65) per misure senza dispersione.** *(conflitto)*
- **Klabnik**: serve un grado sotto ADVISORY per B3 e per il rapporto CLI.
- **Hoare**: «nessun gate nuovo».
- **Composizione**: il PROBE è **espansione** d'apparato, non riparazione
  → **BACKLOG per NOME (A-SK-92-PROBE)**. In S-94.0 la sostanza di
  Klabnik si ottiene a costo zero per via **lessicale**: una misura a un
  run per braccio si scrive «indistinguibile dal rumore», mai «delta
  nullo». Questo è vincolante da subito e non tocca una riga di codice.

**K3 — Numeratore della leva: touched o capacità?** *(coppia non risolta)*
- **Hoare**: A-TH-75 pinna il numeratore al **touched ≈25,8 MB**;
  KS-TH-95-3 annulla ogni predizione firmata con 39.534.144.
- **Hejlsberg**: emenda con `Bump::with_capacity` per unit **proprio per
  recuperare i 13.738.592 B di coda mai toccata** — cioè esattamente la
  differenza capacità−touched.
- **Composizione**: non è contraddizione ma **accoppiamento non
  dichiarato**: con il pre-size, una parte del risparmio viene da capacità
  mai toccata e NON è predicibile dal touched. Vincolo: **A-AH-72 deve
  emettere entrambe le grandezze per-unità** (`allocated=` capacità E
  touched), e la predizione-misurata WP-48 va firmata come **DUE
  predizioni separate** (touched; coda di capacità), mai una sola.
  Altrimenti scatta KS-TH-95-3 sul residuo.

**K4 — Collisione di NUMERAZIONE (registrata come difetto di verbale).**
`A-AH-69` e `A-AH-70` esistono in DUE significati diversi:
- Klabnik: A-AH-69 = ancorare la glob dell'esenzione pre-ledger
  (`battery-8[0-8]pre`); A-AH-70 = `--selftest-identity` esteso al consumo.
- Hejlsberg: A-AH-69 = `.done` parsato per-RIGA; A-AH-70 = ancora
  `sha256=…` sul triangolo + grammar sulle righe PASS; A-AH-71 = writer
  autenticato.
- **Disambiguazione proposta al plenario**: il blocco di Hejlsberg
  (69→73, contiguo e già esteso a 72/73 senza collisione) **conserva** i
  numeri; i due di Klabnik diventano **A-AH-74** (ancoraggio glob) e
  **A-AH-75** (selftest-identity sul consumo). Nessun ID riusato.
  *Un registro con due significati per lo stesso ID è un registro rotto:
  questa non è pedanteria, è la stessa classe di C1 (l'etichetta non è
  la cosa).*

**Non-conflitti** (registrati per completezza): la refutazione aritmetica
di Hoare (Q2, commento `lower/mod.rs:1013-1015`) non è contestata da
nessuno; i due hazard di Hoare (`var_os` nell'alloc-path, `thread::current()`
nel `GlobalAlloc`) non sono toccati dalle altre due sedie e sono
riparazioni di UB/deadlock, non apparato.

---

## 3. Cura minima ordinata

**Esiste UNA cura che chiude tutti e tre i canali di Klabnik insieme.**
Non è la somma delle tre toppe: è il **cambio di classe di ancoraggio**
richiesto da C1 — smettere di legare il nome, ristabilire il contesto
d'esecuzione sotto controllo del giudice PRIMA di qualunque lavoro.

**Nucleo: A-SK-90 (re-exec sanificante) su path FISICO (A-SK-89).**
Primo atto eseguibile del giudice:

```
exec env -u BASH_ENV -u ENV -u SHELLOPTS bash -p "$SELF_PHYS" "$@"
```

Perché chiude i tre canali con un colpo solo:
- **F-K1/K2** (`bash -c` + `BASH_SOURCE` iniettato): dopo il re-exec il
  testo **non viene più dal chiamante** ma dal file a `$SELF_PHYS` — il
  testo patchato muore all'exec, la corsa prosegue sul giudice pristino;
- **F-K3/K4** (symlink logico): `SELF_PHYS` risolto con `cd -P … && pwd -P`
  nomina il file che il kernel apre davvero → il tether A-SK-78 confronta
  lo sha del file PATCHATO, REFUSE;
- **F-K7** (`BASH_ENV`, funzioni esportate): `env -u` + `bash -p`
  neutralizzano sia il file di startup sia le funzioni ereditate
  (**verificato a macchina**, §5).

**Ordine di atterraggio — UN SOLO COMMIT, in questa sequenza:**

1. **A-SK-89** *(indispensabile, precondizione)* — `HERE`/`SELF_ABS`/
   `SELF_PHYS` **fisici** (`cd -P`, `pwd -P`). Senza questo il re-exec
   ri-esegue un omonimo e il tether continua a firmare un nome.
2. **A-SK-90** *(indispensabile, il nucleo)* — re-exec sanificante come
   **primo atto**, prima di `git`, `perl`, `HERE`, di qualunque lettura.
   ⚠️ **Hazard nominato dal relatore, da progettare nello stesso commit**:
   il marker anti-loop non può essere una env var che il chiamante
   pre-imposta per saltare il re-exec — sarebbe C1 di nuovo, un livello
   più in là. Il marker va **validato**, non solo letto: se il marker è
   presente ma `BASH_SOURCE` non è un array che nomina `SELF_PHYS` con
   sha == blob HEAD, **REFUSE**.
3. **A-SK-88** *(non ridondante, ma declassato: da blocco a asserzione)* —
   `declare -p BASH_SOURCE` deve iniziare con `declare -a`. Dopo il
   re-exec `BASH_SOURCE` è un array genuino, quindi 88 **non blocca più
   nulla da solo**: serve (a) a rendere SANO il marker del punto 2, (b) a
   morire con un NOME anziché per effetto collaterale. Verificato che
   discrimina esattamente (`declare -a` normale vs `declare -x` iniettato).
4. **A-SK-91** *(indispensabile — è il falsificatore)* — dente che esegue
   il giudice con `perl`/`git` dirottati e con i tre canali, e pretende
   il **rc ESATTO**. Senza 91 la cura è una promessa: KS-SK-95-3 (dente
   che copre un canale e non la sua variante d'ambiente = sotto-portata).
   Assorbe la sotto-portata di **T23** denunciata da Klabnik: arm-b deve
   asserire l'assenza dell'**escalation a rc=0 firmato**, non solo il 64.

**Ridondanze/sovrapposizioni dichiarate:**
- A-SK-88 è **ridondante come blocco**, necessario come guardia del marker
  e come nome dell'errore → resta, ma non è quello che chiude i canali.
- **A-AH-74** (ex Klabnik A-AH-69, ancoraggio glob) e **A-AH-75** (ex
  A-AH-70, selftest sul consumo) NON si sovrappongono a nulla di
  Hejlsberg: oggetti diversi (esenzione di scope; predicato di consumo).
- Sul lato Hejlsberg: **A-AH-71** (autenticare `writer=` contro lo sha del
  battery a HEAD) **sussume il rischio** che A-AH-70/H (ancore `sha256=`
  + grammar sulle righe PASS) mitiga soltanto → 71 prima, 70 dopo.
- **A-AH-69/H** (`.done` per-RIGA) non è ridondante con 71: verificato che
  oggi i `.done` reali sono **a riga singola** (`m90.done`, 97 B, 1 riga)
  MA il parse è `grep -q "^rev=$BREV "` accoppiato a `sed … | head -1`:
  con un `.done` a due righe i 4 campi nascono da righe diverse. Il
  `.done` è scritto dal battery, cioè interamente dal forgiatore.

---

## 4. Priorità S-94.0 (FONDAMENTALI-first, timebox mezza sessione d'apparato)

**Regola applicata** (utente 2026-08-03): un emendamento d'apparato entra
SOLO se blocca il prossimo passo sull'OGGETTO. Discriminante usata:
*la cifra che S-94.0 produrrà passa da questo percorso?*

### DENTRO la mezza sessione d'apparato (tetto duro; se sfora, si taglia dal fondo)

| # | Emendamento | Perché BLOCCA l'oggetto | Costo |
|---|---|---|---|
| A1 | **A-SK-89 + A-SK-90 + A-SK-88 + A-SK-91** (blocco unico, ordine §3) | Le cifre della leva sono consumate a rc=0 di `--all`; oggi quel rc=0 è **fabbricabile** (verificato §5) ⇒ ogni numero di S-94.0 nascerebbe senza autorità | ~30 righe + 1 dente |
| A2 | **A-AH-71** (writer autenticato contro sha del battery a HEAD) | battery-91pre / battery61 sono l'oggetto di S-94.0 e si consumano per quel percorso; è **una** riga: forma → origine | 1 riga |
| A3 | **A-AH-69/H** (`.done` per-RIGA: i 4 campi dalla riga che porta `rev=$BREV`; più righe `rev=` ⇒ REFUSE) | stesso percorso di consumo di A2; ancora `sed` già presente, cambia solo il pattern | 2 righe |
| A4 | **A-TH-73 + A-TH-74** (env-read fuori dall'allocatore; via `thread::current()` dal `GlobalAlloc`) | **non è apparato**: è un deadlock latente e un panic-path che è UB da contratto nel percorso che genera le misure | 2 righe |

*Se il timebox si esaurisce, l'ordine di taglio è A3 → A2: il blocco A1
non si taglia. Motivo: A1 chiude canali che un operatore può imboccare
**senza volerlo** (un `bash -c`, un symlink nel path); A2/A3 richiedono
una fabbricazione deliberata.*

### FUORI dalla mezza sessione — BACKLOG PER NOME (non «più avanti»: per nome)
- **A-SK-92-PROBE** — grado PROBE rc=65 sotto ADVISORY (K2). *Sostituito
  in S-94.0 dalla regola lessicale «indistinguibile dal rumore».*
- **A-AH-70/H** — ancore `sha256=[0-9a-f]{64}( |$)` sul grep del triangolo
  e grammar-anchor esteso alle righe PASS.
- **A-AH-74** (ex Klabnik A-AH-69) — `battery-8[0-8]pre` ancorato.
  *Diventa bloccante se S-94.0 gira una batteria con nome a tre cifre 8x0-8x8.*
- **A-AH-75** (ex Klabnik A-AH-70) — `--selftest-identity` esteso al CONSUMO.
- **A-AH-73** — dente HIR plain-data (precondizione della via precompilata,
  che è leva #2, non #1).

### L'OGGETTO (ciò per cui si fa la sessione — non conta nel timebox)
1. **Leva arene per-file del preludio + pre-size**, con i 7 obblighi di
   prova di Hoare (A-TH-76), il contatore per-unità **A-AH-72 PRIMA**
   (KS-AH-95-1) emesso **su due grandezze** (K3), gate parità COMPLETI e
   corpus per NOME nello **stesso commit**. Numeratore pinnato da
   **A-TH-75** (touched ≈25,8 MB) — la predizione a 39.534.144 è NULLA.
2. **Misura CLI hello/refl post-leva vs oracle**: il 4,42× deve muoversi.
3. **battery61 riproducibile in modo nativo** (criterio 5).

---

## 5. Verifica eseguita (a macchina, HEAD `a9a1b364`, nulla committato)

Scelta: **la catena F-K2 di Klabnik** — la più grave, perché è l'unica che
produce un `PASS … --all` **verdict-grade, rc=0, firmato col judge_sha
pristino, mentre gira testo patchato**. Le altre due degradano o
fabbricano una riga; questa **firma**.

**Baseline pristina.**
```
$ bash wp81-harness/gate-measure-cifre.sh --all      # 25,5 s
rc=1   (9 × FAIL … UNCOMMITTED php-rust/wp95-harness/verbali/verbale-*.md)
$ git rev-parse -q --verify HEAD:php-rust/wp81-harness/gate-measure-cifre.sh
2f37f386d153d6ea6fe4f86b2d26e85b953ac2e3     (== git hash-object del working tree)
```

**Primitiva del canale** (bash 3.2.57 arm64-apple-darwin25):
```
$ BASH_SOURCE=/tmp/pristine bash -c 'declare -p BASH_SOURCE' /tmp/pristine
declare -x BASH_SOURCE="/tmp/pristine"          ← scalare ESPORTATO
```
⇒ la guardia A-SK-82 `[ "$SELF_SRC" != "$0" ]` è **soddisfatta**: entrambi
valgono la stringa scelta dal chiamante.

**Forge.**
```
$ perl -pe 's/\$all_rc = 1;//g if /UNCOMMITTED/' "$SELF" > $SCR/patched.sh   # 2 righe cambiate
$ BASH_SOURCE="$SELF" bash -c "$(cat $SCR/patched.sh)" "$SELF" --all
FORGE rc=0
```
Output riga 265:
```
PASS gate-measure-cifre --all (A-SK64/A-SK-67): manifest perimeter,
bidirectional, authorities from HEAD [judge_sha=2f37f386d153d6ea
manifest_sha=f2ebfa986614710a budget_sha=34dda74c63b87eba
revs_sha=7797b9d7ebd5b880 head=a9a1b3646e6f]
```
Nessuna riga `REFUSE`, nessuna `NOTE` A-SK-78/A-SK-82.

**ESITO: forge CONFERMATO, e più grave di come è riportato nel verbale 3.**
Il rc passa da 1 a 0 e la riga PASS firmata **coesiste nello stesso output
con le nove righe FAIL** che il patch ha reso non-fatali: chi consuma il
rc, o la riga terminale, legge PASS verdict-grade su una campagna che il
giudice pristino aveva bocciato. Il tether A-SK-78 **firma il blob
pristino di codice che non è quello che è girato** — precisamente
KS-SK-95-1.

**Controprova delle cure proposte** (stesse invocazioni, oggetti-giocattolo):
```
A-SK-88  normale:   declare -a BASH_SOURCE='([0]="…")'     ← discrimina
         iniettato: declare -x BASH_SOURCE="…"                   REFUSE
A-SK-90  BASH_ENV=evil bash  v.sh  → HIJACK     (canale F-K7 aperto)
         BASH_ENV=evil bash -p v.sh → REAL      (canale F-K7 CHIUSO)
```
A-SK-89 non è stato riprodotto end-to-end (servirebbe un symlink nel
repo): resta **verificato da Klabnik**, non dal relatore — lo dichiaro.

Nessun commit, nessun file toccato nel repo; tutti i temporanei
(`patched.sh`, `p.sh`, `evil.sh`, `v.sh`, `L`, `realdir`, `*.out`)
cancellati dallo scratchpad a fine verifica.

---

## Riepilogo del relatore

Non c'è nulla da benedire. Il gate cifre v3 + A-SK-82 **non è
verdict-grade**: l'ho rifatto io a HEAD e ho ottenuto rc=0 firmato. La
cura non è un'altra toppa sul nome — è il **re-exec sanificante su path
fisico**, che sposta l'ancoraggio dal nome all'artefatto; A-SK-88 e
A-SK-91 la rendono rispettivamente sana e falsificabile. Entra in S-94.0
non perché sia bello avere apparato, ma perché **senza di essa nessuna
cifra prodotta da S-94.0 sulla leva sarà un'autorità**: mezza sessione,
tetto duro, poi congelato. Tutto il resto va a backlog per nome.


---

# team-misura — verbale di team (fase 2, Concilio WP-95)

**Sedie**: Bak (5) · Pedersen (6) · Leijen (7) · Gregg (9). **Relatore**: team-misura.
**Fonti vincolanti**: `wp95-harness/verbali/verbale-{5-bak,6-pedersen,7-leijen,9-gregg}.md`;
oggetto giudicato: `wp93-harness/huge-sites.out` (letto per intero).
**Mandato**: riconciliare o registrare i dissensi, MAI benedire.

---

## 0. FONDAMENTALI (in testa, per direttiva)

Il team NON ha misurato nulla di nuovo. Ha giudicato cifre già pubblicate e ha trovato
**un errore aritmetico agli atti** (§4, atto A2) oltre ai difetti di grado. Il contatore
delle sessioni-senza-misura-full è fermo a **WP-85 (8 sessioni)**: nessuna leva di questo
filone ha oggi un giudice, perché la predizione-misurata (WP-48) richiede un «prima» fresco.

---

## 1. Convergenze (4/4 sedie, nessun dissenso)

1. **Il probe slope di B3 è uno SCREEN, non una misura.** Un run per punto, due punti W,
   varianza mai stimata nel probe. La risoluzione dichiarabile è lo **spread inter-build
   38229 B/worker** (baseline b048b697 18852538 vs ctrl 8e966efd 18814309), che **supera** il
   delta della leva **21837 B/worker**. Verdetto legale: «|effetto| < ~0,2% alla risoluzione
   del probe», mai «delta = 21837». (Bak Q1 · Gregg Q1/KS-BG-95-1 · Pedersen Q2 · Leijen Q2)
2. **La qualifica «REFUTATA CON MISURA» va retrocessa nel GRADO.** Tutte e quattro le sedie
   la rifiutano come scritta; nessuna sostiene che LEVER-2 funzioni.
3. **B1 e B2 reggono nella sostanza**: sei chunk di raddoppio (x2+16, firma bumpalo) di UNA
   arena — il parse del preludio stdlib in `lower_prelude_uncached` — con falsificazione per
   NOME dei sospetti (UNIT_CACHE, STUBS, arene dei Module) e coppia alloc/dealloc vista dal
   canale in-band Rust. Questo è il metodo giusto e il canale di fede (Leijen KS-DL-95-2:
   in caso di divergenza, fede alla coppia in-band, la stat mimalloc è solo testimone).
4. **`malloc_huge` dei dump m90 è CUMULATIVO** e va declassato ovunque; ogni derivata di m90
   che l'ha consumato come *retained* è sospetta e da censire (Gregg A-BG-72).
5. **La leva «arene per-file» è la candidata giusta**, ma NON in questa sessione e non prima
   che esistano baseline e giudice (Leijen A-DL-68 dà il via libera dal perimetro allocatore;
   Bak la mette al 3° posto; Gregg al 4°; Pedersen «dopo, sessione dedicata»).
6. **Nessuna cifra di questo filone è verdict-grade.** Il file si autodichiara ADVISORY
   (riga 2) e il team conferma che quello è il **tetto**, non il pavimento: alcune righe
   stanno sotto.

---

## 2. Conflitti registrati (posizione di ciascuna sedia)

### CONFLITTO M-1 — che cosa resta in piedi di LEVER-2 (il più importante)
- **Pedersen**: la conclusione «sopravvive in ADVISORY sulla **gamba m90**» (arena committed
  invariata a exit_collect_mi, huge-sites.out:72-74) — gamba indipendente e già committata.
- **Leijen**: **quella gamba non esiste**. In release `_mi_prim_decommit` su macOS fa
  `madvise(MADV_FREE_REUSABLE)` ma pone `*needs_recommit=false` (prim.c:495-504, gate
  prim.c:503) e `mi_os_decommit_ex` decrementa `committed` SOLO se `needs_recommit`
  (os.c:590-591): **`committed` non scende MAI su purge in release, per disegno**. Inferire
  «committed piatta ⇒ mi_collect non decommitta» NON SEGUE. Resta il solo braccio A/B fisico.
- **Bak / Gregg**: il braccio A/B fisico è **sotto il pavimento di rumore** del probe stesso.
- **Composizione del relatore** (Pedersen non aveva letto Leijen; Leijen non aveva letto
  Bak/Gregg sul pavimento): **la gamba m90 cade per il meccanismo (Leijen), il braccio A/B
  cade per la risoluzione (Bak/Gregg) ⇒ non resta nulla che regga il verbo «refutata».**
  Grado finale di B3: **SCREEN — «effetto non rilevato sotto la risoluzione del probe»**.
  L'ADVISORY concesso da Pedersen è **ritirato** perché poggia sull'unica premessa che Leijen
  refuta per file:riga. Nessuna sedia ha argomenti contro questa composizione; è comunque
  registrata come dissenso di partenza, non come unanimità originaria.

### CONFLITTO M-2 — «transiente» al livello malloc vs «riserva» al livello OS
- **B2 agli atti / Leijen**: spike transiente, liberato e purgabile; nessuna controindicazione.
- **Bak (KS-BB-95-2)**: la dicotomia riserva/spike è falsa al livello provisioning — al
  restart il thundering herd sincronizza gli spike (picco ~W×39,5 MB) che il capacity planning
  DEVE coprire; «freed ≠ decommitted ≠ non-provisioned».
- **Attrito reale**: Bak porta a sostegno l'arena committed 151 MB dei raw m90 — **la stessa
  cifra che Leijen dichiara illeggibile** (os.c:591 ⊣ prim.c:504).
- **Composizione**: la tesi di Bak **regge sul picco sincrono** (argomento di scheduling, non
  di allocatore) e **cade sull'evidenza citata**. Nel testo emendato: tenere la clausola
  provisioning, **sostituire** l'appoggio a `committed` con il picco ×W misurato.

### CONFLITTO M-3 — che cosa si misura per PRIMO in S-94.0
- **Bak**: battery61 riproducibile → probe slope v2.
- **Pedersen**: battery61 (debito 31 sessioni) → campagna m91 con battery-91pre (MAI girata).
- **Leijen**: **sanare il canale stat** (MI_STAT=1 + coppia in-band) PRIMA di m91.
- **Gregg**: sanatoria di carta → **COPPIA FULL stessa-sera** (contatore fermo a WP-85) → m91.
- **Composizione**: §5. Il dissenso è ordinale, non sostanziale — tutte le sedie mettono la
  **leva per-file DOPO**, e nessuna accetta cifre nuove prima dell'apparato di battery.

### CONFLITTO M-4 — sorte del canale stat mimalloc
- **Leijen (A-DL-65)**: census m91 con `MI_STAT=1` esplicito e livello dichiarato nel banner.
- **Pedersen (A-PP-78)**: con `PHPR_HUGE_TRACE=1` i contatori galloc/gfree sono
  auto-contaminati (backtrace+format passano dal GlobalAlloc) ⇒ VOID.
- **Non contraddittori, ma incompatibili nello stesso run.** Regola composta: **MI_STAT=1 e
  TRACE=1 mai nella stessa esecuzione**; il run TRACE serve a NOMINARE, il run MI_STAT a
  CONTARE; ogni cifra da stat gated senza livello MI_STAT nel banner è invalida.

### CONFLITTO M-5 — grado del 4,42× e la sua forza di priorità
- **Gregg (KS-BG-95-2)**: magnitudine di canale su UN workload, non priorità di roadmap
  finché la media non è rimisurata; pin solo dopo R≥3 (A-BG-74).
- **Bak**: il 4,42× non compare come priorità; la sua leva n.1 è lo snapshot del preludio.
- **Composizione**: nessun conflitto duro — il 4,42× **motiva** la leva per-file ma **non la
  autorizza**; l'autorizzazione viene dalla coppia full.

### Punti senza dissenso ma con effetto sul testo
- **Pedersen A-PP-76**: `huge_note` è **asimmetrico su realloc** (main.rs:120-124: nota
  `realloc` sul size nuovo, nessun `dealloc` del size vecchio huge) ⇒ il saldo del canale huge
  deriva sui path realloc. Pedersen stesso esclude l'impatto su B1/B2 (i sei chunk bumpalo
  passano da alloc/dealloc), ma **il canale resta storto** e va sanato prima di riusarlo.
- **Pedersen A-PP-75**: il collect gira DOPO il dec Release di OUTSTANDING
  (worker_pool.rs:619-620 vs 630-633) ⇒ lavoro heap-mutante **fuori dalla finestra del
  testimone**; un censimento a outstanding==0 può sovrapporsi al collect. Fix a costo zero:
  ordine send → collect → dec.

---

## 3. Tabella dei gradi per cifra — `wp93-harness/huge-sites.out`, riga per riga

Legenda: **VERDICT** = pinnabile/ledger · **ADVISORY** = consumabile con provenienza, non pin
· **SCREEN** = indica direzione, vietato il pin e vietato il verbo «refutata/confermata» ·
**VOID** = non consumabile, va rimossa o marcata refutata.

### Intestazione
| riga | cifra / clausola | grado deciso | motivo |
|---|---|---|---|
| 2 | `grade=ADVISORY` (globale) | **da sostituire** con grado PER SEZIONE | il globale è il *tetto*: B3 sta sotto |
| 6-7 | tre hash di build + parent + feature | **FATTUALE** (regge) | identità dichiarata bene, non scambiabile per la release (Pedersen Q1) |
| 8 | `phpr d5ce86e3 INVARIATO (mai ricompilato)` | **ADVISORY** | plausibile ma non ricevutato in-band |
| 9 | `php-server RIPRISTINATO d45b578 dopo il probe` | **DICHIARATO — non ricevuta** (≈ VOID come prova) | nessun shasum in-band né ri-run del gate lever-pins DOPO il probe; il PASS in NEXT_SESSION è pre-S-93.0 (Pedersen KS-PP-95-3) |
| 10 | carico/W/porta 8199 | **FATTUALE**; smaltimento porta e log grezzi **non dichiarato** | A-PP-77 |

### B1 — i sei siti (righe 13-40)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 13-18 | le sei `size` (622576 … 19922928) | **ADVISORY forte** (autoverificante) | catena x2+16 verificata su tutti e 5 i passi; canale in-band, non stat C. Promuovibile a VERDICT nel census m91 con MI_STAT dichiarato |
| 19 | `somma=39423200` | **VOID — ERRATA** | la somma vera è **39223200** (delta +200000 agli atti). Vedi atto A2 |
| 20 | relazione di raddoppio | **VERDICT-grade** (identità aritmetica) | verificata: 2·s+16 esatta 5/5 |
| 21-22 | 6 huge dopo req1 e dopo req3 (numero FISSO) | **ADVISORY** | riproduzione intra-run, non inter-run |
| 23-25 | `nota-somma` residuo os-good-size | **VOID come scritta** | dipende dalla somma errata: il residuo corretto è 39911424−39223200 = **688224**, non 488224 (cifra ripresa anche da Leijen Q4) |
| 28-38 | sito/via/radice/identità (backtrace) | **ADVISORY** | attribuzione per NOME dal vivo; il canale TRACE non falsifica sé stesso |
| 39-40 | falsificati per NOME (UNIT_CACHE, STUBS, Module) | **ADVISORY forte** | è la parte metodologicamente migliore del file (Pedersen) |

### B2 — la natura (righe 43-59)
| riga | cifra / clausola | grado | motivo |
|---|---|---|---|
| 43-48 | le sei `dealloc` (ordine inverso) | **ADVISORY forte** | coppia in-band = canale di fede (KS-DL-95-2); promuovibile a VERDICT in m91 |
| 49-50 | «tutti e sei liberati dentro la prima richiesta» | **ADVISORY** | regge |
| 51-54 | conseguenza-1: «riserva fissa mai liberata» REFUTATA; stat cumulativa | **ADVISORY, ma da RIFORMULARE** | la conclusione sopravvive, la *ragione* agli atti è imprecisa (§ sotto) |
| 55-57 | conseguenza-2: «il decremento NON morde sul canale stat» | **VOID come scritta** | non «non morde»: **non è compilato** (free.c:612 / stub 635-637 a MI_STAT=0) |
| 58-59 | `natura=SPIKE TRANSIENTE … non riserva né leak` | **ADVISORY con riserva** | vero al livello malloc; **falso al livello provisioning** per lo spike sincrono ×W (Bak KS-BB-95-2) |

### B3 — LEVER-2 (righe 61-76)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 61 | titolo «REFUTATA con misura» | **SCREEN** (retrocessione) | 4/4 sedie |
| 62-64 | protocollo (1 run per W, W∈{1,4}) | **FATTUALE** — ed è la prova che è uno screen | zero repliche, zero varianza |
| 65-68 | 4 coppie `max_rss`/`peak_footprint` | **SCREEN** | R=1 per punto; non pinnabili |
| 69-70 | slope ctrl 18814309 / lever 18792472 | **SCREEN** | «mai nel ledger come slope» (Gregg Q1) |
| 71 | `delta slope=21837 (0.12%)` | **VOID come pin**, SCREEN come bound | sotto il pavimento 38229; il bound legale è «|effetto| < ~0,2%» |
| 72-74 | `coerenza=` arena committed invariata ⇒ mi_collect non decommitta | **VOID — inferenza REFUTATA** | prim.c:503-504 (`needs_recommit=false`) + os.c:590-591: `committed` non scende mai su purge in release (Leijen ref. capitale 2) |
| 75-76 | baseline build precedente b048b697 (slope 18852538) | **SCREEN come cifra, ma PROMOSSA DI RUOLO** | è la **risoluzione del probe**: da nota a piè di pagina a parametro dichiarato in testa a B3 |

### B3 — canale della leva vera (righe 79-92)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 79-80 | `allocated_bytes=39534144` | **MAGNITUDINE (ADVISORY)** — pin vietato | contatore in-band deterministico ma R=1 e legato all'identità build (A-BG-74: pin dopo R≥3) |
| 80 | `chunk_capacity=13738592` | **SCREEN** | il nome del campo non prova «coda MAI usata»: capacità residua ≠ non toccata; serve il campo *used* in banda |
| 81-83 | lettura «13738592 di coda MAI usata» | **SCREEN** | discende dalla riga sopra |
| 84-88 | costo CLI: 44630520 / 44597728 / 10093048 e **4.42** | **MAGNITUDINE (SCREEN-ADVISORY)** — pin vietato | 1 run per lato (Gregg Q1/A-BG-74); rapporto ricalcolato 4,4219 ✓ |
| 89-92 | leva nominata + obbligo gate parità completi | **non è una cifra**: dichiarazione d'intento, **valida e rafforzata** | Leijen A-DL-68 dà via libera dal perimetro allocatore |

### Verdetto finale del file (righe 95-101)
| riga | clausola | grado |
|---|---|---|
| 95-96 | «B1 CHIUSO» | **regge in ADVISORY** (con la somma corretta) |
| 97 | «B2 CHIUSO: transiente, liberata; il "mai liberata" era la statistica monca» | **regge in ADVISORY con riformulazione obbligatoria** (§ sotto) |
| 98 | «B3: LEVER-2 refutata con misura (delta nullo)» | **SCREEN** — sostituire il verbo |
| 99 | «slope fisico ~18.8 MB/worker resta l'oggetto» | **SCREEN** — l'oggetto regge, la cifra non è un pin |
| 100-101 | composizione «residuo committed per-thread post spike + retained» | **VOID come ipotesi formulata** — «residuo committed» è esattamente la grandezza illeggibile (Leijen A-DL-66); riformulare su canale fisico on-thread (A-DL-55) |

**Nessuna riga del file raggiunge il grado VERDICT** salvo l'identità aritmetica della catena
di raddoppio (riga 20), che è un teorema, non una misura.

---

## 4. La conclusione B2 sopravvive a Leijen? **SÌ, ma solo riformulata**

Il rischio è sottile e va detto in chiaro: B2 e la nota-canale di `huge-worker.out` dicono
**cose opposte**, ed è la nota di `huge-worker.out` a cadere, non B2. Ma la *ragione* scritta
in B2 («il decremento non morde») suggerisce un decremento eseguito e inefficace, mentre il
meccanismo vero è **preprocessore**: il decremento non esiste nel binario.

> **Tesi corretta, in una frase difendibile**: *in ogni build census (MI_STAT=0, imposto da
> libmimalloc-sys 0.1.49 build.rs:108-115 → types.h:81-87), l'incremento di `malloc_huge` alla
> nascita della pagina huge è incondizionato (page.c:935-936, macro internal.h:392) mentre il
> decremento alla free è compilato VIA (`mi_stat_free` sotto `#if (MI_STAT>0)` free.c:612, stub
> release free.c:635-637; il decremento di theap.c:362 vive in un blocco commentato ed è dead
> code anche in debug): `malloc_huge` è quindi un contatore di NASCITE — **cumulativo per
> costruzione in quella build**, non «monco per assenza di free» — e la free dei sei chunk
> esiste davvero, provata dalla coppia alloc/dealloc in-band lato Rust.*

Corollari da portare nel testo:
- «cumulativo» è una proprietà **della build**, non della statistica in assoluto: con MI_STAT=1
  la stessa riga tornerebbe informativa (ed è il motivo di A-DL-65).
- **Asimmetria latente n.2** (per quando MI_STAT>0): il decrease va sul
  `_mi_theap_default()` del thread che libera (free.c:615), non sul theap che incrementò ⇒ una
  Σ per-theap può andare **negativa** su free cross-thread. Va scritta ORA, prima che qualcuno
  sommi per theap in m91.
- Il **controllo positivo E2 chiesto dal Concilio WP-94 è soddisfatto** — ma dal lato Rust, e
  la sua conclusione corretta è «il decremento C non è nel binario», non «non morde».

---

## 5. Atti di sanatoria sui file già committati

Tutti gli atti sono **di carta**, costo ≈ una sera, e vanno in **UN solo commit** (l'atto
A-BG-71 richiede esplicitamente «stesso commit della riga ledger»).

### (A) `wp93-harness/huge-sites.out`
| id | riga | correzione | formula |
|---|---|---|---|
| **A1** | 2 | grado per sezione + risoluzione dichiarata | `grade=B1 ADVISORY · B2 ADVISORY · B3 SCREEN` e nuova riga `risoluzione-probe=38229 B/worker (spread inter-build b048b697a4c10688 vs 8e966efd6b3d3e69) — ogni |effetto| sotto questa soglia e NON RILEVATO, mai refutato` |
| **A2** | 19 | **errore aritmetico** | `somma=39223200` (era 39423200; 622576+1245168+2490352+4980720+9961456+19922928 = **39223200**) |
| **A3** | 23-25 | residuo os-good-size ricalcolato | `residuo os-good-size = 39911424-39223200 = 688224` (era 488224). **Corroborazione indipendente**: con la somma corretta `39534144-39223200 = 310944`, dell'ordine dell'anello immediatamente inferiore della catena `(622576-16)/2 = 311280` — con la somma errata il residuo 110944 non corrisponde ad alcun anello |
| **A4** | 61 | verbo del titolo | `== B3: leva LEVER-2 (mi_collect on-thread post-preludio) — EFFETTO NON RILEVATO sotto la risoluzione del probe ==` |
| **A5** | 71 | delta come bound, non come pin | `delta slope=21837 B/worker — SOTTO la risoluzione 38229: verdetto legale «|effetto| < ~0,2%». PIN VIETATO (A-BG-73)` |
| **A6** | 72-74 | inferenza refutata | sostituire con `coerenza=RITIRATA (REFUTATA, Concilio WP-95/Leijen): in release committed non scende MAI su purge — _mi_prim_decommit pone *needs_recommit=false (prim.c:495-504, gate 503) e mi_os_decommit_ex decrementa solo se needs_recommit (os.c:590-591). Da committed invariata NON segue alcuna conclusione su decommit. Retention affermabile solo dal probe fisico on-thread (A-DL-55/66)` |
| **A7** | 55-57 | meccanismo per NOME | sostituire `il decremento NON morde sul canale stat` con `il decremento C NON ESISTE nel binario census: mi_stat_free e sotto #if (MI_STAT>0) (free.c:612, stub release 635-637), MI_STAT=0 via build.rs:108-115 di libmimalloc-sys 0.1.49 => types.h:81-87; theap.c:362 e dead code (blocco commentato). L'increase e incondizionato (page.c:935-936) => malloc_huge conta le NASCITE` |
| **A8** | 51-54 | tesi riformulata | applicare la frase del §4 («cumulativo per costruzione **in quella build**») |
| **A9** | 58-59 | caveat provisioning | aggiungere `caveat: transiente al livello malloc; al livello provisioning lo spike e SINCRONO su W thread al restart (~W*39,5MB) e va coperto dal capacity planning — freed != decommitted != non-provisioned (KS-BB-95-2)`. **Non** citare arena committed a sostegno (illeggibile per A6) |
| **A10** | 8-9 | ricevuta di ripristino | `RIPRISTINO=DICHIARATO — nessuna ricevuta in-band (ne shasum post-ripristino, ne ri-run del gate lever-pins DOPO il probe); ogni claim di parita che vi poggia resta ADVISORY (KS-PP-95-3)`; + riga `smaltimento=porta 8199 chiusa, log grezzi del trace <path> — per NOME` |
| **A11** | 4-5 (banner) | livello stat + auto-contaminazione | `MI_STAT=0 (dichiarato)` nel banner; + `contatori galloc/gfree del run con PHPR_HUGE_TRACE=1 = VOID (auto-contaminazione: backtrace e format passano dal GlobalAlloc)` (A-DL-65, A-PP-78) |
| **A12** | 80-83, 84-88 | etichette di grado | `chunk_capacity … [SCREEN: capacita residua != coda mai toccata — serve il campo used in banda]`; `4.42 = MAGNITUDINE R=1, PIN VIETATO fino a R>=3 (A-BG-74)` |
| **A13** | 100-101 | composizione dello slope | rimuovere «residuo committed per-thread» come termine; riformulare su canale fisico on-thread |

### (B) `wp92-harness/huge-worker.out` — supersessione formale (A-BG-71)
UNA riga in testa, **corpo intatto** (è evidenza storica), **due clausole per NOME**:

```
SUPERSEDED-IN-PART (Concilio WP-95, atto A-BG-71 — vedi wp93-harness/huge-sites.out B1/B2):
  clausola-1 REFUTATA: «riserva FISSA mai liberata» — i sei chunk sono TUTTI deallocati
    dentro la prima richiesta (coppia in-band Rust, huge-sites.out:43-48).
  clausola-2 REFUTATA: nota-canale righe 6-8 «il decremento ESISTE ... quindi current==total
    NON e un contatore monco: e assenza di free» — a MI_STAT=0 (ogni build census) il
    decremento e compilato VIA (free.c:612, stub 635-637) e theap.c:362 e dead code; il
    contatore E monco per costruzione in release.
  RESTANO VALIDI: 39911424 B e 6 blocchi per worker, come CUMULATIVO di nascite di pagine
    huge (mai come retained).
```

### (C) `sessions/WP_SESSION_93.md`
| id | riga | correzione |
|---|---|---|
| **C1** | 30 (B2) | «è la statistica malloc_huge … a non decrementare MAI (cumulativa)» → aggiungere `perche a MI_STAT=0 il decremento non e compilato (free.c:612/635-637); increase incondizionato (page.c:935-936) — cumulativo PER BUILD`; e «il decremento NON morde» → «il decremento NON e nel binario» |
| **C2** | 31 (B3) | `LEVER-2 … REFUTATA CON MISURA` → `LEVER-2 … EFFETTO NON RILEVATO (screen, R=1): delta 21837 SOTTO la risoluzione inter-build 38229 B/worker`; **cancellare** «coerente con arena committed invariata a exit_collect_mi» (inferenza refutata); `4.42` → `4.42 (MAGNITUDINE R=1, pin vietato)`; `39534144` → aggiungere `(R=1)` |
| **C3** | 74 | l'attribuzione dello slope va etichettata «solo da probe fisico on-thread (A-DL-55/66)» |

### (D) `gaps/GAP_TREND.md`, riga **WP-93 (S-93.0)**
| id | correzione |
|---|---|
| **D1** | `LEVER-2 mi_collect refutata con misura (delta slope 0.12 per cento)` → `LEVER-2 mi_collect: EFFETTO NON RILEVATO — delta 21837 B/worker sotto la risoluzione inter-build 38229 (screen R=1, non refutazione)` |
| **D2** | `slope fisico probe 18814309 B/worker` → `slope fisico probe 18814309 B/worker [SCREEN, non pin — mai nel ledger come slope]` |
| **D3** | `somma 39423200` → `somma 39223200` (stesso errore propagato) |
| **D4** | `malloc_huge di m90 è un CUMULATIVO` → aggiungere `per costruzione a MI_STAT=0` |
| **D5** | `4.42` → `4.42 (magnitudine R=1)`; la riga resta «non rimisurato — riferimento resta WP-85», che è il punto (§0) |

### (E) Atti che eccedono i quattro file ma sono conseguenza diretta
- **A-BG-72**: audit per NOME delle derivate m90 che hanno consumato `malloc_huge` come
  retained (repair90-estimators, VCOV 0,778, decomposizione b, «invisibile» 4,48 MB/worker);
  esito a ledger. **Il team lo considera parte della sanatoria, non lavoro nuovo.**
- **A-PP-76**: `huge_note` simmetrico su realloc (main.rs:120-124) — il canale va sanato
  prima di essere riusato in m91, anche se B1/B2 non ne soffrono.

---

## 6. Ordine S-94.0 proposto dal team-misura (FONDAMENTALI-first)

**Precondizione (non è una misura, non consuma il budget di misura)**
- **P0. Gli atti di sanatoria §5 (A+B+C+D+E) in UN commit.** Nessuna cifra nuova può essere
  pubblicata mentre agli atti resta una somma sbagliata e un verbo «refutata» non sostenuto.

**Che cosa si MISURA, in ordine**
1. **COPPIA FULL stessa-sera (media + peak footprint + CPU)** — *questa è la prima misura*.
   Motivo: il contatore è fermo a **WP-85, 8 sessioni**; la leva per-file è governata dalla
   regola WP-48 (predizione-misurata) e **senza un «prima» fresco non ha giudice**; e la
   coppia è l'unica cifra del filone che nasce già verdict-grade. (Gregg A-BG-75; nessuna sedia
   la contesta — Bak e Pedersen semplicemente mettono battery61 davanti.)
2. **battery61 riproducibile in modo nativo (criterio 5)** — debito di 31 sessioni, mezza
   sessione (Pedersen Q5, Bak Q4). **Registro il dissenso ordinale**: Bak e Pedersen la
   vogliono al posto 1. Il team propone 1↔2 in quest'ordine perché la coppia full è
   *precondizione della sessione successiva*, ma se il tempo basta per una sola, la scelta è
   del Concilio plenario, non di questo team.
3. **Probe slope v2 = il canale unico di m91**, nella forma **fusa** dei quattro emendamenti —
   non tre probe diversi:
   - `MI_STAT=1` esplicito e **livello dichiarato nel banner**; `TRACE=1` mai nello stesso run
     (A-DL-65 + A-PP-78, conflitto M-4);
   - **coppia alloc/free IN-BAND nel GlobalAlloc Rust** (soglia ≥524288, chiave thread/theap)
     come canale di fede (KS-DL-95-2);
   - **eco d'arm obbligatoria**: `arm=<nome> fired=<n> thr=<id>`; raw senza `fired==W` ⇒ run
     VOID (A-PP-74) — senza questo, ogni delta nullo è indistinguibile da un arm mai scattato;
   - **R≥5 interleaved (A,B,A,B…) sulla stessa binaria, W∈{1,2,4}**, mediana ± banda 2se
     pubblicate NEL probe; NULLO **solo** come test di equivalenza con soglia ex-ante
     (A-BB-74);
   - **doppia metrica peak + residency post-warmup** — una leva che agisce dopo il picco non
     può muovere il peak (A-BB-75, KS-BB-95-1);
   - **finestra del testimone**: ordine `send → lavoro → dec OUTSTANDING`, o dichiarazione
     in-band di lavoro fuori-testimone (A-PP-75);
   - `huge_note` simmetrico su realloc prima dell'uso (A-PP-76).
4. **Attribuzione dello slope ~18,8 MB/worker per NOME**, ammessa **solo** dal probe fisico
   on-thread (A-DL-55) — mai da `committed` (A-DL-66).

**Che cosa NON entra in S-94.0** (esplicito, per non riaprirlo in sessione)
- ❌ **La leva arene per-file.** Approvata nel merito da tutte e quattro le sedie, ma **nessuna
  leva prima che esista il giudice** (coppia full + probe v2). Entra in S-95.0 con predizione
  scritta ex-ante (hello 44,6 → ~11 MB) e gate parità COMPLETI (corpus 1418 + refl 290 + ORM
  3E/13F + hk 1665) + ricertificazione baseline phpr.
- ❌ **Lo snapshot/precompilazione del preludio** (Bak Q3 n.1): endgame, costo e gate massimi.
- ❌ **Il rifacimento di LEVER-2**: è post-picco per costruzione; non merita un probe dedicato
  finché non esiste la metrica residency che potrebbe vederla.
- ❌ **Qualsiasi pin** di 21837, 18814309, 39534144, 4,42×: pin solo dopo R≥3/R≥5 (A-BG-73/74).
- ❌ **Nuove derivate da m90** finché l'audit A-BG-72 non chiude.
- ⏱ **Apparato**: solo se blocca l'oggetto, timebox permanente (direttiva FONDAMENTALI).
- 🔎 **Da tenere in coda, non in S-94.0**: latenza prima-richiesta per worker (A-BB-76) e spike
  di fattibilità `LoweredPrelude: Send+Sync` (A-BB-77) — cifre di canale, non fondamentali.

---

## 7. Riepilogo delle retrocessioni (la lista corta che il plenario deve votare)

1. `huge-sites.out:19` somma **39423200 → 39223200** (errore aritmetico, +200000 B agli atti).
2. `huge-sites.out:23-25` residuo os-good-size **488224 → 688224**.
3. B3 «REFUTATA con misura» → **effetto non rilevato sotto risoluzione 38229 B/worker**.
4. `huge-sites.out:72-74` (coerenza m90) → **VOID/REFUTATA** per prim.c:504 + os.c:590-591.
5. `huge-sites.out:55-57` «il decremento non morde» → **«il decremento non è compilato»**.
6. `huge-sites.out:9` ripristino pin → **DICHIARATO, non ricevuta**.
7. `huge-worker.out` → **SUPERSEDED-IN-PART**, due clausole per NOME.
8. Delta 21837, slope 18814309, 39534144, 4,42× → **pin vietati**.


---

# Team-leva (fase 2) — Concilio WP-95

**Relatore**: sedia-relatore team-leva. **Sedie in perimetro**: Matsakis (2),
Stogov (8), Hejlsberg (4, parte rank/precompilato), Hoare (1, parte leva
per-file e semantica bumpalo). **Mandato**: riconciliare o REGISTRARE i
dissensi. Nessuna benedizione: sotto ogni cifra c'è un giudice o la cifra non
si scrive.

---

## 1. Convergenze (unanimi, senza sconti)

1. **La leva per-file è semanticamente percorribile e safe-only.** Il borrow
   checker prova già che nulla della `LoweredPrelude` borrows dall'arena
   (Matsakis Q1: alias senza lifetime, `Box<[u8]>`/`Vec` owned, parent =
   `ClassId` indice; zero `unsafe`/`Box::leak`/`transmute` in lower/mod.rs);
   la sequenza `{ Bump; parse; hoist; drop }` per unità compila com'è
   (Matsakis Q3); il Drop di fine funzione è già la prova (Hejlsberg Q1);
   il preludio è GIÀ multi-unit — 7 `File::ephemeral` tutti col nome
   `b"prelude"` su UNA sola `Bump` (Hoare Q3).
2. **La cifra pubblicata in `wp93-harness/huge-sites.out:89-90` è SBAGLIATA**
   («peak arena = max file ~74 KB invece del cumulativo ~39.5 MB»): confonde
   byte di SORGENTE con byte di ARENA. Refutazione capitale concorde di
   Matsakis (RC-MS-95-1) e Stogov (RC#1), corroborata da Hoare (Q3-7). Il
   fattore è ~2 ordini di grandezza: **la riduzione NON è ~500×.**
3. **Il rischio primario NON è «l'ordine globale di hoist»** (Stogov Q1,
   audit vivo: zero forward-ref extends/implements CROSS-file; core 52 decl,
   spl 28, reflection 25, tidy 0) **ma la sentinella `b"prelude"`** —
   uguaglianza al byte in ~20 siti portanti (host_reflect.rs:1511,
   vm/mod.rs:853 take_while sul PREFISSO, :13221, :16793, :6598/12145,
   calls.rs:1324, host.rs:3014, hir.rs:203, bytecode.rs:1769).
4. **Nessuna chiusura di fronte su per-file** (Stogov A-DS-70, Hejlsberg rank,
   legge no-front-closure): per-file è palliativo del solo picco transiente;
   lo slope ~18,8 MB/worker resta intatto e non nominato.
5. **Il rank parte da per-file** in entrambe le proposte (Hejlsberg 1, Stogov
   1): nessun conflitto sulla prima mossa.

### Verifica a macchina fatta dal relatore (nuova, non nei verbali)

- **Sorgenti**: 15 file, Σ = **405.628 B**; l'unità concat (9 file) =
  **309.488 B**; le 6 unità extra (ns/bcmath/gmp/mysqli/gd/fileinfo) =
  96.140 B. File massimo = **prelude/reflection.php 74.019 B**; secondo =
  dom.php 68.048 B.
- **Catena dei chunk bumpalo ricostruita a residuo ZERO**: 11 chunk piccoli
  (112 → 155.632, Σ = **310.944**) + 6 chunk huge (622.576 · 1.245.168 ·
  2.490.352 · 4.980.720 · 9.961.456 · **19.922.928**, Σ = **39.223.200**);
  **310.944 + 39.223.200 = 39.534.144 = `allocated_bytes` ESATTO**, e i primi
  cinque huge = 19.300.272 = esattamente la cifra di Hoare (Q2). La catena è
  `x·2+16` verificata su ogni anello.
  ⇒ **`allocated_bytes` = Σ CAPACITÀ di tutti i chunk; `chunk_capacity` =
  RESIDUO LIBERO del chunk corrente (13.738.592 su 19.922.928)**; nell'ultimo
  chunk sono toccati 6.184.336 B. **Touched fisico = 39.534.144 − 13.738.592
  = 25.795.552 B.** Hoare Q2 è CONFERMATO per aritmetica chiusa; il commento
  lower/mod.rs:1013-1015 («allocated_bytes … è il touched fisico») è FALSO.
- **Il forward-ref citato nel commento del `concat!` (mod.rs:740-747),
  «PhpToken prima di Stringable», è INTRA-file**: entrambi in
  `prelude/core.php` (righe 295 e 348). Corrobora l'audit di Stogov e
  l'argomento statico di Hejlsberg Q1(a): il conteggio decl per file
  coincide col suo (core 52, spl 28, reflection 25).

---

## 2. Conflitti (registrati, non lisciati)

**C1 — Unità della cifra: capacità vs touched.** Stogov e Matsakis calcolano
sulla CAPACITÀ (39.534.144 ⇒ ~97× sorgente→arena ⇒ ~7 MB / 5-8 MB per il file
massimo); Hoare pinna il numeratore al TOUCHED (25,8 MB). *Riconciliazione
del relatore*: **non è lo stesso oggetto e vanno etichettati entrambi.** La
valuta del FOOTPRINT è il touched (le pagine mai toccate non sono committed:
la coda di 13.738.592 B è indirizzo, non memoria fisica). La valuta
dell'IGIENE d'arena è la capacità. Regola: *ogni cifra della leva porta la
sua unità o è nulla*.
- capacità/sorgente = 39.534.144 / 405.628 = **97,5×** (Stogov, esatto);
- touched/sorgente = 25.795.552 / 405.628 = **63,6×** (derivato).

**C2 — `Bump::with_capacity` (A-MS-63, Hejlsberg «coda evitabile
13.738.592»).** *Parziale refutazione del relatore*: la coda mai toccata
molto probabilmente **non è nel footprint fisico**, quindi il pre-size può
valere **0 MB** sul denominatore che ci interessa. Vale su spazio
d'indirizzamento, su mmap/CPU e sull'igiene. **Non può essere venduto come
parte del numeratore della leva.** ⇒ pre-size in **commit e misura
SEPARATI** dalla leva, altrimenti si attribuisce alla leva un risparmio che
non c'è (violazione della predizione-misurata WP-48).

**C3 — Forma della leva: N arene vs una arena + `reset()`.** Stogov (A-DS-69)
nomina la variante `reset()`; Matsakis/Hejlsberg parlano di arena per unità.
*Riconciliazione*: **default = una `Bump` + `reset()` fra le unità**
(bumpalo trattiene il chunk maggiore ⇒ niente ri-raddoppio, niente churn di
mmap/commit, e la reuse delle pagine è garantita dall'arena invece che
sperata dall'allocatore — decisivo perché LEVER-2 ha provato che mimalloc non
decommitta). **Fallback = arena per unità** se `reset()` non è compatibile con
i borrow del lowering. Il picco atteso è lo stesso; cambia il rischio.

**C4 — Rank 2 vs 3.** Hejlsberg: 2 = precompilato embedded, 3 = condiviso
Arc (il condiviso aiuta SOLO il server, l'oggetto è il CLI 4,42×; ripple
Rc→Arc + atomics sui path clone-caldi). Stogov: 2 = condiviso per-processo
(è l'omologo Zend, ed è l'unico che attacca lo slope 18,8 MB/worker), 3 =
precompilato. *Riconciliazione*: entrambe le posizioni dipendono da DUE
misure che **oggi non esistono**. Ordine unico condizionale in §4.

**C5 — Posto di battery61 in S-94.0.** Hoare la mette **prima** della leva
(criterio 5); Stogov dice esplicitamente «resta debito separato, non
caricarlo sulla leva»; Hejlsberg mette la leva prima (la leva È l'oggetto).
*Non riconciliato — dissenso registrato*: il team a maggioranza (3 su 4) mette
battery61 DOPO, sotto la condizione-4 (apparato solo se blocca l'oggetto).
**Clausola di ribaltamento**: se la misura di ricezione della leva (§5.4) non
è riproducibile senza battery61 nativa, allora battery61 blocca e sale al
posto 0. Hoare mantiene la sua priorità a verbale.

**C6 — Obbligo Hoare-1 (interleaving classi/funzioni e doppio parse).**
L'audit di Stogov + la verifica PhpToken/Stringable intra-file riducono il
rischio, **ma non cancellano l'obbligo**: resta da provare che
`hoist_function` non leghi classi a hoist-time. Registrato come obbligo
vivo, non come rischio chiuso.

**C7 — «Riduzione ~5,5×» (Matsakis).** Corretta **sull'arena**; sul
footprint CLI la stessa leva porta il rapporto 4,42× a ~2,3-2,7× (≈1,8-1,9×
di miglioramento). Due numeri veri, oggetti diversi: **vietato citare 5,5×
senza nominare l'oggetto.**

---

## 3. Cifra difendibile della leva (predizione ex-ante)

### 3.1 Numeratore — cosa la leva toglie

**Valuta = touched d'arena, per-thread, al picco del preludio.**

```
N = T_tot − T_max
T_tot = 25.795.552 B          (MISURATO: 39.534.144 − 13.738.592, catena chiusa)
T_max = touched del file più grande   (DA MISURARE, §3.4)
stima lineare: T_max ≈ 63,6 × 74.019 = 4.707.608 B (~4,71 MB)
⇒ N ≈ 21.087.944 B (~21,1 MB), banda dichiarata 19–22 MB
```

In valuta di CAPACITÀ (per l'igiene d'arena, non per il footprint):
39.534.144 → ~7,2 MB, cioè **~5,5×** (Matsakis) / «5-8 MB» (Stogov):
entrambi CONFERMATI come cifre di capacità.

### 3.2 Denominatore — su cosa il risparmio si legge

**`peak_footprint` del CLI, binario di parità d5ce86e3342f3926 (pinnato):**

```
D  = 44.630.520 B   (phpr hi.php)      | controllo: 44.597.728 (refl.php)
Dr = 10.093.048 B   (oracle php -n)    | rapporto oggi = 4,42×
```

**Argomento a supporto che il picco È la fase preludio** (nuovo, dai dati già
committati): due script semanticamente lontanissimi — `hi.php` e `refl.php` —
hanno lo stesso picco entro lo **0,073%**. Un picco indipendente dallo script
è fissato da una fase indipendente dallo script: lo spike del preludio. Questo
è ciò che rende la leva la leva.

### 3.3 Predizione firmata (WP-48), con il coefficiente di trasferimento

Il numeratore d'arena **non è automaticamente** numeratore di footprint:
serve α = Δfootprint / Δtouched.

```
peak_post = 44.630.520 − α × N,     α ∈ [0,8 ; 1,0] atteso
⇒ peak_post ≈ 23,5 – 27,7 MB       ⇒ rapporto vs oracle 2,33× – 2,74×
FALSIFICATA se peak_post > 40 MB (α < 0,25: il canale non è quello nominato)
FALSIFICATA se peak_post < 21 MB (α > 1,1: risparmio non spiegato dalla leva)
```

α ≈ 1 è atteso perché con `reset()` la reuse delle pagine è interna
all'arena; con N arene separate α dipende dalla reuse di mimalloc (che NON
decommitta — LEVER-2 — quindi le pagine liberate restano committed e
riusabili: favorevole al picco, che è un high-water).

**Sul multi-worker: nessuna predizione firmabile.** Lo slope 18,8 MB/worker
non è ancora decomposto (committed-free dei sei chunk / live PRELUDE_CACHE /
live UNIT_CACHE-STUBS / residuo theap — Matsakis Q2). Predizione **solo
condizionale**: se una quota f dello slope è residuo committed dello spike,
la leva ne toglie f × (1 − T_max/T_tot) ≈ f × 0,82. Firmarla prima di
A-MS-62/m91 è vietato (KS-MS-95-1).

### 3.4 Come si misura PRIMA di scrivere una riga di leva

Il relatore **non** costruisce il contatore in questa sede: il repo non si
tocca durante il concilio e un build strumentato usato per cifre di footprint
è già vietato (KS-TH-95-1, KB-78-5). La cifra si misura nella sessione della
leva, **prima del codice della leva**, così:

- **M1 — contatore per-unità, parse-only** (A-AH-72 v2 + A-DS-69). Dietro
  `PHPR_PRELUDE_STATS=2`, build ADVISORY: per ciascuna delle 15 unità una
  `Bump` propria, **solo `parse_file`** (nessun hoist ⇒ zero rischio
  semantico, e l'arena tiene l'AST che è il 100% del canale), stampa
  `unit=<file> src=<B> allocated=<B> chunk_remaining=<B> touched=<B>`.
  Fornisce `T_max` e ogni `T_i` REALI, che rimpiazzano la stima lineare.
  **Controllo positivo obbligatorio**: `Σ T_i ≈ 25.795.552 ± 10%`; se la
  somma non torna, il contatore è rotto e la predizione è nulla (lezione
  WP-72: un contatore che non ha un controllo positivo non è un contatore).
- **M2 — dente sulla semantica bumpalo** (A-TH-75). Unit test su
  bumpalo 3.20.3 che asserisce `allocated_bytes` = Σ capacità e
  `chunk_capacity` = residuo del chunk corrente, + correzione del commento
  lower/mod.rs:1013-1015. La ricostruzione della catena qui sopra chiude
  l'aritmetica a residuo zero, ma resta **inferenza** finché non è un dente.
- **M3 — α si misura DOPO, e solo in coppia BUILD-ADIACENTE** stessa sera
  (`/usr/bin/time -l`, MIMALLOC_PURGE_DELAY=0, binari non strumentati).
  Ex-ante α è dichiarato come banda falsificabile, non come numero.

---

## 4. Rank unico delle leve (Hejlsberg ∪ Stogov, con tie-break oggettivo)

| # | Leva | Costo | Deve provare | Guadagno atteso |
|---|------|-------|--------------|-----------------|
| **1** | **Per-file, forma `Bump` + `reset()`** (fallback: arena per unità) | 1 sessione, superficie media-alta (span, sentinella, ordine) | Obblighi O1-O16 (§5) integralmente | CLI 4,42× → **2,3-2,7×**; slope W ignoto |
| **1b** | **Pre-size `with_capacity`** — commit e misura SEPARATI | basso | che il suo Δfootprint sia ≠ 0 (C2) | capacità −13,7 MB; footprint forse 0 |
| **2** | **Precompilato embedded** (`include_bytes!` da build-tool) | alto (derive su tutta la foresta HIR, versioning, determinismo) | A-AH-73 (HIR plain-data, niente `Cell`/`Rc` interni), byte-determinismo dell'artefatto, KS-AH-95-2 (**mai** cache su disco a runtime), parità completa | resto dell'arena **+ il parse CPU** (tocca il 2,06-2,11× full) |
| **3** | **Condiviso per-processo** (Rc→Arc o `&'static`) | ripple su tutta la VM + atomics sui path clone-caldi (rischio full CPU, WP-81) | **PRIMA il numeratore** (A-MS-62/m91: quota preludio dello slope), poi immutabilità post-seed dei decl, poi costo full CPU in coppia. KS-MS-95-3: proposta che non nomina il costo Rc→Arc = vacua | solo server (W−1 thread); zero sul CLI |
| **4** | **Lazy per-unità** | massima superficie | rompe l'invariante id-contigui (mod.rs:793-796) e la parità reflection | duplicato della 1 |

**Tie-break 2↔3 (registrato come regola, non come gusto)**: le due misure
mancanti decidono l'ordine.
- Se il **residuo CLI post-leva-1 resta ≥2× oracle** ⇒ la 2 è la prossima.
- Se **m91 nomina ≥50% dei 18,8 MB/worker come live `PRELUDE_CACHE`** ⇒ la 3
  sale al posto 2 (il numeratore esiste e vale il ripple).
- Se **entrambe** ⇒ prima la 2 (paga anche il parse CPU, e non ha ripple).
- Se **nessuna** ⇒ il fronte footprint-preludio si sospende, non si chiude
  (A-DS-70 + legge no-front-closure).

Le due misure sono già negli obblighi (O3 e O17): il rank si risolve da sé
alla fine della sessione della leva.

---

## 5. Obblighi di prova, ordinati (unione senza duplicati)

Fonti fuse: i 7 obblighi di Hoare (A-TH-76), F1-F8 + A-DS-66/67/68/69 di
Stogov, A-MS-62/63/64 di Matsakis, A-AH-72/73 di Hejlsberg, tutti i KS delle
quattro sedie.

### Fase 0 — PRE-CODICE (niente riga di leva prima che questi siano verdi)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O1** | Dente sulla semantica bumpalo (`allocated_bytes` = Σ capacità; `chunk_capacity` = residuo) + correzione commento lower/mod.rs:1013-1015 | A-TH-75 | `cargo test --release`; **KS-TH-95-3** (predizione firmata col 39.534.144 senza questo dente ⇒ NULLA) |
| **O2** | Contatore per-unità parse-only (M1) con controllo positivo `Σ T_i ≈ 25,8 MB ±10%` | A-AH-72 v2 + A-DS-69 | il numero scritto nel design PRIMA del codice; **KS-AH-95-1**, **KS-DS-95-3** |
| **O3** | Predizione WP-48 **firmata** nel design: N = 25.795.552 − T_max; D = 44.630.520; α ∈ [0,8;1,0]; soglie di falsificazione | relatore (§3.3) | coppia BUILD-ADIACENTE, stessa sera |
| **O4** | Fixture **F1-F8** oracle-morse committate | A-DS-67 | **KS-DS-95-2** (codice prima delle fixture ⇒ STOP) |
| **O5** | Checker cross-file forward-ref (extends/implements) come gate **pre-nascita**, scope dichiarato (NON copre const-expr cross-file né attributi) | A-DS-68 + Hejlsberg Q1(a) | exit-code del checker committato |
| **O6** | Dente `assert_static<LoweredPrelude>` + assenza di `unsafe`/`Box::leak`/`transmute` nel modulo | A-MS-64 | `cargo test` + dente di pattern |
| **O7** | Se il probe tocca l'alloc-path: env letta in `main()`, nessun `thread::current()` nel `GlobalAlloc` | A-TH-73, A-TH-74 | build del probe |

### Fase 1 — ATTUAZIONE (stesso commit di tutti i gate)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O8** | Nome unità `b"prelude"` **identico al byte** per ognuna delle 15 unità + unit test che lo asserisce; alternativa ammessa: predicato unico migrato in **tutti** i ~20 siti — **mai a metà** | A-DS-66 | **KS-DS-95-1** (rinomina senza migrazione ⇒ STOP) |
| **O9** | `hoist_function` NON lega classi a hoist-time (oppure due passate = doppio parse, **da misurare**, non da assumere) | Hoare-1, C6 | audit + F6/F8 + costo CPU misurato |
| **O10** | Ogni frammento resta unità di compilazione con lo scoping odierno (declare/namespace: il file NS separato esiste per questo) | Hoare-5 | corpus per NOME |
| **O11** | `MAIN_CHAIN_FP`: input enumerati aggiornati e **falsifier che si muove nello stesso commit** (mutare uno qualsiasi dei file muove il fp) | Hoare-6 + A-AH22 esteso (F8) | il falsifier che MORDE, non che passa |
| **O12** | Zero `unsafe`, zero leak `'static` dell'arena | **KS-MS-95-2** | grep-dente + review |
| **O13** | Pre-size in commit **separato**, con misura propria | C2 / A-MS-63 declassato | coppia adiacente dedicata |

### Fase 2 — GATE (stesso commit dell'attuazione)

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O14** | Gate parità **per NOME**: corpus 1418 + refl 290; ORM 3E/13F + hk 1665 per la ricertificazione della baseline | Hoare-3, Stogov Q3, ricetta ORM | **KS-TH-95-2**: UNA divergenza per NOME ⇒ **leva RESPINTA**; vietato adattare i gate |
| **O15** | F1-F3 liste INTERE `get_declared_classes/interfaces/functions` + count byte-id; F4 campione per file (`getFileName`/`getStartLine` === false, `isInternal` === true); F5 eccezione lanciata DA codice preludio (nessun `prelude:<riga>` affiora); F6 autoloader-logger a zero invocazioni; F7 "Cannot redeclare" senza file/riga; F8 snapshot name→id invariato | Stogov Q1 | baseline phpr d5ce86e3 per F1-F3/F8; oracle vivo per F4/F5/F7 |
| **O16** | **battery-91pre** (MAI girata: la ricompila la fa scattare) nella stessa sessione | Stogov Q4, Hejlsberg Q5 | la battery |

### Fase 3 — RICEZIONE E DEBITI

| # | Obbligo | Origine | Giudice |
|---|---------|---------|---------|
| **O17** | Live-delta on-thread attorno a `get_or_init` di `PRELUDE_CACHE`, con `heap=<ptr>` in banda; slope decomposto per NOME (committed-free / PRELUDE_CACHE / UNIT_CACHE-STUBS / residuo theap) | A-MS-62, A-MS-53 | **KS-MS-95-1**: «slope attribuito» senza heap=`<ptr>` e senza separazione per NOME ⇒ vietato |
| **O18** | Misura di ricezione: hello + refl CLI pre/post in coppia BUILD-ADIACENTE, slope W{1,4}; α calcolato e confrontato con O3 | Hoare Q4, Stogov Q4 | **KS-TH-95-1**: nessun build strumentato per cifre di footprint |
| **O19** | Precondizione della via 2: dente che tiene `ClassDecl`/`FnDecl` **plain-data** | A-AH-73 | `cargo test` |

---

## 6. Ordine S-94.0 dal team-leva (FONDAMENTALI-first)

**Principio applicato**: l'oggetto è il footprint del CLI (4,42×) e la leva è
l'oggetto; l'apparato entra solo se blocca (condizione 4), con timebox.

0. **(≤1h, tutta misura)** O1 (dente bumpalo) → O2 (contatore per-unità
   parse-only + controllo positivo) → **O3: predizione firmata nel design**.
   In parallelo, se il canale è pronto: **O17 prima metà** (A-MS-62,
   mezz'ora, un thread) — nomina la metà retained prima di ogni leva
   (priorità 1 di Matsakis).
1. **Pre-nascita**: O4 (F1-F8) + O5 (checker cross-file) + O6 committati
   PRIMA del codice (priorità 1 di Stogov).
2. **Leva**: forma `Bump` + `reset()` per unità (9 file splittati + le 6
   unità esistenti), O8-O12 nello **stesso commit** dei gate O14-O16.
   Pre-size (O13) in commit separato con misura propria.
3. **Gate completi per NOME** + battery-91pre; ricertificazione baseline
   (ORM/hk).
4. **Ricezione**: O18 (coppia adiacente stessa sera, hello+refl, slope
   W{1,4}); α calcolato; predizione↔misurato a verbale. Se O3 è falsificata,
   la leva si dichiara REFUTATA con la sua cifra — non si riscrive la
   predizione a posteriori.
5. **Completamento** O17 (attribuzione slope su m91) ⇒ risolve il tie-break
   2↔3 del rank.
6. **Debiti separati, non caricati sulla leva**: battery61 nativa (mezza
   sessione — **con la clausola di ribaltamento C5**); gli emendamenti al
   checker (A-AH-69/70/71) restano a verbale, finestra apparato successiva
   (fuori perimetro team-leva: Q4c di Hejlsberg è capitale e va al
   team-cifre).

---

## 7. Conflitti con la posizione di ciascuna sedia (esplicito)

**Hoare (1)** — Accolto: numeratore = touched 25,8 MB (confermato per
aritmetica chiusa dal relatore); i 7 obblighi entrano tutti (O9-O11, O14);
A-TH-73/74/75 accolti. **Conflitto**: il suo ordine mette battery61 **prima**
della leva; il team la mette dopo con clausola di ribaltamento (C5). Suo
obbligo-1 mantenuto vivo (C6) contro chi lo darebbe per chiuso dall'audit.

**Matsakis (2)** — Accolto: RC-MS-95-1 (la cifra pubblicata è sbagliata),
A-MS-62/64, KS-MS-95-1/2/3. **Conflitti**: (a) il suo «~7 MB/thread» e la
«riduzione ~5,5×» sono cifre di **capacità** — sul footprint valgono ~4,7 MB
e 4,42×→~2,4× (C1, C7); (b) **A-MS-63 declassato**: il pre-size è igiene,
non numeratore, perché la coda mai toccata probabilmente non è footprint
fisico — commit e misura separati (C2).

**Stogov (8)** — Accolto: refutazione della cifra e del rischio inquadrato,
audit cross-file (corroborato dal relatore su PhpToken/Stringable),
F1-F8, A-DS-66/67/68/69/70, KS-DS-95-1/2/3, e la sua tesi «battery61 non si
carica sulla leva» (contro Hoare). **Conflitti**: (a) il suo «~97×» è
capacità/sorgente, il rapporto footprint-rilevante è 63,6× (C1); (b) il suo
rank 2 = condiviso è reso **condizionale** e non a priori (C4); (c) la sua
variante `reset()`, che lui offre come alternativa, è **promossa a forma
DEFAULT** della leva (C3).

**Hejlsberg (4)** — Accolto: rank come scheletro, A-AH-72 (specificato:
parse-only per unità, Σ come controllo positivo), A-AH-73, KS-AH-95-1/2
(cache su disco a runtime NEGATA), l'argomento statico su `extends`
risolto a `ClassId` al lowering. **Conflitti**: (a) «la coda 13.738.592 è il
costo evitabile» → corretto in «costo di capacità, effetto di footprint non
provato» (C2); (b) il rank 2↔3 diventa condizionale a due misure (C4);
(c) la sua refutazione capitale Q4c (append in-window mai autenticati) è
**fuori perimetro** di questo team e va inoltrata al team-cifre — qui è
registrata, non giudicata.

---

*Nessuna cifra di questo verbale è consumabile senza il suo giudice. La sola
cifra qui dichiarata MISURATA è `T_tot = 25.795.552 B` (aritmetica chiusa a
residuo zero sui dati già committati); tutto il resto — `T_max`, α,
`peak_post`, la quota di slope — è DA MISURARE, con il come già scritto.*


---

## ⚖️ SINTESI DI CONVERGENZA — Concilio WP-95 su S-93.0 (compilata dalle ricevute fase 1+2 + estrazioni mirate; verbali = fonte VINCOLANTE)

**Verdetto complessivo: 9 sedie, 8 CON EMENDAMENTI + 1 REFUTATO (Klabnik).
Nessuna opposizione al lavoro dell'OGGETTO; DUE refutazioni capitali di
apparato (autorità gate + canale stat) e una retrocessione di grado
convergente 4/4 sul probe. Tre team di fase 2 (cifre, misura, leva).**

### §FONDAMENTALI (in testa, per direttiva utente 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: POSITIVO, per la prima
volta dopo due sessioni di sola ri-analisi. Sappiamo oggi di phpr tre cose
che ieri non sapevamo: (1) i «40 MB mai liberati per worker» sono l'arena
di parse del PRELUDIO stdlib (sei chunk bumpalo in raddoppio), **spike
transiente liberato alla prima richiesta**, non una riserva né un leak;
(2) `malloc_huge` di m90 è **cumulativo per costruzione** (MI_STAT=0 elide
il decremento C) — l'attribuzione huge-worker.out di WP-92 andava
superseduta; (3) il CLI di parità paga **4,42× l'oracle su hello** per lo
stesso preludio → candidato n.1 del roadmap footprint, ma la priorità è
dichiarabile **solo dopo una coppia full fresca**.

**(b) Contatore sessioni-senza-misura**: ultima full/media = **WP-85, OTTO
sessioni fa**; ultima campagna sull'oggetto = m90 (WP-90). S-93.0 ha
prodotto misure di PROBE (R=1), non verdict-grade. Il contatore NON si è
azzerato: serve la coppia full.

**(c) Rischio d'oggetto più trascurato ORA**: la leva footprint viene
governata dalla regola WP-48 (predizione-misurata) ma **manca il «prima»
fresco** — otto sessioni senza full. Rischio: attuare la leva per-file
contro una baseline stantia e non poter firmare il guadagno.

**Regola di ammissione all'ordine S-94.0**: l'unico apparato ammesso è
quello che BLOCCA il prossimo passo sull'oggetto — la riparazione
dell'autorità del gate (A-SK-82 è AGGIRATA, PASS forgiati verificati a
macchina: ogni cifra futura nascerebbe senza autorità) e il deadlock/panic
latente nel canale di misura. Tutto il resto → BACKLOG PER NOME. Timebox:
mezza sessione d'apparato, tetto duro.

### Refutazioni capitali

1. **🔴 A-SK-82 è AGGIRATA — tre canali producono un `PASS --all` rc=0
   firmato col judge_sha pristino su codice NON pristino (Klabnik, 7/9
   forge; RIPRODOTTO dal team-cifre a macchina, HEAD a9a1b364)**: (a) env
   `BASH_SOURCE` iniettato; (b) symlink logico/fisico; (c) `BASH_ENV` +
   funzioni esportate. La classe comune: **il giudice autentica stringhe
   scelte dal chiamante, mai gli artefatti che il kernel legge**. Cura
   minima UNICA (un commit): re-exec sanificante `exec env -u BASH_ENV -u
   ENV -u SHELLOPTS bash -p "$SELF_PHYS"` su path FISICO (`cd -P`/`pwd -P`)
   come PRIMO atto, marker anti-loop VALIDATO (non una env che salta il
   re-exec), + falsificatore che pretende rc esatto sui tre canali. →
   A-SK-88/89/90/91, KS-SK-95-1..4.
2. **🔴 La statistica `malloc_huge` è cumulativa PER COSTRUZIONE, non
   «monca per assenza di free» (Leijen, meccanismo per NOME)**: MI_STAT=0
   in release (libmimalloc-sys 0.1.49 build.rs → types.h) compila VIA
   `mi_stat_free` (free.c:612, stub 635-637) mentre l'increase huge è
   incondizionato (page.c:935); theap.c:362 è dead code. Corollario
   refutato: «committed invariata ⇒ nessun decommit» NON segue — in
   release committed non scende mai su purge (prim.c:504, os.c:590-591).
   La conseguenza-2 di huge-sites.out era scritta come «non morde»: va
   «non è compilato». → A-DL-65..68, KS-DL-95-1/2.

### Retrocessione di grado — probe B3 (convergenza 4/4: Bak, Pedersen, Gregg, Leijen)

Il probe slope di B3 è uno **SCREEN, non una misura**: R=1 per punto, due
soli W, delta 21837 SOTTO lo spread inter-build 38229 B/worker dello stesso
slope. «REFUTATA CON MISURA» è **RITIRATA** → «effetto non rilevato sotto la
risoluzione»; l'ADVISORY sulla gamba m90 è ritirato (poggiava sull'inferenza
committed refutata da Leijen). L'arm senza readback in banda è muto
(Pedersen): un delta nullo è indistinguibile da un arm mai scattato.

### Refutazione della CIFRA della leva (Matsakis + Stogov, team-leva)

«peak arena ~74 KB» confondeva SORGENTE e ARENA. Cifra difendibile
ricostruita a residuo zero dal team-leva: `allocated_bytes=39534144` è la
CAPACITÀ; il **touched reale è 25.795.552 B** (11 chunk; `chunk_capacity`
= residuo libero dell'ultimo, non «coda mai toccata»). Predizione ex-ante
WP-48: N = 25.795.552 − T_max; D = 44.630.520; peak_post = D − α·N con
α∈[0,8;1,0] → **2,3-2,7× oracle** (falsificata se >40MB o <21MB). Il
~500× è refutato, il ~5,5× è capacità. Audit vivo di Stogov: **zero
forward-reference cross-file** ⇒ la leva per-file è semanticamente
percorribile; il rischio vero è la sentinella `b"prelude"` osservabile.

### Convergenze forti (dai team)

- **team-cifre**: una cura di classe chiude tutti e tre i canali (re-exec
  sanificante + path fisico); l'apparato ENTRA in S-94.0 perché non è un
  gate nuovo ma la riparazione di uno rotto e verificato aggirabile.
  Ledger battery: `writer=` va autenticato contro lo sha del battery a HEAD
  (A-AH-71), `.done` per-RIGA (A-AH-69) — percorso di consumo di
  battery61/91pre.
- **team-misura**: 13 atti di sanatoria di carta in UN commit (già
  applicati: somma 39423200→39223200, residuo 488224→688224, B3→SCREEN,
  coerenza→REFUTATA, «non morde»→«non compilato», ripristino pin→dichiarato,
  huge-worker.out→SUPERSEDED-IN-PART). Ordine: sanatorie → **coppia full
  stessa-sera (prima misura, nasce verdict-grade)** → battery61 → probe
  slope v2 FUSO (MI_STAT=1 + coppia alloc/free in-band + eco d'arm fired==W
  + R≥5 interleaved + doppia metrica peak+residency). Leva per-file
  ESCLUSA da S-94.0 (nessuna leva prima del giudice).
- **team-leva**: rank unico 1) per-file+`reset()` (1b pre-size separato),
  2) precompilato embedded, 3) condiviso Arc, 4) lazy; tie-break 2↔3
  deciso da due misure (residuo CLI post-leva-1 ≥2× oracle → 2; m91 nomina
  ≥50% dello slope come live PRELUDE_CACHE → 3). 16 obblighi di prova
  ordinati in 3 fasi, ciascuno col giudice; controllo positivo del
  contatore per-unità `Σ T_i ≈ 25,8 MB ±10%` (lezione WP-72).

### Delibera di consumabilità

**S-93.0 è consumabile in ADVISORY dopo le sanatorie (APPLICATE in questa
chiusura).** B1 regge in ADVISORY (catena di raddoppio VERDICT come
identità aritmetica); B2 regge riformulato; B3 è SCREEN (nessun pin). La
falla A-SK-82 è would-have-allowed a HEAD ma il forge è VIVO: la
riparazione è la prima voce di S-94.0.

### Ordine vincolante di apertura S-94.0 (FONDAMENTALI-first)

**P0 (precondizione, già fatta in chiusura WP-93)**: sanatorie §5 team-misura
in un commit — nessuna cifra nuova mentre agli atti resta una somma sbagliata.

**Mezza sessione d'apparato (tetto duro, ordine di taglio A3→A2, A1 mai)**:
1. **A1 = A-SK-89 + A-SK-90 + A-SK-88 + A-SK-91** (blocco unico): path
   fisici, re-exec sanificante come primo atto, marker validato,
   falsificatore rc-esatto sui tre canali. Assorbe la sotto-portata di T23.
2. **A2 = A-AH-71** (writer autenticato contro sha del battery a HEAD).
3. **A3 = A-AH-69** (`.done` per-RIGA: 4 campi dalla riga che porta rev=$BREV).
4. **A4 = A-TH-73 + A-TH-74** (env-read fuori dall'allocatore, nessun
   panic-path nel GlobalAlloc — non è apparato, è UB latente nel canale
   di misura).

**L'OGGETTO (non conta nel timebox, è il corpo della sessione)**:
1. **COPPIA FULL stessa-sera** (media + peak footprint + CPU): prima
   misura, verdict-grade, precondizione della leva (contatore fermo a WP-85).
2. **battery61 riproducibile in modo nativo** (criterio 5, debito 31
   sessioni). *Dissenso ordinale registrato: Bak/Pedersen la vogliono al
   posto 1; il team-misura al 2. Se il tempo basta per una sola, la scelta
   è del plenario — qui: coppia full prima, è precondizione della leva.*
3. **Probe slope v2 FUSO** = canale unico di m91 (i quattro emendamenti in
   un solo strumento, non tre probe): MI_STAT=1 dichiarato, coppia
   alloc/free in-band, eco d'arm `fired==W`, R≥5 interleaved W∈{1,2,4}
   mediana±2se, doppia metrica peak+residency, `huge_note` simmetrico su
   realloc.

**S-95.0 (NON S-94.0)**: leva arene per-file del preludio, con i 16
obblighi del team-leva, predizione ex-ante firmata (2,3-2,7× oracle) e
gate parità COMPLETI + ricert. baseline phpr nello stesso commit.

**BACKLOG PER NOME** (non «più avanti»): A-SK-92-PROBE (grado rc=65),
A-AH-70/74/75 (ancore ledger), A-AH-73 (HIR plain-data, precond. leva #2),
audit A-BG-72 (derivate m90 che consumarono malloc_huge come retained).

### Kill-switch nuovi consolidati (attivi da subito)

KS-TH-95-1/2/3 · KS-MS-95-1/2/3 · KS-SK-95-1..4 · KS-AH-95-1/2/3 ·
KS-BB-95-1/2 · KS-PP-95-1/2/3 · KS-DL-95-1/2 · KS-DS-95-1/2/3 · KS-BG-95-1/2
— tabella nei verbali. Ereditati ATTIVI: WP-94 (22) + WP-93 (23) + WP-92
(22) + WP-91 (27) + WP-88..90. **KS-SK-91-1 resta NON sollevabile.**

**NON riproporre (nuovi)**: «tether su una stringa che il chiamante
sceglie» (A-SK-82 su `$0`/`BASH_SOURCE` da solo — KS-SK-95-1); «un
contatore stat come prova di retention senza leggere il #if di build»
(MI_STAT, KS-DL-95-1); «committed invariata ⇒ nessun decommit» (REFUTATA);
«refutare una leva con un probe R=1» (SCREEN, mai refutazione — KS-BB-95-1);
«peak arena = taglia sorgente» (confonde sorgente e arena); «leva prima del
giudice» (nessuna leva footprint prima di una coppia full fresca).
