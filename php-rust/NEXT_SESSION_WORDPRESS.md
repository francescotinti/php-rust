# NEXT_SESSION_WORDPRESS.md — S-86.0: LA-CONTRO-PROVA-E-LE-TRE-REFUTAZIONI (verdict86 PASS, scomposizione confermata, purge refutato) → WP-87(sessione)

**Ultima sessione**: S-86.0 (2026-08-02, commit 6be233c…4fe33dc + chiusura) —
gli 8 punti del Concilio WP-87 eseguiti: sanatorie (A-SK44 bite-tested +
RE-RUN byte-identico, A-SK45, MODEL-GRADE A-BG42, retro-dich. A-AH42/
A-BB48), sigilli v4 (A-MS32 `&mut ()`, A-MS33 !Send/!Sync compile_fail
E0277, A-TH36 guardia-in-testa, A-TH37 `#[used]` con KH87-2 positivo
VERIFICATO, A-TH38/39), battery v4 (A-SK41 stamp ledgerato, A-SK42
porcelain+contato — ha REFUSED due volte il proprio checker e i difetti
erano DEL checker, A-AH40 matrix dal .done, A-SK43 allowlist), probe/
eredità (reqns-guard.pl, dispatch in-band, PROBE_ACTIVE, A-DL33/34),
**campagna measure86 + VERDICT86 PASS**: **contro-prova A-BB45 SUPERATA
(hello-own(ord2) = 6.842 B ESATTO sulla predizione ex-ante)**, burst
+1.048.576 B esatto (pipeline fail-closed PROVATA), VW500 sul modello
(4/2.002 B), ABBA 9+9 (su RSS inconclusive — MA v. Concilio: su
footprint SEPARA), catalogo §3.3-ter/quater + A-DS35 deciso CORRECT.
Dettaglio: `sessions/WP_SESSION_86.md` + `wp86-harness/MEASURE86_RESULTS.md`.

## Stato gate

- **phpr (CLI, parità release)**: **ea56b874c76d3558** (stash additivo
  `phpr-wp86`; bit mossi da A-TH36/A-DS36 putord/A-MS32/33/35/A-DS34) —
  corpus Zend per NOME: **1418 IDENTICO** + refl 290 IDENTICO
  (parity-full verde in TRE battery-86pre).
- **php-server**: battery-86pre **15/15 CONTATO a c259bc6** (A-SK42;
  stamp LEDGERATO committed A-SK41; matrix nel .done A-AH40); campagna
  measure86 alla STESSA rev (nessuna equivalenza consumata — ma v.
  KS-SK-88-1: il fast-path stessa-rev va dotato dei denti v4). Union
  b2074e451cbc7fc3 · mem-census 874e744ede57b4ca ·
  driver_sha 699db00a9808489e (≠ 85: parametro purge in measure78).
  Gate lever: pins v4 (A-MS32/A-TH38/39 con decoy) / fixtures /
  fixtures2 tutti PASS.
- **Misure**: `wp86-harness/MEASURE86_RESULTS.md` + verdict86.out —
  VCAL terza riproduzione al byte (7.349.977/7.803.281 B) · VINV 6.842 B
  esatto · VBURST +1.048.576 B esatto · VW500 4/2.002 B · VABBA spread
  A 21.315.584 B / B 14.139.392 B (RSS) · envelope braccio A
  252.772.352 B = 241,06 MiB. ⚠️ Concilio WP-88: VOVL e VABBA vanno
  RI-GIUDICATI dai raw esistenti (v. §WP-87 p1).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).

## Permanent Binding Rules (invariate)

1. **Output capture BEFORE request_end()**. 2. **Isolamento = semantica
FPM** (A-DS2). 3. **RetainSet thread-affine PER-RICHIESTA, sigillato**;
porta vm_new/park_main = token `VmGate<'gate>` LIFETIME-BOUND nel
micro-modulo foglia `gate` (v4: anchor `&mut ()` — constant promotion
morta per E0716; !Send/!Sync per PhantomData<*mut ()> con compile_fail
E0277). 4. **Panic = FAIL-FAST**. 5. **Un body ≠ oracolo sulla leva ⇒
git revert, mai fix-forward** (KS-DS-80-3, mai innescata). 6. **Mai rm
di raw: quarantena + manifest** (KS-AH-83-2; esteso da KG-88-1: mai
RIUSARE un filename — attempt= nel nome). 7. **Cifre memoria BYTES-FIRST
con companion VERIFICATO, MiB-only, bande da allowlist** (A-DL26/A-SK40/
A-SK43). 8. **Cifra da binario = hash contro matrix + marker #[used]**
(KH86-1+A-TH37). 9. **Peak/spread SEMPRE con NOME della metrica —
footprint vincolante, mai RSS nudo** (KB-88-2).

## ⚖️ Concilio WP-88 ESEGUITO (2026-08-02, verbali VINCOLANTI): `wp88-harness/COUNCIL_WP88_REVIEWS.md`

