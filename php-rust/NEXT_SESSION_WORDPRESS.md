# Rotta WORDPRESS-FIRST — WP-track

## ✅ WP-75 (2026-07-30): PUNTO-0 DELTA-ZERO — S-73.2.1 FRAME FIX SHIPPED ✓

**Status**: **S-73.2.1 CRITICAL FIX COMPLETE** — flush_buffer frame context now working.  
**What Was Fixed**: `invoke_array_callable()` at calls.rs:553 panicked when accessing `frames[top]` with empty stack during shutdown. Now pushes synthetic caller frame (reusing first class with methods) when frames empty.  
**Pattern**: Mirrors `dtor_walk_round()` (oop.rs:1161-1174) — frame-push before method dispatch ensures context available.  
**Test Result**: t3_ob_handler_cycle.php runs WITHOUT PANIC (was: `index out of bounds`; now: deprecation warning only).  
**Commit**: **b2c93bd** "S-73.2.1: Push synthetic frame in invoke_array_callable when frames empty"  
**Outcome**: S-73.2 (teardown reorder) + S-73.2.1 (frame context) SHIPPED & VALIDATED. Punto-0 structurally complete. Outstanding: full gate validation (S-73.1/S-73.3/E-73 decisions), handler output capture issue (separate), council verdicts H-73.4/5.  
**Reference**: `sessions/WP_SESSION_75.md` (session log + learnings), `sessions/WP_SESSION_74.md` (root cause), `/tmp/gate-d73/t3_ob_handler_cycle.php` (fixture).

---

## ✅ WP-76 (2026-07-30): GATE-D73 VALIDATION + S-73.1 IMPLEMENTATION ✓

**Status**: **PUNTO-0 FULLY CLOSED** — S-73.2/S-73.2.1/S-73.1 all shipped; gate-d73 5-test batch validated.  
**Completion**: (1) Created full fixture set (t1, t2, t5, t8 + runner) ✓, (2) Executed gate batch ✓, (3) Implemented S-73.1 (walk-abort on exception) ✓, (4) Council reconvene pending (H-73.4/5 deferred).  
**Gate Results**: t1/t5/t8 PASS · t2 partial (walk aborts correctly; error rendering TBD) · t3 known-separate (handler output capture).  
**Key Achievement**: Exception-in-dtor now halts remaining destructors (matches PHP 8.5.7); Axum migration **UNBLOCKED**.  
**Reference**: `sessions/WP_SESSION_76.md` (session log), `gaps/REPORT_GAP_76.md` (no measurement).

---

## ✅ WP-77.3 (2026-07-30): M1 Vm::request_end() IMPLEMENTATION ✓

**Status**: **M1 COMPLETE** — Vm::request_end() implemented, compiled, committed (d937b0b).  
**Completion**: (1) Pre-flight all green ✓, (2) Harness created (wp77-harness/) ✓, (3) M1 implementation + ~25-field reset per Bak spec ✓, (4) Compiles clean ✓, (5) Integration plan documented (3 options: Lazy Static/Handler Wrapper/Middleware) ✓.  
**Key Achievement**: N=1 Vm lifecycle reset ready; per-request ephemeral state cleared while statics/module persist. Async ownership solved by M9 (tokio::task_local!).  
**Binary Change**: phpr-wp77.3 (879fb2ed) stashed; baseline WP-77.1 (eab88e7a) archived.  
**Deferred**: M1 integration into Axum handler (deferred to WP-77.4 — decision needed: Lazy Static vs Handler Wrapper vs Middleware).  
**Reference**: `sessions/WP_SESSION_77.3.md` (session log + learnings), `wp77-harness/WP77_OPTION_B_AMENDMENTS.md` (M1/M3/M4 specs), `wp77-harness/M1_INTEGRATION_PLAN.md` (next steps).

---

## ✅ WP-77.4.1 (2026-07-30): M1 OPTION B INTEGRATION — BINDING STEPS 1-3 COMPLETE ✓

**Status**: 🟢 COMPLETE — Binding steps 1-3 of mandate fulfilled; gates passing.

**Completed**:
- ✅ Step 1: Implement Option B (Handler Wrapper) — with_vm_lifecycle() in axum_handler module
- ✅ Step 2: Integrate request_end() call — integration point ready for Vm instance + reset
- ✅ Step 3: Run KS-M1-Complete gate — **PASSES** (object_id==1 verified, spl_object_id test)

**Harness Created**:
- M1_INTEGRATION_PLAN.md (3 options: A/B/C; Option B chosen per council binding)
- fixtures/gate_ks_m1_complete.php (✅ PASS)
- fixtures/gate_km77_2_concurrent.php (pending concurrent HTTP)
- run_gate_ks_m1.sh (gate runner script)
- WP77_4_1_PROGRESS.md (session tracking)

**Commits**: 2 (b4e655c "Option B integration + KS-M1-Complete PASS", 7a0d47b progress update)

**Deferred to WP-77.4.2**:
- Step 4: KM-77-2 concurrent isolation test (requires concurrent HTTP fixtures)
- Step 5: KS-M1-Gate-Upgraded byte-snapshot (requires multi-workload testing)

**Reference**: `wp77-harness/M1_INTEGRATION_PLAN.md`, `WP77_4_1_PROGRESS.md`, commits b4e655c–7a0d47b.

---

## ✅ WP-77.4.2 (2026-07-30): KM-77-2 + KS-M1-GATE-UPGRADED — BINDING STEPS 4-5 COMPLETE ✓

