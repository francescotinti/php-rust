# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-101 (QUESTA
sessione — coppia nei DUE modi sul binario cumulativo, contatore 0)** ·
ultima campagna sull'OGGETTO = **S-101 (questa: H-C1a+b promosse e misurate)**.

**Ultima sessione (S-101, 2026-08-06)**: l'ordine del Concilio WP-102
eseguito nei punti 1, 2, 3a, 3b, 3c, 5 e 6-parziale (il punto 4 —
attribuzione crescita d'albero — è in handoff: nessuna finestra per ~10 run
full dopo la coppia). **DUE LEVE PROMOSSE dai loro criteri pre-registrati**:
H-C1a (guardia `#[inline]` scalari su `gc_note`: Δ=7,3 ns/iter, banda
[4,13]) e H-C1b (**MOVE dell'handle ricevitore owned** — non prestito, non
addref: l'handle poppato è già di proprietà; Δ=6,0 ≥ pavimento, SOTTO la
banda attesa [7,20] — sovrastima contabile del profilo REGISTRATA).
**prop 12,4 → 11,5**; trasversali: arith 12,7→12,2, calls 7,9→7,3, arr
4,6→4,3. Census dinamico ha ARBITRATO le tre predizioni (P1 ✓ 0 refcounted;
P2 aggravata: 6 coppie clone+drop Rc/iter; P3 raffinata: 4 gc_note/iter
tutte Long, residuo = `reg_store_slot`; alloc/iter≈0); profilo inline-aware
(samply+dSYM+atos -i): il 50% di run_loop è APERTO = 21,2% dispatch/corpi +
**~26,6% meccanica della pila operandi** (accessor Vec inlined) — più
grande del ciclo di vita Zval (28,2% confermato). Gate: 13 fixture
semantiche 2-modi (attese PRIMA) + §3.13 catalogata; batteria **1737/0**
(coi denti A-HE-102-1 a polarità corretta-dopo-morso e A-KL-102-3); corpus
1418×2 per NOME + diff per-test ZERO per ENTRAMBI gli stadi; coppia WP
bimodale: media 0 identici, full = SOLO delta pre-esistente wp_is_stream.
Dettaglio: `sessions/WP_SESSION_101.md` + `wp101-harness/*.out`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-101 chiusura, binario cumulativo, modo default)

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **12,2** | residuo = corpi non-Binary + pila operandi |
| proprietà | **11,5** (da 12,4; H-C1a+b spedite) | **H-C ATTIVA** — gambe residue NOMINATE: pila operandi ~26,6% + ciclo vita Zval residuo ~28% + 3 clone LoadVar/iter (canale EMISSIONE) |
| chiamate | **7,3** (da 7,9) | **H-D ATTIVA** — census: 5 gc_note/iter scalari, 3/iter non taggate da nominare |
| stringhe | 7,0 · array 4,3 · regex 3,5 | — (regex = parte sana) |

## LE IPOTESI — stato dopo S-101

- ~~H1, H-A1, H-B1, H-B2~~ — chiuse (rotazioni precedenti).
- **H-C (proprietà, 11,5×)**: H-C1a e H-C1b SPEDITE E PROMOSSE (criteri,
  census-control, gate cumulativi + coppia WP). **H-C1c (copy+addref
  condizionale per refcounted) NON scritta**: su prop.php i valori sono
  Long — serve un giudice/fixture per SPECIE (string/array in proprietà)
  prima di aprirla (A-BA-102-3). Le gambe NUOVE nominate dal profilo:
  (i) **meccanica della pila operandi** (~26,6% del tempo phpr dentro
  run_loop: as_slice/len/pop/push su Vec<Zval>) — trasversale, non solo
  prop; (ii) i **3 clone LoadVar/iter dell'handle** = canale EMISSIONE
  (il compilatore sa che è lo stesso $o); (iii) drop-glue sui temporanei
  scalari (Bak). Ognuna esige il SUO controfattuale contato.
- **H-D (chiamate, 7,3×)**: prima pietra census fatta (`hd-census-primo.out`);
  resta la tavola completa: census call-path bi-regime (nominare i 3
  gc_note/iter residui + i canali che non passano da read_slot) + profilo
  co-equale su calls.php.

