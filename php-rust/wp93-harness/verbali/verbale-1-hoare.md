# Verbale sedia 1 — Hoare — Concilio WP-93 (revisione S-91.0)

**Perimetro**: sigilli lessicali v9, pin, fail-fast, doc (gate-lever-pins.sh; vm/mod.rs).
**VERDETTO: CONCORDO CON EMENDAMENTI.**

## Q1 — TH49ML_PROG (gate-lever-pins.sh:1246-1264)

Grafie che ANCORA sfuggono (verificate sulle regole, riga per riga):

1. **Punto-con-commento-in-coda**: `let x = u. // c` + riga `vm_gate(x)`. La tolleranza `(\/\/.*)?$` fu data alla regola NOME (1247) ma NON alla regola punto-nudo (1248: `/[.][[:space:]]*$/`) — asimmetria A-TH-59.
2. **Riga-commento a blocco interposta**: `u.vm_gate` \n `/* c */` \n `(x)` — pend/dot/nmp tollerano solo `//` e blank (1254/1257/1261); la riga `/* c */` cade su `{ pend=0 }`. Il residuo dichiarato ("multi-LINE block comments inside a path", 1237-38) NON copre questa riga-commento intera, lessicalmente raggiungibile.
3. **Nome-spazio-parentesi e turbofish, monoriga**: `retain.production_gate (x)` e `retain.production_gate::<>(x)` eludono TUTTO — `production_gate[(]` (86/118/164), sweep 340, TH49RE_V8 (1245: il ramo `.name…(` esige `/*…*/`), e l'awk (stateless su monoriga). Il pin ==2 su vm/mod.rs resta VERDE con un terzo sito. Buco più grave del perimetro.
4. **`use … CachedUnit as` con `as` a fine riga** + alias alla riga dopo: la regola 1252 esige `[[:space:]]as[[:space:]]` sulla stessa riga.
5. Dot+commento+nome-con-commento (`u.` \n `//c` \n `vm_gate //c2` \n `(x)`): COPERTO (1257→1259 nmp→1262). `use` dentro split: non-caso (illegale in posizione espressione); la regola 1249 resetta pend correttamente.

**awk-slash**: nel file è CHIUSA — ogni `/` nei pattern literal è escapata (`\/\/` a 1247/1254/1257/1261) o vive in regex DINAMICHE (-v re; TH49RE_V8 usa `[/]`; 1156/1167 `/` raw ma dinamiche = sicure). Invariante da dichiarare: TH49RE_V8 non contiene backslash — il processing `-v` li mangerebbe.

## Q2 — A-TH-57 (1416-1432)

Eludono ntot (grep fixed-string 'ProbeWindow::arm', 1426): **`ProbeWindow :: arm` spaziato**; **alias `use …::ProbeWindow as PW`** (TH49RE_V8 copre SOLO CachedUnit|VmGate); **type-alias `type P = ProbeWindow`**; **split multiriga `ProbeWindow::`\n`arm()`**; **siti fuori worker_pool.rs** (census scoped a $WPOOL, nessuno sweep workspace). Il vincolo ntot==2==narm regge strutturalmente (narm⊆ntot per riga) ma SOLO entro la grafia fissa: un terzo sito eluso lascia 2==2 VERDE. La dicitura "census TOTALE" è quindi FALSA alla lettera — è totale su una grafia, in un file.

## Q3 — Guardia numericità (1305)

Copre SOLO v8n/v8ml. Restano FAIL-OPEN su conteggio vuoto (`[ "" -ne N ]` → exit 2 → falso → dente vacuo): GATE_MINTS (140), PROD_N/SELF_N (552), RS_FLUSH (851), r1..r5 (1161), a/ml (1015), d1..d4 (1349), d5 (1447) — tutti output awk o grep-c-su-file-assente. I confronti in forma STRINGA (`[ "$x" = 0 ]`, es. 1357, 1465) sono fail-closed: vuoto≠"0". La guardia è un punto, non un sistema.

## Q4 — Doc A-TH-61 (vm/mod.rs 16548-16564)

L'enum a tre stati (destroyed/alive/never-initialized) è completa rispetto al modello std SOLO se l'accesso durante il PROPRIO destructor è sussunto in already-destroyed (std marca destroyed prima del drop): va DICHIARATO, altrimenti è il quarto caso. Incompletezze certe: (a) lost-write nominata solo per uc_log — **UC_STATS** (stessa lista TLS, 16544) ha lo stesso canale; (b) sul fallback path il never-init produce anche un box LEAKED (scrittura persa + memoria ritenuta). Il pin nth61 (1486-87) è `-ge 1` presence-only: doc svuotabile tenendo il marker.

## Emendamenti

- **A-TH-62**: TH49ML_PROG — tolleranza `//`-in-coda sulla regola punto-nudo + riga `/*…*/` intera nei tre stati; decoy same-commit.
- **A-TH-63**: rete monoriga nome-spazio-paren e turbofish (pin PREFISSO `[.]…(production_gate|vm_gate)` senza àncora `(`, modello A-TH-57) su vm/mod + sweep.
- **A-TH-64**: helper num-assert su OGNI cattura awk/grep-c (o `-ne`→`=` stringa); generalizza la guardia 1305.
- **A-TH-65**: ntot in ERE `ProbeWindow[[:space:]]*::[[:space:]]*arm` + sweep workspace + rami alias/type-alias ProbeWindow in TH49RE_V8; emendare la dicitura "TOTALE".
- **A-TH-66**: doc A-TH-61 — UC_STATS lost-write, leak fallback, collocazione caso durante-proprio-dtor con cite std; pin nth61 ==1 + frase distintiva.
- **A-TH-67**: ml — `as` a fine riga nel ramo use.

## Kill-switch

- **KS-TH-93-1**: nessuna cifra di campagna m91 con probe VERDICT-grade finché A-TH-63/65 non mordono (A-TH-57 è la base della validità probe).
- **KS-TH-93-2**: gate VOID se un conteggio non-numerico sfugge dopo A-TH-64.

## Refutazioni capitali

**NO.** Nessuna evidenza già consumata è falsificata; ma la LETTERA di A-TH-57 ("census TOTALE") e la copertura sistemica della guardia di numericità sono claim eccessivi da emendare prima di appoggiarci verdetti.
