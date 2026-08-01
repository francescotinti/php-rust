# NEXT_SESSION_WORDPRESS.md — S-85.0: IL-CANARY-CHE-DISCRIMINA (verdict85 PASS, ×W VERDICT-GRADE, pin peak RITIRATO) → WP-86(sessione)

**Ultima sessione**: S-85.0 (2026-08-01 sera, commit 675e7dc…d660fd7) —
gli 8 punti del Concilio WP-86 eseguiti: sanatorie (KL-85-2 byte esatti,
A-PP32/A-MS30/A-BG37/A-DS33/A-MS31), battery/equivalenza v3 (A-SK40
cifre-gate `--all` NEL 15/15 — buco GRAVE sanato; A-SK36 stamp ancorato +
.done sha256; A-SK37/39, A-AH39), sigilli v3 (A-TH32 micro-modulo `gate`
foglia, A-MS29 probe anchor-bound, A-TH33/34, A-TH35 guardia rientranza
che MORDE, KH86-1 cablato), sig() esaustiva (A-DS30 destructuring senza
`..` + 3 mutanti nuovi, A-DS31 ordine eventi), eredità meccanica (A-PP31
sweep reqns, KG-86-1 gate-slope-verdict, A-PP30 pre-warm), **campagna
measure85 + VERDICT85 PASS**: **VDL28 canary monolaterale → PER-THREAD
CONFERMATO byte-exact, ×W PROMOSSO a verdict-grade**; **VP R=9 → pin
identità RITIRATO** (spread non attribuito, KL-86-1; envelope max
252.526.592 B = 240,8 MiB); **delibera peak ESEGUITA** (×W accettato;
registry bloccata su A-MS27). Dettaglio: `sessions/WP_SESSION_85.md` +
`wp85-harness/MEASURE85_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **bf278d55fd5efb0a** (stash additivo
  `phpr-wp85`; bit mossi da A-TH32/A-MS29/A-TH35/A-DS30/A-DL29/A-PP30) —
  corpus Zend per NOME: **1418 IDENTICO** + refl 290 IDENTICO
  (battery-85pre parity-full, `wp85-battery-out/` fuori repo).
- **php-server**: battery-85pre **15/15 per NOME a 368c91d** (riga PASS
  ANCORATA + .done rev+sha256, A-SK36; `gate-measure-cifre --all` DENTRO
  il perimetro, A-SK40); union a8b65c0578c42fb7 · mem-census
  a3c901dfddd474c0 — ogni arm ENFORCE matrix + gate-binary-noprobe
  (KH86-1, entrambe le metà). Gate lever: pins (v3: classe VmGate ==3 in
  mod gate, A-TH33/34, sweep 'static ==0, A-PP31, KG-86-1) / fixtures /
  fixtures2 (F16 con pin '1 passed'+rustc, A-SK39) tutti PASS.
- **Misure**: `wp85-harness/MEASURE85_RESULTS.md` + verdict85.out —
  VDL28 PER-THREAD byte-exact (NET_H 7.349.977 B · NET_P 7.803.281 B,
  W=2 attribuzione per PATH) · scomposizione additiva (residuo thread
  7.343.135 B + own per fixture) · VP R=9 envelope max 252.526.592 B =
  240,8 MiB, pin RITIRATO · ledger A-DS29 invariato.
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
porta vm_new/park_main = token `VmGate<'gate>` LIFETIME-BOUND nel
micro-modulo foglia `gate` (v3: il campo è privato al modulo — rustc
giudica anche in-module; probe anchor-bound, mai 'static). 4. **Panic =
FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒ git revert, mai
fix-forward** (KS-DS-80-3, mai innescata). 6. **Mai rm di raw:
quarantena + manifest** (KS-AH-83-2). 7. **Cifre memoria BYTES-FIRST con
companion VERIFICATO** (A-DL26/A-SK40, per-FIGURA, unità
case-insensitive). 8. **Cifra da binario = hash contro matrix** (KH86-1:
il simbolo assente non prova la feature assente — dead-strip).

## ⚖️ Concilio WP-87 ESEGUITO (2026-08-01, verbali VINCOLANTI): `wp87-harness/COUNCIL_WP87_REVIEWS.md`

**9× CONCORDO/PASS CON EMENDAMENTI, 0 opposizioni; 27 KS nuovi.** CIFRE
confermate a ricomputo indipendente (Gregg+Bak al byte; Matsakis ha
COMPILATO le prove; Stogov sull'oracle vivo). REFUTATI: **A-MS29 a metà**
(`&()` PROMOTABILE — token 'static abitabile senza grafia; fix `&mut ()`,
E0716); sigillo mint chiuso solo per LETTERA (metodi `.production_gate(`/
`.vm_gate(` fuori sweep — A-TH38); **A-TH35 non sound sotto unwind**
(guardia dichiarata DOPO gli sfollati, commento falso — A-TH36); **A-SK38
VIOLATO in verdict85** («VDL28 PASS» da blocco sporco — A-SK44 + re-run);
battery forgiabile (15/15 CABLATO, no porcelain — A-SK41/42); classe
**MODEL-GRADE** istituita (cifre da raw VOID mai load-bearing — A-BG42);
PER-THREAD = proprietà del protocollo SEQUENZIALE (concorrenza esige
re-canary — A-BB47/KL-87-2); **phys_peak<phys SPIEGATO** (fetch_max solo
nel throttle — fix 1 riga A-DL32); «envelope SALITO» = vizio d'ordine
statistico (max R=9 domina max R=3 — A-BG41); sweep A-PP31 vacuo al
falso-positivo (`w=10` passa — A-PP35); 🔴 **GRAVE (Stogov): covariance
LSP non verificata** — classe SBAGLIATA invece che assente,
correct-or-absent violato (A-DS35, primo item engine ROADMAP); hoisting
phpr = semantica OPCACHE (fedele a opcache_cli, divergente dal CLI-oracle
— catalogare).

## §WP-86(sessione) — S-86.0 = ordine Concilio WP-87 §Sintesi (non rinegoziare)

1. **Sanatorie di verdetto/documento**: A-SK44 (verdict85 per-blocco +
   RE-RUN) · A-SK45 (dente ord≥2) · delibera ×W riformulata su SOLO
   m85.dl28s + tag MODEL-GRADE (A-BG42) · correzione overclaim A-BG37 +
   A-BG40 · retro-dichiarazioni A-AH42/A-BB48 · A-DS34 · doc A-DS36.
2. **Sigilli v4**: A-MS32 (`&mut ()`) · A-TH36 · A-MS35 · A-TH38 · A-TH39
   · A-MS33 · A-TH37 (#[used] marker, controllo positivo che DEVE fallire,
   KH87-2).
3. **Battery v4**: A-SK41 (stamp ledgerato) · A-SK42 (k/k contato +
   porcelain) · A-AH40 (matrix dal .done) · A-AH41 (rustc -V) · A-SK43.
4. **Probe/eredità**: A-PP35 · A-PP33/34 · A-PP36 · A-MS34 · A-DS37 ·
   A-DL33/34.
5. **Attribuzione spread VP**: A-DL32 → campagna ABBA purge R≥8
   (A-BB46/A-DL35, soglie 50%/80%) → SOLO DOPO eventuale pin
   (KL-87-1/KB-87-2).
6. **Modello scomposizione**: A-BB45 contro-prova ordine invertito
   (predizione 6.842 B) — fino ad allora mai addendo di budget (KB-87-1).
7. **Metà fisica + concorrenza**: A-DL31/A-DL35 (chiusura 3.605.572±5%) ·
   A-BB47 — con A-MS27 precondizioni registry.
8. **ROADMAP**: A-DS35 (covariance LSP) primo item engine + catalogo
   hoisting-opcache; poi [[php-rust-todo-master]]. Deferred invariati:
   A-PP18/A-PP27; KS-DS-80-3 invariata.

**Kill-switch di rotta (WP-87, attivi)**: KH87-1/2/3, KS-MS-87-1/2/3,
KS-SK-87-1/2/3/4, KS-AH-87-1/2, KB-87-1/2/3, KS-PP-87-1/2/3, KL-87-1/2/3,
KS-DS-87-1/2/3, KG-87-1/2/3 — tabella in
`wp87-harness/COUNCIL_WP87_REVIEWS.md`. Ereditati ancora attivi: KH86-1
(forma ponte nm+hash), KS-MS-86-2 (registry solo con A-MS27), KS-SK-86-*,
KB-86-*, KG-86-*, KL-86-* come da tabella WP-86.

**NON riproporre**: tutti i NON-riproporre WP-83/84/85 restano; in più —
pin identità peak 232±1 (RITIRATO: solo envelope max 252.526.592 B =
240,8 MiB finché lo spread non è attribuito); canary path-len≥384 per il
net (il bracket avvolge SOLO lower_source — refutato dalla sede);
"per-process/specchio" per la finestra DL24 (falsificato dal canary);
`(\S+)$` su path di questo repo (contiene spazio).

---
**Chiusura**: 2026-08-01. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