### ⚖️ Concilio WP-102 — ESEGUITO in S-101 (punti 1-3, 5, 6-parziale; punto 4 in handoff)

Verbali in `wp102-harness/verbali/` (formato indice). KS soddisfatti in
S-101: KS-ST-102-3/KS-BA-102-2 (ri-baseline con ns/op dove censito),
KS-GR-102-1 (criteri dal controfattuale contato), KS-ST-102-2 (H-C1a
misurata DA SOLA), KS-MA-102-1/KS-ST-102-1 (fixture attese-prima),
KS-MA-102-3 (ordine distruttori mai divergente: fixture 12 + full per
NOME), KS-KL-102-3/KS-MA-102-4 (coppia WP non derogata), KS-KL-101-3
(assert conteggi↔nomi su ogni gamba). BACKLOG per NOME invariato in
`wp102-harness/verbali/SYNTHESIS.md` + nuovi da S-101: sentinella server
sul pin 2c4242b6 (NON collaudato); attribuzione dei 3 gc_note/iter di
calls; fixture per specie (string/array in proprietà) per H-C1c.

## Regole di metodo (invariate)

1. Il giudice è la micro-categoria. 2. WordPress è un collaudo di PARITÀ
(si esegue quando cambia l'emissione O il runtime — S-101 l'ha eseguito
per il runtime). 3. Ogni ipotesi porta il criterio di caduta scritto PRIMA.
4. L'apparato non entra nell'ordine se non blocca; timebox mezza sessione.

## Stato gate

- **phpr (pin release)**: **48a5d4384970d8ff** @ HEAD f808017 (hash churna
  col relink: fa fede HEAD) — DEFAULT flag-ON; contiene H-C1a (split
  gc_note) + H-C1b (move ricevitore su PropGet/PropSet). Batteria
  **1737/0** (1735 + denti A-HE-102-1/A-KL-102-3). Corpus **1418 per NOME
  nei 2 modi + diff per-test ZERO fuori carve-out** (entrambi gli stadi).
  Stash ADDITIVO `phpr-s101`; braccio B degli A/B: `phpr-s100-fix`
  (=f29883eb) e `phpr-s101-hc1a` (=0ef9498d).
- **php-server**: **2c4242b6c8120b8e** — ricetta OBBLIGATORIA
  `cargo build --release -p php-server --features axum-server` RISPETTATA.
  **NON collaudato** (sentinella bimodale non eseguita in S-101; la
  batteria/corpus/coppia coprono il runtime ma non il capture-boundary del
  server): grado parziale, collaudo = primo atto se si tocca il server.
  Stash `php-server-s101`. Registro = `PIN_REGISTRY.md`.
- **Launcher S-101** (`wp101-harness/`): `hc1-fixtures.sh` (13 fixture +
  carve-out §3.13 a diff esatto) · `s101-corpus-gate.sh` / `s101-corpus-diff.sh`
  (OUT in wp101) · `pair101.sh <off|on>` · `hc1a-ab.sh` (A/B interleaved,
  bracci via env A/B).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA (rust-analyzer):
  rimuoverla nel pre-flight. Disco locale ~22G dopo la pulizia S-101.

## Voci APERTE per NOME (misura/attribuzione dovuta)

- **PUNTO 4 WP-102 NON ESEGUITO — attribuzione crescita d'albero peak**:
  A/B pin S-99 (52330330, stash `phpr-s99-sigillo`) ↔ S-100 (f29883eb,
  stash `phpr-s100-fix`) STESSA-SERA con R≥5 e bande UNILATERALI sul
  rumore misurato per-motore, PRIMA del bisect (KS-GR-102-2). Il rumore
  full-peak oracle è ~10% intra-sera (terza conferma stanotte:
  720,9↔795,5 MiB) — la banda parte da lì.
- **sentinella server sul pin 2c4242b6** (vedi Stato gate).
- divergenze §3.11/§3.12/§3.13 — catalogate, non urgenti (famiglia
  fetch-undef: A-ST-102-1/2).

