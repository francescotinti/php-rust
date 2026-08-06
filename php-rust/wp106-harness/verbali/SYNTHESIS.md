# Concilio WP-106 — SINTESI DI CONVERGENZA (su S-104 e programma S-105)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in S-104**: la leva H-C2 è stata TENTATA col
suo A/B ESEGUITO DUE VOLTE (regola di ritmo SODDISFATTA; KS-GR-105-1
saldato) e il verdetto è CADUTA: Δ=−10,33/−11,33 ns/iter, 5/5, meccanismo
CANDIDATO nominato dal disasm (inliner flippato, bl 1101→0, +8 KB).
Conoscenza nuova e nominata: canale drop-call refutato al numeratore ·
direzione peak B>A FIRMATA (7/7, p=0,0078) · free di calls INCHIODATO
(1 alloc×32 B + 1 free×32 B/chiamata, ret_cell escluso per layout E misura)
· 19a/19b non arbitrano il meccanismo · memory_get_usage = stub. Rapporti
del giudice: **FERMI per la TERZA sessione** (prop 11,5).

**(b) Contatori**: full/media = WP-102 (2 sessioni fa; il debito coppia ha
ora una SCADENZA, sotto). **Contatore SDOPPIATO (A-GR-106-1)**:
ritmo-leve = rispettato (tentata con A/B); **sessioni-senza-Δ-rapporti = 3**.
**KS-GR-106-1: se S-105 chiude a Δ-rapporti zero (contatore = 4) ⇒
riallocazione di CATEGORIA obbligatoria in WP-107** (si cambia bersaglio,
non si itera).

**(c) Rischio d'oggetto più trascurato**: spendere S-105 a FIRMARE
l'ipotesi icache (contatori, PGO, ristrutturazioni) invece di spedire la
leva sul canale già INCHIODATO (args di calls). Composizione: i contatori
sono un braccio PARALLELO breve, MAI prerequisito della leva args
(team-leva: Hejlsberg li voleva prerequisiti, Bak/Leijen no — composto:
prerequisiti SOLO per future leve a tesi icache, es. H-ICS/H-C3).

## Refutazione capitale (Klabnik) — regola PIN-106

**PIN-105 è insoddisfacibile per costruzione**: la batteria relinka SEMPRE
il binario ⇒ la lettera «build→hash→STASH→batteria» declasserebbe ogni pin,
incluso 86a50d1c. **PIN-106 (vincolante da subito)**:
`build → hash₁ → batteria → re-hash₂ → STASH(hash₂) → fixture → corpus×2`
— la batteria certifica il SORGENTE, il pin è hash₂; ogni build dopo lo
stash = gate VOID (KS-KL-106-3); churn hash₁→hash₂ sempre documentato.

## Verdetti fase 1 (9/9 CON EMENDAMENTI, nessun MI OPPONGO)

Ricevute integrali nell'INDICE (`../COUNCIL_WP106_REVIEWS.md`); verbali
VINCOLANTI in `verbale-*.md`; team in `team-*.md`. Colpi maggiori:

1. **«icache-bound» DECLASSATO a ipotesi forte N=1** (Hoare∧Hejlsberg∧Bak∧
   Gregg): l'A/B misurò leva+inliner INSIEME; +8.000 B / 1101 siti ≈ 7,3 B
   ⇒ «inline ovunque» letterale smentito. KS congiunto (KS-HO-106-1 ≡
   KS-HE ≡ KS-BA-106-1): il claim si firma SOLO con contatori
   (INST_RETIRED, L1I-miss) o con inlining pinnato — mai più componenti di
   costo da un A/B che flippa il codegen. La CADUTA della leva regge
   comunque (due forme, 5/5).
2. **Leijen su R=7**: l'estimatore (mediana Δ accoppiati) è stato scelto
   POST-hoc ed è decision-relevant ⇒ la magnitudine si scrive
   «INDETERMINATA», non «sotto banda»; il verdetto sopravvive solo grazie
   allo STOP §3 (metrica esaurita). Futuro: estimatore accoppiato
   PRE-registrato o lettura VOID.
3. **Leijen su H-D**: la «simmetria byte» esclude le TAGLIE, non i SITI
   (bound ~50 ppm); il SiteTag PIENO è sostituito dal **probe cap-bump
   2→4** come gate d'apertura + censimento ARITÀ (Bak) + audit-fuga.
   Leva = **SmallVec inline-2** (pool REFUTATO: duplica la freelist TL di
   mimalloc). Attesa pre-registrata: Δ∈[6,14] ns/iter su calls.
