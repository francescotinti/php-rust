# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-102 (QUESTA
sessione — coppia nei DUE modi + banda rumore peak, contatore 0)** ·
ultima campagna sull'OGGETTO = **S-102 (questa: guardie MOVE, census pila,
fix §3.13 con miglioria corpus; nessuna leva perf — le prossime nominate:
H-C2, H-D alloc-canale)**.

**Ultima sessione (S-102, 2026-08-06)**: l'ordine del Concilio WP-103
SALDATO in tutti i punti eseguibili. 1) php-server **2c4242b6 GRADATO**
(sentinella bimodale + dente capture-boundary NUOVO, fails=0×2; riga
aggiunta a PIN_REGISTRY). 2) Guardie MOVE: **INV-RECV-1 auditata** (12
osservatori assoluti, verdetto INVARIANTE; addendum RC-MA-103-1 al
criterio H-C1b) + **fixture 14-18** 5/5 byte-id ×2 modi + 
**`Zval::is_gc_container`** esaustivo a due livelli (perf-neutro: prop
11,5 invariato). 3) **Banda rumore full-peak PHPR misurata per la PRIMA
volta**: mediana 1896,91 MiB, spread 34,64 MiB (~1,8%) < 48 ⇒ bisect
ammesso; **A/B pin S-99↔S-100 (ABAB off/off R=5) IN VOLO a fine sessione**
— verdetto meccanico in `wp102-harness/peak-ab-out/ab-verdetto.out`.
4) **Census pila operandi**: 23 transiti-sorgente/iter ESATTI (statico
confermato; push=9 pop=9 peek=2 len=1 elem=2; linearità 300:1) =
denominatore dei futuri Δ_A/B; gamba alloc a MEM-CENSUS: prop alloc/iter≡0
CONTATO, ma 🔵 **calls ALLOCA ~2/chiamata** (~35 B, churn bilanciato
invisibile alle pagine) — canale H-D NOMINATO. 5) Denti: absent≡`=1` VERO
in sottoprocesso col dump-diff; body-zoo (residuo `Binary(Add)`
fuori-funnel ESISTE, pinnato per NOME); gate fixture ai 13 NOMI. 6) **Fix
§3.13 FEDELE** (riga all'accodamento; carve-out CANCELLATA, fixture 13/13
diff zero puro) — 🔵 MIGLIORIA: passa `nullsafe_operator/015.phpt` ⇒
**corpus 1418 → 1417 per NOME**. Batteria **1739/0**; coppia WP bimodale:
media 0 ×2, full = solo `wp_is_stream` invariante, full CPU 1,891/1,894.
Dettaglio: `sessions/WP_SESSION_102.md` + `wp102-harness/*.out`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-102 chiusura, binario d0b01362, modo default, R=5)

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **12,3** | residuo = corpi non-Binary + pila operandi |
| proprietà | **11,5** (invariato: S-102 = sessione di guardie, non di leve) | **H-C ATTIVA** — prossime: H-C2 drop fast-out scalare (banda [8,22], pavimento ½ prudenziale, canale ~11 drop/iter) e slot-diretti RISCRITTI (dump-diff prima, ring-fence dal round-trip già eliminato); denominatore pila = 23 transiti/iter CONTATI |
| chiamate | **7,7** | **H-D ATTIVA** — canali NOMINATI: 🔵 ~2 alloc+2 free/chiamata (mem-census S-102) + 6 gc_note/iter non taggate + tavola bi-regime completa |
| stringhe | 6,6 · array 4,6 · regex 3,6 | — (regex = parte sana) |

## LE IPOTESI — stato dopo S-102

- ~~H1, H-A1, H-B1, H-B2~~ — chiuse. H-C1a+b SPEDITE (S-101), ora SOTTO
  GUARDIA COMPLETA (audit INV-RECV-1 + fixture 14-18 + predicato).