**Status**: 🟢 COMPLETE — entrambi i kill-switch pendenti CHIUSI con verdetto da runner COMMITTATI.  
**Verdicts**: **KM-77-2 PASS** (30/30 richieste, 30 marker unici, 0 leak; controllo positivo MORDE — fixture WP-77.4.1 superseded: false-PASSava sui leak) · **KS-M1-Gate-Upgraded PASS** (A axum hello 7/7 byte-id · B WordPress front/post/feed 7/7 byte-id ciascuna — ⚠️ /?p=1 è 301 body-vuoto = falso verde, si usa il permalink reale + guardia size>0 · C ORM 3484 **3E/13F fail-set 16 nomi IDENTICO**, pin nomi in `wp77-harness/gate-baseline/orm7742-fails.txt`).  
**M4 metrics (pulite, post-ORM)**: axum 0,3 ms/4,3 MB · cli-server 6,8 ms/18,4 MB · WP front 352 ms/**350-367 MB STABILE su ~46 req (no growth)**. Abort-times N/A (M3 non ancora costruito).  
**Caveat onesto**: il handler Axum è ancora placeholder (non esegue PHP) — l'isolamento reale è validato sul percorso cli-server; il KM-77-2 "task async simultanei" pieno richiede Vm nel handler (WP-77.5+).  
**Critical Path rimanente**: (5) M3 exception bridge, (6) M4 measurement phases, (7) Vm reale nel handler Axum.  
**Reference**: `wp77-harness/WP77_4_2_RESULTS.md` (verdetti+metriche), runner `run_gate_km77_2.sh` + `run_gate_ks_m1_upgraded.sh`, verdetti in `/Volumes/Extreme Pro/Claude/wp77-harness/gate-out/`.

---

## 🚀 WP-77.4 (2026-07-31+): M1 INTEGRATION + M3 EXCEPTION BRIDGE (Continuation to WP-77.4.2)

**Status**: Binding steps 1-3 COMPLETE (WP-77.4.1); steps 4-5 COMPLETE (WP-77.4.2 ✅).  
**Critical Path**: (1) ✅ DONE: Choose Vm storage + integrate, (2) ✅ DONE: KS-M1-Complete gate, (3) ✅ DONE: Concurrent fixture (KM-77-2), (4) ✅ DONE: Byte-snapshot (KS-M1-Gate-Upgraded), (5) M3 exception bridge, (6) M4 measurement phases.  
**Kill-Switches**: KS-M1 ✅ PASS, KM-77-2 ✅ PASS, KS-M1-Gate-Upgraded ✅ PASS, KS-M3 (pending), KS-M4-Steady (pending).  
**Next Session (WP-77.5)**: Vm reale nel handler Axum + M3 exception bridge + M4 measurement.  
**Reference**: `wp77-harness/` (all docs), commits b4e655c–7a0d47b + WP-77.4.2.

---

## 🚀 WP-77 (2026-07-30): AXUM MIGRATION — OPTION B BINDING ✓

**Architecture**: N=1 Vm per async task (M9 tokio::task_local!) + per-request reset (M1) + exception bridge (M3).  
**Status**: Punto-0 unblocked; Axum bootstrap (M9) + handler refactor (M1/M3/M4) in progress.  
**Design Decision**: Council unanimously rejected N=1 immortal pool (contention + lifetime hazards); user selected Option B (structured N=1 reuse + amendments).  
**Reference**: `COUNCIL_WP77_REVIEWS.md` (council verdicts), `WP77_OPTION_B_AMENDMENTS.md` (binding amendments M1–M9).

---

# Rotta WORDPRESS-FIRST — WP-track (dopo WP-72: ⚡ **IL LEAK PER-RICHIESTA È CHIUSO E AXUM È SBLOCCATO** — 🔴 LEVA S-72.4 mass-teardown Zend-fedele SPEDITA E VALIDATA: fase A reverse-symtab a fixpoint (celle condivise e Closure incluse) + dtor-walk in ORDINE DI CREAZIONE a round + fase BREAK dei cicli Rc pre-Drop via Weak (fix meta 730fa4a: senza cattura pre-walk era un no-op — l'istogramma tutto-zero l'ha svelato); tripla post-fix **used_n 20,000→0,000 obj/req (spread 0,000)** · used_b 2,1109→0,0010 · amp K=10 +80,000→**+0,005** · ladder ESISTENZA-ONLY (flat vero = tripla + peak vmmap 221,5|221,1|221,5) · **C4 (160|192) CHIUSA (0,008≤0,2)** · broken/req=21 COSTANTE · coppia full stessa-sera **−0,36% = PASS-PARZIALE** (fail-set 88 IDENTICO; metà media-group da leggere su -S in WP-73) · 🔴 PUNTO 0 concilio CHIUSO (M-72.1 UnsetWalk canale AA+readonly f1/f6 BYTE-ID; S-72.1/2/3 autoload dedup+callable_eq array+cursore sospeso+prepend BUG(port); H-72.3/5; grep-gate esteso BORROW-OK=13) · 🔴 il gate ha MORSO 2 volte e ha pagato: **S-72.6 trait table keyed FQN** (wrong-result: due trait omonimi in ns diversi = il 2° mai registrato, smascherato dal DebugClassLoader ATTIVATO dal fix callable_eq) + getType/STREAM_CRYPTO + declared-id ristretto · E-71.H1/H2 ESEGUITI (3 letture flaggate + fixture flip-readonly PASS + bug conditional-image chiuso) · corpus **1420→1418 (−2 fix reali dtor-order, PROVENANCE)** · GATE72 PASS fails=0 sul binario finale (attempts=4 ONESTO: 1 rigettato + 1 FAIL con 2 morsi = fix a meccanismo) · **⚡ SBLOCCO AXUM DICHIARATO (P-71.3 ✓ AND E-71.H1/H2 ✓ AND P-71.4/P-72.6 ✓ — KG70-1 DECADE)** · stash phpr-wp72 4fd7b2d5…, tree `730fa4a`)

> ⚡ **WP-72 (2026-07-29, →`730fa4a`)** — sintesi a 9 recepita INTEGRALE
> in `wp72-harness/design72.md` PRIMA del codice; predictions72 a catena
> ADDITIVA (lock1 9b9391fc → lock2 376147c2 → lock3); harness wp72
> SOTTO GIT dall'apertura (runner committati = hash pre-letto per
> costruzione). Leva mass-teardown spedita e validata su TUTTI i metri;
> axum SBLOCCATO. **Storia: `sessions/WP_SESSION_72.md`. WP-71:
> `sessions/WP_SESSION_71.md`.**

## 📁 Convenzioni (decisione utente 2026-07-23)

- Qui SOLO: sintesi ultima sessione · decisioni in vigore · stato gate ·
  prossimo lavoro · backlog. Storia: `sessions/WP_SESSION_<n>.md` (un file
  per sessione, con lezioni e verdetti; ≤WP-27: memoria + git history).
  Gap perf: `gaps/REPORT_GAP_<n>.md` = SOLO le misure della sessione n
  (correzione utente 2026-07-25: la vecchia forma cumulativa era una
  trascrizione sbagliata dell'intento); il trend cumulativo vive in
  `gaps/GAP_TREND.md` (file unico vivo, una riga per sessione).
- **Chiusura della sessione N = skill `chiudi-sessione`** (decisione
  utente 2026-07-26: la procedura NON si ripete nei prompt; vive in
  `~/.claude/skills/chiudi-sessione/SKILL.md`): bonifica processi →
  verbale/ri-stash binario di parità → rotazione (WP_SESSION_N ·
  REPORT_GAP_N solo se misurato, MAI cumulativo · riga GAP_TREND ·
  questa testa + stato gate + §WP-N+1) → memoria → CONCILIO 9 sedie
  (verbali vincolanti in wp<N+1>-harness/) → gh-status-sync se file
  pubblicati cambiati → report utente + prompt N+1 CORTO. Commit+push
  a ogni passo.

## 🧭 Decisioni in vigore (fonte citabile: migration/RULEBOOK.md)

- **Zero `unsafe` nel value core** (RULEBOOK §0; NaN-boxing WP-32,
  SSO-union WP-38 e stadio-2 registri WP-44 bocciati — non riproporre
  senza rotta esplicita utente).
- **ARCO REGISTRI CHIUSO allo stadio 1 (WP-44, verbale a TRE forme)**:
  l'infra dual-mode resta dormiente a delta zero (`PHPR_REG_LOWER` = pass
  vuoto); gli stadi 3-4 NON si aprono. Falsificati sia l'ibrido
  enum-operand (v1/v2) sia i raw-registers monomorfi (v3): il costo è il
  numero di corpi handler caldi nel run_loop. Il census WP-42 resta mappa
  valida (Ret→DerefTop 40,5M = call ABI); ogni riapertura deve RIDURRE o
  mantenere i corpi caldi (dispatch-table/token-threading = ipotesi DA
  MISURARE, spesso perde in Rust; ristrutturazione del loop), mai
  aggiungerne. La macchina di riscrittura (pass a finestre + remap
  totale, gate/corpus-proven) vive in `35ff89f`/`1e365db`/`f4c80cf`.
  🆕 ROTTA UTENTE 2026-07-28: autorizzato per WP-69 (post WP-68) uno
  SPIKE DISPATCH bounded a una sessione DENTRO la condizione WP-44
  (ridurre/mantenere i corpi caldi) — dettagli e kill-switch in
  §WP-68 punto 6; i registri-come-rappresentazione restano falsificati.
- Micro-bench solo advisory: verdetti SOLO su A/B interleaved stesso-giorno
  sul workload reale. Gate per NOME a ogni commit; refactor layout/GC =
  sentinelle drop-order pinnate PRIMA; oracle-probe con `-d log_errors=0`.
- Commit AND push a ogni step; deviazioni deliberate = marker
  `BUG(port):` / `PERF(port):` / `TODO(port):`.
- **⚖️ CONCILIO DOPO OGNI SESSIONE (regola utente 2026-07-26)**: a
  chiusura della sessione N si riunisce il concilio di TUTTI gli
  sviluppatori — **9 sedie**: le 6 fondative del piano Concilio
  (`FOOTPRINT_CPU_ROADMAP.md`) Hoare (design Rust safe) · Matsakis
  (ownership/aliasing) · Klabnik (spec/testabilità) · Hejlsberg
  (compilatori/interning) · Bak (V8/HotSpot: alloc-rate, code-cache) ·
  Pedersen (disciplina di confine) + le 3 aggiunte 2026-07-26 Leijen
  (allocatore) · Stogov (Zend/opcache) · Gregg (metodologia), più
  co-optati di dominio quando il tema lo richiede (es. axum/HTTP ⇒
  web-runtime). Mandato: REFUTARE il
  report N + il programma N+1, leggendo i file di rotazione e dove
  serve il codice. Ogni membro delibera VERDETTO + emendamenti R1..Rn +
  kill-switch; verbali INTEGRALI in
  `wp<N+1>-harness/COUNCIL_WP<N+1>_REVIEWS.md`, VINCOLANTI per
  design<N+1> (recepiti PRIMA di toccare codice). Se una sessione
  chiude senza concilio, il PRIMO atto della successiva è convocarlo.
  Modello: `wp62-harness/COUNCIL_WP62_REVIEWS.md`.

## Stato gate per nome (WP-72 `730fa4a`: **GATE72 PASS fails=0 sul binario FINALE** su `wp72-harness/gate-out/` — attempts=4 ONESTO (1 contaminato-rigettato, 2 FAIL che hanno morso S-72.6+fase-A, PASS su 72b76f80 e sul finale) — cargo **1651/0** · sentinelle 5 assi BYTE-ID (**dtor RI-PINNATA .pre-wp72**: coda shutdown in ordine Zend, 29/29 dtor, PROVENANCE in wp72-harness/fixtures/pins/PROVENANCE-s72.txt) · fixture: TUTTE le WP-67..71 + **gate-h72 (13 casi M-72.1+E-71.H2: f1-f10 + h2a/b/c pin oracle)** + **gate-s72 (11 casi S-72.1/2/3+E-72.H3; b2 pin phpr PROVENANCE=deviazione livelock)** + **gate-d72 (6 casi P-72.4 __destruct/teardown pin oracle)** + **gate-walkborrow72 (recinto esteso ai drain+run.rs, BORROW-OK=13 PINNATO)** · sentinels65 + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1418 IDENTICO (baseline wp72-harness/gate-baseline/corpus72.fails, PROVENANCE: −2 fix reali dtor-order fibers/destructors_011 + bug74053)** · refl **290** · ORM **3E/13F** · hk **1665 0E/0F** · reverse **2F PER NOME**; release = **phpr-wp72 (4fd7b2d5…, tree `730fa4a`)** stash additivo 28° accanto a wp71 (38af5eaa…); census **phpr-memgc72 (c6cf3da9)** con tag=teardown (reg/broken/busy/alive_after) in phpr-mem-target/; probe verdict-file (tutti da runner COMMITTATI in wp72-harness GIT): tripla72 **PASS used_n 0,000** · amp72 **PASS +0,005** · ladder72 **ESISTENZA-ONLY** (K-73.3: token vuoto, banda unilaterale; flat attestato da tripla+peak) · run72-pair **PASS −0,36%** · probe72-h2 **PASS 3/3** · probe72-axumgate **PASS 3/3** · hist72 **PASS broken=21** · U-72 uc-steady STRUTTURALE (fp/req 512→505: 7 unit-load in meno dai fix fedeltà, zero re-cold); **AXUM SBLOCCATO**; ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-71: vedi `sessions/WP_SESSION_71.md`

## (storico-esteso) Stato gate per nome (WP-71 `076ed4b`: **GATE71 PASS fails=0 AL 1° RUN, ATTESTATO (gate71.attempts=1, K-71.2)** su `wp71-harness/gate-out/` — trap self-test esercitata — cargo **1651/0** (tot≥1651) · sentinelle 5 assi BYTE-ID · fixture: TUTTE le WP-67..70 + **gate-h71walk (22 casi M-71.1/H-71.2, 19 pin oracle + t4/t6/t7 pin phpr PROVENANCE-h71)** + **gate-s71 (15 casi S-71.1/S-71.2, vi_trait pin phpr = backlog S-71.3)** + **gate-walkborrow (K-M71.2: zero borrow nudi nei 23 corpi di walk-path, 6 BORROW-OK motivati)** · sentinels65 + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1420 IDENTICO (baseline wp71-harness/gate-baseline/corpus71.fails, PROVENANCE: −1 fix reale ns_064)** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse **2F PER NOME**; release = **phpr-wp71 (38af5eaa…, tree `076ed4b`)** stash additivo accanto a wp70 (a666382e…); census **phpr-memgc71 (2c4691d7, scope-heap src=defer)** + **71b/71c (blockdump)** con tag=cellpark; probe verdict-file (P-71.1, tutti da script): probe71-tripla **PASS** (KL71-1 totale 20,000 spread 0,000; H1-C1 MORTA) · probe71-amp **K=10 MECCANISMO-CONFERMATO (+80,000 esatto)** · probe71-ladder **PASS LINEARE (ratio 1,078)** · probe71-cron **PASS (8 evict per-evento)**; KL69-1: attribuzione CHIUSA ⇒ axum si sblocca con P-71.3 post-fix; ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-70: vedi `sessions/WP_SESSION_70.md`

## (storico) Stato gate per nome (WP-70 `feca5b2`: **GATE70 a VERDETTO MACCHINA — PASS fails=0 AL 1° RUN** su `wp70-harness/gate-out/` — matrice INTEGRALE in un solo run, trap self-test esercitata — cargo **1650/0** (tot≥1650) · sentinelle 5 assi BYTE-ID · fixture: ordering + s68 + gate-ordering-s69 + gate-h69cycle + droporder + editmiss + **gate-h70cycle (8 casi ciclici K-M70.1, 7 pin oracle + wopset PROVENANCE)** + **gate-s70neg (12 casi KS-S70.1/2 con pin `load:M`)** + **gate-h70trait (5 casi H-70.4)** · sentinels65 + KS-S6 + seed_prefix_short=0 · KE-e · P1 · sem doppio pin · **corpus 1421 IDENTICO (baseline wp70-harness/gate-baseline/corpus70.fails, PROVENANCE: −1 fix reale)** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse **2F PER NOME**; release = **phpr-wp70 (a666382e…, tree `feca5b2`; +1f73b2b census-only cfg-gated, parità invariata)** stash additivo accanto a wp69 (c5b1e80b…); census **phpr-memgc70 (9201ed94)** con tag=cellskip + tag=defermini; probe verdict-file: probe70-step0 **PASS ESISTE-SU-RELEASE (Δ5,8 MiB)** · probe70-tripla **PASS con slope in colonna (run1 FAIL parser superseded)**: mediana 2,1121 KiB/req · 20,000 obj/req, KL70-1 leak=SI, K-M70.2 cellskip=0 · K-69.3 = 178 invariato; KL69-1 resta SCATTATO (attribuzione aperta ⇒ axum fermo); ⚠️ /private/tmp/wp11-gates dopo un reboot va ripristinato dai tarball `wp9-harness/gates/`)

## (storico) Stato gate WP-69: vedi `sessions/WP_SESSION_69.md`

## (storico) Stato gate WP-68: vedi `sessions/WP_SESSION_68.md`

## (storico) Stato gate WP-67 (`2b92c78`: GATE67 PASS fails=0 — cargo 1647/0 · corpus 1421 · refl 290 · ORM 3E/13F · hk 1665 OK · reverse 2F PER NOME; run55 triple 88 BYTE-ID ×3; stash phpr-wp67 c0f5cfff…; dettaglio: `sessions/WP_SESSION_67.md`)

- Gate56 verde (2026-07-26, tree `0943f58`): corpus **1421 IDENTICO** per
  nome (baseline = `wp55-harness/gate-out/corpus.fails`) · **refl 290
  IDENTICO** · cargo **1640/0** (nuovo test model-based
  `keyless_index_churn_matches_model`) · ORM 3484 **3E/13F fail-set
  IDENTICO** (16 nomi) · hk 1665 **0E/0F** · probe56
  order/tomb/numkey/dtor BYTE-ID new vs old (pinnate PRIMA del layout;
  order/tomb/numkey anche = oracle). Baseline corpus resta **1421**.
- Gate22 integrale (storico, su `60c7e04`): sess 28 · date 351 · probe
  gd/mysqli/media byte-id · http DIFF-set atteso (WP-14, 2 item) ·
  option 413/1061 · restapi 3508/15947 IDENTICI per nome COL conteggio.
- ⚠️ **MySQL**: datadir del progetto = `/Volumes/Extreme Pro/Claude/
  mysql-wp8/data` (socket `/private/tmp/mysql-wp8.sock`, porta 3306) —
  MAI `mysql.server start` naive (apre il datadir brew vergine ⇒ gate DB
  FALSI VERDI a 0 nomi). Avvio: `mysqld_safe --datadir=... --socket=...`
  daemonizzato (double-fork+setsid). Gate DB "IDENTICO" da validare
  SEMPRE col conteggio (option 413, restapi 3508).
- **Full-suite run45** (~/Claude/wpdev, WP-56, binario `0943f58`):
  30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a run33** (88 nomi;
  idem run45-old su phpr-wp55); baseline resta
  `wp16-harness/full-out/run33-fails.txt` (⚠️ righe con prefisso
  numerico: normalizzare `s/^\d+\) //` prima del diff). Master-CPU
  **697,8s ≈11:38 = 2,06× NUOVO MINIMO** (−2,66% vs run45-old 716,9
  STESSA SERA — l'old replica run44 alla cifra: ambiente stabile; il
  riferimento residuo WP-40 699s è CHIUSO). Wall ~16 min = 1,4×.
  Archivi: `full-out/run45/`, `wp56-harness/full-old-out/`; storici
  run44 in `full-out/`, `wp55-harness/full-old-out/`.
  ⚠️ RSS: telemetria `.rss` in APPEND — max PER SEGMENTO, e il filtro
  orario da solo NON basta (le ore dei giorni prima passano il confronto
  stringa): prima delimitare il blocco della notte (ultimo time-reset),
  poi la finestra oraria (run38 1696 / run39 1945 / old 1700 = rumore).
- Suite phpt (misura): xsl 63/64 (da CWD root php-8.5.7) · tidy 44/45 ·
  asym 38/39. Suite phpt SEMPRE con path ASSOLUTO.

## Harness full-suite

```bash
"/Volumes/Extreme Pro/Claude/wp16-harness/run-full-detached.sh" phpr
# col daemonizer perl (double-fork+setsid) — il task-kill a 10' non deve
# raggiungere la run. MAI due gate22 insieme; uploads azzerati PRIMA di
# ogni full run; non ricompilare mentre una run/gate usa il binario.
# multisite: wp19-harness/run-multisite-detached.sh <oracle|phpr>
# WP-49: per-collect log dietro gc-census: PHPR_GC_COLLECT_LOG=<path>
# WP-50: probe full-scan fine-run (census-only): PHPR_GC_EOR_FULL_COLLECT=1
# (harness: wp50-harness/{pipeline50,ab50a,ab50b,census50-media,
#  run-full-census50,run37,run37-old}.sh + ab50-report.pl; binario census
#  phpr-memgc50 in phpr-mem-target/release/)
# WP-51: wp51-harness/{run-full-census51,run-full-census51b,corpus51,
#  corpus51-diffs,run38,run38-old,ab51}.sh; binario census A+B
#  phpr-memgc51; stash A/B: phpr-wp51a (leva A) / phpr-wp51b (A+B).
# WP-52: wp52-harness/{daemonize.pl (SENZA chdir "/"),corpus52,
#  build-memgc52,run-full-census52,run40,run40-old,ab52,orchestrate52}.sh;
#  binario census phpr-memgc52; stash: phpr-wp52 (sha256 57607da3…).
#  orchestrate52.sh = catena sequenziale census→full-new→full-old→ab.
# ⚠️ launcher: MAI chdir "/" nel daemonizer per i gate phpt — cd root
#  php-8.5.7 DENTRO lo script (bug60771 scrive ./test.php nella CWD).
# WP-53: wp53-harness/{corpus53,build-opcensus53,census53,run41,
#  run41-old,ab53,ab53-report,orchestrate53}.sh; binari OP-census
#  phpr-op53 (phpr-op-target/) e phpr-op52 (phpr-op-target-old/, build da
#  worktree della ROOT repo + php-server/+Cargo.lock copiati a mano);
#  stash: phpr-wp53 (sha256 e75c5abb…). PHPR_OP_CENSUS=1 → dump STDERR.
# WP-54: wp54-harness/{run42,sample42,sample-media,census54,census54b,
#  gate54,ab54,run43,run43-old,orchestrate54}.sh + {attr,aggregate,
#  subtree}.py (parser alberi `sample` + stratificazione) + probe54.php;
#  binario gc-census phpr-gc54{,b} in phpr-op-target/; sample-out/ =
#  finestre win1-6 (full) + mwin1-3 (media) + attribution-table.txt;
#  stash: phpr-wp54 (sha256 e4341e77…). PHPR_GC_CENSUS=<path> → append.
# WP-55: wp55-harness/{build-memgc55,census55-media,gate55,run44,
#  run44-old,ab55,orchestrate55}.sh + probe55-{sem,dtor,concat}.php
#  (sentinelle append + probe O(n), baseline old in probe55-*-old.txt);
#  binario census phpr-memgc55 in phpr-mem-target/ (tree wp54);
#  census-media-out/memcensus.txt = tabella checkpoint Fase 3;
#  stash: phpr-wp55 (sha256 ecc04817…).
# WP-56: wp56-harness/{design56 (quota pin a),gate56,census56-media,
#  build-memgc56,run45,run45-old,ab56,orchestrate56}.sh + probe56-
#  {order,tomb,numkey,dtor}.php (sentinelle 4 assi, baseline old+oracle)
#  + probe56-concat-sites.php (quota `.=` per sito); binario census
#  phpr-memgc56 (tree NUOVO 0943f58) in phpr-mem-target/;
#  stash: phpr-wp56 (sha256 65466c64…).
# WP-57: wp57-harness/{design57.md (note+verdetti),run57-census-full,
#  run57-census-debug,census57-media,aggregate-concat57.pl,daemonize.pl}
#  + probe57-arrlive.php (validazione live-accounting); binari census:
#  phpr-op57 (dcb589ea…, phpr-op-target/) e phpr-memgc57 (41a21c65…,
#  phpr-mem-target/); dati: full-out/opcensus57-debug.txt (tabella `.=`
#  full), census-media-out/memcensus57.txt (arr esatto + arr_shape).
#  Coda di sessione (indagine panic): gate57.sh (baseline = gate-out
#  wp56), probe57-yieldfrom-stale.php (regressione fix e9a1679),
#  run57-census-debug.sh; trappola diagnostica ip>=len nei build census.
#  Stash NUOVO: phpr-wp57 (sha256 a5ae7d27…) — release col fix engine.
# WP-58: wp58-harness/{design58.md (quota su size misurate),gate58.sh,
#  build-memgc58.sh,census58-media.sh,ab58.sh,run46.sh,run46-old.sh,
#  orchestrate58.sh,daemonize.pl} + probe58-*-{old,new}.txt (sentinelle
#  5 assi: order/tomb/numkey/dtor/yieldfrom) + probe58-objlive.php
#  (validazione Ob.2); census-media-out/memcensus58{,b}.txt; binari
#  census phpr-memgc58 (4c9b8da3…) e phpr-memgc58b (bc47f237…, con Ob.2)
#  in phpr-mem-target/; full: full-out/run46/ + wp58-harness/full-old-out/.
#  Stash: phpr-wp58 (sha256 2d7efdf8…).
# WP-60: wp60-harness/{design60.md (criteri P1 pre-registrati + verdetti
#  P2-P4 + scoperta unit-cache),gate60.sh,run48-new.sh,run48-old.sh,
#  orchestrate60-evening.sh,build-memgc60.sh,probe60-listtests.sh,
#  census60-media.sh,census60-full.sh,gap60-media.sh,probe60-p4*.php
#  (+output -{oracle,new,wp58}.txt = sentinelle Stogov pinnate)};
#  out: gate-out/, full-{new,old}-out/ (run48, time -l), census-out/
#  (memcensus60-{listtests,media,full}.txt = seed split + unitpath +
#  ctx), gap-out/ (coppia media). Binario census phpr-memgc60
#  (d1c8dc81…, phpr-mem-target/). ⚠️ telemetria .rss di run48-new
#  agganciata al wrapper time (profilo assente, metro time -l intatto).
# WP-61: wp61-harness/{design61.md (census v2 deep pre-registrato),
#  build-memgc61.sh,probe61-listtests.sh,census61-full.sh}; binario census
#  phpr-memgc61 (9600a662…, phpr-mem-target/); out: census-out/
#  (memcensus61-listtests.txt = v2 attribuito; memcensus61-full.txt =
#  mappa FULL v2, run detached — flag census61-full.done). Batteria HTTP
#  php-server: scratchpad battery61.sh (pattern run-http.sh WP-9, porta
#  8080, docroot ~/Claude/wpdev/src, DB wp).
# WP-62: wp62-harness/{design62.md (recepimento sintesi a 9 + §5 verdetti
#  P0/M2 + §6 decision point),uploads-guard.sh (Gregg R7: backup-wipe|
#  restore, UPLOADS_GUARD_DIR per self-test),build-memgc62.sh,calib62.sh
#  (taratura ±10%),probe62-listtests.sh,census62-full.sh,gate62.sh,
#  probe62-relbase.php,ab62-media.sh,orchestrate62.sh (catena serale:
#  gap media→census full→run49 new/old, guard attorno)}; out: gate-out/,
#  census-out/ (memcensus62-{listtests,full}.txt con colonne prefix +
#  tag=unitcache/cachehit/netguard/reloc), calib-out/, ab-media-out/,
#  eve-out/ (run49). Binari: census phpr-memgc62 (105df139…,
#  phpr-mem-target/); diagnostico phpr-relb62 (97ed6469…,
#  phpr-relbase-target/, feature relbase-probe MAI parità).
#  uploads-backups/ = tar+manifest su drive esterno (guard).
# WP-63: wp63-harness/{design63.md (spec contratto v2 + raffinazione
#  link-time baking + §10 verdetti),probe63-fresh.sh (G1 n=6),
#  sentinels63.sh (matrice E2: KK1'+KS-S2 via PHPR_UNIT_CACHE_LOG),
#  probe63-p1.sh (P1-a..d via -S, docroot p1-doc/),gate63.sh (off|on|
#  flip → gate-out-<modo>/),census63-listtests.sh (KE2/B7/L3),
#  calib63-manysmall.sh (L1),build-memgc63.sh,orchestrate63.sh};
#  out: fresh-out/, sent-out/, p1-out/, census-out/ (memcensus63-lt-
#  {off,on}, memcensus63-full = mappa FULL a flag ON), eve-out/
#  (run50 + media + progress). Binari: census phpr-memgc63 (515b9760…,
#  phpr-mem-target/; tag=reloc + tag=compilens debuttano qui).
#  ⚠️ colonne prefixsum a flag ON dominate dalla mappa eager
#  TRANSIENTE (free fuori-seg): il giudice è net/net_tot, mai le
#  colonne. Env: PHPR_STUB_ELISION=0 torna al contratto v1;
#  PHPR_UNIT_CACHE=0 = cache-off diagnostico.
# WP-64: wp64-harness/{design64.md (recepimento sintesi a 9 + §4-A
#  predizioni K4'' + §4-B spec-sketch leva coi rischi WP-44 + §4-C esiti
#  catena + §9 verdetti),COUNCIL_WP64_REVIEWS.md,gate64-debts.sh (gate63
#  flip + assert KS-S6 alignhit),probe64-s4.php(+inc),build-memgc64.sh,
#  census64-listtests.sh,orchestrate64.sh (media→census full→run51
#  new/old, guard attorno),watch-sample64.sh (B4)}; out: gate-out-debts/,
#  census-out/ (memcensus64-{lt,full} CON colonne pfx_slotnames/pfx_fnvec/
#  sc + compilens map_ns/remap_ns), eve-out/ (run51), sample-out/.
#  Binario census phpr-memgc64 (6d6b518b…, phpr-mem-target/).
#  ⚠️ probe63-p1.sh EMENDATO (detector cache-off cerca `hit (intra|cross)`,
#  mai la sottostringa "hit" — collide con l'evento alignhit).
# WP-65: wp65-harness/{design65.md (recepimento sintesi 9 sedie + §5
#  predizioni K4'' bilaterali + §8 verdetti), COUNCIL_WP65_REVIEWS.md,
#  build-memgc65{,b}.sh, census65-listtests.sh (coppia log-off/on G-65.2),
#  census65-full.sh, sentinels65.sh (+sent-baseline/ = elide per-unit
#  PINNATI), probe65-kee.sh (KE-e stale-id via -S porta 8497),
#  probe65-p1.sh (detector ANCORATI, out-dir propria, porta 8495),
#  probe65-sem.sh (+sem-units/ = S-65.3 a 3 lati), gate65.sh (matrice
#  INTEGRALE K-65.1), orchestrate65.sh, run53-pair.sh (ordine invertito),
#  run54-pair.sh (coppia build-ADIACENTE hyg-vs-leva = il giudice del
#  costo leva), watch-sample65.sh}; out: census-out/ (memcensus65-{lt-
#  logoff,lt-logon,full} pre-leva + memcensus65b-full post-leva +
#  design65.sha256 = snapshot K4''), gate-out/, eve-out/ (run52/53/54),
#  sample-out/, kee-out/. Binari: census phpr-memgc65 (e2a2016a) e
#  phpr-memgc65b (02b7d9d2) in phpr-mem-target/; igiene-only phpr-hyg65
#  (b93a6e91) in phpr-wp65hyg-target/ (worktree: Cargo.lock copiato dal
#  vivo + --locked). Env nuovi: PHPR_MI_COLLECT_EXIT=1 (checkpoint
#  standing exit_collect_mi, census-only).
# WP-66: wp66-harness/{design66.md (recepimento sintesi 9 + §8 verdetti),
#  design66-p2.md (mini-design P-2 de-leak M-66.4), predictions66.md
#  (+.locked + copia repo sessions/WP66_PREDICTIONS.md = snapshot K-66.1),
#  gate66.sh (VERDETTO MACCHINA: FAIL counter, gate66.verdict, done solo
#  a zero; hk/reverse asseriti), sem-baseline/ (sem-oracle.diff.expected
#  + PROVENANCE), probe66-ksp1.sh (KS-P1 replay wpdev N=10, porta 8080,
#  segmentazione uc.log per-richiesta via offset), probe66-s4.sh
#  (batteria S-66.4, docroot s4-doc/, porta 8499), census66-l661.sh
#  (census full su memgc65b + supervisor vmmap del FIGLIO del wrapper
#  time), analyze-l661.pl}; out: gate-out/ (gate66.verdict), p1-out/
#  replay/ (uc.log, seg*.log, fpseq*), s4-out/, census-out/
#  (memcensus66-l661.txt, l661-vmmap-*.txt, l661-attribution.txt;
#  run1 = wrapper, tenuta come lezione). Stash phpr-wp66 (5aa60d56…).
# WP-67: wp67-harness/{design67.md (recepimento sintesi 9 + §8 verdetti),
#  predictions67.md (+.locked + copia repo sessions/WP67_PREDICTIONS.md,
#  commit 34aed31), gate67.sh (K-67.2 trap-FAIL + tot>=1646 + purge;
#  K-67.3 reverse PER NOME; aggancia fixtures/gate-*.sh),
#  gate-baseline/{hk-reverse-fails.txt,PROVENANCE}, analyze-l67.pl
#  (per-causa esatta, unità MiB, dirty/swap separati),
#  build-memgc66.sh, probe67-census.sh (B-67.1: census server
#  PER-RICHIESTA via dump per run top-level; segmentazione reqmark),
#  probe67-nk.sh (P2-a leak-shape N=1000, fixture nk-doc/),
#  probe67-ksp1.sh (rigioco KS-P1 post-P2, segmenti CLASSIFICATI per
#  entry-script del reqmark — il self-traffic WP è visibile),
#  run55-triple.sh (coppia propria + A-A′ nella stessa catena),
#  droporder/ (sentinelle P-2 pinnate, baseline == oracle),
#  fixtures/ordering/ (S-67.2: pin divergenza + gate-ordering.sh)};
#  out: gate-out/ (gate67.verdict PASS), census-out/server/, nk-out/,
#  ksp1-out/, eve-out/ (run55 new/old/new2). Binari census:
#  phpr-memgc66 (cea61455, PRE-P2) e phpr-memgc67 (95d07638, POST-P2)
#  in phpr-mem-target/. Env nuovi: PHPR_MI_COLLECT_REQ=1 (checkpoint
#  standing per-richiesta, census-only).
# WP-68: wp68-harness/{design68.md (recepimento sintesi 9 + §10 verdetti
#  con disposizione K-68.3 di OGNI predizione), predictions68.md (+.locked
#  + copia repo sessions/WP68_PREDICTIONS.md, commit 9bc603f),
#  probe68-pre.sh (precondizioni KS-P68.1 a verdict), probe68-ksp1.sh
#  (KB67-2: attese cold=0/hit_cross=512), probe68-census.sh (quota
#  S-68.4, G-68.4 ×3 run, KS68-2 nel verdetto), probe68-sem.sh (S-65.3
#  forma WP-68: DOPPIO diff pinnato, sem-baseline/+PROVENANCE),
#  probe68-b683.sh (B-68.3 slope wpdev N=1000, slope-only L-68.3),
#  build-memgc68.sh, census68-metro.sh (KL67-1 con protocollo no-swap
#  L-68.4), gap68-media.sh, run56-triple.sh, gate68.sh (K-68.1 verdict
#  .superseded / K-68.4 trap self-test / M-68.1 assert park),
#  fixtures/{gate-ordering-s68.sh (5+3 casi 2 LATI, pins/+PROVENANCE),
#  gate-droporder.sh (P-68.1), gate-editmiss.sh (P-67.4),
#  ordering-h68/ (f-intra g-declared h-livecond)}}; out: gate-out/
#  (gate68.verdict PASS + .superseded-*), pre-out/, ksp1-out/,
#  census-out/ (quota-table, memcensus68-metro), b683-out/, eve-out/
#  (run56 new/old/new2), stubs-out/, gap-out/. Binario census
#  phpr-memgc68 (5027e4e8) in phpr-mem-target/. ⚠️ ri-pin WP-68 con
#  PROVENANCE: gate-ordering (atteso=oracle), sem-baseline (doppio pin),
#  sentinels65 main.elide 177→176 (copia .pre-wp68 accanto).
# WP-69: wp69-harness/{design69.md (recepimento sintesi 9 + §9 verdetti
#  con disposizione PER BANDA), predictions69.md (+.locked b91b4799 +
#  copia repo sessions/WP69_PREDICTIONS.md, commit 07ef494),
#  gate69.sh (gate68 + fixtures s69 + baseline corpus69), gate-baseline/
#  {corpus69.fails (1422), PROVENANCE-corpus69.txt},
#  fixtures/{gate-ordering-s69.sh (9 unit 2 LATI + 6 redecl),
#  gate-h69cycle.sh, gen-pins69.sh, pins/ (+PROVENANCE-s69), s69-*/ (9),
#  h69-cycle/, redecl/}, probe69-l681.sh (L-68.1 3 leg),
#  probe69-cron.sh (B-69.5, inject/restore DISABLE_WP_CRON),
#  count-handler-bodies.sh (K-69.3, baseline 178),
#  build-memgc69.sh}; out: gate-out/ (gate69.verdict PASS run2),
#  l681-out/ (l681-table), cron-out/ (cron-table). Binario census
#  phpr-memgc69 (f034e62a) con PHPR_CENSUS_HITS=0 off-switch +
#  tag=censusown capacity-aware. ⚠️ ri-pin WP-69 con PROVENANCE:
#  corpus69 1422, sentinels65 redecl.elide 2 righe (.pre-wp69).
# WP-70: wp70-harness/{design70.md (recepimento sintesi 9 + §7 verdetti),
#  COUNCIL_WP70_REVIEWS.md, predictions70.md (+.locked cbf251fe con
#  ADDENDUM pre-letto + copia repo sessions/WP70_PREDICTIONS.md),
#  gate70.sh (etichette K-70.4, corpus70, cargo>=1650),
#  gate-baseline/{corpus70.fails (1421), PROVENANCE-corpus70.txt},
#  fixtures/{gate-h70cycle.sh (8), gate-s70neg.sh (12),
#  gate-h70trait.sh (5), h70-*/, s70-neg/, h70-trait/, pins/ con
#  PROVENANCE-h70}, probe70-step0.sh (P70-0-bis vmmap two-boot),
#  probe70-tripla.sh + analyze70-tripla.pl (segmentazione: snapshot =
#  blocco mi_bin chiuso da tag=modrecon — win= NON è il contatore),
#  build-memgc70.sh}; out: gate-out/ (gate70.verdict PASS 1° run),
#  step0-out/ (step0-table), tripla-out/ (tripla-table2, census1-3).
#  Binario census phpr-memgc70 (9201ed94) con tag=cellskip (M-70.2)
#  + tag=defermini (P70-D: calls+net_b run_deferred). ⚠️ ri-pin WP-70
#  con PROVENANCE: corpus70 1421 (−1 class_order_autoload1).
# WP-71: wp71-harness/{design71.md (recepimento sintesi 9 + §7 verdetti
#  con attribuzione chiusa), COUNCIL_WP71_REVIEWS.md, predictions71.md
#  (+.locked, catena lock ADDITIVI 05faefcc→27f512d3 + copia repo
#  sessions/WP71_PREDICTIONS.md), gate71.sh (fixture WP-67..71 +
#  attempt-counter K-71.2 + baseline corpus71), gate-baseline/
#  {corpus71.fails (1420), PROVENANCE-corpus71.txt},
#  fixtures/{gate-h71walk.sh (22), gate-s71.sh (15), gate-walkborrow.sh
#  (K-M71.2 grep-gate), h71-walk/, s71/, pins/ con PROVENANCE-h71},
#  hoare71/ + stogov71/ (fixture dei verbali, salvate dallo scratch),
#  analyze71.pl (parser G-71.3/G-71.4 + straddle + src=defer),
#  probe71-tripla.sh, probe71-reanalyze.sh (causa obbligatoria K-71.4),
#  probe71-ladder.sh (L-71.4), probe71-cron.sh (B-70.1/B-70.4),
#  probe71-amp-analyze.sh (verdict amplificazione), analyze71-blockdump
#  .pl + analyze71-bddelta.pl (KL71-2), build-memgc71.sh}; out:
#  gate-out/ (gate71.verdict PASS + .attempts), tripla-out/,
#  ladder-out/, cron-out/ (cron-table71), amp-out/, blockdump-out{,2}/.
#  Binari census phpr-memgc71 (2c4691d7) + 71b (91038d46) + 71c
#  (blockdump ≤1024) in phpr-mem-target/. Env nuovi (census-only):
#  PHPR_MI_BLOCKDUMP=<base> + PHPR_MI_BLOCKDUMP_AT="500,1000".
#  ⚠️ ri-pin WP-71 con PROVENANCE: corpus71 1420 (−1 ns_064).
# WP-59: wp59-harness/{design59.md (verdetti Ob.0-3 + gate pre-registrati),
#  ob0-media.sh,ob0-full.sh (diagnostiche MIMALLOC_SHOW_STATS+vmmap),
#  build-memgc59.sh,ob1-census59-media.sh,ob1-supervisor.sh (finestre
#  phys: vmmap+footprint per flag-file),ob1-report.pl (tabella
#  riconciliazione+top bin),build-pooloff59.sh,run47-pooloff.sh,
#  run47-new.sh,run47-scan4.sh,orchestrate59-evening.sh}; out: ob0-out/,
#  census-media-out/ (memcensus59-master.txt = solo master pid),
#  full-{pooloff,new,scan4}-out/ (CPU esatta in *.time). Binari: census
#  phpr-memgc59 (00ca4d3d…, phpr-mem-target/); diagnostici phpr-pooloff59
#  (93aea5a3…, phpr-pooloff-target/), phpr-scan4-59 (b955afb1…,
#  phpr-scan4-target/). Probe scratchpad: probe59-{inc,boot,classes,
#  funcs,null} (leak template + moltiplicatore compile-side).
```

## 🎯 PROSSIMO LAVORO — WP-73: punto-0 concilio → ancora-stash + media-pair -S → DESIGN axum lockato → migrazione (forma pool N=1)

**Rotta (utente 2026-07-24)**: `FOOTPRINT_CPU_ROADMAP.md` — footprint-first,
safe-only, TUTTE le fasi, no-revert. Stato WP-72: leak per-richiesta
CHIUSO (tripla 0,000 + peak flat), axum SBLOCCATO (dettaglio:
`sessions/WP_SESSION_72.md` + `wp72-harness/design72.md` §7).

⚖️ **CONCILIO DI CHIUSURA WP-72 ESEGUITO (10 sedie: 9 + web-runtime
co-optata): 0 concordi secchi · 9 con emendamenti/riserve · 1 REFUTATO
PARZIALE (Stogov: canale errore/exit del teardown non Zend — 1 PANIC
da PHP puro su ob-handler a shutdown, t3). Verbali INTEGRALI + sintesi
in `wp73-harness/COUNCIL_WP73_REVIEWS.md` — VINCOLANTI per design73,
recepire INTEGRALI prima di toccare codice. Deliberazioni chiave:
ordine dei fronti RIORDINATO (Bak); forma axum = pool worker N=1
pinnato, byte di risposta al writer attuale (web-runtime); contratto
di teardown NEI TIPI via `Vm::finish_request(self)` (Hoare); il breach
K-72.2 su probe72-h2 è stato SANATO col rigioco in chiusura (6e6d966).**

0. **🔴 PUNTO-0 (delta-zero, PRIMA di ogni riga axum)**: S-73.1/2/3
   canale errore/exit del teardown (eccezione-in-dtor = Fatal+abort
   walk; RIORDINO shutdown-fns → ob_end_all su VM viva → dtor-walk →
   break — il PANIC t3 vm/mod.rs:10729 vive qui, KS-S73.1 =
   pre-condizione del primo gate axum; exit-code propagato; batteria
   t1-t12 → gate-d73 con RC-PARITY) · E-73.H1/H2 trait-identity
   (redecl cross-unit per SPAN + fatal a DeclareTrait; autoload
   nested-use case-preservato — pre-requisito Linux/PSR-4) · M-73.1
   DimBase::This (m9/m11) · H-73.2 tripwire phasea_skip + H-73.3
   fixture d7/d8 · E-73.H3 tag=condfallback (atteso 0) · L-73.2
   attribuzione one-shot dei 20 alive_after per campo · K-73.1 diff
   per-NOME dei 7 fp-load (505 vs 512, log su disco) · G-73.2 guardia
   anti-leak FISSA nel gate73 (hist N=200 + amp K=10 single-leg, con
   controllo positivo EMBEDDED — KG73-1) · correzioni K-73.3 GIÀ
   APPLICATE alla rotazione in chiusura WP-72.
1. **Misure-ancora (Bak, PRIMA del design axum)**: B-73.1 ancora-stash
   wp56-vs-wp72 stessa-sera (deriva +17,7%; KB73-1: wp56 ≤760s ⇒
   righe ×-CPU GAP_TREND congelate, spike ri-prioritizzato) · B-72.4
   media-pair -S post-leva (ULTIMA baseline -S possibile) + K-69.3 +
   recinto P70-S · B-73.3 timer tag=teardown ns/req (cap ≤500 µs).
2. **DESIGN AXUM lockato (predictions73 PRIMA del primo commit)**:
   forma (b) pool worker PHP dedicati IMMORTALI, **N=1 pinnato**
   (spawn_blocking/LocalSet REFUTATE); canale mpsc+oneshot SOLI dati
   piatti (Rc nel tipo o unsafe impl Send = FATAL, grep-gate M-73.2);
   byte di risposta al WRITER ATTUALE (pin WP-4: no Content-Length,
   case header, router-prefix raw — KW73-2 diff RAW sapi-probe);
   H-73.1 `Vm::finish_request(self)` consumante (unico call-site
   CLI+axum); KH73-1 boot assert FAST_SHUTDOWN==false + reg>0;
   contatori aggregati o thread-id costante (KS73-H1); P-73.3
   contratto confine (parità -S vs axum; teardown su OGNI uscita,
   panic incluso; respawn==0); lock G-73.1/K-73.4: tripla post
   [−0,009,+0,5], uc-steady SCOPE PER WORKER pre-definito, latenza
   p50/p95 ≤+1% (hello ≤+100 µs/req), cap CPU BILATERALE.
3. **Migrazione + gate-axum** (fixture concorrente inclusa, KK73-4).
4. **Spike dispatch** (sessione dedicata, baseline DICHIARATA post
   punto 1 — mai mista).
4b. **DEBITI AUDIT SOL (2026-07-30, archiviati come apertura WP-73)** —
   ricognizione esterna in `20260730-chatgpt-sol.md` (fatti verificati,
   allineata alle nostre discipline). **Pacchetto A GIÀ FATTO in
   chiusura WP-72** (f0bb8b8: LICENSE BSD-3-Clause = stessa di
   php-src/Zend da PHP 8.4, Cargo.lock tracciato, rust-toolchain
   1.96.0, .cargo/config untracked, falso-verde differential chiuso).
   Da portare al CONCILIO WP-73 come emendamenti del design axum:
   (i) **P0.4 hash-DoS**: `rustc-hash` sull'indice di `PhpArray` NON è
   DoS-hardened e le chiavi arrivano da input remoto (query/JSON/form)
   — con axum esposto entra nel threat model: hasher resistente sulle
   tabelle raggiungibili da input O(collision-cap) + benchmark del
   costo + test avversariale (co-optare una sedia security al concilio);
   (ii) **P0.2 soundness FFI**: wrapper safe in gdio.rs che
   dereferenziano raw pointer → `unsafe fn` + `# Safety` o newtype
   NonNull; `#![deny(unsafe_op_in_unsafe_fn)]` nei crate FFI;
   (iii) **P0.3 perimetro server**: threat model + resource limits
   (memory/instruction/recursion/body/upload) + etichetta esplicita
   "development server" fino a hardening; (iv) rustfmt/Clippy verdi
   (un commit solo-format + policy) — NON durante la migrazione, dopo.
   **Fase C dell'audit (CI matrix, sanitizer, fuzz, SBOM, release
   engineering) = SESSIONE DEDICATA "hardening di prodotto" DOPO axum**
   (decisione utente: "ok per Axum poi C"), con proprio design e
   concilio (sedia security/release-engineering co-optata).
