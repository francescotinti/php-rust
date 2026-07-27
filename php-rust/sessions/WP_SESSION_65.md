# WP_SESSION_65 — LEVA TRANCHE 2 slot_names v2-style SPEDITA (counted −51,63MB alla cifra, CPU leva −0,21%, tre coppie full 88 BYTE-ID, gate65 matrice INTEGRALE verde)

> ⚡ **WP-65 (2026-07-27 pomeriggio, `ad15ada`→`8a03d6e`)** — sintesi a 9
> del concilio recepita INTEGRALE in `wp65-harness/design65.md` PRIMA di
> ogni codice; PRIMO ATTO = igiene pre-leva (commit `6619b7a`); poi LEVA
> slot_names **forma v2-style di Hoare** (commit `04d559b`): il `{main}`
> eliso cede l'INTERA name-table alla copia canonica `seed_globals` del
> VM (`Func.seed_slots`, slot_names vuota — `program.slots` è tutto nel
> seed al link perché `apply_seed_delta` precede la compile su ogni
> path). Emissione intatta: gli op bakano gli stessi indici;
> `PHPR_STUB_ELISION=0` = v1 pieno (rollback intatto).

## Primo atto (igiene del concilio, `6619b7a`, delta zero)

- **B-65.2** uc_log BUFFERIZZATO (fd once + buffer thread-local; flush
  SOLO ai confini include/unit/exit; eccezione documentata: pre-panic).
- **P-65.4** vocabolario eventi uc_log CHIUSO (`UC_LOG_EVENTS`) +
  debug_assert all'emissione + test cargo anchored-prefix-free (1646°).
- **M-65.1** colonna `seedlen` per-unit + `tag=seedlen units/distinct`;
  **M-65.2** `miss_dc` disaggregato (base/remap/locals); **M-65.3**
  check pun sul bootstrap main (`run_module_with_hir`); **B-65.3**
  colonne `read_ns/lexparse_ns/lowerhir_ns/lowers`; **H-65.1** nome
  classe nel panic; **H-65.2** debug_assert basi in apply_seed_delta;
  **L-65.2** checkpoint `exit_collect_mi` (mi_collect(true), env-gated).

## Misure pre-leva (census65: memgc65 e2a2016a, log OFF)

- **G-65.2 CERTIFICATA**: coppia lt log-off/log-on, TUTTE le finestre
  `*_ns` entro ±10% (max +6,2%) — l'observer-effect WP-64 è morto
  STRUTTURALMENTE (remap lt log-on 0,090s vs 4,573s WP-64).
- **P65-A**: colonne lt BYTE-IDENTICHE a memcensus64-lt (18845/22824/
  278 su version) — a verbale: banda mia mal riferita (full vs lt),
  bordo side-by-side esercitato e risolto; "misurato", mai "✓".
- **M-65.1 FORMALE (full, master)**: units=4726, distinct=116 ⇒
  **D/U=0,025** — K-M65.1 non scatta; v2-style resta per DOMINANZA
  pre-registrata (il letto poteva solo uccidere la Rc-prefix).
- **B-65.3 (KB65-3 saldato)**: lower 34,91s all-proc (≈4,4% del full)
  = read 2,64 + lexparse 14,25 + lowerhir 14,39 (89,6%; coda = lower
  parziali dei retry, 58.477 lowers vs 54.342 unit). map+remap 0,37%
  — canale CPU O(seed) morto RICONFERMATO.
- Baseline: `slotnames_tot=51.615.151` (replica WP-64 esatta),
  net_tot 499,9MB, miss_dc=0 sul full CLI (M-65.2: letto utile = server).

## Leva e gate (design65 §4: censimento consumatori H-65.5 completo)

- 9 siti runtime censiti; 7 convertiti agli helper freddi outlined
  `unit_slot_name/pos/count` (bridge drive_unit, var_dyn r/w,
  bind_global_dyn, get_defined_vars, frame_local); frames[0]/closures
  intoccati (seed_slots=0); nessuna traduzione d'indice; nessun oggetto
  condiviso mutabile (KS-P65.1 per assenza).
- **GATE65 MATRICE INTEGRALE VERDE (K-65.1, zero deroghe)**: cargo
  **1646/0** · sentinelle 5 assi BYTE-ID · sentinelle65 5/5 (coppia
  v1-vs-v2 REALE off-leg=0, elide per-unit PINNATI in sent-baseline/,
  miss ancorati, KS-S6 per-fixture) · **KE-e stale-id DEDICATO verde**
  (miss fp su chain diverso, miss cold su rewrite) · **P1-a..d NEL
  gate, hit_cross=2** (P-65.5 rigiocata post-leva) · S-65.3 byte-id ·
  corpus **1421 IDENTICO** · refl **290** · ORM **3E/13F IDENTICO** ·
  hk **0E/0F** · reverse **2F gemello**.

