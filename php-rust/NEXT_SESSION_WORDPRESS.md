# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-102 (2
sessioni fa; il debito coppia ha SCADENZA: trigger = prima leva promossa,
fallback = chiusura S-106 o il riferimento DECADE, KS-PE-106-2)** · ultima
campagna sull'OGGETTO = **S-104 (leva H-C2 TENTATA: A/B eseguito ×2,
verdetto CADUTA con canale refutato — ritmo-leve RISPETTATO)** ·
**sessioni-senza-Δ-rapporti = 3** (contatore SDOPPIATO A-GR-106-1;
**KS-GR-106-1: se S-105 chiude a Δ zero ⇒ contatore=4 ⇒ riallocazione di
CATEGORIA in WP-107**).

**Ultima sessione (S-104, 2026-08-06 sera)**: ordine WP-105 consumato.
1) **A/B peak R=7 CHIUSO PER SEMPRE**: sign 7/7 B>A p=0,0078 = direzione
FIRMATA (s100-fix cresce su s99-sigillo); magnitudine INDETERMINATA
(l'estimatore accoppiato +22,47 MiB fu scelto post-hoc — emenda Leijen);
metrica full-peak esaurita, futuro solo per-fase (A-LE-105-5, backlog).
2) **LEVA H-C2 CADUTA CON CANALE REFUTATO**: implementata parity-esatta,
Δ=−10,33/−11,33 ns/iter (due forme, 5/5, rumore ~2) ⇒ REVERT verificato
(run_loop 257.632 B esatti, prop 4,87s); disasm: inliner flippato
(bl a drop_in_place 1101→**0**, run_loop +8 KB) ⇒ le chiamate al glue
erano quasi gratis. ⚠️ «icache-bound» = ipotesi forte N=1 NON firmata
(l'A/B misurò leva+inliner insieme): si firma solo con contatori o
inlining pinnato (KS congiunto WP-106). 3) **Denti col rosso VERO**:
absent_eq_one per-corpo nato rosso e corretto con causa; 🔵 19a/19b NON
arbitrano il meccanismo (2 perturbazioni senza rosso — la TERZA su OBS-8
decide, A-MA-106-1); 🔵 **memory_get_usage phpr = STUB costante** (fx20
ridisegnata col braccio RSS: clean 50 vs mutante 301 MiB, rosso
archiviato). 4) **H-D INCHIODATO**: free-hist 4/4 attese — 1 alloc×32 B +
1 free×32 B/chiamata ESATTI, realloc≡0, ret_cell ESCLUSO (layout E
misura) ⇒ indiziato args-Vec cap 2. Dettaglio: `sessions/WP_SESSION_104.md`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-104, micro R=5 SUL PIN DI CHIUSURA 86a50d1c, N emessi)

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **12,4** | corpi non-Binary + pila operandi; fusioni forme registro = H-C3 (backlog) |
| proprietà | **11,5** | drop fast-out REFUTATO (S-104); prossima: fusione prop-RMW H-C3 (2° slot, porta §3.12-i; contatori = prerequisito di tesi) |
| chiamate | **7,6** | **LEVA S-105: args SmallVec inline-2** — canale inchiodato (1×32 B alloc+free), attesa Δ∈[6,14] ns/iter, gate d'apertura probe cap-bump 2→4 + arità + audit-fuga |
| stringhe 6,7 · array 4,5 · regex 3,6 | | — (regex = parte sana) |

(banda tra-sere ±0,3 su 2 giorni-punto; il terzo punto su GIORNO distinto resta dovuto)

## Regole di metodo

1. Il giudice è la micro-categoria. 2. WordPress è collaudo di PARITÀ —
ora con SCADENZA (sopra). 3. Criterio scritto PRIMA. 4. Apparato solo se
blocca; timebox ½ sessione. 5. **SCOREBOARD in testa a ogni report +
almeno una leva TENTATA (A/B eseguito) per sessione**
([[feedback-scoreboard-lever-pacing]]). 6. **PIN-106** (Klabnik, capitale
WP-106): `build → hash₁ → batteria → re-hash₂ → STASH(hash₂) → fixture →
corpus×2`; build dopo lo stash = gate VOID; churn documentato.
7. **Admission di OGNI leva (KS-GR-106-2)**: admission-disasm (bl-count
prima/dopo) + smoke R=2 prima del pieno; mai componenti di costo da un
A/B che flippa il codegen (KS-BA-106-1). 8. Parity-null sempre col
PERIMETRO nominato (KS-KL-106-1).

