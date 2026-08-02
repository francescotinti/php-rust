# COUNCIL_WP90_REVIEWS.md — Concilio a 9 sedie su S-88.0 (due refutazioni pre-esecuzione + campagna measure88 + sigilli v6) + programma WP-89(sessione)

**Convocato**: 2026-08-02, chiusura S-88.0 (regola utente 2026-07-26: 9 sedie
dopo OGNI sessione, mandato REFUTARE). Atti: NEXT_SESSION_WORDPRESS.md,
sessions/WP_SESSION_88.md, wp88-harness/MEASURE88_RESULTS.md +
verdict88.a1.g1.out + verdict88.sh + measure88-campaign.sh +
battery-88pre.sh + gate-axum-tests.sh + design88.md, wp87-harness/design87.md
(integrato A-DL43), wp89-harness/COUNCIL_WP89_REVIEWS.md (gli ordini
eseguiti), codice e raw dei rispettivi perimetri. Verbali VINCOLANTI per il
design WP-89(sessione).

## ⚖️ SINTESI DI CONVERGENZA

(compilata in coda ai verbali — v. fondo file)

---

## VERBALI INTEGRALI

# VERBALE — Hoare, sedia 1, Concilio WP-90

**VERDETTO: CON EMENDAMENTI** — A-TH44/45/46/47 eseguiti nella lettera e verificati nel codice; ma una dichiarazione (A-TH47) è INESATTA rispetto alla semantica std, e due sigilli hanno finestre residue nominabili. Refuto dove va refutato.

**Q1 — A-TH44: coperte, decoy mordenti; grafie che ANCORA sfuggono, PER NOME.** Verificato pins:1057-1085: decoy 2/2/2 con selftest che abortisce se non mordono (:1070-1072), sweep workspace su tre regex (:1076). Sfuggono: (1) **raw identifier** `retain.r#production_gate()` — ogni regex esige il nome adiacente al punto/spazi, `r#` interposto buca sede (:333-336), v5 (:1011) e v6 (:1067) senza alterare i conteggi ==N dei siti legali; (2) **call spezzata su due righe** (`u.vm_gate`⏎`(x)`) — tutti gli sweep sono line-based; l'awk multiline (:1033) copre solo `CachedUnit$`+`{`, mai i metodi; (3) **commento interposto** `retain./*x*/production_gate()`; (4) **path con `::` spaziato o globale** `type CU = vm :: CachedUnit;` / `= ::vm::CachedUnit;` — la regex pq (:1068) esige `ident::` adiacente all'uguale; (5) **use a gruppo** `use crate::vm::{CachedUnit as CU};` — la regex ua (:1069) esige `::` adiacente al tipo, `{` la buca. Carrier-a-valore resta APERTO come dichiarato (KH88-2/A-MS27).

**Q2 — A-TH45: chiude l'echo, NON il decoy-grep; terza copia dichiarata: sì.** Il pin (pins:1094) esige il literal su riga `grep|strings` — l'`echo` nudo ora FALLISCE. Ma il pin non è scoped a `probe_in()` (noprobe:36-41): un commit che rinomina l'argomento del grep di detection e lascia altrove un `grep -q 'payload' /dev/null` decoy passa. Il positivo comportamentale che lo morderebbe (tainted2.bin, noprobe:49-57) vive nel `--selftest`, che NON è nella catena ledgerata: la campagna invoca noprobe solo sul binario (measure88-campaign.sh:123). Terza copia: dichiarata nel coupling comment vm/mod.rs:537-541 (nomina tutte e tre; il literal stesso resta ==1, :544, pin :970). KH89-2 è opponibile da audit umano, ma il rename a tre vie coordinato passa ancora la meccanica: «th41-positive rieseguito same-commit» non ha dente macchina.

**Q3 — A-TH46: precondizione ESEGUITA in a_ds38, sì — ma solo lì, e solo ad canale armato.** L'assert (vm/mod.rs:19617-19621) è eseguito su dati reali, il buffer letto è thread_local (:19576-19577) ⇒ W==1 per costruzione sulle righe viste, e la monotonia stretta morde su wrap/reorder. Due limiti: (a) tutto vive sotto `if uc_log_path().is_some()` (:19575) — in `cargo test` nudo la precondizione NON si esegue (vacua); è eseguita solo nella battery armata F16/F16b (ledgerata: accettabile, ma va detto); (b) **a_ds36 resta senza**: il join same-putord (:19488-19503) non ha né monotonia né asserzione W==1 — KH89-1 è soddisfatta per a_ds38, NON per «i consumer in-cargo» al plurale.

**Q4 — A-TH47: la dichiarazione NON è esatta. MI OPPONGO alla lettera.** Il doc (vm/mod.rs:16534-16541) afferma che `LocalKey::with` «PANICS if the thread's TLS has already been destroyed». Ma `UC_EMIT_GUARD`/`UC_PUT_ORD` sono `const { Cell::new(…) }` di tipi SENZA Drop (:16516-16531): std non registra alcun distruttore per queste chiavi e `with` panica solo a distruttore running/run — per queste chiavi MAI. Un put da distruttore TLS supera `arm()` (:16543-16551) e panica invece al primo TLS CON distruttore (UNIT_CACHE/UC_STATS/UC_LOG_BUF, RefCell) — a guardia ARMATA, parametri oltre il rebind; e il Drop della guardia in unwind (:16553-16556) NON può panicare ⇒ l'abort dichiarato è irraggiungibile per quella via. L'esito resta fail-fast (Regola 4, via il panic del RefCell-TLS), ma la SEDE e il meccanismo dichiarati sono sbagliati: una dichiarazione inesatta è peggio di nessuna.

**Emendamenti:**
- **A-TH48**: pins invoca `gate-binary-noprobe.sh --selftest` (positivo comportamentale in catena) e scopa il pin A-TH45 alla funzione `probe_in` (finestra awk).
- **A-TH49**: sweep esteso alle cinque grafie di Q1 (`r#`, call multiline, commento interposto, `::` spaziato/globale, use-group) con decoy che mordono nello stesso commit.
- **A-TH50**: correggere la dichiarazione A-TH47 — nominare i TLS con distruttore come vera sede del fail-fast; niente claim di panic su chiavi no-Drop const.
- **A-TH51**: assert monotonia+W==1 anche in a_ds36; putord-pair-guard.pl dichiara o verifica il regime W del log consumato.

**Kill-switch:**
- **KH90-1**: rename del payload in qualsiasi copia senza noprobe `--selftest` PASS nello stesso commit ⇒ campagne VOID (indurisce KH89-2 a macchina).
- **KH90-2**: claim di fail-fast TLS-teardown fondato sulla dichiarazione attuale prima di A-TH50 ⇒ claim VOID.
- **KH90-3**: consumer putord citato come «precondizione eseguita» da un run a canale disarmato ⇒ citazione VOID.

Firmato: T. Hoare, sedia 1.

---

# VERBALE — Matsakis, sedia 2, Concilio WP-90

**VERDETTO: CON EMENDAMENTI.** A-MS40/41/42 eseguiti nella lettera e verificati nel codice; l'attribuzione probe è corretta per costruzione su tutti i path reali. Ma il sigillo è ancora di GREP dove potrebbe essere di TIPO: il flag è `pub` e scrivibile da fuori modulo, la belt ha grafie eludibili nominabili, e `.done` resta protetto solo da procedura.

