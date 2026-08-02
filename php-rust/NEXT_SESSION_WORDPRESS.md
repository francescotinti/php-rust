# NEXT_SESSION_WORDPRESS.md — S-87.0: I-RI-GIUDIZI-E-LA-CAMPAGNA-CHE-SI-È-DIFESA-DA-SOLA (VERDICT87 PASS attempt=3) → WP-88(sessione)

**Ultima sessione**: S-87.0 (2026-08-02, commit d50f3b5…8793d65 + chiusura)
— gli 8 punti del Concilio WP-88 eseguiti: ri-giudizi DAI RAW (A-BB49:
**VOVL ribaltato — overlap 10/10**, per-thread sotto concorrenza
REFUTATO-DAI-RAW; A-BB51: VABBA su peak footprint **SEPARA 8/8**, purge=0
abbassa 20.926.464 B = 19,96 MiB, delta tutto nella regione arena;
correzione MEASURE86 + A-BG44-forma; A-DS40: §3.3-ter EMENDATA innesco
PERSIST + 7 fixture-ancora live), fix strumenti (A-DL37 dump atomico ·
A-DL36 clamped end-to-end con dente positivo · A-MS36 flag thread_local ·
A-PP39 dispatch row su union), catena evidenza (A-SK46 --same-rev coi
denti PIENI · A-AH43/44/45 · A-BG44/45), sigilli v5 (A-TH40/41/42/43 con
positive-control tainted same-commit · A-MS37/38/39 · A-SK48 · A-PP37/38/
40 · A-DS38 putord-pair-guard + A-DS39), **CAMPAGNA measure87**:
battery-87pre 15/15 → consumo --same-rev verificato da MACCHINA (prima
volta) → a1 VOID (server ORFANO: $! era /usr/bin/time — VERDICT87 FAIL(67)
ha MORSO) → a2 VOID (dente anti-orphan morso sul PROPRIO wrapper) →
**attempt=3 PULITO, VERDICT87 PASS**: VSLOPE NAMED-DEVIATION (slope LSQ
committed = 25.880.166 B = 24,68 MiB per worker, FUORI banda KL-85-2),
VARMS 0 B (eager/minstack sottratti per NOME), VDISJ predizioni ex-ante
CONFERMATE (firma inghiottimento nest-free +310 B; floor_inc
byte-invariante cal↔conc); delibere p6 (promozione scomposizione
VERDICT-GRADE **SCOPED alle ANNIDATE**; A-DL39+A-BB50 = DESIGN in
design87.md); contratto A-DS41 in todo-master.
Dettaglio: `sessions/WP_SESSION_87.md` + `wp87-harness/MEASURE87_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **eff98c88eb95c254** (stash additivo
  `phpr-wp87`; bit mossi da A-TH40 rebind + campo
  main_program_net_clamped A-DL36 + A-MS36/A-PP39) — corpus Zend per
  NOME: **1418 IDENTICO** + refl 290 IDENTICO (parity-full verde in
  battery-87pre, doctest VmGate CONTATI ==2 A-MS37).
- **php-server**: battery-87pre **15/15 CONTATO a 6d9a80f** (stamp
  ledgerato committed a9e18fc; matrix COMMITTED; A-AH45 snapshot
  bidirezionale); campagna measure87 a 38961f0 **attempt=3** (a1/a2 VOID
  nominati, ledger campagna APPEND-only) consumata via
  **battery-equivalence --same-rev** (A-SK46: anchored PASS + sha256 +
  stamp 4 campi + matrix committed + toolchain A-AH44 — KS-SK-88-1
  CHIUSA). mem-census 2a15f2085d4dcdf8. Gate lever: pins v5 (A-TH41/42)
  / fixtures / fixtures2 tutti PASS.
- **Misure**: `wp87-harness/MEASURE87_RESULTS.md` + verdict87.out —
  VSLOPE min-of-R committed 68.681.728 / 103.153.664 / 116.785.152 /
  150.405.120 B (W=1..4), slope 25.880.166 B = 24,68 MiB per worker
  (NAMED-DEVIATION vs 3.605.572 B ±5%) · VDISJ cal gemelle 7.801.102 B
  ESATTE, conc padB=calA+calB+310 B (2/2), floor_inc delta=0 (2/2) ·
  VARMS delta 0 B. ⚠️ nets concorrenti VOID come cifre per-thread
  (KB-88-1) fino ad A-BB50 attuato.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
porta vm_new/park_main = token `VmGate<'gate>` LIFETIME-BOUND nel
micro-modulo foglia `gate` (v4: anchor `&mut ()`; !Send/!Sync con
compile_fail E0277 CONTATI ==2 in parity-full, A-MS37/KS-MS-88-2).
4. **Panic = FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒ git revert,
mai fix-forward** (KS-DS-80-3, mai innescata). 6. **Mai rm di raw:
quarantena + manifest; mai RIUSARE un filename — attempt= nel nome, ledger
campagna APPEND-only** (KS-AH-83-2/KG-88-1/A-BG43). 7. **Cifre memoria
BYTES-FIRST con companion VERIFICATO, MiB-only, bande da allowlist a
scope di RIGA** (A-DL26/A-SK40/A-SK43/A-SK48). 8. **Cifra da binario =
hash contro matrix + marker #[used] con coupling pinnato** (KH86-1+
A-TH37/A-TH41). 9. **Peak/spread SEMPRE con NOME della metrica —
footprint vincolante, mai RSS nudo** (KB-88-2). 10. **Una battery si
consuma SOLO via battery-equivalence (path equivalence O --same-rev), mai
a mano** (A-SK46/KS-SK-88-1).

## §WP-88(sessione) — S-88.0 = ordine Concilio WP-89 §Sintesi (non rinegoziare)

(compilato alla convocazione del Concilio WP-89 — v.
`wp89-harness/COUNCIL_WP89_REVIEWS.md` §Sintesi; il blocco ⚖️ sotto lo
dichiara VINCOLANTE.)

**Kill-switch di rotta ereditati ancora attivi**: KH88-1..4,
KS-MS-88-1..3 (KS-MS-88-1 sollevata per i run ≥ rev 400fa10: flag
thread_local), KS-SK-88-1..4, KS-AH-88-1..2, KB-88-1..3, KS-PP-88-1..3,
KL-88-1..3, KS-DS-88-1..3, KG-88-1..3 — tabella in
`wp88-harness/COUNCIL_WP88_REVIEWS.md`; più gli ereditati WP-87 come da
tabella. KS-DS-80-3 invariata.

**NON riproporre**: tutti i NON-riproporre WP-83/84/85/86 restano; in più
— "VOVL OPEN / per-thread sotto concorrenza CANDIDATO" (REFUTATO dai raw,
ri-giudicato S-87.0); "purge refutato come driver" su metrica RSS (su
footprint purge=0 ABBASSA, separazione 8/8); Δpeak d'avvio come metà
fisica (la forma è slope committed — che a W∈{1..4} dà NAMED-DEVIATION:
non riproporre la banda KL-85-2 su quel protocollo senza riconciliazione);
canary concorrenti come cifre per-thread PRIMA di A-BB50 attuato;
`pgrep -f` per contare processi wrappati (una cmdline di wrapper matcha);
`$!` come pid del server sotto /usr/bin/time.

---
**Chiusura**: 2026-08-02. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