## Stato gate

- **phpr (pin release)**: **86a50d1c01c6f45a** @ HEAD di chiusura S-104
  (fa fede HEAD; churn 66681884→86a50d1c documentato) — DEFAULT flag-ON.
  Batteria **1740/0** (+1 dente trivial_drop) · fixture **13+5+19a/b+fx20**
  (pin BILATERALE, braccio RSS) · corpus **1417 per NOME ×2** IDENTICO a
  wp82 · micro sul pin (baseline sopra) = parity-null STRUMENTALE
  misurato (perimetro: CLI due modi; server NON misurato). Stash ADDITIVO
  `phpr-s104`. ⚠️ HEAD post-pin ha SOLO commit census/doc (freehist) —
  parity-null per costruzione (feature-gated), ma la prossima build
  churna l'hash: fa fede HEAD.
- **php-server**: pin **31aa7c2eef899cce** @ 37312e8 INVARIATO — è
  indietro rispetto a HEAD; cifre server SOLO da pin same-HEAD gradato
  PIENO (KS-PE-106-1); si rigrada al trigger della leva promossa.
- **Launcher S-104** (`wp104-harness/`): `s104-hc2-ab.sh` (modello A/B
  con sanity output + N emesso) · `s104-fx20-gate.sh` (pin bilaterale +
  braccio RSS) · fixture `fixtures/fx20-stringhe-in-pop.php` · rossi in
  `denti-rossi/`. Census con freehist in `phpr-census-target`.
- GATE72 CLI baseline trasversale (corpus 1417 · refl 290 · ORM 3E/13F ·
  hk 1665). ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA: rimuoverla
  nel pre-flight.

## Voci APERTE per NOME

- **Coppia WP bimodale**: debito con SCADENZA (trigger leva promossa;
  fallback S-106 o riferimento WP-102 decade — KS-PE-106-2/KS-KL-106-2).
- **Terza mutazione 19a/19b su OBS-8** (A-MA-106-1): decide arbitri-del-
  MOVE vs riclassifica per NOME.
- **Contatori L1I/INST_RETIRED sulla coppia H-C2** (ricostruibile:
  checkout 4ea2cff): braccio parallelo breve S-105 o backlog — finché
  manca, «icache-bound» resta ipotesi NON citabile come premessa.
- **memory_get_usage stub**: voce 🔴 a catalogo (KS-MA-106-1: nessun
  verdetto vi si appoggi); cura a due gradini TLS/mi_* (A-ST-106-1) solo
  post-leva.
- **Generator get_gc** (fixture rossa arbitro) + **fixture §3.13 unit
  include/eval**: primi in coda fedeltà.
- divergenze §3.12 regime-i (viaggia con la fusione H-C3) · terzo punto
  banda su GIORNO distinto · A-HE-105-3 zoo esteso (hook/arrow/static).

## Che cosa è SOSPESO (non abbandonato)

- A-ZV2 (liveness+TakeSlot); design per-fase A-LE-105-5; PGO/outlining
  A-HE-106-4; H-ICS cold-out (criterio firmato prima); 21,2% run_loop =
  prefisso di targeting della futura leva icache (A-HE-106-5);
  roadmap footprint (riferimenti S-102: full peak on 1942,05 / off
  1989,88 MiB).

## NON riproporre

Tutti i NON-riproporre WP-83..103 restano. Nuovi da S-104/WP-106:

- **leve che riducono micro-costi di chiamata su run_loop** (canale
  drop-call REFUTATO dalla misura: due forme, 5/5) — la valuta è il
  volume di codice/lavoro, non il costo della singola `bl`.
- **claim architetturali («icache-bound», «chiamate gratis») da un A/B
  che ha flippato il codegen**: si firmano SOLO con contatori o inlining
  pinnato (KS-HO/HE/BA-106-1).
- **fixture il cui verdetto poggia su memory_get_usage** finché è stub
  (KS-MA-106-1).