5. **Backlog**: ways/fp multi-flip (B-72.5) · typed-const retention ·
   get_included_files · S-71.3 · famiglie STRING-OFFSET-BIND /
   DIAG-LINE / quiet-fetch / EVAL-NAMING / parse-error · S-72.5 ·
   L-73.3 riconciliazione Σ una-tantum · L-72.4/L-72.5 (condizionali
   espliciti) · cache deferred E-71.H3 (design-only, E-73.H6).

**VETI (restano vincolanti)**: invariati WP-64..72 + nuovi dal
concilio: **token verdict asserito dal RUNNER** (mai note post-letto —
la deroga ladder non fa precedente, KG73-3/KK73-1) · **letto con
mtime < commit del runner = NULLO** (KK73-2) · **assert di
non-vuotezza sui file confrontati** (KK73-3) · **runner-contatore con
controllo positivo embedded** (KG73-1: mai più PASS su tutto-zero) ·
**cap letto a metà si dichiara PARZIALE** (KG73-4) · **uc-scope mai
ridefinito post-letto** (KG73-2) · verdict di coppia CPU parsa anche
instructions-retired (B-73.2).

## (storico, pre-roadmap) PROSSIMO LAVORO

0. **PRE-FLIGHT = skill `apri-sessione`** (decisione utente 2026-07-26:
   disco×2 / MySQL con datadir giusto / hash binario di parità /
   residui detached / uploads / ordine di lettura — procedura in
   `~/.claude/skills/apri-sessione/SKILL.md`). ⚠️ path CANONICO dello
   stash = drive esterno `phpr-old-target/release/` (lo stash in
   ~/Claude ha prodotto un A/B con 12 run da 0,00s — WP-50).
