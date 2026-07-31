# COUNCIL_WP83_REVIEWS.md — Concilio a 9 sedie su S-81.0 (LEVA A-BB6 + churn verdict) + programma WP-82 (misure residue A/B)

**Data**: 2026-07-31 (chiusura S-81.0)
**Oggetto**: revisione di S-81.0 (commit 62cd100…f6e13c3: ordine WP-82 passi
1-7, leva 57ec7dc, campagna 26/26 R=3, verdict81 PASS) e giudizio del
programma WP-82 (footprint twin, CPU slope, retained ×W, autoload-run,
battery-su-HIT).
**Mandato**: REFUTARE (regola utente 2026-07-26). Ogni sedia ha letto
NEXT_SESSION, WP_SESSION_81, design79 emendato, MEASURE81_RESULTS, il
proprio ordine WP-82 e il CODICE/i raw del proprio perimetro; Gregg e Bak
hanno ricomputato le cifre dai raw in proprio.
**Status**: verbali VINCOLANTI per WP-82 (blocco ⚖️ in NEXT_SESSION).

---

## ⚖️ SINTESI DI CONVERGENZA

**Verdetto complessivo: 9× CONCORDO CON EMENDAMENTI — nessuna opposizione.**
La leva A-BB6 è giudicata REALE e sana nel merito da ogni sedia: il
clone-on-stack è borrowck-enforced (Matsakis), il put-dopo-link tiene su
entrambi i path (Hoare), la purezza del main regge alla lettura del lowering
(Stogov), e la cifra-simbolo **a_calls(HIT)=2 è stata PROVATA dall'aritmetica
dei raw** (Bak: a_bytes == 2×len(path canonico) ESATTO su ogni fixture —
196=2×98 hello, 194=2×97 bare, 212=2×106 include_heavy; Gregg: md5
r1==r2==r3 su TUTTE le 8 coppie, determinismo pieno). Il colpo strutturale
della tornata è doppio: **il one-shot NON passa dall'acquire** (Hoare A-TH21:
`run_source_with_argv` — il vero `phpr script.php` — duplica lower+compile e
bypassa `main_unit_acquire`, quindi F-oneshot t2 è vacuamente vera e il
"secondo path di compile in attesa" di A-TH14 ESISTE GIÀ) e **u64 è chiuso
nel codice ma non nella macchina** (Hoare A-TH19: gate-dr1 dichiara ancora
u32 nel proprio header e nessun dente pinna la larghezza — un revert
silenzioso passerebbe tutto).

### Refutazioni convergenti (≥2 sedie indipendenti)

1. **Gli strumenti di verdetto non sono onesti quanto i dati** (Klabnik
   A-SK19/20 — field() vuoto passa i confronti awk come stringa, P5-P7
   leggono solo r1 senza etichetta; Gregg A-BG26/27/28 — l'header di
   MEASURE81 "nessuna trascrizione a mano" è FALSO alla lettera: le
   percentuali −99,997%/−98,6%/+0,14% non escono da alcuno script; "a2
   assorbita" è smentita dai raw: a2=2 call/196 B su ogni riga steady):
   [KS-SK-83-1, KG-83-1, KG-83-2, KG-83-3].
2. **Il floor 2 è vero ma CONDIZIONALE e va scopato** (Bak A-BB27/28/29 —
   vale per path<384B, cieco al C-malloc di realpath, il source-read vive
   nel resid; il −99,997% è della FINESTRA a, mai "costo per richiesta";
   Leijen A-DL18 — il canale cieco (ii) include la libc del probe):
   [KB-83-1, KB-83-4].
3. **Il one-shot va foldato sull'acquire** (Hoare A-TH21 + Klabnik A-SK23/24
   — evento sintetico probe=false pinnato, sweep di CLASSE su `…true)`):
   [KH83-3, KS-SK-83-4].
4. **Ledger drained: l'identità dichiarata "esatta" è falsa di un termine**
   (Pedersen A-PP18 — il resid del fatal stesso EVAPORA; telescoping valido
   solo con inflight_max==1): [KS-PP-83-3].
5. **I raw VOID rimossi senza quarantena non sono falsificabili** (Hejlsberg
   A-AH27 — VOID ≠ inesistente; Gregg conferma: la rimozione non è
   diff-verificabile, il claim "13+13" è testimoniale): [KS-AH-83-2].
6. **La cifra retained MAIN è un FLOOR, non un budget** (Leijen A-DL15/16 —
   add_program è shallow di 1-2 ordini e il selftest non lo esercita;
   Hejlsberg A-AH29 — manca la battery full-body sul binario mem-census):
   [KL-83-1, KS-AH-83-1].
7. **F13 non discrimina le superfici condizionali/deferred** (Stogov A-DS18
   — serve F14: classe condizionale + eval-mint + extends deferred; Klabnik
   A-SK22 — serve il falsificatore in-cargo della classe baked-id):
   [KS-DS-83-3].
8. **La porta vm_new merita il sigillo di TIPO** (Matsakis A-MS17 — token
   ZST a costruttore privato, l'awk ha 4 eluzioni note; Hejlsberg A-AH26 —
   newtype PreludeBinding per il single-binding del fp): [KS-MS-83-2,
   KS-AH-83-3].
9. **Eviction thrash main/include sulla stessa UnitKey** (Stogov A-DS20 —
   FIFO senza refresh: 5 fp su 4 ways = il main caldo evicted in ciclo;
   contatore main_evicted + F15): [KS-DS-83-1].
10. **Baseline ORM/hk: per NOME va bene ef90cb19, per PERF serve la coppia
    build-adiacente a 7593d8e** (Hejlsberg A-AH30/KS-AH-83-4).

### Ordine vincolante di apertura WP-82 (S-82.0 "honest instruments", poi misure residue)

1. **Strumenti di verdetto fail-closed**: A-SK19 (field vuoto=FAIL, pin
   presenza+steady_n) · A-SK20/A-BG28 (P* su media R con spread per-campo;
   r1-only solo dietro md5-gate scriptato) · A-BG26 (tag [derivata] o script
   che emette le percentuali) · A-BG27 (a2 NOMINATA, byte accanto alle call)
   · KG-83-3 (grep-gate cifre MEASURE↔raw) · MEASURE81 retro-annotato.
2. **Fold one-shot**: A-TH21 (run_source_with_argv/run_source_with →
   run_source_probed(probe=false)) + A-SK23 (evento acquire-oneshot pinnato)
   + A-SK24 (sweep di classe) — F-oneshot smette di essere vacua.
3. **Denti macchina sui sigilli**: A-TH19 (pin u64 in DR-1 + header
   riscritto) · A-TH20 (publish ==2 in vm/mod.rs, commenti skippati, F8c
   nominato falsificatore) · A-MS18 (guardia include-hit su main_program) ·
   A-MS19 (doc-drift pins) · A-AH26 (pin main_chain_fp_from) · A-PP19
   (uclog=1 in-band o FATAL su census+uclog) · A-PP20 (F8c in-cargo con
   probe key MISS) · A-PP21 (publish nel canale b enumerato) · A-DL17
   (etichetta unique-at-drop) · A-DL18 · A-TH22 (opeak test drain-sync) ·
   A-MS20 (park=cintura dichiarato).