**Q1 — TIENE; il drop esplicito è legale e qui non-portante.** `ProbeWindow` (worker_pool.rs:84-97): ctor unico `arm()` :87-90, `Drop`→`set(false)` :93-97. Sito d'uso: arm :369, `catch_unwind` :370-388, `drop(probe_window)` :389, abort :394. Ordine corretto: il hook std gira SUL thread panicante PRIMA dell'unwind ⇒ per un panic del probe legge `true` (il Drop scatta solo DOPO il ritorno di catch_unwind) — nessuna menzogna nel path principale. `drop(probe_window)` è un move legale; ed è ridondante in senso buono: senza di esso il guard morirebbe a fine closure, DOPO l'abort :394 — con flag ancora true all'abort, esito identico (abort, nessun panic ulteriore). Path dove il flag può mentire, per NOME, entrambi TLS-teardown-only: (a) panic DENTRO `arm()` prima di `set(true)` (`LocalKey::with` a TLS distrutta) ⇒ flag false, strumento attribuito al worker; (b) panic dentro il Drop ⇒ panic fresco post-catch: il hook stampa strumento (flag ancora true), l'outer :398-403 stampa "worker panicked" — due righe contraddittorie prima dell'abort. La finestra TLS è dichiarata per UcEmitGuard (A-TH47) ma NON nel doc di ProbeWindow. Fragilità reale non-TLS: `let _ = ProbeWindow::arm();` droppa SUBITO — finestra vuota, probe attribuito al worker; la belt non lo vede (siti restano 2) e manca `#[must_use]`.

**Q2 — ELUDIBILE, con grafie nominate.** Belt a gate-lever-pins.sh:1104-1115: conta `CENSUS_PROBE_ACTIVE.with(|f| f.set(` ==2 + contesto awk ctor (w=3) e Drop (w=4) — fail-closed su riformattazioni (bene). Elusioni: (1) **`CENSUS_PROBE_ACTIVE.set(true)`** — `LocalKey<Cell<bool>>::set` esiste in std: bypassa `.with` per intero, nset resta 2, PASS col terzo sito vivo; (2) closure rinominata `.with(|g| g.set(true))`; (3) `.with(|f| f.replace(true))` / `f.update`; (4) split multilinea (`f.set(` a capo); (5) **scope**: la belt scansiona SOLO worker_pool.rs, ma il thread_local è `pub static` (:68-71) — un armatore in main.rs o altrove non è nemmeno guardato. Il fix giusto non è più regex: flag PRIVATO al modulo, esportare solo `census_probe_active()` (lettura) + `ProbeWindow` — ogni scrittore esterno muore a COMPILAZIONE.

**Q3 — NESSUN UB; parse fail-visible; un residuo di scoping.** memcensus.rs:1022-1038: `counter`/`count3` sono chiusure `Fn` che prendono `&json` (String posseduta, :1012-1013 — `jbuf` non è più borrowed dopo `from_utf8_lossy(...).replace(...)`) — borrow condivisi coesistenti, zero aliasing. Collisioni chiavi: i pattern sono quote-ancorati (`"committed": {`, :1029; `"reset": `, :1023) — in `"page_committed"` il carattere prima di `committed` è `_`, non `"` ⇒ NON matcha; idem per tutte le coppie del set (:1040, :1051-1052). Ordine di ricerca: ogni chiave riparte da inizio stringa (find PRIMA-occorrenza GLOBALE, non scoped alla sezione arena) — se una futura versione mimalloc annidasse una chiave omonima in una sezione precedente, il valore estratto sarebbe quello sbagliato SENZA FAIL; mitigato dalla riga authority `mi_arena_json` (:1018-1020). Troncamenti/formato senza spazio dopo `:` ⇒ `parse=FAILED` in-band (:1045-1047, :1058-1060) — fail-visible, corretto.

**Q4 — Basta per il consumatore ATTUALE, non per i futuri.** Dichiarazione a gate-parity-83p1.sh:75-79, esito = `exit $FAILS` :81; la battery consuma il rc (battery-88pre.sh:40-45), non il `.done`. Ma il touch resta incondizionato (:80) e nessun dente a macchina impedisce un lettore futuro: KS-MS-89-2 è procedura. Sanatoria minima: `.done` porti l'esito DENTRO (`rc=$FAILS git=…`) — così anche il consumatore sbagliato legge la verità.

**EMENDAMENTI**
- **A-MS43**: flag privato al modulo (drop del `pub` su CENSUS_PROBE_ACTIVE); scrittori esterni = errore di compilazione; belt ridotta a conferma.
- **A-MS44**: `#[must_use = "bind the window: let _w = ProbeWindow::arm()"]` su ProbeWindow; doc della finestra TLS-teardown (classe A-TH47) nel tipo.
- **A-MS45**: `.done` di gate-parity con `rc=` in-band; belt estesa alle grafie `.set(`/`.replace(`/`.update(` su tutto il crate.

**KILL-SWITCH**
- **KS-MS-90-1**: attribuzione citata da un binario dove CENSUS_PROBE_ACTIVE è scrivibile fuori dal modulo del ctor ⇒ attribuzione ADVISORY.
- **KS-MS-90-2**: cifra `tag=mi_arena` estratta citata senza la riga authority `mi_arena_json` nello stesso win ⇒ cifra VOID (il parser è first-occurrence, non scoped).
- **KS-MS-90-3**: `ProbeWindow::arm()` non legato a binding vivo fino a valle del probe ⇒ finestra vuota, run VOID per attribuzione.

*Niko Matsakis — la belt conta le grafie di ieri; la visibilità le vieta tutte, comprese quelle di domani.*

---

# VERBALE — Klabnik, sedia 3, Concilio WP-90

**VERDETTO: CON EMENDAMENTI.** VERDICT88 PASS (a1.g1) regge come protocollo; i miei ordini WP-89 (A-SK50/51/52/54) sono nel codice e la campagna li ha consumati. Ma ho fatto MORDERE due forge dal vivo sul gate cifre, e A-SK53 è stato PERSO dalla sintesi (vive solo nel mio verbale WP-89 — mai eseguito).

**Q1 — A-SK50.** Tre esiti, due negativi. **(a) CONFERMATO A MACCHINA**: `gate-measure-cifre.sh` legge il corpus dal **WORKING TREE** (`bsd_glob` + `open`, :121-145) — "committed" vive solo nel commento. Ho creato `measure-out/m88.zzforge.tmp` NON committato con `committed=987654321`: la cifra fabbricata nel doc è passata (RC=0, poi FAIL=1 senza forge). L'allowlist A-SK50 è irrilevante qui: il forge non entra nemmeno nel delta. **(b)** matrix forgiato pre-stamp: il vettore-append resta strutturale — un 4-field stamp FORGIATO appeso in-window passa il prefix-check (l'append è legale by-design) e il matrix da esso nominato passa BAD_MTX (:275); l'unico costo residuo è fabbricare OUT con la riga PASS ancorata. **(c)** `${BL_NEW#"$BL_OLD"}` in bash 3.2: ESEGUITO — quoting corretto (pattern letterale, glob-metachar in OLD non espansi), append/rewrite giudicati bene, MA **l'estensione dell'ultima riga passa come append** (remainder non-vuoto che non inizia con `\n`): `rev=abc…` → `rev=abc…X` è un rewrite non catturato.

**Q2 — verdict88.sh.** Flag: catena corretta. VWARM senza ARENA_CLEAN è **GIUSTO per perimetro**: VARENA gira solo sui `SLOPE_RAWS` (:155) e VWARM non consuma righe `mi_arena`. Le `eval DA_${m}_${r}` sono **sicure sotto `set -u`**: ogni path che salta una eval incrementa `bf`, e l'aggregato (:296-305) è guardato da `bf==0`. `cmp` pointer↔suffixed: sufficiente per la COERENZA (byte-equality; lo sha non aggiunge nulla fra due file locali), **insufficiente per la PROVENIENZA**: il verdict legge `.done` e raw dal working tree, mai contro HEAD, e **non ri-verifica `mem_hash` contro il matrix COMMITTED** (il tether KS-AH-87-1 vive solo nella campagna, :121-122). REFUTAZIONE nuova: **`spans=` è riportato ma MAI giudicato** — `OV` può valere INVALID/OVERLAP su concstag e VWARM PASSA lo stesso; il «CONTROLLO POSITIVO PERFETTO zero-swallow» di MEASURE88 poggia su NO-OVERLAP che nessun dente esige.

**Q3 — generazione gG.** Nessuno la governa. G = primo slot libero (:52); un g2 a HEAD successivo (verdict88.sh eventualmente EDITATO, raw eventualmente ritoccati — working tree, vedi Q2) siederebbe accanto a g1 con pari autorità; il gate cifre globba `wp88-harness/*.out` — TUTTE le generazioni alimentano il corpus, quindi un doc che cita g1 refutato da g2 resta verde. Serve un dente: supersede + identità del giudice.