**9× CONCORDO/PASS CON EMENDAMENTI, 0 opposizioni; 29 KS nuovi.** CIFRE
confermate a ricomputo (Bak+Gregg al byte; Matsakis nei log battery;
Stogov sull'oracle vivo). REFUTAZIONI CAPITALI: **VOVL era FALSO-DAI-RAW**
(qualificatore awk confrontava STRINGHE: overlap in 10/10, hello ~13,8 ms;
i net sotto overlap mostrano pad = NET_H+NET_P ⇒ **per-thread sotto
concorrenza REFUTATO, non OPEN** — ×W sequenziale-only confermato duro);
**VABBA su metrica footprint SEPARA 8/8** (purge=0 abbassa ~20,9 MB — l'
INCONCLUSIVE era artefatto della metrica RSS); **riuso dei nomi = veleno
dell'evidenza** (raw abortiti sovrascritti same-label, ledger OVL
troncato — 3 sedie convergenti); **fast-path stessa-rev bypassa i denti
v4**; **flag probe bool globale mente sotto W≥2** (il garbling della
sessione lo prova); **phys_window_dump ancora non-atomico**; **§3.3-ter
refutata nella ricetta** (persist, non enable_cli; 4°/5° observable
trovati); **VW123 su mapping non verificato** (union senza dispatch row);
**promozione scomposizione SCOPED alle coppie annidate** (Δfloor_inc =
996.838 ESATTO — la simmetria era del NIDO); sigilli: finestra parametri
put, marker↔grep per convenzione, carrier-a-valore Clone (KH88-2).

## §WP-87(sessione) — S-87.0 = ordine Concilio WP-88 §Sintesi (non rinegoziare)

1. **Ri-giudizi DAI RAW ESISTENTI (nessun run nuovo)**: A-BB49
   (qualificatore numerico + verdetto overlap dai 10 raw) · A-BB51
   (VABBA su peak footprint dai vmmap V1/V2 archiviati) · correzione
   MEASURE86 (VOVL/VABBA/VW123 tag) · A-BG44-forma su verdict86 ·
   A-DS40 (catalogo emendato + fixture committate).
2. **Fix strumenti**: A-DL37 (dump atomico) · A-MS36 (flag per-thread) ·
   A-PP39 (dispatch row su union) · A-DL36 (clamped flag).
3. **Catena evidenza**: A-SK46 (fast-path coi denti) · A-BG43/A-SK49
   (attempt= nel filename, ledger append-only) · A-AH43/44/45 · A-BG45.
4. **Sigilli v5**: A-TH40/41/42/43 · A-MS37/38/39 · A-SK47/48 ·
   A-PP37/38/40 · A-DS38/39.
5. **Misura (una campagna)**: A-DL38≡A-BB52 metà fisica = slope
   committed steady-state (W∈{1..4}, R≥5, ±5%) · A-BB50 design net
   window per-thread · A-BB53 coppia disgiunta.
6. **Delibere**: promozione scomposizione SCOPED alle annidate ·
   A-DL39 design split heap per-worker (solo design).
7. **ROADMAP**: A-DS35 fase 1 secondo spec Stogov Q3 (A-DS41 in
   todo-master; KS-DS-88-3 vincola il merge).
8. Deferred invariati: A-MS27 · A-PP18/27 · A-AH38+dry-run; KS-DS-80-3
   invariata.

**Kill-switch di rotta (WP-88, attivi)**: KH88-1..4, KS-MS-88-1..3,
KS-SK-88-1..4, KS-AH-88-1..2, KB-88-1..3, KS-PP-88-1..3, KL-88-1..3,
KS-DS-88-1..3, KG-88-1..3 — tabella in
`wp88-harness/COUNCIL_WP88_REVIEWS.md`. Ereditati ancora attivi: KH87-*,
KS-MS-87-*, KS-SK-87-*, KS-AH-87-*, KB-87-1 (fino a delibera p6),
KS-PP-87-*, KL-87-1/2 (KL-87-3 DECADUTA post-A-DL32), KS-DS-87-*,
KG-87-* come da tabella WP-87.

**NON riproporre**: tutti i NON-riproporre WP-83/84/85 restano; in più —
"VOVL OPEN / per-thread sotto concorrenza CANDIDATO" (REFUTATO dai raw,
Concilio WP-88); "purge refutato come driver dello spread" su metrica
RSS (il giudizio vincolante è su footprint, A-BB51); heap-visit
per-thread sul default heap v3 (condiviso — KL-88-1); Δpeak d'avvio come
metà fisica (la forma è A-DL38); qualificatori awk su campi numerici
senza coercizione `+0`; array vuoti con `set -u` su bash 3.2.

---
**Chiusura**: 2026-08-02. Apertura/chiusura sessioni = skill
`apri-sessione` / `chiudi-sessione`.
