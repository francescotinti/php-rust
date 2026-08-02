# NEXT_SESSION_WORDPRESS.md — S-88.0: LE-DUE-LETTERE-REFUTATE-E-LA-CAMPAGNA-PULITA-AL-PRIMO-COLPO (VERDICT88 PASS attempt=1) → WP-89(sessione)

**Ultima sessione**: S-88.0 (2026-08-02, commit 456c1b0…c0f00d0 + chiusura)
— gli 8 punti del Concilio WP-89 eseguiti, con DUE lettere refutate a
MACCHINA prima dell'esecuzione: (p1) il 6° observable `new` pre-decl dal
vivo dà **persist `C` exit 0 ≡ phpr** (la refutazione capitale n.1 del
Concilio era falsa; claim «phpr ≡ persist» ANCORATO, KS-DS-89-2
soddisfatta; riqualifica const-folding DECADUTA); (p5) la lettera A-DS45
«≥1 coppia main_evicted in produzione» è INSODDISFACIBILE (main_evicted =
tripwire A-MS24, scatto ⇒ VOID per KS-DS-84-4) — riqualificata su lane
SUPERSEDE con smoke a macchina. Fix strumenti (mi_option read-back
A-DL41 col positivo che morde da solo, mi_arena in-band A-BB55,
identity v2 A-BG47, thr-set esatto A-PP42, port-owner A-PP43), catena
evidenza v6 (A-SK50 allowlist finestra + ledger-prefix, A-AH46/47),
sigilli v6 (ProbeWindow RAII A-MS40/41, sweep A-TH44/45, F16b A-DS42,
gate-axum-tests A-PP41 — F16b ha morso sul PROPRIO harness al 1° giro),
**CAMPAGNA measure88**: battery-88pre 16/16 → consumo --same-rev v6
(prima volta con allowlist) → **attempt=1 PULITO, VERDICT88 PASS g1**:
b = 21.195.981 B = 20,21 MiB/worker sui MODI DOMINANTI W∈{4, 8, 12, 16}
(NAMED-DEVIATION: banda KL-85-2 non torna in NESSUN regime; granuli
64 KiB residuo 0; mi_arena==mi_proc commit AL BYTE), VWARM (surplus
timing-attached con INVERSIONE di lato; floor-collapse in warm
1.161.206→164.368 B; **STAG = zero-swallow al byte: l'inghiottimento è
PURO artefatto di overlap**), VUCLOG PASS in forma riqualificata;
delibere design88 (A-BB54 nested-guard, A-PP44 parser dispatch-union) +
design87 integrato (A-DL43); contratto A-DS41 EMENDATO (A-DS44: trait
regola 9 DENTRO, tentative-return = deprecation-lane, lattice nominato)
— **A-DS35 pronto a partire**.
Dettaglio: `sessions/WP_SESSION_88.md` + `wp88-harness/MEASURE88_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **520e1b56708ee678** (stash additivo
  `phpr-wp88`; bit mossi dai sorgenti php-types/php-runtime di p2/p4 —
  parità certificata dalla battery: corpus Zend per NOME **1418
  IDENTICO** + refl 290 IDENTICO in parity-full, doctest VmGate CONTATI
  ==2).
- **php-server**: battery-88pre **16/16 CONTATO a 0b83f2b** (stamp
  ledgerato committed 202f8b1; matrix COMMITTED con snapshot NOME+sha
  A-AH48; 16° gate NUOVO axum-tests A-PP41 con a_pp38+A-PP45 pinnato;
  F16b a_ds38 ARMATO); campagna measure88 a 202f8b1 **attempt=1**
  (nessun VOID — prima campagna pulita al primo colpo) consumata via
  **battery-equivalence --same-rev v6** (A-SK50: allowlist finestra
  evidence-only + matrix-solo-.done + ledger-prefix + toolchain -Vv
  in-repo). mem-census b2fe7a43e62166da. Gate lever: pins v6
  (A-TH44/45 + A-MS41) / fixtures / fixtures2 (F16/F16b) tutti PASS.
- **Misure**: `wp88-harness/MEASURE88_RESULTS.md` + verdict88.a1.g1.out
  — VSLOPE-HI modi dominanti 148.307.968 / 230.948.864 / 324.403.200 /
  399.769.600 B (W=4/8/12/16), b = 21.195.981 B = 20,21 MiB/worker
  (NAMED-DEVIATION vs 3.605.572 B ±5%, Delta monotoni, KB-89-1/2 ok) ·
  VWARM cal 7.801.102 B (3ª campagna al byte), concbase padA
  =calA+calB+314 B (2/2), surplus su padB instabile, concwarm
  floor_inc 164.368 B, concstag net==cal AL BYTE (zero swallow) ·
  VUCLOG supersede=2 putord in-band, main_evicted=0. ⚠️ nets
  concorrenti VOID come cifre per-thread (KB-88-1) fino ad A-BB50.
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
campagna APPEND-only; VOID per-attempt IN-BAND nel ledger; verdetti
PER-ATTEMPT e PER-GENERAZIONE** (KS-AH-83-2/KG-88-1/A-BG43/A-BG46/
KG-89-1). 7. **Cifre memoria BYTES-FIRST con companion VERIFICATO sulla
STESSA riga, [derivata] a scope di riga, MiB-only, bande da allowlist**
(A-DL26/A-SK40/A-SK43/A-SK48). 8. **Cifra da binario = hash contro
matrix + marker #[used] con coupling pinnato a TRE copie** (KH86-1+
A-TH37/A-TH41/A-TH45/KH89-2). 9. **Peak/spread SEMPRE con NOME della
metrica — footprint vincolante, mai RSS nudo** (KB-88-2). 10. **Una
battery si consuma SOLO via battery-equivalence (path equivalence O
--same-rev v6 con allowlist di finestra)** (A-SK46/A-SK50/KS-SK-89-1).
11. **Companion mancante = FAIL del raw, mai default dentro un
aggregato** (A-BG48/KG-89-3). 12. **Un ordine di concilio con premessa
fattuale va VERIFICATO contro oracle/emitter PRIMA dell'esecuzione; la
refutazione si committa con le prove** (lezione S-88.0 ⭐⭐).

## ⚖️ Concilio WP-90 (da convocare/assemblare in chiusura S-88.0): `wp90-harness/COUNCIL_WP90_REVIEWS.md`

(placeholder — il blocco vincolante e il §WP-89(sessione) vengono
scritti all'assemblaggio dei 9 verbali)

## §WP-89(sessione) — S-89.0 (ordine dal Concilio WP-90, non rinegoziare)

(da compilare all'assemblaggio)

**Kill-switch di rotta ereditati ancora attivi**: tutti i KS WP-89
(tabella in `wp89-harness/COUNCIL_WP89_REVIEWS.md`) + KH88-1..4,
KS-MS-88-1..3 (KS-MS-88-1 sollevata ≥400fa10), KS-SK-88-1..4,
KS-AH-88-1..2, KB-88-1..3, KS-PP-88-1..3, KL-88-1..3, KS-DS-88-1..3
(KS-DS-88-1 consumata su log di PRODUZIONE in m88), KG-88-1..3 + gli
ereditati WP-87. KS-DS-80-3 invariata.

**NON riproporre**: tutti i NON-riproporre WP-83/84/85/86/87 restano;
in più — «≥1 coppia main_evicted su log di produzione» (insoddisfacibile
per costruzione: tripwire A-MS24/KS-DS-84-4); «riqualifica const-folding
dell'observable 4» (premessa refutata dall'oracle: persist hoista anche
new-pre-decl); «surplus concorrente come proprietà del LATO» (inverte
fra campagne: timing-attached); «formula discriminatore dA su regime
floor-collapse» (label VOID-di-significato); «banda KL-85-2 come banda
del protocollo slope» (non torna in nessun regime: b reale ~20-25 MiB/
worker — ri-derivare o ritirare la banda, mai riproporla tal quale).

---
**Chiusura**: 2026-08-02. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