1. **Rotta ([[php-rust-roadmap-wp-first]])**: dopo la roadmap
   footprint+CPU si apre la VALIDAZIONE LARAVEL (metodo = ricetta gate
   ORM/hk: oracolo+composer build, phpr esegue).
2. **Backlog pescabile da [[php-rust-todo-master]]** — candidato pronto:
   🆕 bug isset via prefisso `__get` con indici annidati
   (probe wp42-harness/probe-isset-div.php §3).
3. **NON riproporre**: stadi 3-4 registri; leve locali canale churn
   (esaurite WP-33/34/41/42); NaN-boxing; SSO union.

## Backlog aperto (non legato a una sessione)

- 🆕 **Panic census-only** (WP-57): prima run full con binario op-census →
  panic master a ~6 min, `run.rs:478` `index out of bounds: len 78,
  index 78` (PC oltre fine func.ops); rerun identico completa la suite ⇒
  non-deterministico; MAI visto sui binari di parità (~45 full run).
  Evidenza: `wp57-harness/full-out/full-census.txt` + WP_SESSION_57.
- 🆕 **isset via prefisso `__get` con indici annidati** perde il walk sul
  risultato — bug funzionale preesistente (WP-42). Candidato fix.
- 🆕 Deprecation PHP 8.5 (chiave null/float-frac) non emesse dentro
  isset/empty (`coerce_key_silent` muto); `isset($nonAA['k'])` non lancia
  Error — famiglia quiet-fetch, catalogare in PHPR_DIVERGENCES se si
  decide di non chiudere.