**Q4 — MEASURE88.** Gate eseguito: **PASS**, companion tutti verificati a mano da me sui punti caldi (141,44/220,25/19,45/24,68 ✓). MA due cecità: **(i) MORSO DAL VIVO**: `[derivata]` è a scope di RIGA (:223) — la riga fabbricata `W=4 999.948.288 B = 953,56 MiB [derivata: companion /1048576]` PASSA (RC=0): il companion auto-coerente si verifica da solo e il token MISURATO sfugge al corpus. Le righe chiave 41-44 del doc usano esattamente questo pattern: le loro cifre sono in corpus per fortuna, non per macchina. **(ii)** il `±5%` di riga 56 sfugge perché la riga fisica non porta token-unità — è la classe A-SK53 mai implementata.

**Emendamenti**
- **A-SK55**: corpus cifre COMMITTED-only (`git ls-files`/`git show HEAD:`), mai working tree.
- **A-SK56**: `[derivata]` a scope di FIGURA: ogni token `N B` misurato resta corpus-bound anche su riga taggata.
- **A-SK57**: prefix-check a granularità di RIGA (remainder vuoto o inizia con `\n`).
- **A-SK58**: VWARM giudica `spans`: stag⇒NO-OVERLAP, base/warm⇒OVERLAP, INVALID⇒FAIL.
- **A-SK59**: verdict ri-verifica mem_hash contro matrix committed; header con sha del giudice; doc cita la generazione MASSIMA per attempt.
- **A-SK53-bis**: ri-emesso tal quale (perso dalla sintesi WP-89 — la sintesi dichiari i dropped per NOME).

**Kill-switch**
- **KS-SK-90-1**: cifra legalizzata da file measure-out non raggiungibile da HEAD ⇒ PASS del gate cifre VOID.
- **KS-SK-90-2**: figura misurata esentata dal corpus via `[derivata]` di riga ⇒ riga VOID.
- **KS-SK-90-3**: doc che cita gN mentre esiste gM>N divergente sullo stesso attempt ⇒ claim VOID.

*— Klabnik, sedia 3. Due forge morsi dal vivo valgono più di dieci letture: il gate che dice "committed" e legge il working tree non è un gate, è una preghiera.*

---

# VERBALE — Hejlsberg, sedia 4, Concilio WP-90

**VERDETTO: PASS CON EMENDAMENTI.** Le quattro superfici A-AH46/47/48/49 reggono la refutazione sul punto capitale (fail-closed confermato a macchina); trovati tre difetti reali: dente A-AH48 estremi-only, battery FALLITE fuori da ogni ledger, giudice del verdict non ancorato per sha.

**Q1 — A-AH47: nessuna asimmetria che ACCETTA, una che DIAGNOSTICA male.** Il `|| echo unknown` del recorder (`gate-feature-matrix.sh:83`) è codice quasi-morto: se rustc manca, la pipeline `rustc -Vv | tr` esce con lo status di `tr` (0 su input vuoto) → il fallback NON scatta e la riga diventa `rustc=` VUOTA. Il comparatore però rifiuta comunque: `NRUSTC==1` passa (la riga `^rustc=` c'è), ma `MTX_RUSTC` vuoto innesca il FAIL A-AH44 — con diagnosi ERRATA ("no rustc= header": il header c'è, è il valore che manca). Fail-closed preservato, messaggio bugiardo. `unknown` può nascere solo da `cd` fallito → mismatch → FAIL per nome. **NRUSTC copre l'archivio COMMITTATO** (`git show "HEAD:${GITPREFIX}${MTX_REL}"`, battery-equivalence.sh:198), non il working — corretto, l'ancora è HEAD. Verificato vivo: NRUSTC=1 sull'archivio 0b83f2b, equality recorder↔comparatore OK oggi (rustc 1.96.0, host aarch64 incluso, `;`-joined identico).

**Q2 — A-AH48: swap reale catturato, ma solo fra ESTREMI; spazi = FAIL spurio, mai vacuo.** Test eseguito: `join -j 2` spezza sul PRIMO token del nome — due file `foo bar.log`/`foo baz.log` con sha IDENTICHE producono cross-pair e SWAPPED non-vuoto → FALSE FAIL (rumoroso, non silente; direzione giusta). I nomi sono senza spazi per costruzione (`feature-matrix.$REV.$STAMP.log`), quindi oggi innocuo. Lo swap reale su nome persistente è catturato (test: `aaa→zzz swapped.log` → FAIL). **Limite vero**: snapshot before/after agli estremi della battery — uno swap-in + restore in-window è invisibile. Nessun gate rilegge l'archivio in-window (rischio teorico), ma il dente va dichiarato per quello che è: estremi-only.

**Q3 — A-AH46: prefix errato = RUMOROSO, la vacuità di (iv) è schermata dal dente stamp.** Prefix vivo `php-rust/` (`rev-parse --show-prefix`, calcolato da `$REPO` non dal CWD — classe A-AH43 chiusa). Dimostrato: `git show HEAD:SBAGLIATO/...` → `fatal: path does not exist` → il grep A-SK41/A-SK46 sulla stringa vuota FAILa per nome. `object_changed` da solo con prefix errato SAREBBE vacuo (grep senza match = "non cambiato"), ma condivide la STESSA variabile `GITPREFIX` del dente stamp che morde prima → nessun percorso arriva a PASS. Nel `--same-rev`, prefix errato rende BAD_DELTA l'intero delta (A-SK50) — rumoroso anche lì. Nessun literal `php-rust/` residuo nel codice vivo.

**Q4 — catena m88: solida da 0b83f2b al verdict, ma il ledger NON racconta da solo i due fail pre-battery.** Verificato: stamp 4-field a HEAD + matrix committata nello STESSO commit 202f8b1; m88.a1.done/ledger/raws tutti tracked (27f1dae); riga `phase=verdict esito=PASS` in-band (A-AH49 rispettata); campaign_sha=8cdf64506001c1fa combacia con measure88-campaign.sh. **Buchi**: (1) battery-stamps.ledger appende SOLO su PASS (battery-88pre.sh:103-105) — il fail F16b a 94661d8 vive solo nel commit message di 0b83f2b + archivio orfano; il REFUSE porcelain a b4ad60b non lascia NESSUN artefatto a macchina. Asimmetria con KG-88-1: le campagne hanno VOID in-band, le battery no. (2) La riga verdict porta `campaign_sha=judge` (verdict88.sh:344/348) — placeholder che riusa il campo con semantica diversa: il GIUDICE non è identificato per sha.

**Emendamenti**:
- **A-AH50**: battery-attempts ledger append-only — OGNI tentativo (PASS/FAIL/REFUSE, incluso il porcelain-refuse) lascia riga in-band con motivo per NOME.
- **A-AH51**: riga verdict con `judge_sha=sha256(verdict.sh)[0:16]`; vietato riusare `campaign_sha` come marcatore.
- **A-AH52**: A-AH48 dichiara il limite estremi-only nel commento; guard esplicita "nome archivio con spazio ⇒ FAIL per NOME" (oggi FAIL spurio da cross-join).
- **A-AH53**: recorder — il fail di rustc non deve essere mascherato da `tr` (check esplicito, mai riga `rustc=` vuota); diagnosi A-AH44 corretta ("valore vuoto" ≠ "header assente").

**Kill-switch**:
- **KS-AH-90-1**: tentativo di battery terminato senza riga in-band ⇒ il PASS successivo sulla stessa rev NON consumabile.
- **KS-AH-90-2**: riga `phase=verdict` senza giudice identificato per sha ⇒ verdetto non citabile come verdict-grade.

— *Anders Hejlsberg, sedia 4 (compilatori/toolchain, catene di identità/evidenza, dedup)*

---

# VERBALE — Bak, sedia 5, Concilio WP-90