- **H-C (proprietà, 11,5×)**: gambe residue = pila operandi (denominatore
  CONTATO: 23 transiti/iter) + ciclo vita Zval (~28%) + 3 clone
  LoadVar/iter (emissione). **H-C2** pronta ad aprirsi: canale contato,
  banda [8,22] ns/iter dal Concilio WP-103; esige criterio pre-registrato
  + A/B da sola + ciclo gate pieno (corpus×2 + coppia). **Slot-diretti**:
  solo dopo leva-nulla di taratura (A-BA-103-4, non fatta) e dump-diff.
- **H-D (chiamate, 7,7×)**: 🔵 il call-path alloca ~2/chiamata
  (`hd-census-secondo.out`) — identificare LE DUE allocazioni per sito
  (args Vec? frame?) col census PRIMA di ogni leva; tavola bi-regime
  completa (6 gc_note/iter residue da nominare; Call/Ret non ancora nello
  stackcensus).
- **H-C1c**: resta GATED su fixture/giudici per SPECIE (KS-ST-103-2).

## Regole di metodo (invariate)

1. Il giudice è la micro-categoria. 2. WordPress è un collaudo di PARITÀ
(si esegue quando cambia l'emissione O il runtime — S-102 l'ha eseguito
per il runtime). 3. Ogni ipotesi porta il criterio di caduta scritto PRIMA.
4. L'apparato non entra nell'ordine se non blocca; timebox mezza sessione.

## Stato gate

- **phpr (pin release)**: **d0b01362433b3039** @ HEAD b6f8098 (hash churna
  col relink: fa fede HEAD) — DEFAULT flag-ON; contiene H-C1a+b +
  `is_gc_container` + fix §3.13 + denti WP-103. Batteria **1739/0**.
  Corpus **1417 per NOME nei 2 modi + diff per-test ZERO** (riferimento
  aggiornato con la miglioria 015.phpt DOCUMENTATA). Stash ADDITIVO
  `phpr-s102`.
- **php-server**: pin gradato = **2c4242b6c8120b8e** (runtime S-101,
  grado minimo A-PE-103-2 in S-102). ⚠️ **Il runtime è cambiato in S-102**
  ⇒ build con ricetta obbligatoria + collaudo del pin NUOVO = **debito NON
  condizionato, PRIMO ATTO S-103** (dottrina Pedersen; il binario
  workspace 49a91e4d è effetto collaterale SENZA axum-server: NON è un
  pin). Registro = `PIN_REGISTRY.md`.
- **Launcher S-102** (`wp102-harness/`): `s102-collaudo-server.sh <off|on>`
  (sentinella + capture-boundary) · `s102-move-fixtures.sh` (14-18) ·
  `s102-corpus-gate.sh` / `s102-corpus-diff.sh` · `s102-stack-census.sh` ·
  `s102-peak-noise.sh` / `s102-peak-ab.sh` (criterio in
  `s102-peak-criterio.out`) · `pair102.sh <off|on>`. Fixture 13 in
  `wp101-harness/hc1-fixtures.sh` (pinnate per NOME, SENZA carve-out).
- GATE72 CLI resta baseline trasversale (corpus ora 1417 · refl 290 · ORM
  3E/13F · hk 1665). ⚠️ gh-status-sync NON eseguito in S-102 (A/B in
  volo): coverage/README citano ancora 1418 — sync a S-103.
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA: rimuoverla nel
  pre-flight.

## Voci APERTE per NOME (misura/attribuzione dovuta)

- **A/B peak S-99↔S-100 IN VOLO** (lanciata S-102 ~12:45, ~3h):
  `wp102-harness/peak-ab-out/ab.done` + `ab-verdetto.out` (verdetto
  MECCANICO dalla regola pre-registrata: |Δmediane| ≤ banda ⇒ RUMORE e
  voce chiusa; > banda ⇒ crescita reale, bisect ammesso se spread < 48).
  **Lettura del verdetto = primo atto S-103 col collaudo server.**
