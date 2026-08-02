# COUNCIL_WP89_REVIEWS.md — Concilio a 9 sedie su S-87.0 (ri-giudizi dai raw + campagna measure87 + sigilli v5) + programma WP-88(sessione)

**Convocato**: 2026-08-02, chiusura S-87.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_87.md, wp87-harness/MEASURE87_RESULTS.md + verdict87.out
+ verdict87.sh + measure87-campaign.sh + battery-87pre.sh + design87.md +
rejudge86.out + ds40-verify.out, wp88-harness/COUNCIL_WP88_REVIEWS.md (gli
ordini eseguiti), codice e raw dei rispettivi perimetri. Verbali VINCOLANTI
per il design WP-88(sessione).

## ⚖️ SINTESI DI CONVERGENZA

(compilata in coda ai verbali — v. fondo file)

---

## VERBALI INTEGRALI

# VERBALE — Hoare, sedia 1, Concilio WP-89

**VERDETTO: CON EMENDAMENTI** — A-TH40/41/42/43 eseguiti nella lettera, KH88-1..4 non scattati, il positive-control tainted esiste same-commit. Ma ho verificato nel codice: restano grafie eludibili e un limite dichiarato solo su carta. Refuto dove va refutato.

**Q1 — A-TH40: la finestra dei parametri è CHIUSA; ne dichiaro una residua, marginale.** Verificato: arm a vm/mod.rs:16559, rebind `let (key, cu) = (key, cu);` a :16566 — fra i due SOLO commenti, nessun punto di panico; da lì key/cu sono locali post-guardia e in unwind droppano a guardia armata. `kpath` (:16576), `putord` (:16580), superseded/victim/replaced (:16585-16587) tutti post-rebind: guardati. Early-return (:16547-16549) droppa `cu` non guardata — benigno e ora DICHIARATO nel commento (:16564-16565). Residuo PER NOME: un panico DENTRO `UcEmitGuard::arm` (:16531, `LocalKey::with` panica a TLS distrutta — put da un distruttore TLS in teardown) droppa i parametri non guardati; e il Drop della guardia stessa (:16542) nella stessa condizione panica in unwind ⇒ abort. Fail-fast di fatto (regola 4), ma non dichiarato.

**Q2 — A-TH41/KH88-1: il coupling è pinnato a doppia via e il tainted morde.** Pins:967-981: payload ==1 in vm/mod.rs (il literale a :538) + literale su riga ESEGUIBILE in noprobe; noprobe:39 greppa il payload PIENO, selftest tainted2.bin solo-payload (noprobe:49) morde. th41-positive.out: tainted FAIL rc!=0 + gemello clean PASS, `git_dirty_run=pre-commit-of-same-change` — KH88-1 soddisfatto per QUESTO cambio (metà nm dichiarata, matrix ENFORCE come cintura altrove). Resta eludibile: (a) il pin verifica riga non-commento, NON che il literale sia argomento del grep di detection — spostarlo in un `echo` passa; (b) esiste una TERZA copia del literale (pins:967): un rename a tre vie coordinato passa la meccanica e KH88-1 torna procedura umana.

**Q3 — A-TH42: le cinque grafie nominate sono coperte (decoys 2/1/≥3/1/≥1, pins:1010-1018; sweep 1019-1047). Grafie che ANCORA sfuggono, PER NOME:** (1) **`.vm_gate (` / `.production_gate (`** — spazio solo pre-parentesi: il pin di sede esige `[(]` adiacente (pins:327-328), lo sweep spaced esige spazio DOPO il punto (`[.][[:space:]]+`, pins:1021) — questa grafia buca ENTRAMBI; (2) **alias path-qualified** `type CU = vm::CachedUnit;` — la regex `=[[:space:]]*CachedUnit;` esige CachedUnit adiacente all'uguale; (3) **`use …::CachedUnit as CU;`** — il belt A-TH28 (pins:171-182) copre solo `vm_gate_probe`, mai i TIPI; (4) **`CachedUnit /*c*/`⏎`{`** — l'awk esige `CachedUnit[[:space:]]*$` (pins:1034); (5) **transmute→VmGate oltre la finestra `[^;]{0,200}`** (pins:1037) o via binding intermedio con alias path-qualified; (6) carrier-a-VALORE: resta APERTO come dichiarato — KH88-2 in piedi fino ad A-MS27 (backlog, WP_SESSION_87 §Residui).

**Q4 — A-TH43: putord sul supersede c'è (:16658), la doc dei due spazi ordinali c'è ed è esatta** (:16513-16525: per-thread, collisione a W>1, `UC_STATS.main_put` spazio DIVERSO senza mapping, wrap dichiarato). Ma il limite è opponibile solo per PROCEDURA (KH88-4 + commento): nessun dente macchina rifiuta un join di putord a W>1 — i consumer (a_ds36:19466-19488, a_ds38:19565-19592) girano single-thread per contingenza, non per asserzione.

**Emendamenti:**
- **A-TH44**: sweep — classe `[.][[:space:]]*(production_gate|vm_gate)[[:space:]]*[(]`; alias path-qualified `=[[:space:]]*([A-Za-z_]+::)*CachedUnit`; `use …::(CachedUnit|VmGate) as` ==0; decoys che mordono nello stesso commit.
- **A-TH45**: pin che il literale in noprobe sia ARGOMENTO del grep nella funzione di detection; terza copia (pins) dichiarata nel commento del coupling.
- **A-TH46**: putord opponibile a MACCHINA — thr= in-band sulle righe uc_log, oppure ogni consumer asserisce W==1 come precondizione eseguita.
- **A-TH47**: dichiarare nel doc della guardia la finestra TLS-teardown di arm()/Drop (panic ⇒ abort fail-fast).

**Kill-switch:**
- **KH89-1**: claim fondato su putord fra righe prodotte a W>1 senza thr= in-band ⇒ claim VOID (indurisce KH88-4).
- **KH89-2**: rename del payload in QUALSIASI delle tre copie senza th41-positive rieseguito nello stesso commit ⇒ noprobe ADVISORY, campagne VOID (estende KH88-1 alla terza copia).
- **KH89-3**: grafia di mint nuova nominata da un audit e non aggiunta allo sweep con decoy mordente nello stesso commit ⇒ sweep ADVISORY.

Firmato: T. Hoare, sedia 1.

---

# VERBALE — Matsakis, sedia 2, Concilio WP-89

**VERDETTO: CON EMENDAMENTI.** A-MS36/37/38/39 e A-DL36 eseguite fedelmente e verificate nel codice; KS-MS-88-1 legittimamente sollevata per rev ≥ 400fa10. Restano due classi evolutive non sigillate dal tipo (arm manuale, marker `.done`). (Repo: `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust`.)

**Q1 — TIENE, con un residuo evolutivo.** Il flag è `thread_local Cell<bool>` (worker_pool.rs:68-71), arm a :342 e disarm a :362 attorno all'UNICO probe site (`memcensus_unitcache_main_rows` :354 + `mi_bin_thread_rows` :360); il hook (main.rs:485-502) legge via `census_probe_active()` (:494). Il hook std gira SEMPRE sul thread panicante, PRIMA dell'unwind: per il panic del probe la lettura è program-order sul proprio thread ⇒ falso-negativo del bool globale (B legge false dopo il disarm di A) e falso-positivo (dispatcher letto true) sono chiusi ENTRAMBI per costruzione. Panic in async context: i probe non girano mai su thread tokio (worker = `std::thread::spawn` :295); un panic in task axum legge il PROPRIO flag (false) ⇒ attribuzione corretta. Abort da altro thread (`process::abort` :367) non passa dal hook ⇒ nessuna riga, nessuna menzogna. Il residuo: arm/disarm è manuale, non RAII — oggi il blocco :342-362 è straight-line, ma un early-return futuro lascerebbe il flag sticky-true e OGNI panic successivo di quel worker verrebbe liquidato come strumento. Corretto oggi, non sigillato domani.