**VERDETTO: PASS CON EMENDAMENTI.** Ogni cifra del verdetto riproduce al byte dai raw; ma il claim «Delta MONOTONI» è vacuo sul MARGINALE, e la banda KL-85-2 va RITIRATA, non ri-derivata.

**Q1 — Ricomputo ESATTO.** Committed win=0 riletti dai 20 raw (`tag=mi_proc win=0 commit=`): combaciano riga per riga col verdetto; `mi_arena key=committed current` == commit AL BYTE su 20/20, arena_count=1, `purge_delay val=0` presente. Residuo 64 KiB = **0 su TUTTI i 20 run**, non solo sui dominanti (più forte del verdetto). Mode-census confermato: W=4 {148.307.968×2, 150.339.584×2, 150.011.904×1}, W=8 dominante 230.948.864×2, W=12 **5 modi tutti ×1**, W=16 399.769.600×2. LSQ dominanti: **b=21.195.980,8 → 21.195.981 B/worker, a=63.897.600** ✓; b_min=20.399.718,4 ✓. Sullo stimatore: a W=12 «dominante con tie→min» degenera in min-of-R (nessun modo esiste); a W=16 il dominante è il MASSIMO dei run. La mediana-dei-run dà b=21.100.954 — i tre stimatori stanno tutti in [20,40M; 21,20M]: la NAMED-DEVIATION da KL-85-2 è **robusta allo stimatore**, quindi non serve cambiarlo, serve dichiararne l'incertezza: residui LSQ {−373.555; −2.516.582; +6.153.830; −3.263.693}, **se(b)=586.270 B → b=21,20M ± 1,17M (2σ)**.

**Q2 — La monotonia degli assoluti NON basta.** I marginali per-worker sono 20.660.224 / 23.363.584 / 18.841.600 B = 0,975·b / 1,102·b / 0,889·b: spread ±11%, e la sequenza dei Δ NON è monotona (sale poi scende). KB-89-1, come l'ho scritta, è stata soddisfatta nella lettura debole (Δ>0). Serve un criterio del marginale: ogni Δ/ΔW ∈ b·(1±δ) con δ NOMINATO ex-ante (qui δ=0,15 copre; δ=0,10 avrebbe MORSO). KL-85-2: **RITIRARE** come banda cross-protocollo — nasce a W=10 steady su un fixture con altra metrica di forma; anche l'IC 2σ di b [20,02M; 22,37M] non la sfiora. Ri-derivarla solo PER-PROTOCOLLO, mai come costante universale.

**Q3 — Tutto reale nei raw.** Cal 7.801.102/1.161.206 B, 4/4 al byte. concbase: dA=314 B (2/2, firma-inghiottimento ora su A), dB=1.560.464 / 3.176.029 B — inversione e instabilità CONFERMATE. concstag: net==cal **AL BYTE** (7.801.102, 2/2, entrambi i lati), spans disgiunti ([0;14.145] vs [33.772;47.967]). concwarm: floor 1.161.206→164.368 (2/2). Dai `lower_span` ricavo net=da−df per-tid: in concbase r1 il surplus sta su tid12 (t0=2), r2 su tid12 (t0=0) — l'identità del thread non discrimina a questa granularità. **Esperimento minimo**: stagger-sweep asimmetrico INVERTITO — coppie con Δt∈{0,1,2,5,10,20} ms e ordine A-prima/B-prima alternato, + swap-fixture control; predizione ex-ante: il surplus segue il lato con finestra APERTA durante il first-touch dell'altro (ORDINE, non fixture). Coi soli process-counters resta driver-discrimination (KB-88-1), mai cifra.

**Q4 — A-BB54 ha tre buchi.** (i) La regex `function\s+(\w+)` è cieca alle **closure anonime** (vivono in m.closures, A-DS28): un piccolo con una closure extra passa il subset senza essere annidato. (ii) I nomi di funzione PHP sono **case-insensitive**: confronto senza normalizzazione lowercase dà falsi refuse/pass. (iii) Il witness usa costanti di collaudo (996.838 B) come oracolo: per coppie NUOVE il criterio (b) è **non falsificabile** — il Δfloor atteso va derivato A MACCHINA dalla calibrazione di L, e ε=1.040 B non è scale-invariante (va per classe-di-coppia o scalato sui fns).

**Emendamenti**: **A-BB57** slope verdict-grade solo con fascia del marginale (δ ex-ante) e b±2σ in-band; **A-BB58** W con modes==R (census non-informativo, cfr. W=12 5/5) ⇒ raddoppio R o punto ADVISORY; **A-BB59** stagger-sweep invertito + swap-fixture per il surplus; **A-BB60** nested-guard: lowercase, closure-subset per hash del corpo, witness derivato dalla calibrazione di L.

**Kill-switch**: **KB-90-1** slope citata senza fascia δ del marginale o senza 2σ ⇒ ADVISORY, mai pin; **KB-90-2** KL-85-2 citata contro protocolli con metrica/forma diverse ⇒ VOID (banda RITIRATA); **KB-90-3** nested-guard consumato su coppia con closure anonime senza confronto dei corpi ⇒ NESTED non provato, scomposizione MODEL-GRADE.

— Bak, sedia 5

---

# VERBALE — Pedersen, sedia 6, Concilio WP-90

**VERDETTO: CON EMENDAMENTI** — i quattro ordini eseguiti nella lettera e verificati nel codice (A-PP41 in battery, contato; A-PP42 thr-set esatto; A-PP43 in ogni run; A-PP45 seconda richiesta). Ma il fail-path del port-owner assert lascia il server VIVO, e l'osservabile publish è ancora una ripetizione di path, non un publish.

**Q1 — A-PP43: assert giusto, fail-path bucato.** Port-owner c'è (measure88-campaign.sh:164-166, `lsof -t` LISTEN, multi-pid ⇒ FAIL fail-closed; SO_REUSEPORT co-listener cadrebbe su `sort -u`≠srv); `assert_server_gone` in run_conc c'è (:297) e conc non è più l'ultimo run (uclog:338 chiude). Buchi PER NOME: (1) **BUCO-SUBSHELL**: `SRV=$(assert_single_server …) || exit 1` (:199/:235/:266/:323) — il `fail()` gira nella command-substitution: ledger VOID sì, ma l'uscita via `|| exit 1` NON uccide né SRV né DPID ⇒ proprio il fail del port-owner produce l'orfano che dovrebbe prevenire. (2) **Body-mismatch usa `kill -TERM "$SRV"` solo** (:209/:239/:275/:277/:290-291/:330): bersaglio corretto (il FIGLIO, lezione a1 applicata), ma nessun `assert_server_gone` a valle — un TERM ignorato sopravvive fino al pre-flight della campagna SUCCESSIVA. (3) **TOCTOU residuo**: assert one-shot post-wait_up; a run_arm W=16 seguono 1600 richieste senza re-assert — mitigato da body-oracle + server_exit (VIDENT), non chiuso. lsof lento: al più FAIL spurio (direzione fail-closed), mai PASS falso.

**Q2 — A-PP42: la mappa regge, la COMPOSIZIONE è assunta.** VDISP (verdict88.sh:109-117) esige set esatto {0..W-1} (thr duplicato ⇒ "0,0,1"≠"0,1" FAIL — KS-PP-89-2 chiusa) e count==nreq/W. Warm 2+2 e stag 1+1 reggono perché i curl warm sono SERIALIZZATI (:274-277) e il contatore round-robin è condiviso: ogni retry/richiesta estranea sposta un count ⇒ FAIL — nessuna assunzione che un retry rompa in silenzio (curl non ritenta; -m 10 abortito ⇒ body≠oracle ⇒ fail). MA il verdetto non verifica QUALE fixture su quale thr: la lettura "hello+pad per worker" (:270-272) poggia sul round-robin da next_worker=0, assunzione vera ma NON nominata nel verdetto; `worker_dispatch` non porta path=. Il caso hello+hello/pad+pad cade solo per effetto collaterale (VORD entries/mains, VWARM atid≠btid) — copertura per fortuna, non per disegno.