4. **Matsakis**: 19a/19b non si condannano con due mutazioni-specchio —
   la TERZA, mirata al LORO osservabile (OBS-8/holder-esterno), decide:
   rossa ⇒ restano arbitri del MOVE; verde ⇒ riclassifica per NOME.
   fx20: cap 150 fisso → BANDA derivata + guardia erosione (cap/2) +
   mutante leak-PARZIALE. KS-MA-106-1: nessun verdetto futuro su
   memory_get_usage finché è stub.
5. **Stogov**: cura memory_get_usage a DUE GRADINI (contatore per-thread
   TLS o mi_* on-demand; functional-parity DICHIARATA, mai byte-parity);
   REFUTATI gli atomics process-global promossi a release (conflazionano i
   worker, tassano calls). Fedeltà in ordine: generator get_gc > §3.13
   unit > §3.12 regime-i (che viaggia nella futura fusione RMW).
6. **Pedersen∧Klabnik su coppia WP**: SCADENZA — trigger = prima leva
   promossa (stessa sessione: rebuild server @ HEAD + grado PIENO +
   coppia bimodale); fallback DURO: entro chiusura S-106 o il riferimento
   WP-102 DECADE a storico (KS-PE-106-2). Parity-null d'ora in poi sempre
   col PERIMETRO nominato (KS-KL-106-1); cifre server solo da pin
   same-HEAD gradato PIENO (KS-PE-106-1).
7. **Gregg (mandato inverso)**: protocollo admission per OGNI leva —
   admission-disasm (bl-count prima/dopo) + smoke R=2 prima del pieno +
   hash dell'output + seconda canna; leva senza admission/smoke = VOID
   (KS-GR-106-2).

## Conflitto registrato e composto

- **team-fedeltà propone prop-RMW (H-C3) DAVANTI a calls** (doppio
  rendimento perf+fedeltà) vs team-leva/Gregg: **leva #1 = args-calls**
  (canale inchiodato da due census, criterio quasi pronto, tre sedie
  convergenti). COMPOSIZIONE: S-105 apre con args-calls; H-C3 prop-RMW è
  il SECONDO slot (o il primo di S-106) e porta con sé §3.12-i e il
  braccio-contatori come prerequisito di tesi icache. Nessun altro
  conflitto sostanziale.

## Ordine DEFINITIVO S-105

1. **LEVA H-D args (SmallVec inline-2) — NON NEGOZIABILE**, nella sua
   finestra: a. atto zero ~30′: criterio SCRITTO PRIMA (attesa Δ∈[6,14]
   ns/iter su calls; co-primari: timing E census alloc/chiamata→**0,0000**;
   caduta sotto max(rumore ~3, banda-layout); admission-disasm + smoke
   R=2, KS-GR-106-2; mai cifre da codegen flippato, KS-BA-106-1);
   b. gate d'apertura: **probe cap-bump 2→4** + censimento ARITÀ dei
   call-site + audit-fuga (fixture: il Vec args non sfugge alla finestra);
   c. implementazione → smoke R=2 → A/B R=5 ABAB stessa sera → verdetto
   col criterio; d. **PROMOZIONE ⇒ trigger fedeltà STESSA SESSIONE**:
   rebuild php-server @ HEAD + grado PIENO (A-PE-106-1) + **COPPIA WP
   bimodale** (salda il debito; fallback S-106 NON prorogabile);
   e. gate **PIN-106**.
2. **Braccio parallelo BREVE (timebox ~30′, NON prerequisito)**: contatori
   INST_RETIRED/L1I-miss sulla coppia H-C2 ricostruibile (checkout
   4ea2cff); se non entra ⇒ backlog per NOME, il claim icache resta
   «ipotesi non firmata».
3. **Denti/arbitri nella finestra del gate**: terza mutazione OBS-8 su
   19a/19b (decide arbitri-del-MOVE vs riclassifica); fx20 cap→banda +
   guardia erosione + mutante leak-parziale; sigillo Copy sui payload
   trivial + doc verdetto S-104 nel predicato + «al byte»→«taglia+timing»
   (A-HO-106-1/2/3).
4. **Fedeltà (timebox ½)**: memory_get_usage voce 🔴 a catalogo SUBITO;
   gradino TLS SOLO se la leva è chiusa e il tempo resta (KS-ST-106-1);
   generator get_gc + fixture §3.13 unit = primi in coda.
5. **BACKLOG per NOME**: H-C3 fusioni prop-RMW (secondo slot, con
   §3.12-i + contatori-prerequisito); H-ICS cold-out (criterio firmato);
   design per-fase A-LE-105-5; PGO/outlining A-HE-106-4; 21,2% =
   prefisso di targeting della leva icache (A-HE-106-5); retention
   backup uploads (A-PE-106-3); banda-layout N≥3; disposizione mutanti
   sopravvissuti (A-MA-106-2).