**Q2 — TIENE; il dente muore RUMOROSO, non in silenzio — con un'eccezione.** gate-parity-83p1.sh:66 conta su `$W/cargo-test.log` (scritto a :51 nello stesso run, ancoraggio fresco) con `grep -cE 'vm::gate::VmGate.*compile fail.*ok'`, confronto `==2` a :67. Morti rumorose: re-export caduto/`doc(hidden)` ⇒ 0 ⇒ FAIL; `--lib` aggiunto ⇒ 0 ⇒ FAIL; terzo doctest ⇒ 3 ⇒ FAIL (delibera forzata, bidirezionale ✓); log mancante ⇒ NDOC="" ⇒ FAIL; doctest FAILED non matcha " ok". Il pin E0277 vive nell'attributo stesso (vm/mod.rs:512,520). L'eccezione: `.done` è toccato INCONDIZIONATAMENTE (:75) anche a FAIL — un consumer che leggesse `.done` come esito farebbe morire in silenzio TUTTO il gate, non solo il conteggio (classe già denunciata da Gregg su m86.done).

**Q3 — SPECCHIO FEDELE.** `a_ms38_key_present_guard_bites` (vm/mod.rs:19634-19650) replica riga-per-riga il pattern di a_th35 (:19612-19627): arma `UC_EMIT_GUARD`, `catch_unwind` su `unit_cache_key_present` (:16024, guardia :16032-16036), asserisce il panic, disarma, asserisce il ritorno legale `false`. Il dente ha morso; KS-MS-88-3 resta armata sul caller unico :7102.

**Q4 — TIENE: stessa corsia one-lifetime, nessun campo fuorilegge.** Catena verificata: `finish_parts` (:2319, locale in acquire :16246) → parametro di `main_publish_decision` (:16291) → `MainPublishTicket.lower_clamped` (:16116, doc esplicita "rides the same one-lifetime lane") → letto SOLO in `main_publish_ticket` (:16486) → `CachedUnit.main_program_net_clamped` (:15444). Nessun thread_local, nessun campo Vm, nessun carrier intermedio: il valore viaggia ESATTAMENTE dove viaggia `lower_net`. La entry pubblicata è la DESTINAZIONE sanzionata (identica a `main_program_net` :15439), non un carrier — KS-MS-84-4 rispettata nella lettera e nello spirito. Nit: sugli include il campo è `false` per convenzione (:7296), indistinguibile da "non clampato" — accettabile perché `main_program: None` discrimina, ma un parser di righe che ignorasse il discriminante conterebbe falsi.

**EMENDAMENTI**
- **A-MS40**: arm del probe via guard RAII (`ProbeWindow` con `Drop`→`set(false)`), costruttore UNICO; l'arm manuale :342/:362 diventa illegale.
- **A-MS41**: la belt pinna siti `CENSUS_PROBE_ACTIVE.with(set` ==2 (una coppia) — un secondo probe site senza delibera muore al gate.
- **A-MS42**: `.done` di gate-parity dichiarato NON-esito (commento in testa allo script); esito = rc.

**KILL-SWITCH**
- **KS-MS-89-1**: attribuzione instrument/measurand citata da un run con un probe site che arma il flag fuori dal costruttore unico (post-A-MS40) o con early-return nel blocco armato ⇒ attribuzione VOID.
- **KS-MS-89-2**: gate-parity consumato via `.done`/presenza-file anziché rc ⇒ PASS VOID.
- **KS-MS-89-3**: nuovo lettore di `main_program_net_clamped` fuori dal dump census, o carrier del clamp fuori dalla corsia ticket→entry ⇒ KS-MS-84-4 riaperta, cifra VOID.

*Niko Matsakis — il flag giusto abita il thread; il sigillo giusto abita il tipo: ciò che oggi è disciplina di riga, domani dev'essere un Drop.*

---

# VERBALE — Klabnik, sedia 3, Concilio WP-89

**VERDETTO: CON EMENDAMENTI.** Il PASS di measure87 (attempt=3) regge come protocollo; ho verificato negli script i miei quattro ordini WP-88 (A-SK46/47/48/49 eseguiti). Ma trovo UNA violazione di lettera di KS-SK-88-3 dentro verdict87.sh, e la finestra evidence-only del `--same-rev` è CIECA su tre superfici.

**Q1 — --same-rev rilassato.** La semantica (battery-equivalence.sh:71-77) delega "stesso codice" ai denti (i) (:108, solo `crates Cargo.toml Cargo.lock`) e (iv) (:114-125, solo script-gate + oggetti trascritti). Vettori aperti nella finestra BREV..HEAD: **(a)** il CHECKER stesso — battery-equivalence.sh non è oggetto di nessun gate: un commit che allarga PASSRE o toglie una riga da GATES/NEVER_ABSENT passa (i)+(iv) e cambia il significato del 15/15; idem **verdict87.sh** e **measure87-campaign.sh**. **(b)** ledger + matrix: un commit che appende uno stamp forgiato a `battery-stamps.ledger` E committa un matrix-archive forgiato (con `rustc=` corrente e un `bin[mem-census]` di un binario dopato) soddisfa TUTTI i denti :167-194 — la forge da me descritta in WP-88 Q1 ora richiede un solo commit su path che nessun dente guarda, e chiude anche il check KS-AH-87-1 della campagna (measure87-campaign.sh:95-98). **(c)** il corpus di gate-measure-cifre: `wp87-harness/*.out` è globbato come fonte (:118) — un `.out` fabbricato committato nella finestra legalizza qualunque cifra, e non è oggetto (iv). → **A-SK50**.

**Q2 — verdict87.sh.** I flag gateano: VDISP/VORD su IDENT (:63,:85), VSLOPE su tutti e tre (:107), VDISJ idem (:167). Due refutazioni: **(a) VARMS (:152) non richiede ORD_CLEAN**: con cache non quiescente (entries>1) il committed dei bracci è inquinato differenzialmente, e "VARMS PASS" resta greppabile da un out con VORD FAIL — advisory non esime dal flag a monte (spirito KS-SK-88-2). **(b) GRAVE di lettera: :116 `tr -d '\0' < …log`** — i log di `/usr/bin/time -l` sono PARSATI con strip dei NUL; `nul_free` (:33) copre solo i .memcensus (:47-50). Le cifre `peak_memory_footprint_bytes` in verdict87.out:25-28 e il companion in MEASURE87_RESULTS.md provengono da raw NUL-strippati: per KS-SK-88-3 quelle RIGHE sono VOID-a-rischio. Minore: identity line presa con `tail -1` (:51) senza count==1 — un raw a doppia generazione concatenata passa VIDENT. → **A-SK51/52**.

**Q3 — A-SK48.** La banda a scope di RIGA (gate-measure-cifre.sh:208) morde solo se '±' e unità coabitano sulla STESSA riga. Falsi negativi residui: **(a)** split cross-riga — `pin 232 ± 16` a capo, `(MiB)` sulla successiva: (2b) non scatta, "16" è 2-cifre, "232" in corpus, "(MiB)" nudo senza cifra adiacente sfugge a (3, :216); **(b)** unità a parole ("232 ± 16 megabyte") — nessun token `[KMGT]i?B`; **(c)** il dente (2c, :212) è `[KMGT]ib` maiuscolo: `1.024 B = 1,00 mib` passa la coppia verificata (1) senza flag bit-vs-byte. → **A-SK53**.

**Q4 — m87.done.** Convenzione ACCETTABILE, non buco KG-88-1 pieno: KG-88-1 protegge i RAW (che portano attempt= e il dente name-reuse :160/:197/:227); .done è un claim-pointer auto-identificante (git+attempt+mem_hash dentro, :293), il verdict lo consuma e ristampa attempt, il ledger :57 è append-only con la storia. MA l'asimmetria va sanata: il pointer NON ha il guard `[ -e ]` che i raw hanno — una campagna futura lo sovrascrive senza traccia nel tree (solo il pre-flight :66-67 lo blocca se non committato). → emendamento in A-SK54.