- **Le 2 allocazioni/chiamata di calls** (H-D): da attribuire per sito.
- **21,2% run_loop senza nome** (dispatch/corpi): dopo il costo/transito
  della pila (Δ_A/B ÷ 23).
- divergenze §3.11/§3.12 — catalogate (famiglia fetch-undef); §3.13 CHIUSA.

## Che cosa è SOSPESO (non abbandonato)

- **A-ZV2** (liveness+TakeSlot): invariata; `cargo check --release
  --features zval-census` in batteria resta a backlog (A-HE-103-7).
- **Rollout Add nelle forme registro**: chiuso salvo misura ≥ pavimento.
- **Roadmap footprint**: ferma (riferimenti S-102: full peak on 1942,05 /
  off 1989,88 MiB).

## NON riproporre

Tutti i NON-riproporre WP-83..101 restano. Nuovi da S-102:

- **estendere un verdetto «≈0» a un giudice diverso da quello che l'ha
  prodotto** (alloc/iter=0 era di prop; calls alloca 2/iter — ogni
  categoria esige il SUO census).
- **gamba alloc su stats a pagine** (KS-LE-103-1 permanente: solo
  mem-census diretto).
- **atteso in ns derivato dai transiti contati o dalla quota 26,6%**
  (KS-GR-103-1 ≡ KS-BA-103-3): il costo/transito fa fede SOLO da Δ_A/B.
- **nominare file `.rs` nei comandi git** (il hook morde anche i messaggi:
  Write del messaggio + `commit -F`).
- ereditati e ribaditi: denti scritti senza leggere il corpo; misure senza
  ABAB sotto rumore; bande < rumore; pin effetto-collaterale (il 49a91e4d
  NON si registra); borrow nudo dello slot valore.

---
**Riscritto**: rotazione S-102 il 2026-08-06. Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione`. Harness di sessione: `wp102-harness/`.

## §S-103 — ORDINE (PROVVISORIO in attesa del Concilio WP-104; il blocco ⚖️ sotto sarà aggiornato con l'ordine DEFINITIVO)

1. **Pre-flight + verdetto A/B peak** (`peak-ab-out/ab-verdetto.out`,
   regola pre-registrata — se la finestra ha coppie a segno opposto ≥3/5
   si ripete, non si interpreta) + **collaudo php-server NUOVO** (build
   ricetta axum-server sul HEAD + `s102-collaudo-server.sh` con
   PIN_SRV_ATTESO aggiornato — debito NON condizionato).
2. **H-C2 «drop fast-out scalare»**: criterio pre-registrato (banda
   [8,22] ns/iter, pavimento ½ PRUDENZIALE a segno ignoto), misurata DA
   SOLA contro il pin, ciclo gate pieno (fixture 13+5 + batteria + corpus
   1417×2 + coppia WP se promossa).
3. **H-D apertura**: attribuire per SITO le ~2 allocazioni/chiamata
   (census con tag nei costruttori di frame/args) + nominare le 6
   gc_note/iter residue + estendere stackcensus a Call/Ret.
4. **Leva-nulla di taratura** (A-BA-103-4) se si apre la strada
   slot-diretti; altrimenti resta gate dichiarato.
5. **Igiene**: gh-status-sync (corpus 1417); backlog per NOME dal WP-103
   invariato (Generator birth-track A-HO-103-2, budget enabled()
   A-HE-103-2, assert no-Ref-wrapper A-MA-103-2, check census in batteria
   A-HE-103-7).

Pre-flight S-103: pin phpr **d0b01362433b3039** @ HEAD (fa fede HEAD) ·
php-server gradato **2c4242b6** ma runtime VECCHIO (primo atto: pin nuovo)
· corpus **1417** per NOME nei 2 modi SUL PIN · default flag-ON · debug/
si rigenera: rimuoverla · NON lanciare run pesanti se `ab.done` manca.
