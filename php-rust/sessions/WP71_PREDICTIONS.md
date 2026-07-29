# predictions71 — pre-registro ATTRIBUZIONE retained-form (G-71.1, lock PRIMA di ogni letto)

Residuo da attribuire (pin WP-70, tripla): **2,1121 KiB/req · 20,000
obj/req** (mediane su 3 leg; spread 0,024 KiB/req · 0,003 obj/req).
Margini (G-71.2, ≥3×spread): **±0,072 KiB/req · ±0,009 obj/req**.

Firma RI-PINNATA a STRADDLE (L-71.3 — nuovo lock per nuovi letti,
KK70-3): **9 obj a cavallo 96|112 + 6@128 + 3@64 + 1 a cavallo
160|192 + 1@32**. L'invariante è il TOTALE 20,000; lo split per-bin è
boot-variabile (~1 KiB di payload con taglia boot-dipendente). Il bin
160|192 a ~0,99/req è FRAZIONARIO-compatibile: crescita ammortizzata
di un container (doubling) = canale separato dai canali interi.

## Strumento (L-71.1) — SCOPE-HEAP mimalloc su run_deferred

Build census (phpr-memgc71): push/pop del default-heap mimalloc
attorno all'esecuzione di `run_deferred` (mi_heap_new/set_default,
guard assert stesso-thread). Il census mi_bin segmenta già per src= ⇒
il RITENUTO del canale defer è per costruzione ciò che resta nelle
pagine del heap defer dopo il pop (il churn che muore non accumula).
NIENTE trap per-bin (L-71.2 la vieta sotto l'80%: 112+128 = 77,8% dei
byte su leg3).

**Clausola di validazione strumento (G-71.1(iii), KG71-1/KL71-1)**:
sui leg scope-heap il TOTALE used_n (tutti gli src) deve restare
**20,000 ± 0,009 obj/req** e used_b **2,1121 ± 0,072 KiB/req** — se lo
strumento muove il totale, il letto è NULLO. Nessun letto per-causa è
citabile prima che questa riconciliazione passi SULLO STESSO leg.

## Libro mastro ADDITIVO (G-71.1(ii), KG71-3) — attese che sommano a 20,000