**Q3 — A-PP45: il publish NON è ancora reale.** Il dente (worker_pool.rs:1400-1425) cattura l'inserter che pubblica garbage SOTTO LA STESSA KEY (body2 cambia) e la key-drift (miss ⇒ ricalcolo… identico, vedi sotto). NON cattura: (a) **publish soppresso** — cache mai popolata, fatal ricalcolato deterministicamente a ogni richiesta: status2=500 e byte-stabilità sono garantiti dal determinismo, il test passa; (b) garbage pubblicato sotto key diversa mai consultata; (c) hit/put invertiti. La byte-uguaglianza di un output deterministico non discrimina "servito dal published" da "ricomputato": è ripetizione del path, con in più il solo vincolo di stabilità. Il vero discriminatore è il contatore: armare `PHPR_UNIT_CACHE_LOG` nel test (pattern F16b/A-DS42, il canale esiste già) e asserire put=1/hit=1 dal log del worker.

**Q4 — A-PP41: pin per nome sì, non-vacuità annacquata.** Il rename di a_pp38 FALLA il grep (:34-38, fail-closed). MA: (1) NPASS = `sort -rn | head -1` su TUTTE le righe `test result: ok` (:28) — prende il MASSIMO fra binari (unit+doctest): una suite lib potata a 1 test passa se i doctest arrivano a 5; (2) soglia ≥5 arbitraria, nessun conteggio pinnato; (3) il pin prova che il test PASSA, non che conservi i denti A-PP45 — un gutting degli assert passa per nome (classe A-DS30, nessun mutante).

**Emendamenti:**
- **A-PP46**: fail-path sano — kill_server_tree PRIMA di `exit 1` sul ramo `|| exit 1` di assert_single_server; body-mismatch seguito da server-gone best-effort ledgerato.
- **A-PP47**: a_pp38 arma PHPR_UNIT_CACHE_LOG e asserisce put=1/hit=1 — publish a contatore, non a ripetizione.
- **A-PP48**: gate-axum: NPASS dalla riga unit-suite SPECIFICA (non max multi-binario) + conteggio atteso pinnato; lever-pins conta gli assert A-PP45 nel sorgente.
- **A-PP49**: `worker_dispatch` con path= in-band (o cross-check fixture-per-thr in VORD) — la composizione warm diventa provata.

**Kill-switch:**
- **KS-PP-90-1**: attempt successivo a un fail-path senza server-gone verificato consumato senza pre-flight port-owner ⇒ attempt VOID.
- **KS-PP-90-2**: "publish observable" citato per a_pp38 senza contatore/log armato ⇒ claim declassato a path-coverage.
- **KS-PP-90-3**: composizione warm per-worker citata senza path= in-band ⇒ ASSUNZIONE, mai prova.
- **KS-PP-90-4**: non-vacuità gate-axum fondata su NPASS multi-binario ⇒ ADVISORY.

— Pedersen, sedia 6.

---

# VERBALE — Leijen, sedia 7, Concilio WP-90

**VERDETTO: PASS CON EMENDAMENTI.** Ordinali verificati CONTRO l'header vendored; l'uguaglianza arena==proc è una TAUTOLOGIA di read-path da declassare; il positivo purge copre UN ordinale su due; l'integrazione A-DL43 è fedele ma una mitigazione è vacua se non pinnata al thread-owner.

**Q1 — Ordinali GIUSTI; il positivo copre SOLO il 15.** Contati nell'enum `mi_option_e` del vendored (`libmimalloc-sys-0.1.49/c_src/mimalloc/v3/include/mimalloc.h`): 0-2 stable, 3 `deprecated_eager_commit`, **4 `arena_eager_commit`** ✓; blocco deprecated 10-14, **15 `purge_delay`** ✓. Raw conformi: `ord=15 val=0` (env 0 vs default 10 — morde), `ord=4 val=2` (=default documentato). MA: il positivo vincola il 4 solo contro shift UNIFORMI (inserzione ≤4 sposta anche il 15); un recycling nel blocco deprecated 10-14 compensato da un cambio ≤4 lascerebbe 15 giusto e 4 muto — e la riga eager non è MAI giudicata (`env_readback MIMALLOC_ARENA_EAGER_COMMIT val=unset`: metà mai armata). → A-DL44.

