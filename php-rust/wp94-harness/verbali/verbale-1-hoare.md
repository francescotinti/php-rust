# Verbale sedia 1 — Hoare (Concilio WP-94)

**Perimetro**: sigilli lessicali v10 (wp81-harness/gate-lever-pins.sh §10 + reti A-TH-62/63/65/67), pin, fail-fast (num_or_void A-TH-64), doc TLS (A-TH-61/66, vm/mod.rs r.16534-16576). Gate eseguito a HEAD e52a634: PASS rc=0. Controesempi ESEGUITI (scratchpad hoare94-counterexamples.sh) contro le reti di produzione copiate al byte; controllo positivo morde (v8re/th63/th65/th44 = 1).

**VERDETTO: CONCORDO CON EMENDAMENTI.**

## Q1 — grafie che eludono ANCORA le reti v10: SÌ, sette, eseguite (0 su ogni rete)
- **CE1** UFCS a parentesi staccata `RetainSet::production_gate (&r)` e turbofish `MainUnit::vm_gate ::<>(&u)`: A-TH42 esige `(` adiacente, A-TH-63 esige il punto, il token-pin `production_gate[(]` non matcha — un TERZO sito resta verde ANCHE nei due file pinnati.
- **CE2** fn-pointer `let g = RetainSet::production_gate;` poi `g(&r)`: nessuna rete (le alias-net coprono solo TIPI).
- **CE3** split a tre righe con blocco `/*…*/` IN CODA alla riga del nome (`u.` / `vm_gate /*c*/` / `(x)`): le regole ML 1275/1276/1288 tolleran0 solo `//` in coda; asimmetria sistematica //-vs-/*…*/ negli iniziatori pend/dot/nmp. Elude anche nei file pinnati.
- **CE4** `<ProbeWindow>::arm()`: TH65_RX esige `ProbeWindow` seguito da spazi e `::` — il `>` interposto lo buca; ntot==2==narm resta verde con un terzo sito.
- **CE5** `ProbeWindow::` a fine riga, `arm()` alla successiva: grep line-based; il programma ML non conosce ProbeWindow.
- **CE6** alias ProbeWindow multiriga: use-group ML, `as` a fine riga, `type P =\n ProbeWindow;` — il ramo use del ML traccia solo `(CachedUnit|VmGate)` (r.1279).
- **CE7** punto nudo + `/*c*/` in coda, `vm_gate(x)` alla successiva: in un TERZO file elude tutto (nessuno sweep conta `vm_gate(` nudo fuori dai pinnati).
Residuo dichiarato (blocchi multi-RIGA, macro) confermato fuori portata; i sette sopra sono IN portata lessicale.

## Q2 — num_or_void «su OGNI cattura»: NO. La dicitura di S-92.0 p3 è FALSA come universale
Guardate ~12 catture su ~45. **check_pin — l'elica del gate, ~25 denti — è fail-open**: dimostrato eseguito (regex rotta ⇒ awk muore ⇒ n="" ⇒ `[ "" -ne 1 ]` errore ⇒ stampa «OK … sites (pinned)»). Stessa classe: check_class, region_split (PS/PP/PC, SS/SP/SC), body_scoped_some sito reale r.255, count_noncfgtest r.693, publish_if_armed r.476-479, tooth-4 r.525/532, sweep_class (aritmetica su n vuoto), e TUTTI gli sweep in `$( )` salvo TH63/TH65 (`[ "$n" -gt 0 ] && echo` ⇒ file saltato in silenzio); self-test decoys r.81/249/287/417/423/640/666/741/796/817/852/1089-92 (vuoto ⇒ nessun exit 2 ⇒ vacui). I confronti stringa (`= "2"`) sono invece fail-closed per costruzione. Nota: num_or_void dentro `$( )` sarebbe comunque inefficace (exit 2 uccide solo la subshell) — serve il pattern inline-case di TH63/65.

## Q3 — fix bash-3.2 dei case in `$( )`: COMPLETO nel file
Census: case a r.62 (corpo funzione, parse top-level — sicuro), r.1343 (top-level), r.1490/1533 (in `$( )`, ENTRAMBI con `(` aperta). Nessun altro case in subshell; `bash -n` su 3.2.57 pulito. gate-binary-noprobe.sh: unico case (r.87) top-level; un'eventuale rottura lì sarebbe fail-closed (selftest FAIL). Nessun emendamento.

## Q4 — pin content A-TH-66: ELUDIBILE, dimostrato eseguito
I quattro grep -c sono FILE-WIDE: una SOLA riga di commento altrove in vm/mod.rs con marker + le tre frasi passa 1/1/1/1 col doc TLS interamente cancellato (q4demo: «ELUSION CONFIRMED»). Anche l'inversione semantica («NOT SUBSUMED…») matcha il substring.

## Emendamenti
- **A-TH-68**: chiudere CE1/2/3/7 — rete PREFISSO su `::(production_gate|vm_gate)` senza àncora di chiamata (modello A-TH-63); tolleranza `/*…*/` in coda speculare a `//` nei tre stati ML; decoy eseguiti stesso-commit.
- **A-TH-69**: chiudere CE4/5/6 — census del TOKEN `ProbeWindow` per file con allowlist (worker_pool + lib re-export), che assorbe qualified-self, split e alias multiriga in una rete sola.
- **A-TH-70**: guardia numericità UNIVERSALE: catture di check_pin/check_class/region_split/self-test sotto num_or_void; sweep col case inline NON-NUMERIC (modello TH63/65); meta-dente che grep-a il gate stesso per `-ne`/`-gt` su catture non guardate.
- **A-TH-71**: A-TH-66 SCOPATO — finestra awk dall'àncora A-TH54/A-TH-61 a `impl UcEmitGuard`; le tre frasi DENTRO la finestra, count file-wide == count in-window (anti-squat), minimo righe di commento della finestra.
- **A-TH-72**: emendare la riga di WP_SESSION_92 p3 «num_or_void su OGNI cattura» in «12 catture per NOME» (supersessione da ledger, mai riscrittura silenziosa).

## Kill-switch
- **KS-TH-94-1**: finché A-TH-68/69 non mordono con decoy eseguiti, i pin 2==2 (arm) e 1/1 (dot-name) sono ADVISORY — nessuna cifra m9x con probe li cita come VERDICT-grade.
- **KS-TH-94-2**: dente il cui conteggio passa per `-ne`/`-gt` senza guardia ⇒ le sue righe OK sono VOID (vacuità presunta) finché guardato.
- **KS-TH-94-3**: pin content soddisfatto da frasi FUORI dalla finestra ancorata ⇒ doc-pin VOID.

## Refutazioni capitali: NO
Nessuna cifra di campagna cade: i sigilli sono cintura, il giudice dei mint resta rustc (privacy VmGate). Cade però la DICITURA universale di A-TH-64 (A-TH-72) e la pretesa che il pin A-TH-66 sia content-pinned (è marker-pinned distribuito).