| causa | contatore/segmento | attesa (obj/req) | banda DENTRO |
|---|---|---|---|
| C1 parked run_deferred (E-71.H4, candidato n.1: parked_modules +16/req, budget 132 B/defer × 16 = 2,112 KiB ≈ il residuo intero) | scope-heap src=defer used_n slope | **16,000** | [12, 20] |
| C2 interning mere-mention (P-69.5; bin 32/64/112 = taglie stringa) | src=main, Δ dopo C1 | **2,000** | [0, 4] |
| C3 attempted-guard run_deferred (P-71.4) | ispezione scope/reset | **1,000** | [0, 2] |
| C4 container doubling (bin 160&#124;192 frazionario) | per-bin frazionale | **1,000** | [0,5, 1,5] |
| C5 per-id re-lower (famiglia WP-28) | solo se C1 FUORI | **0** | [0, 8] |
| C6 accumulate_seed dedup (S-71.4) | solo se C1 FUORI | **0** | [0, 6] |
| C7 RefLeaf path d'errore | drainfails (tripwire H-71.3) | **0** | {0} |
| **Σ** | | **20,000** | ± 0,009 |

Ipotesi primaria H1: **C1 domina** (16/20 obj = 80%; 132 B/defer ×
16,00 calls/req = 2,112 KiB/req ≈ 2,1121 dentro il margine). Esiti:
- **src=defer used_n ∈ [12, 20]** ⇒ C1 DENTRO ⇒ attribuzione
  dominante al park del canale defer; si prosegue con la coda (C2-C4)
  solo se serve per arrivare a ≥80%.
- **src=defer used_n ∈ [−0,009, 0,5]** ⇒ C1 MORTA a verdetto macchina
  ⇒ ordine Pedersen sul candidato successivo (C5, C6) con lo stesso
  trucco scope-heap o contatore dedicato.
- altrimenti ⇒ PARZIALE: C1 = valore letto, il resto va cercato in
  C2/C5/C6 fino a Σ ≥ 80%.

**Criterio ≥80% (KK71-3)**: Σ retained attribuito ≥ 0,8×20,000 −
0,009 obj/req E ≥ 0,8×2,1121 − 0,072 KiB/req, SOLO da retained-form
(mai net/flow). Validazione release-side P-71.3: two-boot vmmap
post-fix (o post-attribuzione con fix del canale dominante) atteso
**Δ < 2 MiB** su ΔN=4000.

## Finestra e parser (G-71.3, G-71.4)

- **WSTART = 100, WEND = 1000** — COSTANTI DICHIARATE QUI (nessuna
  derivazione da eventi in questo protocollo); finestra [R100, R1000].
- Assert macchina di validità finestra (tutti e tre, pena leg NULLO):
  vivi_n == N snapshot; ins (snapshot con inserimenti) > 40; nessun
  gradino di used_n > 500 obj tra snapshot adiacenti dentro finestra.
- Invarianti parser (assert, non prosa): snapshots == N;
  |Σ slope per-bin tracciati − slope totale| < 1,0 obj/req; modifiche
  al parser post-primo-letto ammesse SOLO su guard outcome-independent
  e col superseded che porta la CAUSA nella riga (K-71.4).
- Verdict emessi SOLO da script committato (P-71.1/KS-P71.1); curl
  CONTROLLATE (KS-P71.3: leg con curl fallite = NULLO); bonifica
  artifact di misura in apertura leg (P-71.2); stato canonico wpdev
  assertito in apertura (P-70.1 = contratto, P-71.5).

## Ladder di linearità (L-71.4, KL71-3)

Three-boot RELEASE strumento-free N=1000/5000/9000 (protocollo
P70-0-bis, stesso metro vmmap Physical footprint):
- rate12 = Δ(5k−1k)/4000, rate23 = Δ(9k−5k)/4000.
- **LINEARE (DENTRO)**: rate23/rate12 ∈ [0,6, 1,67].
- **SATURANTE (FUORI)**: rate23/rate12 < 0,6 ⇒ la banda [5,15] decade
  da conferma di linearità; l'obiettivo di attribuzione si ri-scopa
  sul componente persistente (KL71-3).
- Il rate phys resta un metro LOSSY: cifre citabili come esistenza e
  rapporto, mai come slope fine (L-70.2 invariato).

## Kill-switch armati su questo pre-registro

KG71-1 (letto per-sito senza riconciliazione ⇒ NULLO) · KG71-2
(reanalysis che tocca banda/finestra/stimatore ⇒ NULLO) · KG71-3
(attese che non sommano ⇒ pre-registro rigettato — Σ sopra = 20,000
esatto) · KL71-1 (strumento muove il totale ⇒ NULLO) · KL71-2 (dopo 2
candidati < 80% ⇒ mi_heap_visit_blocks + ispezione contenuto) ·
KS-S71.1 (strumentazione che muta l'esito di una delle 15 fixture
Stogov o gate-s70neg ⇒ STOP) · KB71-3/KS71-H2 (nessuna leva sul
canale defer prima dell'attribuzione; prototipo che muove la firma ⇒
revert) · KK71-2 (predizione senza timbro macchina a fine sessione =
NON-ESEGUITA d'ufficio).

Alias di token dichiarati NEL lock (K-71.1): "DOMINANTE" ≡ DENTRO di
H1-C1; "MORTA" ≡ FUORI-basso di H1-C1; "PARZIALE" ≡ FUORI con letto
valido. Set chiuso invariato: {DENTRO, FUORI, NON-CALCOLABILE,
NON-ESEGUITA}.