## Che cosa è SOSPESO (non abbandonato)

- **A-ZV2** (liveness+TakeSlot): invariata. ⚠️ lezione S-101: `liveness.rs`
  è feature-gated e NON è protetto dalla batteria senza feature — il fix
  BinaryAdd è entrato in S-101; una `cargo check --features zval-census`
  in batteria è a backlog.
- **Rollout Add nelle forme registro**: chiuso salvo misura ≥ pavimento.
- **Roadmap footprint**: ferma (full peak on 1863,8 MiB al riferimento).

## NON riproporre

Tutti i NON-riproporre WP-83..100 restano. Nuovi da S-101:

- **denti scritti senza leggere il corpo che pinnano** (A-HE-102-1 prima
  stesura: polarità di emit_binary dedotta dal riassunto ⇒ rosso; la
  polarità si legge nel codice).
- **atteso contabile dal costo/evento del profilo a campioni** senza
  dichiararlo banda LARGA: i simboli inlined sovracontano (~2× su H-C1b);
  il costo/evento fa fede solo dall'A/B.
- **misure VERDICT in finestra con burst remoto senza interleave**: la
  prima ri-baseline è stata VOID (spread 2,53 s); ABAB nella stessa
  finestra o si ripete.
- **`git add -u` a valle di edit multipli** (commit f9e9f22 ha mischiato
  gate-evidence e righe H-C1b): staging per FILE nominati.
- **borrow nudo dello slot valore** resta VIETATO (unanimità WP-102);
  il MOVE dell'handle owned NON è un borrow e non lo riapre.
- ereditati e ribaditi: pin effetto-collaterale; rc di pipe come gate;
  bande < rumore dello strumento; premesse ambientali nella batteria.

---
**Riscritto**: rotazione S-101 il 2026-08-06. Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione`. Harness di sessione: `wp101-harness/`.

## §S-102 — ORDINE (BOZZA della rotazione; il Concilio WP-103 lo giudica e lo fissa in `wp103-harness/verbali/SYNTHESIS.md`)

Oggetto proposto: **le due gambe trasversali nominate dal profilo S-101 —
la meccanica della pila operandi (~26,6%) e il canale emissione del
ricevitore (3 clone LoadVar/iter)** — più il debito del punto 4.

1. **Punto 4 WP-102 (debito)**: A/B pin S-99↔S-100 stessa-sera (R≥5,
   mediane, bande unilaterali sul rumore ~10% misurato) PRIMA del bisect.
2. **Ipotesi PILA OPERANDI (da iscrivere col SUO nome)**: census del
   traffico push/pop per categoria (contatore nel census build) +
   controfattuale contato PRIMA di ogni forma (es. slot-diretti al posto
   del round-trip in pila per gli operandi dei Prop-op — è EMISSIONE:
   dump-diff, non cronometro, come primo giudice).
3. **Canale emissione ricevitore**: i 3 clone LoadVar/iter dello stesso
   `$o` — forma candidata: PropGet/PropSet a SLOT-OPERANDO (à la forme
   registro) che leggono il ricevitore dallo slot senza transitare dalla
   pila; controfattuale = 3 coppie × costo A/B (~2 ns/coppia MISURATO in
   S-101, non stimato).
4. **H-D tavola completa**: census call-path bi-regime (nominare i 3
   gc_note/iter + i canali fuori read_slot) + profilo co-equale calls.
5. **Denti residui**: sentinella server sul pin 2c4242b6 (se si tocca il
   server); `cargo check --features zval-census` in batteria (protegge i
   moduli feature-gated); fixture per specie per H-C1c (A-BA-102-3).
6. (timebox) fix §3.13/§3.11 famiglia fetch-undef (A-ST-102-1/2) SE una
   leva tocca quel percorso.

Pre-flight S-102: pin phpr **48a5d4384970d8ff** @ HEAD f808017 (fa fede
HEAD) · php-server **2c4242b6c8120b8e** (NON collaudato) · corpus 1418 per
NOME nei 2 modi SUL PIN · default flag-ON · debug/ si rigenera: rimuoverla.