- Residui strutturali: `ast_printing.phpt` (serve zend_ast_export
  sull'HIR) · xsl `bug69168` (nodi php:function devono aliasare il doc
  live) · tidy `010` (free-order var_dump-di-albero).
- Ret-hook usa ancora gc_cascade (non gc_release_cascade) per oggetti con
  `__destruct` nel subtree — nessun test lo copre oggi.
- Verbo "increment/decrement" per `$null->p++` (oggi "assign").
- Se si toccano date/prelude DateTime: gate ext/date OBBLIGATORIO (351).
- str residuo walk 3% (~39MB metadati moduli) · interning const-array ·
  gc_047 (release iteratore al break) · gc_030 (trace shape) · gc_022
  (temp mid-statement).

## 📊 Report gap perf — ricorrente di fine sessione

Trend cumulativo e metodo di misura: **`gaps/GAP_TREND.md`** (file unico
vivo); misure per-sessione in `gaps/REPORT_GAP_<N>.md`. A ogni chiusura:
misurare media (user CPU + footprint) e full-suite master-CPU, scrivere
REPORT_GAP_N (sola sessione N), aggiungere la riga a GAP_TREND, riportare
il gap all'utente.
Ultimo stato (WP-70, sessione fedeltà+caccia — gap classico NON
rimisurato): **riferimenti invariati: media CPU 2,58× · footprint
media ~3,0-3,1 (banda 2,9-3,2) · full CPU 2,06-2,11× · peak
~1,98-2,03GB · server hit_cross 512/512 · corpus RI-PIN 1422→1421
(−1 fix reale) · 🔴 residuo per-request CONFERMATO E MISURATO:
esiste su RELEASE (Δ5,8 MiB/4000 req) + tripla 2,1121 KiB/req ·
20,000 obj/req (spread 0,024/0,003 = MDE fondato) · defer path
16,00 calls/req · attribuzione ≥80% APERTA ⇒ axum FERMO · baseline
parità phpr-wp70 (a666382e…, gate70 PASS fails=0 1° run) ·
dettaglio: `sessions/WP_SESSION_70.md`, trend: `gaps/GAP_TREND.md`**.