**Emendamenti**
- **A-SK50**: dente (i-bis) per --same-rev: il delta BREV..HEAD deve essere SOTTOINSIEME di un'allowlist nominata; checker/verdict/campagna byte-identici BREV..HEAD; ledger@BREV prefisso di ledger@HEAD; matrix committabile SOLO quello nominato dal .done.
- **A-SK51**: VARMS gated su ORD_CLEAN; VIDENT esige identity-line count==1.
- **A-SK52**: .log parsati entrano in nul_free con REFUSE, o il canale footprint diventa DECLARED-DEVIATION con conteggio NUL in-band; le righe peak di verdict87.out vanno ri-etichettate finché non sanato.
- **A-SK53**: '±' a scope di FINESTRA (±1 riga) e qualunque '±' in MEASURE8[4-9] fuori allowlist {232±1MiB, ±5%} = FAIL; (2c) case-insensitive sul prefisso.
- **A-SK54**: `m87.a$ATT.done` + pointer con guard di sovrascrittura e coerenza pointer↔suffixed nel verdict.

**Kill-switch**
- **KS-SK-89-1**: consumo --same-rev con delta BREV..HEAD contenente checker/verdict/ledger-rewrite/matrix non nominato ⇒ consumo VOID, campagna da ri-verificare.
- **KS-SK-89-2**: cifra giudicata proveniente da .log NUL-strippato senza declared-deviation in-band ⇒ riga VOID (estensione meccanica di KS-SK-88-3).
- **KS-SK-89-3**: PASS advisory flushata con un flag di pulizia a monte assente dal proprio gate ⇒ blocco VOID.

*— Klabnik, sedia 3.*

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-89

**VERDETTO: CONFERMATO CON EMENDAMENTI** — A-AH43/44/45 attuati e verificati nel codice; il consumo `--same-rev` a MACCHINA (prima volta) è reale. Restano tre fori nominati sotto.

**Q1 — A-AH43: UNA porta, sì, con un residuo.** `GITPREFIX` risolto UNA volta (`battery-equivalence.sh:79`) e usato in TUTTE le query committed: ledger stamp (:173), matrix show (:178), cat-file (:179), rustc header (:187). Grep dell'intero harness: nessun altro checker usa `git show` — scrittore (battery-87pre.sh:100-101, working copy) e lettore (checker, `HEAD:`) separati. Prefix verificato = `php-rust/`. **Residuo**: `object_changed` (:122-123) usa `php-rust/` HARDCODED più fallback bare — è la stessa patologia due-sorgenti bandita da A-AH43, non deduplicata sul `GITPREFIX` già in mano; su checkout con nome directory diverso il dente (iv) diventa vacuo in silenzio.

**Q2 — A-AH44: morde sul drift di versione, non su tutto.** Confronto stringa `rustc=` (matrix, sigillata da matrix_sha256 A-SK46) vs `rustc -V` corrente (:187-192). PASSA correttamente: stessa release via path rustup diversi (`-V` porta hash+data, non il path). **Fori**: (a) `rustc -V` è CWD-sensibile e il repo HA `rust-toolchain.toml` — né il recorder (`gate-feature-matrix.sh:77`) né il comparatore (:188) fanno `cd "$REPO"`: invocati da fuori-repo campionano il default rustup mentre `cargo` (:85, con `cd`) usa il pin — il comparatore può confrontare due campioni entrambi sbagliati e pari; (b) `rust-toolchain.toml` è FUORI dal pathspec del dente (i) (:108: solo crates/Cargo.toml/Cargo.lock) — un cambio di pin fra BREV e HEAD passa; (c) archivio con più righe `rustc=`: `head -1` (:187) giudica la PRIMA in silenzio (oggi il gate ne emette una, :77; non forgiabile grazie alla sha, ma il collasso è muto); (d) `-V` non porta l'host triple: drift di arch a pari versione è invisibile (`-Vv` sì). Mitigante reale: il hash del binario contro matrix (KH86-1) catturerebbe il drift materializzato.