4. **Quarantena raw**: A-AH27 (mai rm; evidence/void/ + manifest;
   retro-annotazione MEASURE81) + KH83-2 (run VOIDate contate nell'header).
5. **Strumento retained onesto PRIMA del budget**: A-DL15 (deep-visitor o
   bracket net-alloc + controllo differenziale) · A-DL16 (selftest
   add_program) · A-AH29 (battery mem-census + A-AH6 esteso) · A-AH30
   (driver_sha include lo script di campagna).
6. **Fixture nuove**: F14 (A-DS18) · F15 + main_evicted (A-DS20) · F2
   same-key a macchina (A-SK21) · trigger F4 (A-DS19) · A-DS16 (§5 esteso
   per NOME) · A-DS17 (double-compile test) · KB-83-2 (spike resid req=11
   NOMINATO prima di ogni "resid invariato").
7. **SOLO POI le misure residue**: footprint twin V2 N/2N + peak W=num_cpus
   con i floor A-DL19 · CPU slope due-N a N=1000/2000 con soglie Bak
   (slope_leva ≤ slope_base×1,05+25µs/req; scan supersede ≤1µs/key) ·
   battery-su-HIT in forma TWIN-PAIR (Pedersen Q5 + A-SK25: run B uclog
   put==0/hit==nreq nel segmento, run A census a_calls≤floor) ·
   autoload-run (KB-82-5) · ORM/hk perf SOLO build-adiacente (KS-AH-83-4) ·
   budget ×W SOLO dopo il punto 5 (KL-83-1).
8. Revert policy KS-DS-80-3 invariata.

### Kill-switch nuovi consolidati (attivi da subito)

| KS | Condizione | Effetto |
|---|---|---|
| KH83-1 | revert sintetico u64→u32 che non fa fallire alcun gate | "residuo DR-1 chiuso" retrocede ad ADVISORY |
| KH83-2 | run VOIDate (inflight>1) non contate nell'header di campagna | campagna nulla |
| KH83-3 | claim "one-shot = stesso path" prima di A-TH21 a macchina | claim NULLO |
| KS-MS-83-1 | fp main servito a probe include (o viceversa) osservato | STOP; separazione per NEWTYPE, mai per tag |
| KS-MS-83-2 | il sigillo di tipo scopre un call-site invisibile all'awk | allowlist advisory, re-audit A-MS13 |
| KS-MS-83-3 | dedup nel park senza bump PER NOME di F6 | reject |
| KS-SK-83-1 | campo giudicato vuoto nel verdetto | VERDICT FAIL d'ufficio |
| KS-SK-83-2 | F2 senza prova same-key a macchina | F2 non verde, battery incompleta |
| KS-SK-83-3 | "battery-su-HIT chiusa" senza pin main_hit sul medesimo binario | claim VOID, resta "path misto" |
| KS-SK-83-4 | call-site di classe `…true)` fuori pin, qualunque spelling | leva respinta fino a re-audit |
| KS-AH-83-1 | retained citato senza battery PASS sul binario mem-census | cifra NULL, budget ×W VOID |
| KS-AH-83-2 | rimozione raw senza manifest di quarantena | campagna sostitutiva VOID finché manca |
| KS-AH-83-3 | main_chain_fp_from fuori pin o arg ≠ PRELUDE_SRC | leva de-certificata, fp ADVISORY |
| KS-AH-83-4 | delta perf ORM/hk su baseline non build-adiacente | VOID (ef90cb19 vale solo per NOME) |
| KB-83-1 | fixture path-lungo senza il delta dichiarato | modello floor falso ⇒ floor VOID |
| KB-83-2 | spike resid req=11 non nominato prima di "resid invariato" | verdetto churn WP-82 bloccato |
| KB-83-3 | slope CPU con risoluzione ex-ante non soddisfatta | NULLO, mai ADVISORY-promosso |
| KB-83-4 | a_bytes(HIT) ≠ 2×len(path) | alloc anonima nel path caldo ⇒ FATAL census |
| KS-PP-83-1 | cifra census da run con PHPR_UNIT_CACHE_LOG armato | run VOID |
| KS-PP-83-2 | battery-su-HIT senza twin B (put==0 nel segmento) | path misto, VOID |
| KS-PP-83-3 | riconciliazione Δglobal senza i termini A-PP18 (o W>1) | VOID |
| KL-83-1 | budget ×W da rw_bytes con entry MAIN pre A-DL15/16 | budget NULLO, A/B VOID |
| KL-83-2 | "supersede libera N byte" dal solo ledger | claim respinto |
| KL-83-3 | verdetto footprint/peak/supersede senza i floor A-DL19 | ADVISORY d'ufficio |
| KS-DS-83-1 | A/B con main_evicted>0 non dichiarato | cifra VOID |
| KS-DS-83-2 | "compile_program pura" senza double-compile test | claim NULLO |
| KS-DS-83-3 | F14 rossa/assente nell'A/B che cita la classe 2 | classe 2 RIAPERTA |
| KG-83-1 | md5 r1/r2/r3 diversi + P* su singola run | FAIL automatico |
| KG-83-2 | campo giudicato con spread oltre banda | VOID di quel P* |
| KG-83-3 | cifra MEASURE senza match raw/tag derivata | documento VOID finché riconciliato |

---

## VERBALI INTEGRALI
### 1. Hoare — design linguaggio/runtime, safe-only — CONCORDO CON EMENDAMENTI

**Verdetto: CONCORDO CON EMENDAMENTI.**

**Q1 — Epoch u32→u64 (KH82-1).** Nel CODICE il passaggio è completo: `IC_EPOCH: Cell<u64>` (bytecode.rs:164), `ic_epoch() -> u64` (:176), `PropIc(Rc<Cell<(u64,u32,u32,u32)>>)` (:199) e `MethodIc` identico (:255); i confronti sono u64==u64, nessun `as u32` sull'epoch nel workspace. `bump_ic_epoch` produzione ==1 (vm/mod.rs:564). `WalkMark.epoch: Cell<u32>` è fuori classe (per-richiesta, contatore per-Vm). **MA il residuo è chiuso nel codice e NON nella macchina**: gate-dr1 dichiara ancora nell'AUDIT VERDICT `Rc<Cell<(u32,…)>>` e «the epoch is u32 and wraps… Recorded, not gated», e NESSUN dente pinna la LARGHEZZA: un revert silente u64→u32 passa DR-1, lever-pins e ogni fixture (il wrap è inosservabile ai test). Doc del gate falsa + chiusura prose-guarded: la classe M-68.5 di questo stesso progetto.

**Q2 — Put dopo link_fatal_check.** Verificato su entrambi i path (worker 603-610; vm/mod.rs 828-837); `main_publish_ticket` ha UN solo chiamante; un main link-fatale non si pubblica su nessun path. Nota dichiarativa: il put precede `vm.run()` — un main runtime-fatale SI pubblica (conforme opcache, ma §6 enumera solo lower/compile/link: una riga di doc lo espliciti). **Il pin lessicale check_order È falsificabile**: (i) guard-blind — un publish incondizionato lessicalmente dopo passa; il falsificatore reale è F8c-contatori, non nominato nel gate; (ii) prima-occorrenza senza skip dei commenti; (iii) vm/mod.rs pinna `m>=1`, non ==2: un secondo call-site non guardato dopo la riga 828 passa TUTTO.

**Q3 — A-TH15 e opeak==1.** Il commento rescoped è ONESTO (finestra send→dec ammessa, fail-closed, giudice=driver). **Ma il test lo contraddice**: `census_queue_depth_no_underflow_inc_before_send` asserisce `opeak==1` ESATTO su un loop che può dispatchare mentre il worker è deschedulato tra send e dec — esattamente la finestra ammessa. Il test PUÒ flakare a opeak=2. Stessa finestra = run VOIDabili da inflight_max>1: fail-closed, ma va contato.

**Q4 — probe=false verbatim.** Sì per il path che lo attraversa (probe_state=None ⇒ lower+compile identici; delta invisibile, corpus conferma). **Però «one-shot = stesso path senza probe» è vero solo per il path ini**: `run_source_with_argv` (il VERO `phpr script.php`) e `run_source_with` sono DUPLICATI lessicali di lower+compile che bypassano `main_unit_acquire` — il «secondo path di compile in attesa» di A-TH14 esiste già, non pinnato; F-oneshot t2 è vacuamente vero su un path che l'acquire non lo chiama proprio.

**Emendamenti:**
- **A-TH19**: dente DR-1 che pinna `Cell<u64>` su IC_EPOCH + `Cell<(u64,` ×2 in PropIc/MethodIc; header del gate riscritto SAME-COMMIT (u32-residual → chiuso-u64).
- **A-TH20**: gate-lever-pins: pin vm/mod.rs `publish_if_armed()` ==2 esatto; check_order skippa i commenti; il gate NOMINA F8c-contatori come falsificatore della guardia (il lessicale è solo posizione).
- **A-TH21**: `run_source_with_argv`/`run_source_with` foldati su `run_source_probed(probe=false)` (parametro argv passante) O pinnati PER NOME nel gate; F-oneshot t2 deve girare su un path che CHIAMA l'acquire.
- **A-TH22**: il test opeak==1 aggiunge drain-sync (spin `OUTSTANDING==0` post-recv) prima del dispatch successivo — o contraddice la sua stessa doc rescoped.

**Kill-switch:**
- **KH83-1**: mutation-test — revert sintetico u64→u32 dell'epoch che non fa FALLIRE almeno un gate ⇒ «residuo DR-1 chiuso» retrocede ad ADVISORY.
- **KH83-2**: le run VOIDate da inflight_max>1 (finestra send→dec) si CONTANO e si riportano nel header di campagna; re-roll silenzioso = campagna nulla.
- **KH83-3**: ogni claim «il one-shot è lo stesso path» è NULLO finché A-TH21 non è a macchina.

*Tony Hoare*
### 2. Matsakis — ownership/aliasing/Send-Sync — CONCORDO CON EMENDAMENTI

**VERDETTO: CON EMENDAMENTI** (nessun difetto di soundness trovato; due coperture INCIDENTALI da nominare, un buco testuale nel gate, un doc-drift).

**Q1 — clone-on-stack (KS-PP-81-2): TIENE, borrowck-enforced.** Worker: `unit` (riga 564, `worker_pool.rs`) vive fino a fine `execute_with_retain`; `vm_new(retain, &unit.module, …, Some(&unit.program))` (599) prende borrow `'m` su `unit` — il compilatore rifiuta ogni drop/move anticipato; `unit.module.file` è ancora letto a 654. CLI probed: `unit` sta nello stack di `run_source_probed` (306), `run_module_with_hir` riceve solo `&'m Module`/`&'m Program`. Nessun ramo senza park sul path probed: worker 598 incondizionato post-acquire (gli error-arm ritornano PRIMA di ogni Vm, e il `SplitDrain` a 554 li precede — dente 3 verificato); `run_module_with_hir` parka a 820-822 sse `main_lever.is_some()`, e sul ramo `probe=false` la safety non dipende dal park (owner = stack del chiamante). **Constatazione da mettere a verbale (A-MS20)**: il park è CINTURA, l'owner primario mid-request è il clone-on-stack.

**Q2 — park prima di ogni prestito: SÌ** (598→599; 820→823). Il supersede mid-request sulla stessa thread **NON è impossibile**: un include del file main con (mtime,size) mossi fa un put a chiave nuova → supersede-per-path droppa la entry main DURANTE la richiesta. Ma non lascia mai la cache unico owner: `unit` + park detengono gli Rc; il Program droppato dalla cache ha il gemello sullo stack. `UNIT_CACHE` è TL+RefCell e `unit_cache_put` non esegue codice utente dentro il `borrow_mut`: niente double-borrow, niente put concorrente.

**Q3 — allowlist A-MS13: fail-closed sulla domanda posta, fail-OPEN altrove.** Un test-module RINOMINATO non elude: non arma `in_tests`, i suoi call-site CONTANO, il gate FALLISCE (direzione giusta). I buchi veri: (a) `in_tests` non viene MAI disarmato — codice di produzione DOPO il blocco `mod tests` è invisibile; (b) `mod tests;` out-of-line a inizio file acceca l'intero file al sweep; (c) match per prefisso (`mod tests_util` arma); (d) alias `use vm_new as x` elude il token. ⇒ A-MS17.

**Q4 — collisione fp main/include: UNA direzione nominata, l'altra incidentale.** Il double-check `main_program.is_some()` (15851) copre SOLO main-probe-consuma-include. L'include-hit (6817) NON controlla `main_program.is_none()`: la direzione inversa regge su (i) disgiunzione fp probabilistica e (ii) il dc strutturale 6834, che rigetta la entry main solo perché `static_off=0/reserved_base=0/class_remap=[]` non possono combaciare con un Vm mid-request (prelude ⇒ `classes.len()>0`) — vero oggi, ma non NOMINATO né pinnato: un refactor del dc riapre la porta. Nessuno stato di relocation sbagliato è consumabile OGGI (reject→fresh path); la copertura resta però KS-DS-82-1-nulla per enunciato. ⇒ A-MS18/KS-MS-83-1.

**Q5 — F6 (retain==3) resiste al dedup (KS-MS-82-3): SÌ, ed è il falsificatore esatto.** Sul doppio include non-`_once`, il primo park pinna l'Rc fresco che il put clona in cache (7076); il secondo è un HIT che parka lo STESSO puntatore. Un dedup per puntatore/chiave collassa 3→2 e F6 morde. Un dedup solo-main è vacuo (un park-evento per richiesta, A-TH14).

**Emendamenti:**
- **A-MS17**: sigillo di TIPO sulla porta — token ZST a costruttore privato richiesto da `vm_new`/`park_main`; rustc giudice, awk declassato a cintura (chiude a, b, c, d di Q3).
- **A-MS18**: guardia esplicita `cu.main_program.is_some() ⇒ rifiuta+contatore` sull'include-hit, con test.
- **A-MS19**: doc-drift in `gate-lever-pins.sh` — il commento dice «vm/mod.rs = 2 (…eval-image sub-VM)», il pin è 1: correggere same-commit (doc falsa peggio di nessuna, WP-78).
- **A-MS20**: dichiarare a design79 §4 che park=cintura, owner=clone-on-stack.

**Kill-switch:**
- **KS-MS-83-1**: un fp main servito a probe include (o viceversa) osservato UNA volta = STOP; separazione per NEWTYPE (MainFp/IncludeFp), mai per tag.
- **KS-MS-83-2**: se il sigillo A-MS17 scopre un call-site che l'awk non vedeva = allowlist advisory, re-audit A-MS13 integrale.
- **KS-MS-83-3**: dedup nel park proposto senza bump PER NOME di F6 (3→2) e dell'invariante park-EVENTI = reject.

*File verificati: worker_pool.rs (500-758), vm/mod.rs (277-333, 483-523, 800-837, 6771-6905, 15748-15996), gate-lever-pins.sh, design79 §4-§5.*
### 3. Klabnik — spec/testabilità/gate — CONCORDO CON EMENDAMENTI

**VERDETTO: CON EMENDAMENTI.** La leva è la più gate-ata mai spedita (harness nell'identità, campagna rifatta a un rev, self-test nei denti 1-2 dei pins). Ma trovo QUATTRO classi di PASS-vacuo latente, nessuna dichiarata.

**1. verdict81.sh — parser senza fail-closed.** `field()` su chiave assente ritorna VUOTO; a valle `awk -v v="" 'v<4000'` fa string-compare e PASSA (P1a/P1b/P4/P5/P6 vulnerabili; solo `v==0` e P8 `${DM:-9}` sono fail-closed). Un rename di campo in analyze80.pl produce VERDICT PASS con celle vuote. KS-SK-82-1 è soddisfatto solo a metà: analyze80.pl muore su 0 righe, ma verdict81 non pinna né steady_n né la presenza dei campi. **Peggio: P5-P7 leggono SOLO r1** e lo spread è calcolato SOLO su a_calls/total_calls — b, resid, retain non hanno spread in-campagna; la tabella li stampa senza etichetta "r1" e MEASURE81 li presenta come steady means. Parzialità NON dichiarata: con spread>0 futuro il verdetto diventa r1-only in silenzio.

**2. F2 — same-key non provato a macchina.** `touch -r` su APFS può troncare i ns (utimes µs vs mtime ns): se il mtime restaurato differisce di sub-µs, la UnitKey cambia e F2 passa per key-MISS senza mai esercitare il fingerprint — esattamente il vacuo che il commento del gate dice di evitare, ma NESSUN check verifica che la key sia identica (né stat ns pre/post, né contatore fp-MISS vs key-MISS). **F13**: il controllo positivo f13x esercita il COMPARATORE (body≠oracolo morde) — corretto e dichiarato come tale — ma NON la classe baked-id A-DS12/2: un id run-scoped cotto verrebbe visto solo se altera l'output. Manca il falsificatore in-cargo della classe; il claim "classe 2 holds" è ADVISORY.

**3. F-oneshot t1.** reqmark prova che il WRITER uc_log è vivo (stesso prefisso `unitcache`, stesso file: accettabile), ma non che il one-shot PASSI per acquire: un refactor che bypassa acquire del tutto dà t2==0 con canale vivo. Il pin testuale A-TH14 (`probe=false`) copre oggi; un evento sintetico `acquire(probe=false)` pinnato ==1 chiuderebbe l'elusione per costruzione.

**4. WP-82 punto 6 — NON ancora falsificabile.** "Pin ESPLICITO main_hit per richiesta" non dichiara: quale binario (uc_log è union-build; la battery-su-HIT gira sul binario CENSUS — pin misurato su un twin ≠ battery = loophole di divergenza), quale conteggio (main_hit==nreq−1 per path), quale criterio di FAIL. Così com'è si può "chiudere" con qualunque numero.

**5. gate-lever-pins.** Denti 1-2 con decoy: bene. Dente 3 (SplitDrain) è l'UNICO senza negativo. Dente 4: il pin letterale è fail-closed sul rename (rumoroso, accettabile se dichiarato), ma la classe di elusione vera è un SECONDO call-site con nomi diversi (`main_unit_acquire(nm, src, r, true)`): il conteggio dello spelling pinnato resta 1 e PASSA. Lo sweep copre vm_new/park_main, non `...true)`.

**Emendamenti (serie A-SK):**
- **A-SK19**: verdict81 fail-closed — campo assente/vuoto = FAIL; pin di presenza per ogni campo giudicato + pin steady_n NEL verdetto.
- **A-SK20**: P5-P7 su media R=3 con spread per-campo (b/resid/retain); finché r1-only, colonne etichettate "r1" e parzialità dichiarata in MEASURE.
- **A-SK21**: F2 con prova macchina di same-key (stat ns pre/post o contatore key-MISS==0 ∧ fp-MISS==1 per f2.php).
- **A-SK22**: falsificatore in-cargo della classe baked-id per F13; fino ad allora "A-DS12/2 holds" = ADVISORY.
- **A-SK23**: evento sintetico acquire-with-probe=false sul one-shot, pinnato ==1.
- **A-SK24**: sweep di CLASSE `main_unit_acquire\([^)]*true\)` ==1 workspace-wide (idem run_source_probed) + decoy negativo per il dente SplitDrain.
- **A-SK25**: WP-82 punto 6 riformulato: binario nominato (stessa build della battery), pin numerico main_hit==nreq−1 per path, FAIL dichiarato.

**Kill-switch (serie KS-SK-83):**
- **KS-SK-83-1**: qualunque campo giudicato vuoto nel verdetto ⇒ VERDICT FAIL d'ufficio, mai coercizione.
- **KS-SK-83-2**: F2 senza prova same-key ⇒ F2 non conta verde (battery incompleta, KS-DS-80-2).
- **KS-SK-83-3**: "battery-su-HIT chiusa" senza pin main_hit sul MEDESIMO binario ⇒ claim VOID, resta "path misto".
- **KS-SK-83-4**: nuovo call-site di classe `…true)` fuori pin, con QUALUNQUE spelling ⇒ leva respinta fino a re-audit (estende KS-MS-82-1).

*— Klabnik, sedia 3. Firmo CON EMENDAMENTI: il churn è vero, i parser che lo certificano non sono ancora onesti quanto i dati.*
### 4. Hejlsberg — compilazione incrementale/feature-gating/igiene di build — CONCORDO CON EMENDAMENTI

**VERDETTO: CON EMENDAMENTI** — il churn verdict regge; tre buchi di igiene da chiudere prima che WP-82 li erediti.

**1. MAIN_CHAIN_FP / single-binding.** Il seam regge per *visibilità* (`main_chain_fp_from` è privata di modulo, `lower/mod.rs:851`) e per *doc* ("MUST NOT call with anything but PRELUDE_SRC") — ma dentro il modulo una futura chiamata con copia mascherata è legale, e il falsificatore A-AH22 non la vedrebbe: verifica solo che il SUO percorso muove l'fp. Il gate-lever-pins ha già la macchineria `count_nontest` e pinna `vm_new(`/`park_main(` — ma NON `main_chain_fp_from`. Sì, serve il grep-gate; meglio ancora il sigillo nel type system (classe A-MS3): newtype `PreludeBinding` a costruttore privato + ctor `#[cfg(test)]` per il falsificatore.

**2. Incidente campagna.** Identità PULITA: il tripwire A-AH21 ha morso il proprio autore, campagna rifatta INTERA a un rev, rimozione dichiarata (3f32c16). MA il commit aggiunge solo l'archivio matrix: i 13+13 raw VOID sono stati `rm`-ati senza mai essere tracciati — il claim "13 a 2dc11eb, 13 rifiutate" non è ri-verificabile dal repo. VOID ≠ inesistente: la dichiarazione senza corpo del reato non è falsificabile. Serve la policy quarantena.

**3. Bump 20→23 (d3437d9).** I 3 siti sono nominati per DESCRIZIONE + rinvio alla "gate-printed site list at 552e6fb (93..1302)" — ma il gate stampa la lista solo su FAIL e non la archivia: quell'output vive nel transcript. La verifica resta deterministica (checkout + `grep -n`), quindi KS-AH-81-3 è rispettato *in sostanza*; "(93..1302)" però è un range, non una lista, e costringe il revisore a rieseguire. Prossimi bump: righe `grep -n` dei soli siti nuovi NEL corpo del commit.

**4. Sestetto/mem-census: asimmetria REALE, tre denti.** (a) Il matrix step 7 (A-AH6) compila i test target SOLO axum-server: warning nei test-build mem-census invisibili localmente (in CI il run filtrato compila i test target php-server ma senza fatalità A-AH6). (b) php-runtime/php-cli sotto mem-census: solo lint rustc, che non compila `cfg(test)` — stessa classe A-AH17/A-AH24 (mitigata dai pub-selftest chiamati dal bin, ma il pin di compilabilità resta dovuto). (c) MANCA il twin funzionale: nessuna battery full-body sul binario mem-census (analogo del tooth 2 census-twin) — e WP-82 residuo 4 vuole citare retained proprio da QUEL binario, dentro il budget KS-MS-82-2.

**5. ORM/hk POST-leva (KS-AH-82-3).** Due assi distinti. *Funzionale* (set per NOME): baseline legittima = evidence passo 2 su ef90cb19 (`wp81-harness/evidence/`) vs phpr f33151fb — il set per NOME non è sensibile alla build-adiacenza. *Perf/memoria*: ef90cb19 NON è build-adiacente (altra sera, altri commit); la coppia corretta è rebuild pre-leva a **7593d8e** (ultimo commit crates prima di 57ec7dc) stessa-sera, `--locked` (regola WP-65), vs post-leva. Un delta perf ORM citato contro ef90cb19 è VOID.

**Emendamenti:**
- **A-AH26**: pin in gate-lever-pins.sh: `main_chain_fp_from` non-test == 2 (def + call) e la call di produzione testualmente `main_chain_fp_from(PRELUDE_SRC`; al prossimo tocco del file, newtype `PreludeBinding` compiler-enforced.
- **A-AH27**: mai `rm` di raw, anche VOID — quarantena `evidence/void/<stamp>/` + manifest committato (nomi, sha, motivo, rev); retro-annotare in MEASURE81 la non-riverificabilità dei 13+13.
- **A-AH28**: bump census-twin = righe `grep -n` dei siti nuovi nel commit; il gate archivia la site-list anche su PASS.
- **A-AH29**: simmetria mem-census: A-AH6 esteso ai test-build census/mem-census nel matrix step 7; `--no-run` pin per php-runtime/php-cli mem-census in CI; battery full-body sul binario mem-census PRIMA di ogni cifra retained.
- **A-AH30**: driver_sha copre solo measure78.sh+gate-feature-matrix.sh — includere lo script di campagna (measure81-campaign e successori): il wrapper ordina i run, è protocollo.

**Kill-switch:**
- **KS-AH-83-1**: retained citato senza battery PASS sul binario mem-census (stesso hash matrix) = cifra NULL, budget ×W VOID.
- **KS-AH-83-2**: rimozione di raw senza manifest di quarantena committato = la campagna sostitutiva è VOID finché il manifest non esiste.
- **KS-AH-83-3**: call-site di `main_chain_fp_from` fuori pin o con argomento ≠ `PRELUDE_SRC` letterale = leva de-certificata, MAIN_CHAIN_FP torna ADVISORY.
- **KS-AH-83-4**: delta perf/memoria ORM/hk contro baseline non build-adiacente = VOID; ef90cb19 vale solo per il confronto per NOME.

*Anders Hejlsberg — sedia 4. File citati: `crates/php-runtime/src/lower/mod.rs:841-892`, `wp78-harness/gate-feature-matrix.sh:177-188`, `wp78-harness/gate-census-twin.sh:40-49`, `wp78-harness/measure78.sh:160-174`, `.github/workflows/ci.yml:88-97`, `wp81-harness/gate-lever-pins.sh`, commit d3437d9/3f32c16.*
### 5. Bak — alloc-rate/code-cache/path caldi — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — a_calls(HIT)=2 è CREDIBILE, e stavolta lo provo io dai raw; ma il floor è CONDIZIONALE e il titolo "−99,997%" va scopato.**

**1. Conteggio a mano del probe (HIT).** `main_unit_key`: `Path::new` borrow (0); `fs::canonicalize` — la CString d'ingresso passa dal buffer di STACK di std (`run_path_with_cstr`, path<384B ⇒ 0), il buffer di `realpath` è **C-malloc, INVISIBILE al GlobalAlloc** (0 contate), la copia in PathBuf = **alloc #1** (len path); `fs::metadata` CString su stack (0); `unit_key_for` Vec del path = **alloc #2** (len path). `fp_mix` 0; `unit_cache_get` + `MainUnit` = Rc::clone ×2 (0). Atteso = 2. **Prova regina dai raw**: a_bytes == 2×len(path canonico) ESATTO — hello 196=2×98, bare 194=2×97 (−1 char di filename ⇒ −2 byte), include_heavy 212=2×106 (+8 char ⇒ +16 byte). La cifra non è troppo bella: è aritmetica del filesystem. NON è un errore di finestra.

**2. Ma il contratto di bare.php regge solo con due scope.** (i) L'INPUT del probe — il `fs::read` del source che `fp_mix` hasha — sta nel **resid** (design79 §10, dichiarato): resid 44,1/43,5KB invariato vs baseline conferma nessun travaso, però ogni citazione di "2 call/req" DEVE portare l'appendice "più source-read nel resid". (ii) Nel resid hello r1 c'è uno **spike periodico NON NOMINATO**: req=11 = 157 call/166.771B (+114/+124KB), req=33 = 44/44.023. "44,1 invariato" è una MEDIA che ingloba spike anonimi — esattamente il letto dove dorme un leak lento (rehash di tabella che cresce?).

**3. b invariato ESATTO: possibile, con una condizione.** Il park_main (put+publish) vive SOLO sulla MISS req=1 (include_heavy: a_calls=128.551, retain_len già 6 da req=1) — fuori dalla finestra steady req≥11. Su HIT `publish: None`, zero alloc nel canale b dal main; gli include erano già HIT pre-leva. 27.982==baseline è quindi aritmeticamente pulito dato il determinismo — ma la riga MISS req=1 va PINNATA, o "b invariato" non distingue "put fuori finestra" da "put mai avvenuto".

**4. Soglie ex-ante WP-82 (KL-82-3), numeri:** (a) slope CPU due-N su hello HIT, W=1, **N=1000/2000** (non 100/200: il probe stimato ~15-25µs/req — realpath cammina i componenti, ~10 syscall — a N=100 sta sotto la granularità); soglia: **slope_leva ≤ slope_base×1,05 + 25µs/req**; risoluzione: spread R=3 delle slope < 1/3 della banda, altrimenti N raddoppia d'ufficio. (b) scan supersede: fixture forced-MISS (touch del main) con K stale keys pre-seminate, due-K **K=8 vs K=64**: costo lineare, **coefficiente ≤ 1µs/key**, put a K=64 ≤ put base ×1,10.

**Emendamenti (serie A-BB):**
- **A-BB27**: il floor 2 è CONDIZIONALE: vale solo per path canonico <384B (buffer stack CString di std) e dichiara la cecità al C-malloc di realpath; i due SITI (PathBuf canonicalize + Vec UnitKey) citati per nome accanto alla cifra.
- **A-BB28**: pin aritmetico nel gate: **a_bytes(HIT) == 2×len(path canonico)**, per fixture — verifica gratuita, muove in lockstep col docroot.
- **A-BB29**: ogni headline "2 call/req" porta l'appendice source-read-nel-resid; il −99,997% è relativo alla FINESTRA a, mai "costo totale per richiesta".
- **A-BB30**: la riga MISS req=1 diventa pin del verdetto b (put/park contati lì, per nome).

**Kill-switch (serie KB-83):**
- **KB-83-1**: fixture path-lungo (≥384B): se a_calls(HIT) non mostra il delta DICHIARATO (+CString heap), il modello del floor è falso ⇒ floor VOID.
- **KB-83-2**: lo spike resid req=11 va NOMINATO (sorgente identificata) prima di qualunque "resid invariato" WP-82; se periodo/ampiezza crescono con N ⇒ struttura che cresce ⇒ verdetto churn VOID.
- **KB-83-3**: claim di slope CPU con risoluzione ex-ante non soddisfatta = NULLO, mai ADVISORY-promosso.
- **KB-83-4**: a_bytes(HIT) ≠ 2×len ⇒ un'alloc anonima è entrata nel path caldo ⇒ FATAL in census-build.

*Firmato: Bak — le cifre troppo belle si refutano con l'aritmetica; questa l'ha vinta.*
### 6. Pedersen — disciplina di confine per-richiesta, lifecycle — CONCORDO CON EMENDAMENTI

**VERDETTO: CON EMENDAMENTI.** La forma del lifecycle è giusta (acquire→park→link_fatal→publish→start→run→capture→end); l'identità del ledger drained è dichiarata "esatta" ma NON lo è — un termine manca davvero.

**Q1 — REFUTO l'esattezza di Δglobal = Σlines + drained.** Enumerazione (worker_pool.rs):
- `SplitDrain::drop` (r. 537-550) booka **s0→drop** e setta `LAST_S3=drop`; il churn drop→s0-successivo (invio 500, dec OUTSTANDING) finisce nel **resid della richiesta SUCCESSIVA** — coperto, ma NON è "drained": chi legge census-drained come "tutto il churn del fatal" sbaglia.
- **TERMINE MANCANTE**: il **resid del fatal stesso** — la finestra LAST_S3(prev)→s0(fatal). La richiesta fatale non emette riga (resid mai letto), il drop booka solo da s0: quel gap **evapora**. L'identità com'è scritta (r. 130, 698) è FALSA di quel termine.
- Termini esclusi ulteriori: boot pre-primo-s0 per worker (LAST_S3 None ⇒ resid=0, r. 722); coda post-ultima-emissione fino al teardown (inclusa la eprintln census-drained stessa); **W>1**: LAST_S3 è thread-local ma snapshot() è globale — il telescoping regge SOLO con inflight_max==1.
- **A-PP18**: o il drop booka LAST_S3(prev)→drop (telescoping esatto per-thread), o il doc-comment enumera i 4 termini e pinna inflight_max==1 come precondizione. Vietata la parola "esatta" senza l'enumerazione.

**Q2 — flush uc_log**: le call-site (vm/mod.rs 330, 362, 5876, 7089…) sono fuori dalle finestre *_ns per costruzione (B-65.2), ma su census-**global**/census-cli il flush È dentro Δ processo. La guardia è solo la FRASE "never during a census measurement run" (r. 328) — disciplina non macchina. **A-PP19**: nel build census, se `uc_log_path()` è Some la riga census porta `uclog=1` (o FATAL); il driver ENFORCE registra l'env nell'header e rifiuta. → **KS-PP-83-1**: cifra census da run con PHPR_UNIT_CACHE_LOG armato = VOID.

**Q3 — F8c**: concordo che put==1 post-fix prova solo il NUOVO main (fp cambia ⇒ key nuova). Ma il gate non poggia lì: il paio **put==0 ∧ hit==0 con probe==2** (r. 128-136) regge — hit==0 è il dente comportamentale indipendente (un fatal pubblicato darebbe HIT alla req 2), probe==2 è il controllo di vitalità che rende i due zeri non-vacui. Nota: **F8b da solo è cieco** — link_fatal_check gira per-richiesta anche su HIT, il body sarebbe byte-identico pure con il fatal in cache. Residuo: put==0 assume publish-path UNICO (A-PP16, grep-gated). **A-PP20**: dente in-cargo — dopo `execute_with_retain` link-fatale, delta UcStats.main_put==0 E probe diretto della key = MISS (la key MAI in entries); chiude la seconda-via-d'inserimento senza dipendere dal grep.

**Q4 — ordine INTATTO**: capture (take rendered, r. 676) precede request_end (r. 680); la leva vive tutta PRIMA di request_start. Il pin retain==2 letto post-end resta esatto (RetainSet per-richiesta, park_main una volta). Neo dichiarativo: `publish_if_armed` (r. 610) cade nella finestra **b** della richiesta MISS ma non è enumerato nella definizione di b — **A-PP21**: emendare design79 §contatori.

**Q5 — battery-su-HIT, forma TWIN-PAIR** (uc_log e census non possono coesistere per A-PP19): stesso rev+driver_sha, stessa sequenza (warmup 1 req/file, poi battery). **Run B** (uclog, no census): offset log registrato tra warmup e battery; nel segmento battery, per ogni path: main_put==0, main_hit==nreq_battery, main_probe==nreq_battery — un solo put nel segmento = "path misto", FAIL. **Run A** (census, no uclog): a_calls≤floor(=2·margine) per riga come testimone HIT. → **KS-PP-83-2**: verdetto battery-su-HIT senza il twin B con put==0 nel segmento = path misto, VOID. **KS-PP-83-3**: qualunque riconciliazione Δglobal senza i termini A-PP18 enumerati (o con W>1) = VOID.

*Emendamenti: A-PP18..A-PP21. Kill-switch: KS-PP-83-1/2/3. Il resto della leva, per il mio mandato, regge.*
### 7. Leijen — allocatore mimalloc/footprint fisico — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI — lo strumento retained ESISTE ed è controllato sul lato Module, ma la cifra delle entry MAIN è NULLA come budget; il ledger supersede sottostima per costruzione proprio il caso della leva.**

Recepimento verificato SUI FILE: A-DL12 → walker UNICO sull'intera cache con visited-set per puntatore; selftest S1+S2−Sshared con sotto-grafo condiviso, guard `s_shared>0`, idempotenza; mem-census nel SESTETTO matrix+CI (KS-AH-82-4). A-DL14 → gross=1 in-band. KL-82-1 → frase esatta + 4 canali in MEASURE81 §Idle. A-DL6 → `stranded_bytes_dropped` al sito di drop. Il residuo "budget NULLO" è dichiarato onestamente (KS-MS-82-2 rispettata).

**Q1 — add_program: la cifra MAIN è INUTILIZZABILE come budget.** Body a `capacity()×size_of::<Stmt>()` senza discesa, FnDecl/ClassDecl a size_of piatto: il corpo HIR — la massa DOMINANTE di un main WP — è escluso. L'etichetta "≥, Const esclusi" nasceva per una sottostima sistematica ma *bounded*; qui il termine escluso domina e il "≥" degenera in un floor 1-2 ordini sotto il vero: un budget ×W derivato da lì sarebbe vacuamente soddisfatto. Vale SOLO come floor/sanity. Walk minimo onesto: visitor esaustivo Stmt/Expr (match senza wildcard = drift-gate gratis dal compilatore) + heap dei campi (Box/Vec/stringhe), controllato per DIFFERENZA contro il bracket GlobalAlloc del lower di una fixture nota; in alternativa, retained del Program = net-alloc bracket al lower, archiviato nella entry. In più: **il selftest non esercita MAI add_program** — il codice nuovo della sessione è l'unico senza controllo positivo; per la lettera di KL-82-2 i contributi Program dentro `rw_bytes` sono cifra non controllata.

**Q2 — surrogato LEGITTIMO, da dichiarare.** `s_shared` è misurato con lo STESSO metro del walker (func_census_bytes): il controllo falsifica l'ALGEBRA del dedup (double-count/skip — esattamente ciò che KL-82-2 chiedeva), non il METRO. "Taglia nota" va letta "nota per lo stesso metro"; un bug sistematico del metro passa indenne e si scopre solo col differenziale di Q1. Soddisfa KL-82-2 per add_module, con dichiarazione.

**Q3 — REFUTO la lettura "byte liberati".** `module_census_bytes` salta `strong_count>1` ⇒ al drop conta la sola parte UNICA immediatamente liberabile. Sul prelude condiviso col successore lo skip è semantica GIUSTA; ma il main deferred-free via RetainSet della richiesta corrente (design79 §7 lo dichiara) non viene MAI accreditato — il ledger sottostima il caso della leva. E la riga unitcache porta DUE regole di ownership: walker cache-as-owner vs ledger skip-shared. Dimensiono: ledger = lower bound "unique-at-drop"; il verdetto resta il twin vmmap.

**Q4 — frase completa, un'estensione dovuta.** Il probe non aggiunge canali alla finestra IDLE (0 richieste = 0 probe). Ma "probe quasi alloc-invisibile" impone che il canale (ii) citi anche la libc del probe (realpath(3)/stat possono mallocare libc-side); la residenza del path probe si giudica solo col twin V2.

**Emendamenti:**
- **A-DL15**: deep-visitor esaustivo per add_program (o bracket net-alloc) + controllo differenziale vs GlobalAlloc su fixture nota, banda dichiarata; fino ad allora ogni `rw_bytes` con entry MAIN si pubblica etichettato "FLOOR, non budget".
- **A-DL16**: selftest esteso ad add_program (due Program con `Rc<FnDecl>` condiviso, composizione nota, idempotenza).
- **A-DL17**: etichetta semantica ledger "unique-at-drop, deferred/shared esclusi" + regola di ownership dichiarata accanto a OGNI cifra-byte della riga unitcache.
- **A-DL18**: KL-82-1 canale (ii) esteso a "qualunque malloc libc, incluso realpath/stat del probe".
- **A-DL19 (floor numerici, KL-82-3)**: (a) footprint twin: N=100/200, no-leak = |V2(2N)−V2(N)| ≤ 0,1MB (bound 1KB/req; claim sotto 0,1KB/req ⇒ N=1000/2000); (b) peak W=num_cpus, R≥3: verdetto solo se |Δpeak| > max(2%, spread osservato IN-campagna), entro = ADVISORY; (c) probe twin supersede: edit-workload con Σ stranded attesa ≥1MB = 10× floor vmmap 0,1MB (E edit dimensionati sull'unique-per-entry misurato, comunque E≥20), V-prima/V-dopo R≥3.

**Kill-switch:**
- **KL-83-1**: budget retained ×W citato da `rw_bytes` contenente entry MAIN prima di A-DL15+A-DL16 = budget NULLO; A/B che lo usa = VOID.
- **KL-83-2**: "il supersede libera N byte" dal solo ledger, senza etichetta unique-at-drop E senza twin vmmap = claim respinto.
- **KL-83-3**: verdetto footprint/peak/supersede senza i floor A-DL19 nel verbale di misura = ADVISORY d'ufficio.
### 8. Stogov — semantica Zend engine — CONCORDO CON EMENDAMENTI

**VERDETTO: CONCORDO CON EMENDAMENTI.** Ho verificato nel codice, non nella prosa: `vm/mod.rs:15815-15889` (main_unit_acquire), `15934-15996` (put/ways), `compile/mod.rs`, `compile/class.rs:549-572`, `lower/mod.rs:107-111/841-891`, `lower/class.rs:485-489`, F7/F10/F13 in gate-lever-fixtures2.sh.

**1. A-DS12 — enumerazione INCOMPLETA nella lettera, non nel merito.** Sul path main `compile_program → compile_program_stubbed(p,r,&[]) → impl(prelude=&[], link=None)`: l'interner STUBS thread-local mai-cleared è FUORI dal path (stub_mask vuoto); `class_index` è FxHashMap (seed fisso) costruita in ordine `program.classes` e consultata per NOME; `cond_retained` entra nel Module ma è membership-only. Nessun contro-esempio trovato — ma "pura" resta un claim di LETTURA, non di macchina. L'enumerazione §5 però MANCA per-campo: `MakeClosure{fn_idx}`, `DeclareTrait{idx}`, `DeclareDeferred`/`NewAnonDeferred{idx}`, `EnumCase{class,case}` — tutti Module-relative (stessa giustificazione), ma KS-DS-82-1 dice: enumerazione o nullità. Si emenda, non si riapre. Nota: il singleton EnumCase "cached" DEVE vivere VM-side (DR-1 = 0 celle nel Module oltre le IC) — dichiararne la sede.

**2. Purezza del main: regge.** `lower_source = lower_source_impl(name, source, None, DeferPolicy::All)` — seed None; `used_conditional_seed` si setta SOLO in `note_seed_super` sotto `idx < seed_class_len` (0 sul main): nessun path lo setta senza seed. ACTIVE_VM compare in lower/ solo in un doc-comment. DeferPolicy::All è la forma Zend corretta: ogni supertype irrisolto è deferred e ri-lowerato per richiesta, mai scritto nel Module condiviso.

**3. F13 NON basta.** Esercita solo id baked verso classi base incondizionate. Non esercitate su HIT: (a) `DeclareClass` condizionale NEL main interleavato con eval-mint (id module-relative fisso, mappa runtime che cambia per richiesta — lezione WP-28); (b) `DeclareDeferred` (extends su parent da include) con ordine variato E una classe mintata che sposta l'id runtime del parent — F10 copre l'EDIT della lib, non lo shift d'id. **Serve F14.**

**4. F4: deviazione ACCETTATA** (ramo strutturalmente morto; il contatore rende un hit-rate 0 legittimo distinguibile da cache rotta) — MA un ramo publish-decision non falsificabile è tripwire senza denti (lezione WP-80): esigo trigger test-only.

**5. fp domain: disgiunzione ok, EVICTION no.** Il check difensivo su `main_program` rifiuta il collision-fallthrough — giusto. Il caso cattivo è reale ma non di correttezza: stessa UnitKey, slot ways=4, evizione FIFO `remove(0)` SENZA refresh su hit (vm/mod.rs:15983-15992). Un file richiesto come main E incluso sotto ≥4 chain-fp distinti (l'include-fp muta col `unit_chain_fp`) evince ciclicamente l'entry più vecchia e più calda = il main: 5 fp su 4 ways FIFO è thrash permanente. Opcache non ha il caso: una persistent_script per path, stesso op_array in entrambi i ruoli. Contamina footprint/CPU dell'A/B in silenzio.

**Emendamenti (serie A-DS):**
- **A-DS16** — §5 esteso per NOME a MakeClosure/DeclareTrait/DeclareDeferred/NewAnonDeferred/EnumCase, con sede VM-side del singleton EnumCase dichiarata.
- **A-DS17** — purezza a MACCHINA: test in-cargo double-compile (stesso Program ×2 + su thread nuovo, confronto strutturale del Module) + grep-gate stile DR-1 su compile/: nessuna iterazione di `Hash*` che alimenti un'emissione.
- **A-DS18** — **F14**: main con classe condizionale + eval-mint + extends deferred, ordine variato, ≥3 richieste == oracolo, controllo positivo.
- **A-DS19** — trigger test-only del ramo `main_impure_skip` (Program con `used_conditional_seed=true` → ticket None + contatore ++).
- **A-DS20** — contatore `main_evicted` (ways-eviction di entry con main_program) + fixture F15 (stesso file main+include sotto ≥4 chain-fp); mitigazione ammessa (main esente da ways, o ways-bump) ma PINNATA.

**Kill-switch (serie KS-DS-83-\*):**
- **KS-DS-83-1**: A/B con `main_evicted > 0` non dichiarato = cifra VOID.
- **KS-DS-83-2**: claim "compile_program pura" senza double-compile test verde = claim NULLO.
- **KS-DS-83-3**: F14 rossa o assente nel prossimo A/B che cita la classe 2 = classe 2 RIAPERTA (F13 da sola non discrimina le superfici condizionali/deferred).

Il resto del programma S-81.0 (fp computato single-binding con falsificatore, publish unico post-link_fatal, park della coppia Module+Program) lo sottoscrivo.
### 9. Gregg — metodologia di misura/attribuzione — CONCORDO CON EMENDAMENTI

**VERDETTO: CON EMENDAMENTI** — la sostanza regge a ricomputo indipendente; refuto due punti di forma e un claim d'intestazione.

**Ricomputi eseguiti (dai raw, `wp78-harness/measure-out/`):**

**1. Spread 0,00% — REALE, anzi sotto-dichiarato.** `analyze80.pl` su hello r1/r2/r3: a_calls=2.0, a1=0.0, a3=0.0, b=731.0/95.659B, resid=44,1/43.463B, retain=1.0, identici. md5 dei `.census`: r1==r2==r3 (84e0fa78…) e vale per **tutte le 8 coppie (arm,fixture): uniq_md5=1**. I raw sono BYTE-IDENTICI: non è spread basso, è determinismo pieno. Ricomputati anche bare (a=2/194B, b=722), cli hello (total=1.140) e heavy (b=27.982): tutti conformi a verdict81.out.

**2. a_calls=2 — sì, su TUTTE le 100 righe steady** (righe 11–110), con **a_bytes=196 su ogni riga**. REFUTO l'omissione: il report non cita a_bytes, e soprattutto le 2 call residue vivono nel canale **a2 (a2_calls=2/a2_bytes=196 su ogni riga steady)** — la frase «a2 assorbita nel salto» lascia credere a2==0, il raw dice il contrario. Inoltre «delta hello−bare su HIT = 0» vale per le CALL; i **byte differiscono di 2** (196 vs 194). Nota: la riga req=11 dentro la finestra "steady" è ancora transiente (resid 157/166.771 vs 43/42.199) — la media 44,1 la ingloba; coerente con la baseline (stesso metodo), ma l'etichetta "steady" è imprecisa.

**3. Incidente mixed-campaign — dichiarato, ma testimonialmente.** Il commit 3f32c16 dice nel messaggio «VOID mixed-rev campaign raws removed», però il diff mostra solo **+1 file** (archivio matrix): i raw misti non furono mai committati, quindi la rimozione non è diff-verificabile. Superstiti: 26/26 summary, **git=3f32c16 uniforme (78/78 occorrenze)**, **driver_sha=436b453ef0980c03 uniforme (52/52)**. ✓

**4. Idle — drift 0/0 CONFERMATO.** Tail-3 census: 136.692→136.730→136.768 (Δcall 38/38, Δbyte 41.524/41.524); cli: Δ 49/49 call, 9.639/9.639 B. Finestra == self-cost esatto su entrambi gli arm; le cifre sono emesse dall'awk tail-3 del driver: scriptate. ✓

**5. Cifra a mano TROVATA.** Le percentuali «−99,997%», «+0,14%», «−98,6%» e la giustapposizione cli 81.613→1.140 **non escono da alcuno script committato**: verdict81.sh non emette percentuali né tocca l'arm cli oltre total/a1/a3. L'aritmetica è giusta (ricomputo: 1−2/80.476=99,9975%; 731/730=+0,137%; 1−1.140/81.613=98,60%), ma l'header di MEASURE81 («OGNI cifra viene dal ricomputo SCRIPTATO… nessuna trascrizione a mano») è **falso alla lettera**. Refuto il claim, non le cifre.

**6. P5–P7 su solo r1** (variabile `L1` nel .sh): innocuo QUI perché i raw sono byte-identici, ma è forma sbagliata — con spread>0 il verdetto sarebbe funzione dell'ordine dei run.

**Emendamenti (serie A-BG, da A-BG26):**
- **A-BG26**: un MEASURE non può intestarsi «nessuna trascrizione a mano» se contiene derivate (percentuali, delta cross-campagna) non emesse da script: o lo script le emette, o portano tag esplicito `[derivata: formula + sorgenti]`.
- **A-BG27**: il canale dove vivono le call residue va NOMINATO (a2=2/196B), mai «assorbito»; ogni tabella churn porta anche i byte del canale giudicato (a_bytes), e i delta dichiarati specificano call vs byte.
- **A-BG28**: i giudizi P* si computano sulla MEDIA delle R run con min/max per campo; leggere solo r1 è lecito unicamente dietro gate d'identità byte (md5 r1==r2==r3) verificato DALLO script.

**Kill-switch (serie KG-83):**
- **KG-83-1**: nel verdict futuro, md5 r1/r2/r3 non identici + un P* valutato su singola run ⇒ **FAIL automatico** (mai r1 silenzioso).
- **KG-83-2**: spread per-CAMPO (non solo a_calls/total_calls): campo giudicato da P* con spread oltre banda dichiarata ⇒ VOID di quel P*.
- **KG-83-3**: grep-gate committato sul MEASURE: ogni cifra numerica del .md deve matchare una riga di un .out/.summary committato o portare il tag A-BG26 — verifica a script, non promessa.

*File verificati: `wp81-harness/verdict81.{sh,out}`, `wp81-harness/MEASURE81_RESULTS.md`, `wp80-harness/analyze80.pl`, `wp78-harness/measure78.sh`, raw `census.81.*`/`censuscli.81.*` (26 run).*