## Verdetti (design65 §8, predizioni K4'' bilaterali + snapshot sha256)

- **KS65-3 ESATTO**: slotnames_tot 51.615.151 → **0** (residuo 0%).
- **P65-C DENTRO banda [40,62], al centro**: net_tot 499,9→448,3MB =
  **−51,63MB**; compile-side counted cumulato da WP-62:
  1.973,3→448,3 = **−77,3%**.
- **KL-65.1 non scatta (L-65.3)**: Δcommitted bin 8-64B al checkpoint
  collect = −32,7MB = 63,3% del counted (il resto nei bin medi dei
  fat-pointer array). Slack NON migrato.
- **P65-D con attribuzione**: coppia ADIACENTE run54 (hyg vs leva)
  peak **−51,8MB ≈ counted ×1,00**; coppie vs stash {−86,3, −15,8} =
  rumore peak ±35MB attorno alla stessa media. Peak full ora
  **~1,98-2,03GB**.
- **P65-E: bordo eseguito e chiuso** — run52 +2,16% e run53 (ordine
  invertito) +1,23% vs stash wp64; sample: helper 0,03% (scagionati);
  **run54 DISCRIMINANTE (build adiacenti): −0,21% ⇒ costo CPU della
  leva ZERO**; il +1,2-2,2% vs stash = spread build-vs-stash (classe
  run51 −1,5%, segno opposto — non si cita).
- **Fail-set: TRE coppie full (run52/53/54) tutte 88 BYTE-ID ×2 =
  run33.**
- **G3 terza coppia media**: CPU 2,57× (20,89/53,77u) · footprint
  **3,01×** (382,2/1.150,6MB) — coppie {2,98, 3,01, 3,01}, mediana
  3,01: KG63-A ha le sue ≥3 coppie, banda consolidabile ~3,0×
  (ratifica al concilio).
- Catalogo: PHPR_DIVERGENCES **§3.8(vii)** (S-65.3: get_defined_vars
  unit-toplevel, ordine $GLOBALS su scritture esplicite, warning
  undef-var muto sul main CLI — pre-esistenti, identiche in wp64).

## Stash e harness

Release = **phpr-wp65 (`778f8ead…`)** additivo accanto a wp64; census
phpr-memgc65 (e2a2016a…, pre-leva) e phpr-memgc65b (02b7d9d2…,
post-leva); igiene-only phpr-hyg65 (b93a6e91…, phpr-wp65hyg-target/,
per la coppia discriminante). Harness: `wp65-harness/{design65.md,
COUNCIL_WP65_REVIEWS.md, build-memgc65{,b}.sh, census65-listtests.sh,
census65-full.sh, sentinels65.sh (+sent-baseline/ elide per-unit),
probe65-kee.sh, probe65-p1.sh (detector ancorati), probe65-sem.sh
(+sem-units/), gate65.sh, orchestrate65.sh, run53-pair.sh,
run54-pair.sh, watch-sample65.sh}` + out-dir (census-out/ con
design65.sha256, gate-out/, eve-out/ run52-54, sample-out/, kee-out/).

## ⭐ Lezioni

- ⭐⭐ **La coppia build-adiacente è l'unico giudice del costo di una
  leva**: vs lo stash il delta CPU era +1,2-2,2% (riprodotto su due
  coppie, ordine invertito!) ma la coppia hyg-vs-leva dice −0,21% —
  lo spread build-vs-stash da linking layout è ±1,5-2% e AVEVA già
  un precedente etichettato (run51 −1,5%); una banda CPU pre-registrata
  più stretta dello spread storico documentato scatta da sola.
- ⭐⭐ **Outlined = misurabile**: gli helper #[cold] compaiono per nome
  nei sample — l'assoluzione (0,03%) è diretta, non inferita.
- ⭐⭐ **Il peak di coppia singola ha rumore ±35MB** (run52 −86, run53
  −16, run54 −52 sulla STESSA leva): il phys si cita dalla coppia
  adiacente o dal checkpoint standing, mai da una coppia sola vs stash.
- ⭐ `[^/]+$` in perl cattura anche il newline finale (`.` no): il
  pin per-unit usciva spezzato — `[^/\n]+$`.
- ⭐ daemonize.pl: la out-dir del log DEVE esistere PRIMA (due lanci a
  vuoto in sessione).
- ⭐ Worktree per build storiche: Cargo.lock del worktree può differire
  dal vivo — copiare quello del repo e buildare `--locked`.

## Prossimo (WP-66) — vedi NEXT_SESSION §WP-66