**Q2 — Uguaglianza ATTESA PER COSTRUZIONE: è la STESSA atomica.** `mi_process_info` legge `subproc->stats.committed.current` (stats.c, load diretta); `mi_stats_get_json` → `mi_subproc_stats_get` (stats.c:611) = memcpy dello STESSO `subproc->stats` + aggregazione heap vivi — ma `committed` è incrementato SOLO via `mi_subproc_stat_increase` in arena.c (:271/:294): il termine heap è zero. L'uguaglianza al byte prova la fedeltà di emissione/parse, NON è cross-check indipendente — va dichiarata. Termini mancanti nell'aggregato: **theap dei worker VIVI al collect** (merge solo a theap-done, theap.c:113/:311) — chiavi heap-level sottocontano; i morti sono merged in heap_main (incluso dal visitor); abandoned: stats storiche merged, blocchi vivi contati. Le chiavi che consumiamo (committed/reserved/purged/arena_count) sono subproc-atomiche ⇒ immuni. Refuto una cifra: **`commit_calls val=0` con committed 150 MB è auto-incoerente** (os.c:545 incrementa; o wiring assente o il parse substring section-blind piglia l'occorrenza sbagliata) — non citabile.

**Q3 — Candidati per NOME.** Arena per-thread ESCLUSA (`arena_count=1` in-band); purge MORDE (`purged` 295 MB a W4, 1,16 GB a W16) ⇒ il committed residuo è pagine VIVE o ritenute, non purge pigro. Candidati: (1) **`page_full_retain=2` (ord 36)** — ogni theap ritiene fino a 2 pagine small PIENE per size-class (page.c:749-773), committed e mai-free ⇒ mai purgate anche a delay=0; (2) **pagine per-theap parzialmente usate** — ogni bin toccato da ogni worker tiene ≥1 pagina propria (W×bins×page); (3) `minimal_purge_size` (ord 44, default 64 KiB) — range liberi sotto soglia saltano il purge; (4) pagine abandoned con blocchi vivi (`page_reclaim_on_free=0`). Riga in-band mancante per WP-89(sessione): **census per-theap al collect** `tag=mi_theap_pages thr= bin= pages_live= pages_retained= committed=` + delta `purged` per finestra; braccio discriminante `MIMALLOC_PAGE_FULL_RETAIN=0` CON read-back ord 36 (KL-89-1).

**Q4 — Fedele; UNA vacuità nominata.** design87 §A-DL43: (a) boundary pre-prologo con marker `heap=` al prologo + VOID KL-89-3 — conforme al mio ordine; (b) semantica temporale «free eseguiti DENTRO la finestra» + due mitigazioni — conforme. MA la mitigazione (i) «quiesce+collect del thread» non pinna CHI colleziona: i delayed dell'owner si processano SOLO sul thread owner — un collect dal main è MUTO per la finestra del worker. Va pinnata: collect SUL thread della finestra, marker in-band. Manca inoltre l'interazione con gli heap A-DL39 (delayed su heap abbandonato).

**Emendamenti**: **A-DL44** — positivo per l'ordinale 4: braccio con `MIMALLOC_ARENA_EAGER_COMMIT` armato e read-back atteso, o recount a macchina dell'enum vendored nel gate. **A-DL45** — arena==proc dichiarata IDENTITÀ di read-path nel header di campagna. **A-DL46** — `commit_calls` bandito finché parse section-aware; census per-theap + braccio retain=0 per l'attribuzione di b. **A-DL47** — design87 (b)(i) pinnata al thread-owner.

**Kill-switch**: **KL-90-1** — bump del vendored senza ri-verifica dei DUE ordinali con positivo armato ⇒ righe mi_option ADVISORY, campagna VOID. **KL-90-2** — arena==proc citata come cross-check indipendente ⇒ claim VOID. **KL-90-3** — chiave heap-level da mi_arena_json con worker vivi al collect senza dichiarare i theap non-merged ⇒ riga VOID (chiavi subproc-only esenti). **KL-90-4** — attribuzione di b senza census per-theap o braccio retain armato ⇒ resta envelope, mai verdict-grade.

*Leijen, sedia 7 — mandato refutare: eseguito sull'header vendored, su stats.c/arena.c/theap.c e sui raw, non sui verbali.*

---

# VERBALE STOGOV — sedia 8, Concilio WP-90

**VERDETTO: RATIFICO — la refutazione-della-refutazione S-88.0 è CONFERMATA dal mio stesso oracle, e ho trovato l'origine plausibile del mio errore WP-89. La riqualifica A-DS45 è corretta dall'emitter. Il contratto A-DS44 regge le tre premesse dal vivo, con UN lato non nominato (emissione del Deprecated).**

**Q1 — CONFERMO: persist `C` exit 0. §3.3-ter emendata RATIFICATA.** Rieseguito ora (php 8.5.7 brew, `t_hoist_new_predecl.php`): plain = «Uncaught Error: Class "C" not found» **exit 255**; persist run1 (file_cache fresco, `.bin` 2,6K effettivamente scritto) = `C` **exit 0**; run2 cache calda = `C` exit 0; SHM-only = `C` exit 0; phpr = `C` exit 0. Il mio verbale WP-89 era falso. **Origine trovata a macchina**: `file_cache_only=1` + `file_cache` validi ma **SENZA `-d opcache.enable_cli=1`** (default CLI: Off, verificato con `-i`) ⇒ opcache spento in silenzio ⇒ comportamento plain, fatal «Class "C" not found» IDENTICO al mio risultato WP-89 — è il braccio-persist-monco, silenzioso dove il file_cache rotto è rumoroso (path relativo ⇒ «Fatal Error opcache.file_cache must be a full path», exit 254 — riprodotto anch'esso). L'ipotesi S-88.0 (dir inaccessibile) refutava solo il fallback rumoroso; il mio errore era la flag caduta. Riqualifica const-folding correttamente DECADUTA; il meccanismo resta hoist.

**Q2 — CONFERMO dall'emitter** (`crates/php-runtime/src/vm/mod.rs:16624-16692`): nel lane main, fp diverso su stessa chiave fa `mem::replace(&mut slot.main, …)` — MAI `victim`; `main_evicted` (16687-16691) scatta SOLO se un ways-victim del lane **includes** porta `main_program`, strutturalmente impossibile post-partizione A-MS24. Quindi sì: in produzione è SOLO tripwire (KS-DS-84-4). La forma «supersede-with-putord ≥2 + main_evicted==0 + pair-guard» è il positivo giusto a **W=1**; il limite resta KH88-4 (niente `thr=` sulle righe) — vedi KS-DS-90-1.

**Q3 — Le tre premesse REGGONO dal vivo**: (a) trait abstract: oracle «Declaration of C::f(): int must be compatible with T::f(): string» exit 255 su plain E persist; phpr `alive` exit 0 — regola 9 DENTRO è giusta; (b) tentative return (`JsonSerializable::jsonSerialize` non tipato): «Deprecated: Return type of J::jsonSerialize() should either be compatible … mixed …» + `{"a":1}` **exit 0** su entrambi i bracci — MAI fatal, lane confermata; `#[\ReturnTypeWillChange]` SOPPRIME (verificato, exit 0 pulito); (c) `never` covariante su `string`: `declared` exit 0 ×3. Esenzioni verificate in più: **costruttore** e **metodi private** esenti da LSP (exit 0 oracle e phpr). **Buco residuo, per NOME**: il contratto classifica la fatalità ma NON l'emissione — phpr oggi tace il Deprecated che l'oracle emette (byte-divergenza viva nel mio run). → A-DS46.

**Q4 — COERENTI.** Il log (8 righe, W=1): 3 generazioni `main_probe`+`main_put`, poi `supersede entries 1 putord=2` e `putord=3` — putord è l'ordinale del put che INNESCA il supersede (put#1 nessun supersede, put#2/3 dopo i touch), riga supersede emessa nella fase di emissione post-borrow (16651-16673) prima del blocco victim: ordine e cardinalità esattamente da modello. Putord strettamente crescenti (A-TH46 ok).

**Emendamenti**: **A-DS46** — deprecation-lane con EMISSIONE: fase 1 A-DS35 emette il Deprecated byte-fedele (messaggio+riga) e onora `#[\ReturnTypeWillChange]`; esenzioni costruttore/private nominate DENTRO il contratto. **A-DS47** — ricetta §3.3-ter: il braccio persist dichiara SEMPRE tutte e tre le flag per NOME + verifica che il `.bin` esista nel file_cache (il braccio-monco senza `enable_cli` è silenzioso e mima plain — la mia recidiva).

**Kill-switch**: **KS-DS-90-1** — fase uclog a W>1 consumata senza `thr=` in-band su supersede/evict ⇒ fase VOID (chiude KH88-4). **KS-DS-90-2** — qualunque run «persist» citato senza `.bin`-check o senza le tre flag esplicite ⇒ observable UNANCHORED.

Firmato: D. Stogov, sedia 8.

---

# VERBALE — B. Gregg, sedia 9, Concilio WP-90

**VERDETTO: PASS CON EMENDAMENTI.** Sonda condotta coi prefissi GIT ROOT `php-rust/` (lezione WP-89 applicata; catena raw: unico commit su measure-out = 27f1dae, verdict88.a1.g1.out ivi committato; le righe m87 appese committate in 456c1b0).

**Q1 — SÌ, con due buchi nominati.** m87: le 3 righe terminali (attempt=1 VOID `orphan-server-wrapper…FAIL67-committed-at-d0132fb`, attempt=2 VOID `anti-orphan-tooth-bit-own-wrapper`, attempt=3 PASS con verdict_file) sono appese con nota `A-BG46-forma-S-88.0-append-only`; git=5392153 nelle righe = HEAD pre-sessione, coerente col commit 456c1b0. m88: `fail()` appende `verdict=VOID` (measure88-campaign.sh:86) e verdict88.sh appende `phase=verdict esito=PASS|FAIL` (:344/:348) — la riga PASS c'è nel ledger. VOID-ness leggibile dal SOLO ledger: confermato. **Buco 1**: `ATT=0` pre-flight (:80) — due pre-flight falliti sarebbero indistinguibili e la riga attempt=0 non è un "attempt" per la lettera di KG-89-1: lane da dichiarare. **Buco 2 (il più grave)**: le due battery FALLITE non hanno traccia in NESSUN ledger — battery-stamps.ledger (wp83-harness/evidence) appende SOLO nel ramo PASS (battery-88pre.sh:104-106, unica riga: rev=0b83f2b); i FAIL vivono solo nella narrativa di sessione e nei commit-fix (b4ad60b = F16b flush). Asimmetria diretta con A-BG46.

**Q2 — quasi completo; due denti del mio ordine mancano, gravità BASSA.** srv_pid è riconciliato con OGNI riga `pid=` del raw (verdict88.sh:94-95), server_exit==0 giudicato (:91), count identity==1 (:84). `srv_boot_epoch` c'è IN-BAND nell'identity (verificato sul raw slope.w4.r1) ma NESSUN dente lo consuma (anti-riciclo-pid non giudicato). Il check X-Phpr-Pid alla prima request per fase NON esiste (grep vuoto sul campaign script): il canale HTTP di risposta non echo-a il pid. Mitiganti reali: port-owner assert (A-PP43) + riconciliazione census chiudono lo scenario-orfano a1. Gravità: bassa — ma il boot_epoch scritto e mai giudicato è un contatore compilato-non-armato (classe già denunciata).

**Q3 — pulito sul canale peak; UN default latente trovato.** NUL scan sui raw di macchina: **0 file .log m88 con NUL>0** (scan integrale, nessuna riga); nessun `DECLARED-DEVIATION nul_count=` nel verdict; le righe peak (`peak_memory_footprint_bytes`) sono tutte pulite e MEASURE88 cita comunque `committed_postcollect_win0` per VSLOPE. Peak mancante ⇒ FAIL (:189), log mancante ⇒ FAIL (:180): A-BG48 rispettato sul companion. **PERÒ**: l'estrazione di C/SL/AC usa `awk … END{print v+0}` (:175-177) — riga `tag=mi_proc win=0` assente ⇒ **0 silenzioso DENTRO TMP e dentro l'LSQ**. Guardato solo indirettamente da `exit_collect_mi` e VARENA; violazione LATENTE della lettera di KG-89-3 (non innescata in m88: tutti i C>0 nel verdict).

**Q4 — SÌ, documentazione ri-giudicabile.** p1: 456c1b0 committa le fixture (`t_hoist_new_predecl.php`, `t_hoist_conditional.php`), ds40-verify aggiornato e catalogo divergenze — riproducibile da terzi. p5: riqualifica in 94661d8 con motivazione a codice nel verdict (:324-328) e positivo ≥1-coppia spostato su F16b ARMATO. GAP_TREND riga WP-88: «non rimisurato (nessun full/media run) — riferimento resta WP-85» — onesto. Nota: il label VWARM «PER-REQUEST» sta nel verdict SENZA qualificatore; la riqualifica VOID-di-significato vive solo nella prosa MEASURE88.

**Emendamenti**: **A-BG49** — esito battery SEMPRE ledgerato: ramo FAIL di battery-*.sh appende `esito=FAIL fails=N rev=` a battery-stamps.ledger (append-only, come A-BG46). **A-BG50** — presence-guard sugli estrattori: `tag=mi_proc win=0` contato ==1 e C>0 PRIMA dell'echo in TMP; mai `v+0` come sorgente d'aggregato. **A-BG51** — srv_boot_epoch GIUDICATO (boot_epoch < primo epoch di fase, per raw) + pid-echo sul canale HTTP alla prima request per fase; lane pre-flight dichiarata (`phase=preflight` nella riga VOID attempt=0). **A-BG52** — ogni label di discriminatore porta il REGIME di validità in-band nel verdict, non solo nella prosa.

**Kill-switch**: **KG-90-1** — battery il cui FAIL non lascia riga esito nel ledger ⇒ il PASS successivo same-rev è PROVISIONAL, non consumabile. **KG-90-2** — estrattore che può emettere 0 su riga assente dentro min/LSQ ⇒ blocco VOID alla prima occorrenza provata dal selftest. **KG-90-3** — identity con srv_boot_epoch assente o incoerente col primo epoch di fase ⇒ raw VOID.

*— B. Gregg, sedia 9*

---

---

## ⚖️ SINTESI DI CONVERGENZA (compilata a valle dei 9 verbali)

**Verdetto complessivo: 9× PASS/CONCORDO CON EMENDAMENTI — nessuna
opposizione; UNA RATIFICA piena (Stogov ratifica ENTRAMBE le refutazioni
S-88.0 rieseguendo l'oracle e trova l'origine del proprio errore WP-89: la
flag `enable_cli` caduta ⇒ braccio-persist-monco SILENZIOSO che mima
plain).** Le cifre di attempt=1 sono confermate a ricomputo indipendente
(Bak al byte sui 20 raw slope + cal/conc; Gregg su ledger/NUL; Hejlsberg
sulla catena; Klabnik sul gate cifre). DUE FORGE MORSI DAL VIVO da
Klabnik sul gate cifre (working-tree corpus; [derivata] a scope di riga)
— i colpi più gravi della tornata. Dropped della sintesi WP-89 dichiarato
per NOME: A-SK53 (banda ± a finestra) non fu mai messo in ordine —
ri-emesso come A-SK53-bis. I colpi:

### Refutazioni convergenti (≥2 sedie o dall'oracle/raw/forge)

1. **🔴 Il gate cifre legge il WORKING TREE, non il committed (Klabnik,
   FORGE MORSO)**: un raw non committato in measure-out legalizza
   qualunque cifra (RC=0 dal vivo con `committed=987654321` fabbricato);
   «committed» viveva solo nel commento → A-SK55, KS-SK-90-1.
2. **🔴 `[derivata]` a scope di RIGA esenta anche i token MISURATI
   (Klabnik, FORGE MORSO)**: una riga con byte fabbricati + companion
   auto-coerente passa; le righe chiave di MEASURE88 sono in corpus per
   fortuna, non per macchina → A-SK56, KS-SK-90-2.
3. **Le battery FALLITE non lasciano traccia in NESSUN ledger
   (Hejlsberg+Gregg, convergenza)**: battery-stamps.ledger appende solo
   su PASS — i fail F16b (94661d8) e porcelain-refuse (b4ad60b) vivono
   solo nei commit; asimmetria diretta con A-BG46 → A-AH50≡A-BG49,
   KS-AH-90-1/KG-90-1.
4. **Il fail-path del port-owner assert PRODUCE l'orfano che dovrebbe
   prevenire (Pedersen)**: fail() in command-substitution + `|| exit 1`
   non uccide né SRV né DPID → A-PP46, KS-PP-90-1.
5. **arena==proc è un'IDENTITÀ di read-path, non un cross-check
   (Leijen)**: mi_stats_get_json e mi_process_info leggono la STESSA
   atomica subproc — l'uguaglianza al byte prova solo la fedeltà di
   emissione/parse; il claim «cross-check riuscito» di MEASURE88 va
   riqualificato; `commit_calls val=0` con 150 MB committed è
   auto-incoerente (parse section-blind o wiring) → A-DL45/A-DL46,
   KL-90-2; l'estratto mi_arena senza riga authority nello stesso win è
   VOID (Matsakis, KS-MS-90-2).
6. **La monotonia degli ASSOLUTI è vacua sul MARGINALE (Bak)**: i Δ
   per-worker spread ±11% e non monotoni fra loro; serve fascia δ del
   marginale ex-ante + b±2σ in-band (se(b)=586.270 B ⇒ b=21,20M±1,17M);
   **banda KL-85-2 RITIRATA** come costante cross-protocollo (l'IC 2σ
   non la sfiora) → A-BB57, KB-90-1/2; W con modes==R è census
   non-informativo (W=12: 5/5) → A-BB58.
7. **La dichiarazione A-TH47 è INESATTA (Hoare, opposizione alla
   lettera)**: le chiavi `const Cell` senza Drop non registrano
   distruttori TLS — `LocalKey::with` NON panica mai per esse; la vera
   sede del fail-fast è il primo TLS con distruttore (RefCell); il Drop
   della guardia non può panicare per quella via ⇒ l'abort dichiarato è
   irraggiungibile → A-TH50, KH90-2.
8. **Il flag probe è `pub` e la belt conta le grafie di ieri
   (Matsakis)**: `LocalKey::set` bypassa `.with` senza toccare i
   conteggi; scrittori fuori modulo non sono nemmeno guardati; il fix è
   di VISIBILITÀ (privato al modulo = sigillo di tipo), non di regex →
   A-MS43/44/45, KS-MS-90-1/3.
9. **Il publish observable di a_pp38 è ancora ripetizione di path
   (Pedersen)**: il determinismo del fatal garantisce la byte-stabilità
   anche a cache spenta; serve il contatore (uc_log armato, put=1/hit=1)
   → A-PP47, KS-PP-90-2; NPASS multi-binario annacqua la non-vacuità →
   A-PP48, KS-PP-90-4.
10. **`spans=` è riportato ma MAI giudicato in VWARM (Klabnik)**: il
    «controllo positivo perfetto zero-swallow» poggia su NO-OVERLAP che
    nessun dente esige → A-SK58; il label «PER-REQUEST» sta nel verdict
    senza qualificatore di regime (Gregg) → A-BG52.
11. **Estrattori `v+0` = zero silenzioso latente dentro l'LSQ (Gregg)**:
    riga mi_proc assente ⇒ 0 in TMP; non innescato in m88 ma è la classe
    KG-89-3 → A-BG50, KG-90-2; srv_boot_epoch scritto e mai giudicato →
    A-BG51, KG-90-3.
12. **Il giudice non è identificato per sha (Hejlsberg+Klabnik)**:
    `campaign_sha=judge` placeholder; le generazioni gG non sono
    governate (g2 divergente siederebbe accanto a g1 con pari autorità,
    e TUTTE le generazioni alimentano il corpus) → A-AH51/A-SK59,
    KS-AH-90-2/KS-SK-90-3.
13. **Il contratto A-DS44 regge le tre premesse dal vivo (Stogov)** ma
    manca l'EMISSIONE: phpr tace il Deprecated che l'oracle emette;
    `#[\ReturnTypeWillChange]` va onorato; esenzioni ctor/private
    nominate DENTRO → A-DS46; ricetta persist con 3 flag + `.bin`-check
    (il braccio-monco è silenzioso) → A-DS47, KS-DS-90-2.
14. **Sigilli residui (Hoare)**: 5 grafie nuove (r#, call multiline,
    commento interposto, `::` spaziato, use-group) → A-TH49; noprobe
    --selftest fuori dalla catena ledgerata → A-TH48, KH90-1; a_ds36
    senza precondizione → A-TH51, KH90-3; ledger-prefix bucato
    dall'estensione dell'ultima riga (Klabnik) → A-SK57; nested-guard
    con tre buchi (closure anonime, case-insensitivity, witness
    non-falsificabile su coppie nuove — Bak) → A-BB60, KB-90-3.

### Ordine vincolante di apertura WP-89(sessione) (non rinegoziare)

1. **Sanatorie forma/dichiarazioni (nessuna campagna)**: A-TH50
   (dichiarazione TLS corretta: sede reale = TLS con distruttore) ·
   A-DL45-forma (nota in MEASURE88: arena==proc = identità di read-path,
   mai «cross-check») · A-DS47 (ricetta persist 3-flag + .bin-check in
   ds40-verify + catalogo) · A-AH52-forma (limite estremi-only
   dichiarato) · A-BG52-forma (label discriminatore VWARM qualificato
   anche nel verdict/MEASURE).
2. **Fix gate cifre (i due forge)**: A-SK55 (corpus COMMITTED-only) ·
   A-SK56 ([derivata] a scope di FIGURA) · A-SK57 (prefix a granularità
   di riga) · A-SK53-bis (± a finestra + allowlist) — OGNI fix con
   bite-test committato nello stesso commit.
3. **Fix strumenti campagna/verdetto**: A-PP46 (failpath kill nel
   subshell + server-gone su body-mismatch) · A-SK58 (spans GIUDICATO:
   stag⇒NO-OVERLAP, base/warm⇒OVERLAP, INVALID⇒FAIL) · A-BG50
   (presence-guard estrattori, mai v+0 in aggregato) · A-DL46-parse
   (commit_calls bandito; estrazione section-aware) · A-AH51+A-SK59
   (judge_sha in-band + governo generazioni: doc cita la MASSIMA) ·
   A-BG51 (boot_epoch giudicato + lane preflight) · A-AH53 (recorder
   rustc mai riga vuota).
4. **Catena evidenza**: A-AH50≡A-BG49 (battery-attempts ledger
   append-only per OGNI esito, incluso REFUSE) — KS-AH-90-1/KG-90-1
   attive da subito.
5. **Sigilli v7**: A-MS43 (flag PRIVATO al modulo = sigillo di tipo) ·
   A-MS44 (#[must_use] su ProbeWindow + doc TLS) · A-MS45 (belt grafie
   set/replace/update + .done parity con rc= in-band) · A-TH48 (noprobe
   --selftest in catena + pin scoped a probe_in) · A-TH49 (5 grafie con
   decoy) · A-TH51 (a_ds36 precondizione) · A-PP47 (publish a contatore
   con uc_log armato) · A-PP48 (NPASS dalla riga unit-suite + conteggio
   pinnato).
6. **Misura (una campagna, ATTRIBUZIONE di b)**: census per-theap
   `tag=mi_theap_pages` + braccio `MIMALLOC_PAGE_FULL_RETAIN=0` con
   read-back ord 36 (A-DL46-census, KL-90-4) · A-DL44 (braccio eager
   ARMATO = positivo dell'ordinale 4) · A-BB57 (b±2σ + fascia δ ex-ante
   in-band) · A-BB58 (R raddoppiato sui W multi-modo o punto ADVISORY) ·
   A-BB59 (stagger-sweep invertito Δt∈{0,1,2,5,10,20} ms + swap-fixture,
   predizioni ex-ante: il surplus segue l'ORDINE, non il fixture).
7. **ROADMAP — A-DS35 fase 1 IMPLEMENTAZIONE**: contratto A-DS44 +
   A-DS46 (emissione Deprecated byte-fedele + ReturnTypeWillChange +
   esenzioni ctor/private DENTRO) — gate di merge KS-DS-88-3 +
   KS-DS-89-3; fixture nuove committate PRIMA del codice.
8. **Delibere/deferred**: banda KL-85-2 RITIRATA a registro (KB-90-2) ·
   A-BB60 (nested-guard v2: lowercase + closure-hash + witness derivato
   dalla calibrazione di L) e A-PP49 (path= in-band) = DESIGN ·
   A-MS27/A-PP18/A-PP27/A-AH38 invariati; KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH90-1 | rename payload senza noprobe --selftest PASS same-commit | campagne VOID |
| KH90-2 | claim fail-fast TLS fondato sulla dichiarazione pre-A-TH50 | claim VOID |
| KH90-3 | consumer putord citato come precondizione-eseguita a canale disarmato | citazione VOID |
| KS-MS-90-1 | flag probe scrivibile fuori dal modulo del ctor | attribuzione ADVISORY |
| KS-MS-90-2 | cifra mi_arena estratta senza riga authority stesso win | cifra VOID |
| KS-MS-90-3 | ProbeWindow::arm() non legato a binding vivo | run VOID per attribuzione |
| KS-SK-90-1 | cifra legalizzata da file non raggiungibile da HEAD | PASS gate cifre VOID |
| KS-SK-90-2 | figura misurata esentata via [derivata] di riga | riga VOID |
| KS-SK-90-3 | doc cita gN con gM>N divergente sullo stesso attempt | claim VOID |
| KS-AH-90-1 | tentativo battery senza riga in-band | PASS successivo non consumabile |
| KS-AH-90-2 | riga verdict senza giudice per sha | verdetto non verdict-grade |
| KB-90-1 | slope senza fascia δ del marginale o senza 2σ | ADVISORY, mai pin |
| KB-90-2 | KL-85-2 citata cross-protocollo | VOID (banda RITIRATA) |
| KB-90-3 | nested-guard su coppia con closure senza confronto corpi | MODEL-GRADE |
| KS-PP-90-1 | attempt dopo fail-path senza server-gone verificato | attempt VOID |
| KS-PP-90-2 | publish observable senza contatore/log armato | declassato a path-coverage |
| KS-PP-90-3 | composizione warm citata senza path= in-band | ASSUNZIONE, mai prova |
| KS-PP-90-4 | non-vacuità gate-axum su NPASS multi-binario | ADVISORY |
| KL-90-1 | bump vendored senza ri-verifica DUE ordinali con positivo armato | mi_option ADVISORY, campagna VOID |
| KL-90-2 | arena==proc citata come cross-check indipendente | claim VOID |
| KL-90-3 | chiave heap-level con worker vivi senza dichiarazione theap non-merged | riga VOID |
| KL-90-4 | attribuzione di b senza census per-theap o braccio retain armato | envelope, mai verdict-grade |
| KS-DS-90-1 | fase uclog W>1 senza thr= in-band | fase VOID |
| KS-DS-90-2 | run «persist» senza .bin-check o 3 flag esplicite | observable UNANCHORED |
| KG-90-1 | battery FAIL senza riga esito nel ledger | PASS successivo PROVISIONAL |
| KG-90-2 | estrattore che può emettere 0 su riga assente in min/LSQ | blocco VOID |
| KG-90-3 | identity con srv_boot_epoch assente/incoerente | raw VOID |