- **PIN-105 letterale** (stash prima della batteria): la batteria
  relinka SEMPRE — vale PIN-106.
- **estimatori A/B scelti post-hoc** (la lettura R=7 è sopravvissuta
  solo per lo STOP): estimatore accoppiato pre-registrato o lettura VOID.
- **pool per gli args** (refutato Leijen: duplica la freelist TL di
  mimalloc) e **threaded-dispatch** (vietato Bak, A-BA-106-3).
- ereditati: banda dal proprio run; denominatori a memoria; assert senza
  chiamanti; output di run nel repo; .rs nei comandi git (Write +
  commit -F).

---
**Riscritto**: rotazione S-104 il 2026-08-06 sera. Apertura/chiusura =
skill `apri-sessione`/`chiudi-sessione`. Harness di sessione:
`wp104-harness/`; concilio in `wp106-harness/`.

## §S-105 — ORDINE DEFINITIVO (Concilio WP-106, `wp106-harness/verbali/SYNTHESIS.md` — VINCOLANTE)

⚖️ **Concilio WP-106 (2026-08-06 sera, su S-104 e programma S-105)**.
Indice + ricevute in `wp106-harness/COUNCIL_WP106_REVIEWS.md` (testi SOLO
in `verbali/`). 9/9 CON EMENDAMENTI; **capitale Klabnik: PIN-105
insoddisfacibile ⇒ PIN-106**; «icache-bound» declassato a ipotesi N=1
(KS congiunto); conflitto prop-RMW-prima composto: **args-calls è la
leva #1**. Ordine:

1. **LEVA H-D args (SmallVec inline-2 per gli args di calls) — NON
   NEGOZIABILE**: a. atto zero ~30′: criterio SCRITTO (attesa Δ∈[6,14]
   ns/iter su calls 7,6; co-primari timing E census alloc/chiamata→
   **0,0000**; caduta sotto max(rumore ~3, layout); admission-disasm +
   smoke R=2); b. gate d'apertura: **probe cap-bump 2→4** (se il costo
   NON sale ⇒ il canale non è l'alloc: STOP prima di implementare) +
   censimento ARITÀ call-site + audit-fuga (fixture: il Vec args non
   sfugge alla finestra di chiamata); c. implementazione → smoke R=2 →
   A/B R=5 ABAB stessa sera → verdetto; d. **PROMOZIONE ⇒ trigger
   fedeltà STESSA SESSIONE**: rebuild php-server @ HEAD + grado PIENO
   (A-PE-106-1) + **COPPIA WP bimodale** (salda il debito);
   e. gate **PIN-106** completo.
2. **Braccio parallelo BREVE (~30′, NON prerequisito)**: contatori
   INST_RETIRED/L1I-miss sulla coppia H-C2 (checkout 4ea2cff); se non
   entra ⇒ backlog, l'ipotesi icache resta non-firmata.
3. **Denti/arbitri nella finestra del gate**: terza mutazione OBS-8 su
   19a/19b (decide); fx20 cap→banda derivata + guardia erosione cap/2 +
   mutante leak-parziale; sigillo Copy payload trivial + doc verdetto
   S-104 nel predicato + «al byte»→«taglia+timing» (A-HO-106-1/2/3).
4. **Fedeltà (timebox ½)**: memory_get_usage 🔴 a catalogo SUBITO;
   gradino TLS solo post-leva (KS-ST-106-1); generator get_gc + fixture
   §3.13 unit se entra.
5. **BACKLOG per NOME** (non aprire): H-C3 fusioni prop-RMW (2° slot,
   con §3.12-i e contatori-prerequisito); H-ICS cold-out; per-fase
   A-LE-105-5; PGO/outlining; retention uploads (A-PE-106-3);
   banda-layout N≥3; disposizione mutanti (A-MA-106-2); zoo A-HE-105-3.

Pre-flight S-105: pin phpr **86a50d1c01c6f45a** @ HEAD S-104 (fa fede
HEAD, la build churna) · server 31aa7c2e (NON citarlo come HEAD) ·
corpus 1417 per NOME ×2 SUL PIN · default flag-ON · debug/ si rigenera:
rimuoverla · MySQL wp8 con l'elenco dei database · uploads sotto guardia.
