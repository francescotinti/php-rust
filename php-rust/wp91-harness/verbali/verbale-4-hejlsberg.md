# VERBALE — Anders Hejlsberg, sedia 4, Concilio WP-91

VERDETTO: CONCORDO CON EMENDAMENTI.

## Q1 — battery-attempts.ledger e il dente KS-AH-90-1

La storia si racconta da sola e regge il confronto coi commit: FAIL a
692c697 (epoch 23:48:56, first_fail=lever-fixtures2) → fix 65086e6
(23:49:38) → REFUSE tree-not-porcelain a 65086e6 (23:49:47: il ledger
appena appeso era esso stesso lo sporco) → evidenza committata 05726aa →
PASS 00:07:55 (stamp 3b7effb) → emendamento allowlist 30f5a87 (che
invalida la finestra del terzo PASS: il checker stesso è delta
non-allowlistato) → PASS attempt 4 a 30f5a87 (stamp ed427f4). Epoche e
commit interfogliano senza contraddizioni; anche il porcelain-refuse è
ledgerato come A-AH50 esige.

Riga forgiata: da sola NON legalizza un PASS — il dente A-AH50
(battery-equivalence.sh:329-344) è necessario-non-sufficiente; il carico
positivo resta su stamp 4-campi committato + matrix committato +
sha256(OUT) ricomputato. MA l'allowlist HA aperto un canale: il
prefix-check a granularità di riga (A-SK57, righe 313-328) copre SOLO
battery-stamps.ledger. Il ledger attempts, ora ammesso in finestra, può
essere RISCRITTO in-window (cancellare un FAIL/REFUSE, fabbricare la
riga PASS che il dente esige) senza che nessun dente morda; e il dente
non confronta il campo sha256 della riga col DSHA dello stamp. Refuto
la lettura "allowlist innocua": è innocua solo per il forge, non per la
riscrittura di storia. → A-AH54, KS-AH-91-1.

## Q2 — judge_sha e tether lato giudice

g1=ae4f528f93095979, g2=24cd290ae0a9fc2b, g3=294898431ef72d16: tre sha
DIVERSI, tutti ledgerati (m89.campaign.ledger righe 61-63, esiti
FAIL(9)/FAIL(41)/PASS). Tether lato giudice presente e armato:
verdict89.sh:88-98 ri-verifica mem_hash contro bin[mem-census] del
matrix COMMITTED a HEAD (A-SK59). Il supersede g3>g2>g1 è leggibile dal
solo ledger SOLO applicando la convenzione fuori-banda max-generation
(KS-SK-90-3): epoch crescenti + generation= + esiti, nessun campo
supersedes=. Ho risolto le sha: g1 = verdict89.sh@418def8 (committato),
g3 = @778d96f == HEAD. g2 NON risolve ad ALCUN blob committato: il
giudice che emise FAIL(41) è identificato ma IRRECUPERABILE — un
dangling pointer che vanifica lo scopo di A-AH51 per quella
generazione. → A-AH56, KS-AH-91-2.

## Q3 — recorder rustc e diagnosi del comparatore

Recorder (gate-feature-matrix.sh:88-92): A-AH53 attuato — sample-first,
rifiuto esplicito del valore vuoto, la riga rustc= non nasce mai vuota.
cargo= (riga 94) NON è coperto: il fallback `|| echo unknown` è vivo
(niente pipeline che ne maschera lo status), ma scrive `cargo=unknown`
senza fallire e NESSUN dente a valle legge cargo= — disciplina
asimmetrica. Comparatore (battery-equivalence.sh:221-233): SÌ, ora
distingue — NRUSTC!=1 ⇒ "carries N rustc= rows" (header assente = 0
righe, A-AH47); NRUSTC==1 con valore vuoto ⇒ "header present but VALUE
EMPTY" (A-AH53). → A-AH55.

## Q4 — finestra 30f5a87..ed427f4

`git log 30f5a87..ed427f4` = UN solo commit, ed427f4; tre file: la
matrix-archive feature-matrix.30f5a87.20260803-000923.log (esattamente
quella nominata dallo stamp e dalla riga phase=identity), +1 riga
attempts, +1 riga stamps. SOLO evidenza, tutta allowlistata. La
campagna parte a git=ed427f4 (phase=start epoch 1785709081, 2 s dopo il
commit). Finestra PULITA. Nota: il verdetto SAME-REV CONSUMPTION LEGAL
vive solo nello stdout della campagna, non nel ledger. → A-AH57.

## Emendamenti

- **A-AH54**: estendere il prefix-check A-SK57 (st_judge, riga-granulare)
  a battery-attempts.ledger nella finestra --same-rev; il dente A-AH50
  esiga sha256==DSHA nella riga PASS consumata (triangolo
  attempts↔stamp↔OUT).
- **A-AH55**: A-AH53 esteso a cargo= — il recorder rifiuta valore
  vuoto/`unknown`; il comparatore dichiari cargo= almeno ADVISORY.
- **A-AH56**: judge_sha risolvibile — ogni generazione va giudicata con
  giudice COMMITTED, o la riga è citabile solo come
  "superseded/judge-unrecoverable"; g2 di m89 va così annotata nel doc.
- **A-AH57**: la consumazione --same-rev va ledgerata nel campaign
  ledger (phase=consume, BREV, sha del checker).

## Kill-switch

- **KS-AH-91-1**: delta in-window su battery-attempts.ledger che non sia
  append riga-granulare, O riga PASS consumata con sha256 != DSHA ⇒
  consumazione VOID, battery re-run.
- **KS-AH-91-2**: judge_sha della generazione MASSIMA non risolvibile a
  blob committato a HEAD ⇒ VERDICT VOID; generazioni intermedie dangling
  vanno dichiarate o il doc che le cita è VOID.
- **KS-AH-91-3**: matrix con cargo= vuoto o unknown ⇒ matrix NULL per le
  identità che citano cargo (rustc= resta il giudice primario).

Firmato: Anders Hejlsberg, sedia 4, Concilio WP-91.