**Q3 — A-AH45: delezioni ==0 verificate (battery-87pre.sh:83-89); il transiente resta, in una variante.** `comm` confronta solo gli ENDPOINT: create+delete interamente dentro la finestra è invisibile a entrambi i denti. Variante nome-diverso: innocua (il .done nomina solo l'aggiunta persistente). Variante **same-name swap** (delete del prodotto del gate + create intruso omonimo): NMTX=1, NDEL=0, e la `MTX_SHA` (:95) è calcolata a FINE battery sul contenuto dell'intruso ⇒ il .done ledgera come "proprio" un archivio non prodotto dal matrix-gate — KS-AH-88-2 aggirabile in-window. Il porcelain iniziale non protegge il runtime.

**Q4 — Onorata nella sostanza, con un'asimmetria.** Nessuna equivalenza abortita in senso stretto è occorsa (consumo `--same-rev`, niente append per costruzione, :240-248); la convenzione è stata applicata per analogia: a1/a2 VOID nominati con causa in `MEASURE87_RESULTS.md:12-17`, ledger campagna APPEND-only con storia completa e mai editato (`m87.campaign.ledger:1-73`), catena commit corretta (6d9a80f battery → a9e18fc stamp → d0132fb/38961f0 VOID nominati → e9eb4a2 PASS). **Neo**: a1 chiude nel ledger con `phase=done esito=ok` (:35) — la sua VOIDità vive SOLO in MEASURE e nel commit; a2 invece ha la riga abort in-band (:38). Il ledger da solo racconta a1 come pulito.

**Emendamenti**:
- **A-AH46**: `object_changed` usa `${GITPREFIX}` (drop del literal `php-rust/` e del fallback bare).
- **A-AH47**: recorder e comparatore campionano `( cd "$REPO" && rustc -Vv )` (host incluso); `rust-toolchain.toml` entra nel pathspec del dente (i).
- **A-AH48**: snapshot matrix-archive per NOME+sha256 (before/after); sha mutata su nome persistente ⇒ FAIL (chiude il same-name swap).
- **A-AH49**: il runner appende `phase=verdict esito=…` per OGNI attempt — la VOIDità deve essere leggibile dal solo ledger.

**Kill-switch**:
- **KS-AH-89-1**: consumo/equivalenza con `rust-toolchain.toml` mutato in BREV..HEAD o rustc campionato fuori-repo ⇒ consumo VOID.
- **KS-AH-89-2**: matrix con ≠1 riga `rustc=` o sha di archivio persistente mutata in-window ⇒ battery VOID.

---

# VERBALE — Bak, sedia 5, Concilio WP-89

**VERDETTO: PASS CON EMENDAMENTI.** Tutti i ricomputi dai raw combaciano al byte; la deviazione VSLOPE è fisica reale ma la statistica usata non è quella giusta per la forma dei dati.

**Q1 — Ri-giudizi riproducibili al byte: SÌ.** A-BB49: awk NUMERICO (`+0`) rieseguito su a2/a5/a8: a2 h=[0,13756] tid12 / p=[2,14395] tid13, a5 h=[1,13802]/p=[4,14468], a8 h=[3,13885]/p=[1,14250] — overlap 3/3, firma `pad_net−(NET_H+NET_P)=150` ESATTA 3/3 (15.153.408 = 7.349.977+7.803.281+150), inclusa l'anomalia a8 hello_net=17.887.038. Combacia riga per riga con rejudge86.out. A-BB51: campione dai .log — pa.r2=213.828.232 (=maxA), pa.r6=203.457.208 (=minA), pd.r7=240.288.440 (=maxB), pd.r6=226.132.640 (=minB): separazione maxA<minB confermata dai raw. VOVL resta REFUTED-BY-RAWS.

**Q2 — VSLOPE: statistica sbagliata per la forma giusta.** Ricomputo: min-of-R 68.681.728/103.153.664/116.785.152/150.405.120 ✓; LSQ = 129.400.832/5 = 25.880.166,4 ✓. MA i raw mostrano che il committed è una FUNZIONE A GRADINI, non una retta: W=3 è **bimodale al byte** (3 run a 116.785.152/116.850.688, 2 run a 124.321.792/125.435.904 — Δ modi ≈ 8,3 MiB), W=2 e W=4 hanno 4/5 run byte-identici + 1 outlier. Il min-of-R non viola KB-88-3 (che vieta max−min come giudice, non il min come locatore di un metrica contaminata solo verso l'alto), ma con R=5 su distribuzione bimodale il min SELEZIONA UN MODO senza garanzia di averlo campionato — la LSQ su 4 punti così ottenuti eredita la selezione. La mediana dei 3 Δ (32,06 MiB) sarebbe peggio. Forma giusta: **W∈{4,8,12,16}, slope sul segmento alto** dove i termini di granularità d'arena si ammortizzano, + **mode-census** per W (conteggio run per valore byte-identico), + chunk-count d'arena in-band. R=5 basta per il mode-census, non per il min nudo.

**Q3 — Surplus padA.** Ricomputo: netA=18.748.620/18.748.924, dA=3.146.416/3.146.720 ✓; netB dB=310 in 2/2 ✓; floor_inc 1.161.206 IDENTICO cal/conc 4/4 ✓; calibrazioni 7.801.102 byte-riprodotte 4/4 ✓. Scomposizione del surplus: **3.145.728 B = 3 MiB ESATTO + residuo 688/992 B** — un termine grande quantizzato + rumore piccolo. Candidati per NOME: (1) first-touch runtime del secondo worker (theap/prelude lazy-init); (2) buffer per-connessione axum/tokio del worker B; (3) crescita interner/registry condivisa. Esperimento MINIMO discriminante (senza attendere A-BB50): **warm-both-then-pair** — una richiesta hello a ENTRAMBI i worker PRIMA della coppia concorrente; predizione ex-ante: se il driver è il first-touch, dA crolla da 3.146.416 a ~310; se restano 3 MiB, è per-richiesta (axum buffer). Secondo discriminatore: stagger di 20 ms su B (il first-touch esce dalla finestra di A). A-BB50 resta il giudice finale.

**Q4 — Scope NON ancora falsificabile.** "Annidato" oggi vive in prosa ("simboli ⊆", classe hello⊆pad85) e regge sulla COSTRUZIONE del fixture, non su una verifica: i conteggi 317⊂377 NON provano l'inclusione degli insiemi. Definizione operativa proposta: NESTED(S,L) ⟺ (a) insieme dei NOMI di funzione di S ⊆ di L, verificato da macchina sui simboli, (b) witness in-band: floor_inc(S, ord2) ≤ ε nominato (osservato 1.040 B) con Δfloor(ord1→ord2) = costo del set condiviso ESATTO (996.838 B). Senza guard, l'appello allo scope è invocabile ma non refutabile.

**Emendamenti**: **A-BB54** nested-guard macchina (subset per NOME + floor-collapse witness) precondizione di ogni uso VERDICT-GRADE della scomposizione; **A-BB55** slope W∈{4,8,12,16} con mode-census e chunk-count in-band, LSQ W1..4 declassata ad ADVISORY; **A-BB56** warm-both-then-pair + stagger per il surplus padA, predizioni ex-ante nominate.

**Kill-switch**: **KB-89-1** regressione su ≤4 punti W con Δ non monotoni citata come slope verdict-grade ⇒ VOID; **KB-89-2** min-of-R citato senza mode-census quando i run mostrano ≥2 modi byte-distinti ⇒ ADVISORY, mai pin; **KB-89-3** uso VERDICT-GRADE della scomposizione senza A-BB54 eseguito ⇒ VOID, declassa a MODEL-GRADE.

— Bak, sedia 5

---

# VERBALE — Pedersen, sedia 6, Concilio WP-89

**VERDETTO: CON EMENDAMENTI** — i quattro ordini eseguiti nella lettera; ma l'eredità A-PP38 vive FUORI dal perimetro ledgerato, VDISP ha un buco di unicità, e la riga union è oggi un dente senza consumatore.

**Q1 — A-PP38: il path è coperto, il perimetro no, l'invariante nemmeno.** `a_pp38_link_fatal_first_dispatched_task` (worker_pool.rs:1352-1377) è vero full-path: pool reale W=1, `dispatch` → canale → `worker_loop` recv (:430) → `dispatched+=1` (:431) → `execute_request` (:463) → oneshot → `shutdown()`; l'approssimazione di a_pp33 è dichiarata (:1283-1289). MA: (a) il modulo è `#[cfg(feature = "axum-server")]` (:45) e i test compilano solo lì (:907-911); la battery esegue SOLO il workspace `cargo test --release` a feature default cli-server (gate-parity-83p1.sh:51) e feature-matrix fa `--no-run` (gate-feature-matrix.sh:188) ⇒ **a_pp38 non GIRA mai nella battery**; gira solo in CI (ci.yml:60 axum-server, :67 census-instrumentation), che non è nella catena stamp/.done/--same-rev. Il claim d'eredità non è vuoto in assoluto, ma è **CI-only, non ledgerato**. (b) Le assert sono solo status/banner: un inserter fra recv ed execute che PUBBLICA il fatal passerebbe — il path è coperto, l'osservabile publish di a_pp33 no (la cache è thread-local, invisibile dal test thread).

**Q2 — anti-orphan, buchi per NOME.** (1) **BUCO-PORTA**: nessun assert che il LISTEN-owner di :8296 sia SRV — il censimento è per NOME (`pgrep -x`, measure87-campaign.sh:135): un binario rinominato o un processo terzo sulla porta serve bodies identici e a/b/c sono ciechi. (2) **BUCO-CONC-GONE**: `run_conc` (:221-254) NON chiama `assert_server_gone` — e conc.r2 è l'ULTIMO run: il .done può nascere con un server vivo. (3) **BUCO-FAILPATH**: su wait_up fallito, run_arm fa `kill -TERM $DPID` (:168) = il wrapper time — la lezione attempt=1 applicata al teardown (:182) ma NON all'abort-path. (4) Pre-flight `pgrep -f` (:63) matcha cmdline estranee (tail/editor) e resta l'unico dente d'apertura. (5) Zombie (stato Z) contato da pgrep: direzione fail-closed ma attribuzione errata, semantica non verificata. (6) TOCTOU: assert one-shot post-wait_up; SO_REUSEPORT co-listener teorico ma non escluso.

**Q3 — VDISP.** Regge su: server fresco (next_worker=0), __reqns/404 non dispatchano (verificato WP-88), nreq divisibile per W (100·W garantito dal protocollo). Ogni richiesta extra o retry dispatchato ha firma "per-thread map != PER each" — fail-closed. MA verdict87.sh:69-74 controlla CARDINALITÀ (NTHR==W) e valore per riga, **non l'unicità di thr=**: due righe thr=0 count=PER con thr=1 assente danno NTHR=2, BAD vuoto ⇒ **PASS falso**. Ed è la classe del garbling S-86.0 (righe census duplicate/mescolate). Il set thr deve essere {0..W-1} esatto.

**Q4 — A-PP39.** Riga emessa a worker_pool.rs:318-321 (`cfg(not(mem-census))`, arm=union, stderr, teardown). Consumatori di `worker_dispatch`: verdict86.sh:66, va-census-template, verdict87.sh:69 — **tutti su raw mem-census**; nessuno greppa `arm=union` (verificato su wp78..88-harness). Oggi è **classe A-AH41: dente senza consumatore** — il verdetto union futuro (VW123/VABBA) dovrà catturare lo stderr union come raw e parsare la mappa, altrimenti il tag KS-PP-88-2 resta.

**A-PP40**: VERIFICATO — tier A esecutivo `^[^#]*perl[^#]*reqns-guard[.]pl` (gate-lever-pins.sh:903), decoy comment-only pp40decoy + positivo (:919-922), scan esteso a *.pl (:901-902). Residuo: un'occorrenza in stringa (`echo "perl reqns-guard.pl"`) è non-comment e soddisfa tier A — non coperta dai decoy.

**Emendamenti:**
- **A-PP41**: gate battery che ESEGUE `cargo test --release -p php-server --features axum-server`, contato nel k/k — l'eredità a_pp38 entra nel perimetro ledgerato.
- **A-PP42**: VDISP esige thr-set == {0..W-1} (unicità+completezza), non la sola cardinalità.
- **A-PP43**: port-owner assert (`lsof -nP -iTCP:$PORT -sTCP:LISTEN` pid==SRV) + `assert_server_gone` in run_conc + failure-path kill su SRV, mai sul wrapper.
- **A-PP44**: parser condiviso dispatch-union (classe reqns-guard.pl, con selftest che morde) + stderr union catturato come raw — precondizione di ogni VW123/VABBA union.
- **A-PP45**: a_pp38 acquisisce l'osservabile publish (probe env-armato o seconda richiesta discriminante).

**Kill-switch:**
- **KS-PP-89-1**: eredità a_pp38 citata da campagna consumata via battery che non ESEGUE il test ⇒ claim CI-only, sigillo declassato.
- **KS-PP-89-2**: VDISP PASS con thr duplicati/mancanti ⇒ mappa VOID, blocchi a valle VOID.
- **KS-PP-89-3**: attribuzione union-arm senza parser dispatch-union consumato ⇒ resta envelope (estende KS-PP-88-2).
- **KS-PP-89-4**: "server nostro" affermato su porta fissa senza port-owner assert ⇒ assunzione, mai identità.

— Pedersen, sedia 6.

---

# VERBALE — Leijen, sedia 7, Concilio WP-89

**VERDETTO: PASS CON EMENDAMENTI.** A-DL36/37 verificati NEL CODICE, dente positivo morde; VSLOPE è una NAMED-DEVIATION legittima; la DECLARED DEVIATION del collect è quantificata e innocua per la metrica di processo.

**Q1 — A-DL37: atomicità completa sul canale census, un residuo nominato.** In `phys_window_dump` (memcensus.rs:833-981) OGNI riga è format-in-buffer + `write_all` singolo su O_APPEND: mi_proc (861), ctx (871), NULLHEAP (904), FAILED (928), per-bin (938), OVERFLOW (948), header stats (966); `dump_line` accumula in un'unica String e scrive una volta (194); `census_line` (1073-1074) e le righe thr via `census_line` idem. **Residuo**: `mi_stats_print_out` (971) scrive su `$PHPR_MI_STATS` per frammenti via callback mimalloc — canale NON atomico. Emendamento A-DL40: dichiararlo NON-corpus (o bufferizzare il callback).

**Q2 — l'ipotesi granularità REGGE sui numeri.** I quattro min-of-R (68.681.728 / 103.153.664 / 116.785.152 / 150.405.120 B) sono TUTTI multipli esatti di 65.536 B: 1.048/1.574/1.782/2.295 granuli da 64 KiB; i Δ sono 526/208/513 granuli — l'alternanza ~32,9 / 13,0 / 32,1 MiB è il passo di chunk d'arena, non costo marginale. Lo slack è piatto (68.416–76.576 B): la ritenzione nei bin è esclusa; a W=4 il gap fra commit di processo (152.829.952) e committed dei bin visitati (`mi_bin_thr_sum` 63.560.848) è ~89,3 MB fuori-bin — lì vive il termine. **Riconciliazione (A-DL42)**: (i) slope stesso protocollo a W∈{8,10,12} (il regime di KL-85-2); (ii) righe in-band `tag=mi_arena` (reserved/committed per arena) per nominare il termine quantizzato; modello `committed(W)=a+b·W+c(W)` con c quantizzato a chunk — la banda 3.605.572 B si confronta SOLO con b, mai con la slope grezza W∈{1..4}.

**Q3 — deviazione quantificata: NON invalida la slope.** Dal raw m87.slope.w4.r5.a3: pre-collect (riga 198, chiusa da `exit_mi`) phys=3.899.944, commit=152.829.952; post-collect (riga 235, `exit_collect_mi`) phys=3.555.880, commit=152.829.952. **Il collect ha drenato 344.064 B di phys (84 pagine) e 0 B di commit**; peak invariati, rss +16.384 B (rumore). Un collect per-theap perfetto potrebbe recuperare al più l'ordine dello slack (~76 KB min-of-R, ≤345.792 B per-run) — 2-3 ordini sotto la slope 25.880.166 B/worker. Per la metrica committed di PROCESSO la deviazione resta DECLARED e sufficiente; A-DL39 serve all'ATTRIBUZIONE, non al livello.

**Q4 — design87: due costi MANCANO.** (a) **A-DL39, boundary pre-prologo**: `mi_theap_set_default` nel prologo non riscrive il passato — le alloc del worker PRIMA del prologo (spawn std, bookkeeping) restano sul heap condiviso: il per-worker heap SOTTOCONTA di un termine non nominato nel design; va dichiarato con marker `heap=` al prologo. (b) **A-BB50, semantica TEMPORALE dei free cross-thread**: in mimalloc il free altrui va in delayed-free list ed è ESEGUITO dall'owner più tardi (a un alloc/collect successivo) — "free eseguiti dal thread" slitta nel TEMPO, non solo di thread: una finestra può chiudersi prima che l'owner processi i delayed ⇒ net gonfiato in modo silente e non-deterministico (il clamp copre solo il lato deflattivo). Inoltre il VARMS eager delta 0 B ESATTO (103.153.664 = 103.153.664) senza read-back dell'opzione è un braccio potenzialmente MUTO.

**Emendamenti**: **A-DL40** ($PHPR_MI_STATS non-corpus o bufferizzato) · **A-DL41** (ogni braccio VARMS stampa in-band il read-back `mi_option_get` — positivo del braccio) · **A-DL42** (riconciliazione due-forme come in Q2) · **A-DL43** (design87 integrato con i due costi di Q4 PRIMA di ogni delibera di merge).

**Kill-switch**: **KL-89-1** braccio d'ambiente citato senza read-back in-band ⇒ braccio MUTO, delta VOID · **KL-89-2** slope committed citata fuori dal regime W della derivazione senza termine di granularità nominato ⇒ cifra non confrontabile · **KL-89-3** cifra per-worker-heap post-A-DL39 senza boundary pre-prologo dichiarato ⇒ VOID · **KL-89-4** righe da $PHPR_MI_STATS citate come corpus ⇒ escluse (canale non atomico).

*Leijen, sedia 7 — mandato refutare: eseguito su codice e raw, non su verbali.*

---

# VERBALE STOGOV — sedia 8, Concilio WP-89

**VERDETTO: CON EMENDAMENTI — due refutazioni nuove dall'oracle vivo (il modello «phpr ≡ braccio persist» è FALSO per l'istanziazione pre-decl; a_ds38 gira in battery col dente DISARMATO) e due casi Zend non classificati dal contratto A-DS41, provati dal vivo.**

**Q1 — §3.3-ter emendata: ancorata ma INCOMPLETA, e il modello over-claims.** Ricetta PERSIST corretta (PHPR_DIVERGENCES:413-441), 7 fixture + ds40-verify.out/sh tutti tracked (verificato `git ls-files`); ho riprodotto dal vivo parent-later (false/true) e const (Error/7) — KS-DS-88-2 formalmente soddisfatta. MA manca il **6° observable, e refuta la classe**: `new C` pre-decl (parent-later) fatala «Class "C" not found» exit 255 su plain **E persist**, mentre phpr stampa `C` exit 0 — **phpr diverge da ENTRAMBI i bracci**: l'hoisting di phpr è PIÙ AMPIO del persist, non «ESATTAMENTE il braccio PERSIST» (:436). Corollario: `C::K`=7 sotto persist con `new C` not-found suggerisce const-folding, non hoist — il MECCANISMO dell'observable 4 va riqualificato. Manca anche il negativo anti-vacuità: classe in blocco condizionale NON hoistata in nessuno dei tre bracci (verificato: bool(false)×3).

**Q2 — Il vincolo «riga successiva» NON è troppo forte: è strutturale.** Nell'emettitore la coppia è adiacente per costruzione (vm/mod.rs:16674-16685, nulla fra le due `uc_log`; supersede emesso PRIMA del blocco victim, :16638-16659); il guard salta le righe non-`unitcache` (putord-pair-guard.pl:26) quindi census/eventi estranei interleaved non lo rompono; UC_LOG_BUF è thread-local con flush whole-buffer `write_all` (:15905-15925) ⇒ a W>1 i blocchi non spezzano coppie salvo partial-write — che KL-88-2 (grammatica) già vieta. Rischio inverso: le righe non portano `thr=` (A-TH43 inevaso su evict/main_evicted) ⇒ sotto collisione putord cross-thread il guard può ACCETTARE una coppia falsa (KH88-4). Vincolo giusto; da dichiarare W=1-only finché thr= non è in-band. Nota: il guard oggi morde solo nel selftest (gate-lever-pins.sh:941-946) e in measure87 nessun uc_log era armato — KS-DS-88-1 soddisfatta VACUAMENTE, mai su log di produzione.

**Q3 — Le 8 regole (todo-master §⓪) NON coprono, per NOME:** (1) **trait abstract-method signature** — `use T` non è extends/implements/abstract: dal vivo Zend fatala «Declaration of C::f(): int must be compatible with T::f(): string» exit 255 in ENTRAMBI i bracci, phpr stampa `alive` exit 0 — caso NON classificato (né DENTRO né nei 3 FUORI): correct-or-absent violato vivo; (2) **tentative return types su interfacce interne** (`JsonSerializable::jsonSerialize` non tipato): Zend DEPRECATION exit 0, mai fatal — se fase 1 fatala qui rompe app reali; NON è nei FUORI dichiarati; (3) **lattice `never`/`void`/`mixed`** non nominato in regola 1/4 (dal vivo `never` covariante OK 3/3 — oggi innocuo, ma la regola non lo dice). I FUORI dichiarati (const 8.3, DNF, enum edge) restano correttamente fuori; (1) e (2) sono i buchi.

**Q4 — a_ds38 è un dente DISARMATO di default.** Il test esiste (vm/mod.rs:19510) e gira in parity-full (cargo test workspace), ma il loop all-pairs è gated su `uc_log_path().is_some()` (:19561) e NESSUNO in battery esporta `PHPR_UNIT_CACHE_LOG` al cargo test; **F16 arma SOLO a_ds26** (`--exact`, gate-lever-fixtures2.sh:418-419). In battery a_ds38 verifica solo `ev==2` (:19560): l'invariante TUTTE-le-coppie non è mai eseguito armato — PASS parzialmente vacuo, classe A-SK39.

**Emendamenti**: **A-DS42** — F16b: a_ds38 armato nel gate fixtures2 (env + `--exact` + pin «1 passed» + ≥2 main_evicted nel log). **A-DS43** — §3.3-ter: 6° observable `new C` pre-decl (phpr diverge da ENTRAMBI), negativo condizionale committato, meccanismo const-folding riqualificato; fixture in fixtures-ds40. **A-DS44** — contratto A-DS41: trait-abstract DENTRO (fatal byte-fedele) o FUORI per NOME; tentative-return-types = deprecation-lane, MAI fatal; lattice never/void/mixed nominato. **A-DS45** — prima campagna con uc_log armato: pair-guard su log di PRODUZIONE con ≥1 coppia (positivo non-selftest).

**Kill-switch**: **KS-DS-89-1** — battery citata come copertura A-DS38 con a_ds38 eseguito senza canale armato ⇒ invariante all-pairs ADVISORY. **KS-DS-89-2** — claim «phpr ≡ persist» citato senza la fixture `new`-pre-decl ⇒ claim VOID, entry parzialmente UNANCHORED. **KS-DS-89-3** — merge A-DS35 che fatala sui tentative-return-types o lascia i trait non classificati ⇒ REJECT (estende KS-DS-88-3).

Firmato: D. Stogov, sedia 8.

---

# VERBALE — B. Gregg, sedia 9, Concilio WP-89

**VERDETTO: PASS CONDIZIONATO.** Le cifre di attempt=3 sono pulite e i denti hanno morso davvero (l'incidente orfano è stato rifiutato dalla macchina, non da un occhio umano). Ma la disciplina attempt= ha retto solo sui RAW: gli artefatti di secondo ordine (verdict87.out, m87.done) sono mono-generazione, e la frase "VERDICT87 FAIL(67) agli atti" (MEASURE87_RESULTS.md r.14) è **falsa alla lettera**: quel file non esiste in NESSUN commit.
[NOTA DI ASSEMBLAGGIO, verificata a valle: il FAIL(67) È committato — `git show d0132fb --stat` elenca `php-rust/wp87-harness/verdict87.out` (create). La sonda della sedia 9 ha interrogato il path SENZA il prefisso GIT ROOT `php-rust/` — la stessa classe 🔵 di WP-86 ("git show path = GIT ROOT"). La refutazione di lettera decade; l'emendamento A-BG46 (artefatti per-attempt, VOID in-band nel ledger) resta valido nel merito e viene mantenuto.]

**Q1 — La disciplina ha retto a metà.** Retto: generazioni preservate nei filename (a1: 33 raw, a2: 3 file troncati alla prima fase, a3: 65 — KG-88-1 sui raw ✓); ledger APPEND-only con i 3 attempt e l'abort a2 IN-BAND (`m87.campaign.ledger` r.38: `abort: … MULTIPLE php-server processes`). NON retto: (a) l'attempt=1 chiude nel ledger con `phase=done esito=ok` (r.35) e **nessuna riga VOID viene mai appesa** — dal solo ledger a1 sembra una campagna sana; (b) `verdict87.sh` r.16 (`: > "$VOUT"`) riusa lo stesso filename ad ogni run [ma v. NOTA: la generazione FAIL è preservata nella storia git a d0132fb]; (c) `m87.done` è anch'esso same-name tra attempt e senza epoch. La VOID-ness di a1 vive nei commit e nella prosa MEASURE, non nel ledger — il morbo A-BG43 spostato di un livello.

**Q2 — seq/epoch reggono INTRA-attempt, non oltre.** Verificato su raw: `m87.slope.w1.r1.a3` seq=1 epoch=1785677119 → w2.r3 seq=8 → w4.r5 seq=20 → eager seq=21 → conc.r2 seq=32; monotonìa perfetta e epoch coincidenti riga-per-riga col ledger. Manca: (1) epoch a risoluzione 1 s con tie reali (conc.r2 e done entrambi 1785677261) — intra-secondo decide solo seq, che però NON copre gli eventi fuori campagna (battery, stamp, esecuzione del verdict, vita e morte dell'orfano: il suo census con count=6208 è piombato DENTRO slope.w1.r1.a1 senza epoch proprio); (2) nessun epoch di INIZIO fase né boot del server ⇒ l'interleave dell'orfano con le fasi non è ricostruibile in-band; (3) VIDENT non giudica campaign_sha dell'identity contro il .done (r.53 confronta solo mem_hash/git/attempt).

**Q3 — Violazioni di forma trovate.** (a) VDISJ conc (verdict87.out r.42/46): `padA net=18748620 floor_inc=1161206` — numeri NUDI, senza `B`, senza companion MiB, metrica non ripetuta sulla riga (Binding Rule 7/KB-88-2; VARMS invece è conforme: nomina arm E base). (b) `slack=` e `peak_footprint=` nelle righe min-of-R senza companion MiB. (c) **Fail-open sostanziale**: verdict87.sh r.117 scrive `${FP:-0}` nel file giudicato (r.118 emette `${FP:-NA}`) — un footprint mancante diventerebbe 0 e vincerebbe il min-of-R in silenzio. (d) VIDENT r.51 prende `tail -1` dell'identity: molteplicità non giudicata — proprio il raw a1 aveva l'identity interrata a riga 2 fra righe estranee.

**Q4 — Il segnale c'era già e nessun dente lo consumava**: ogni identity a1 dichiara `server_exit=143` (a3: `server_exit=0`) e i body a1 sono vuoti — VIDENT non guarda server_exit. Forma minima: (i) dente immediato `server_exit==0` al write del raw; (ii) identity v2 con `srv_pid=` (figlio reale del wrapper, non `$!`) + `srv_boot_epoch=`; (iii) il server risponde `X-Phpr-Pid`/`X-Phpr-Boot-Epoch`, verificati alla PRIMA request di ogni fase: l'orfano (pid=95843 vs pid fresco) sarebbe caduto alla request 1 della fase 1, con abort ledgerato come a2. Il boot-epoch copre il riciclo dei pid.

**Emendamenti**: **A-BG46** — VOID in-band: riga ledger terminale `attempt=N verdict=VOID reason=…` obbligatoria; verdict e done per-attempt (`verdict87.aN.out`, committati) — KG-88-1 esteso dai raw a TUTTI gli artefatti di campagna. **A-BG47** — identity contract v2 (srv_pid, srv_boot_epoch, server_exit, count identity==1, campaign_sha) GIUDICATO da VIDENT + check pid alla prima request per fase. **A-BG48** — companion mancante = FAIL del raw, mai 0/NA dentro un aggregato; unità+companion MiB anche sulle righe VDISJ conc.

**Kill-switch**: **KG-89-1** — attempt non-accettato senza riga VOID nel ledger E verdict.aN committato ⇒ campagna intera non-consumabile. **KG-89-2** — identity con server_exit≠0 o srv_pid non riconciliato con le righe census ⇒ raw VOID a prescindere. **KG-89-3** — companion metric degradata a default dentro min/LSQ ⇒ aggregato VOID.

*— B. Gregg, sedia 9*

---

---

## ⚖️ SINTESI DI CONVERGENZA (compilata a valle dei 9 verbali)

**Verdetto complessivo: 9× CONCORDO/PASS CON EMENDAMENTI — nessuna
opposizione.** Le CIFRE di attempt=3 e i ri-giudizi S-87.0 sono confermati a
ricomputo indipendente (Bak al byte su OVL/ABBA/slope/disjoint; Leijen sui
raw win=0; Gregg su seq/epoch e ledger; Matsakis/Hoare/Pedersen nel codice;
Stogov sull'oracle vivo). UNA refutazione di un verbale è stata a sua volta
refutata in assemblaggio: la sonda della sedia 9 sul FAIL(67) usava il path
senza prefisso GIT ROOT (`git show d0132fb --stat` CONTIENE
`php-rust/wp87-harness/verdict87.out`, ultima riga `== VERDICT87 FAIL(67) ==`)
— recidiva della lezione 🔵 WP-86; l'emendamento A-BG46 resta valido nel
merito. I colpi della tornata:

### Refutazioni convergenti (≥2 sedie o dall'oracle/raw)

1. **🔴 «phpr ≡ braccio persist» è FALSO per l'istanziazione pre-decl
   (Stogov, dall'oracle vivo)**: `new C` pre-decl (parent-later) fatala su
   plain E persist; phpr stampa ed esce 0 — l'hoisting di phpr è PIÙ AMPIO
   del persist; il 4° observable (C::K=7) sotto persist è probabilmente
   const-folding, non hoist → A-DS43 (6° observable + negativo condizionale
   + riqualifica del meccanismo), KS-DS-89-2.
2. **Il contratto A-DS41 ha DUE buchi provati dal vivo (Stogov)**:
   trait-abstract-method (Zend fatala, phpr no — non classificato) e
   tentative return types (Zend DEPRECATION exit 0 — fatalarci romperebbe
   app reali) → A-DS44, KS-DS-89-3.
3. **Il committed è una FUNZIONE A GRADINI da 64 KiB (Bak+Leijen,
   convergenza indipendente)**: i 4 min-of-R sono TUTTI multipli esatti di
   65.536 B; W=3 è bimodale al byte (Δ modi ~8,3 MiB); a W=4 ~89,3 MB vive
   FUORI dai bin visitati — la LSQ su W∈{1..4} è declassata ad ADVISORY;
   riconciliazione = slope W∈{4..16} con mode-census + righe `tag=mi_arena`
   in-band + modello committed(W)=a+b·W+c(W), banda KL-85-2 confrontata
   SOLO con b → A-BB55≡A-DL42, KB-89-1/2, KL-89-2.
4. **La finestra evidence-only del --same-rev è CIECA su tre superfici
   (Klabnik; + toolchain Hejlsberg)**: checker/verdict/campagna non sono
   oggetti di nessun dente; ledger+matrix forgiabili in un commit;
   `wp87-harness/*.out` entra nel corpus cifre senza essere oggetto (iv);
   `rust-toolchain.toml` fuori dal pathspec (i) e rustc campionato
   CWD-sensibile → A-SK50, A-AH46/47, KS-SK-89-1, KS-AH-89-1.
5. **KS-SK-88-3 violata nella LETTERA dentro verdict87.sh (Klabnik)**: i
   .log di time -l sono parsati con `tr -d '\0'` — le righe
   peak_memory_footprint sono VOID-a-rischio finché il canale non è
   REFUSE-o-DECLARED; più il fail-open `${FP:-0}` dentro il min-of-R
   (Gregg) → A-SK52, A-BG48, KS-SK-89-2, KG-89-3.
6. **VDISP giudica cardinalità, non UNICITÀ del thr-set (Pedersen)**: due
   righe thr=0 e thr=1 assente passerebbero — la classe del garbling
   S-86.0 → A-PP42, KS-PP-89-2; identity line senza count==1
   (Klabnik/Gregg) → A-SK51/A-BG47.
7. **Gli artefatti di SECONDO ordine sono mono-generazione (Gregg+Klabnik+
   Hejlsberg)**: verdict87.out/m87.done same-name fra attempt (le
   generazioni vivono solo nella storia git), la VOID-ness di a1 non è
   in-band nel ledger (`phase=done esito=ok`) → A-BG46/A-SK54/A-AH49,
   KG-89-1.
8. **L'orfano aveva UN segnale in-band mai consumato (Gregg)**:
   server_exit=143 su ogni identity a1 — VIDENT non lo guardava; identity
   contract v2 (srv_pid + srv_boot_epoch + server_exit==0 giudicati) +
   port-owner assert (Pedersen: il censimento per NOME non prova il
   LISTEN-owner) → A-BG47, A-PP43, KG-89-2, KS-PP-89-4.
9. **Bracci d'ambiente senza read-back = bracci MUTI (Leijen)**: VARMS
   eager delta 0 B ESATTO senza `mi_option_get` in-band non distingue
   "nessun effetto" da "opzione mai armata" → A-DL41, KL-89-1; VARMS senza
   gate ORD (Klabnik) → A-SK51.
10. **a_pp38 e a_ds38 sono denti FUORI dal perimetro ledgerato
    (Pedersen+Stogov)**: i test axum-server non girano in battery (solo
    CI); l'invariante all-pairs di a_ds38 è gated su un env che la battery
    non esporta (F16 arma solo a_ds26) → A-PP41, A-DS42, KS-PP-89-1,
    KS-DS-89-1.
11. **Scope «annidato» non falsificabile (Bak)**: l'inclusione dei simboli
    è di costruzione, non verificata — nested-guard macchina (subset per
    NOME + floor-collapse witness) come precondizione d'uso VERDICT-GRADE
    → A-BB54, KB-89-3.
12. **Surplus padA = 3 MiB ESATTO + residuo ~688/992 B (Bak)**: termine
    quantizzato; discriminatori senza A-BB50: warm-both-then-pair +
    stagger 20 ms, predizioni ex-ante → A-BB56.
13. **Sigilli residui (Hoare+Matsakis)**: grafia `.gate (` spazio
    pre-parentesi buca ENTRAMBI gli sweep; alias path-qualified e
    `use … as` fuori regex; terza copia del payload nel pin; putord
    opponibile solo per procedura (manca thr= o assert W==1); arm del
    probe manuale non-RAII; `.done` di gate-parity toccato anche a FAIL →
    A-TH44/45/46/47, A-MS40/41/42, KH89-1/2/3, KS-MS-89-1/2.
14. **Canale $PHPR_MI_STATS non atomico (Leijen)**: mai corpus → A-DL40,
    KL-89-4; il collect condiviso è QUANTIFICATO innocuo per il committed
    di processo (drena 344.064 B di phys, 0 B di commit) — la DECLARED
    DEVIATION regge; design87 va integrato con boundary pre-prologo
    (A-DL39) e semantica temporale dei delayed-free (A-BB50) → A-DL43.

### Ordine vincolante di apertura WP-88(sessione) (non rinegoziare)

1. **Sanatorie di forma e catalogo (nessuna campagna)**: A-DS43 (6°
   observable new-pre-decl + negativo condizionale + riqualifica
   const-folding nel catalogo, fixture committate) · A-SK52-forma
   (righe peak di verdict87/MEASURE87 ri-etichettate DECLARED-DEVIATION
   finché il canale .log non è REFUSE) · A-BG46-forma (annotazione VOID
   per-attempt append-only nel ledger campagna, MAI edit) · nota
   correttiva sintesi sedia 9 già agli atti qui.
2. **Fix strumenti verdetto/campagna**: A-SK51 (VARMS gate ORD +
   identity count==1) · A-PP42 (thr-set == {0..W-1}) · A-BG48 (companion
   mancante = FAIL, mai default) · A-DL41 (read-back bracci in-band) ·
   A-DL40 (MI_STATS non-corpus) · A-BG47 (identity contract v2:
   srv_pid/srv_boot_epoch/server_exit giudicati) · A-PP43 (port-owner
   assert + server_gone in run_conc + failpath kill sul figlio).
3. **Catena evidenza**: A-SK50 (allowlist del delta BREV..HEAD per
   --same-rev: checker/verdict/campagna byte-identici, ledger prefisso,
   matrix solo quello del .done) · A-AH46 (object_changed su GITPREFIX) ·
   A-AH47 (rustc -Vv in-repo + rust-toolchain.toml nel pathspec (i)) ·
   A-AH48 (snapshot matrix per NOME+sha, chiude il same-name swap) ·
   A-SK54/A-BG46 (verdict87.aN.out + done per-attempt) · A-AH49.
4. **Sigilli v6**: A-TH44/45/46/47 · A-MS40/41/42 · A-PP41 (gate battery
   axum-server ESEGUITO nel k/k) · A-PP45 · A-DS42 (F16b: a_ds38 armato).
5. **Misura (una campagna)**: A-BB55≡A-DL42 (slope W∈{4,8,12,16},
   mode-census, tag=mi_arena in-band, banda solo su b) · A-BB56
   (warm-both-then-pair + stagger, predizioni ex-ante) · A-DS45 (una fase
   con uc_log ARMATO: pair-guard su log di produzione).
6. **Delibere**: A-BB54 (nested-guard macchina come precondizione d'uso
   della promozione SCOPED) · A-DL43 (design87 integrato: boundary
   pre-prologo + delayed-free temporali) · A-PP44 (design parser
   dispatch-union).
7. **ROADMAP**: A-DS44 (contratto A-DS41 emendato: trait-abstract
   classificato, tentative-return-types = deprecation-lane, lattice
   never/void/mixed nominato) — l'implementazione A-DS35 parte SOLO a
   contratto emendato (KS-DS-88-3 + KS-DS-89-3).
8. Deferred invariati: A-MS27 · A-PP18/27 · A-AH38+dry-run; KS-DS-80-3
   invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH89-1 | claim su putord fra righe W>1 senza thr= in-band | claim VOID |
| KH89-2 | rename payload in QUALSIASI delle tre copie senza th41-positive stesso-commit | noprobe ADVISORY, campagne VOID |
| KH89-3 | grafia di mint nominata da audit senza sweep+decoy stesso-commit | sweep ADVISORY |
| KS-MS-89-1 | probe armato fuori dal costruttore unico (post-A-MS40) o early-return nel blocco armato | attribuzione VOID |
| KS-MS-89-2 | gate-parity consumato via .done anziché rc | PASS VOID |
| KS-MS-89-3 | lettore di main_program_net_clamped fuori dal dump census | cifra VOID |
| KS-SK-89-1 | --same-rev con delta contenente checker/verdict/ledger-rewrite/matrix non nominato | consumo VOID |
| KS-SK-89-2 | cifra da .log NUL-strippato senza declared-deviation in-band | riga VOID |
| KS-SK-89-3 | PASS advisory flushata con flag a monte assente dal gate | blocco VOID |
| KS-AH-89-1 | consumo con rust-toolchain.toml mutato o rustc campionato fuori-repo | consumo VOID |
| KS-AH-89-2 | matrix con ≠1 riga rustc= o sha archivio mutata in-window | battery VOID |
| KB-89-1 | regressione su ≤4 punti W con Δ non monotoni come slope verdict-grade | VOID |
| KB-89-2 | min-of-R senza mode-census con ≥2 modi byte-distinti | ADVISORY, mai pin |
| KB-89-3 | uso VERDICT-GRADE della scomposizione senza nested-guard A-BB54 | VOID → MODEL-GRADE |
| KS-PP-89-1 | eredità a_pp38 citata da battery che non ESEGUE il test | claim CI-only, sigillo declassato |
| KS-PP-89-2 | VDISP PASS con thr duplicati/mancanti | mappa VOID, blocchi a valle VOID |
| KS-PP-89-3 | attribuzione union senza parser dispatch-union consumato | resta envelope |
| KS-PP-89-4 | "server nostro" su porta fissa senza port-owner assert | assunzione, mai identità |
| KL-89-1 | braccio d'ambiente senza read-back in-band | braccio MUTO, delta VOID |
| KL-89-2 | slope committed fuori regime senza termine di granularità nominato | cifra non confrontabile |
| KL-89-3 | cifra per-worker-heap post-A-DL39 senza boundary pre-prologo | VOID |
| KL-89-4 | righe da $PHPR_MI_STATS citate come corpus | escluse |
| KS-DS-89-1 | copertura A-DS38 citata con a_ds38 disarmato | invariante ADVISORY |
| KS-DS-89-2 | claim «phpr ≡ persist» senza fixture new-pre-decl | claim VOID, entry UNANCHORED |
| KS-DS-89-3 | merge A-DS35 che fatala sui tentative-return-types o trait non classificati | REJECT |
| KG-89-1 | attempt non-accettato senza riga VOID nel ledger E verdict.aN committato | campagna non-consumabile |
| KG-89-2 | identity con server_exit≠0 o srv_pid non riconciliato | raw VOID |
| KG-89-3 | companion degradata a default dentro min/LSQ | aggregato VOID |
