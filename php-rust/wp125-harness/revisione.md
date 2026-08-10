# Revisione S-125 — lente MISURA

## Reperto principale

Il registro «SOVRAPPOSTO ⇒ il salto micro PhpStr NON muove il full» (GAP_TREND, roadmap) è un claim di ASSENZA oltre il potere dello strumento, e il bordo alto dell'intervallo poggia su una gamba inquinata: `pair-out/leg1-off/full-phpr.time` registra **249.003 involuntary context switches contro 78.314–80.805 delle altre tre gambe (3,1×)** — è quella gamba, da sola, a produrre la cella max 1,896 e lo spread phpr 3,2% (senza off1: off2 817,94 vs on 802,2x ≈ 2%, tutto effetto-modo). L'effetto full atteso dalle micro (str/arr/re ~1–2% del full) è sotto la risoluzione di cella; sugli assoluti phpr ON è fermo (802,25/802,21 vs 801,02/807,67 S-120): la frase lecita è «effetto ≤~2% non risolvibile», non «non muove». Il verdetto stesso si copre («non distinguibile dal rumore N=2»); la copertura cade nella rotazione.

## Reperti secondari

1. Modo confuso con posizione: off occupa sempre le posizioni 1 e 3 della notte; un drift monotono carica sistematicamente off. Stesso difetto in S-120, quindi il confronto è coerente, ma l'effetto-modo (~2%) resta non separabile.
2. Census «6/6»: 5 predizioni su 6 replicano cifre già misurate sullo stesso head (admission S-124, lato A); l'unico bit nuovo a rischio è B prop=5,00. Inoltre `s125-census-verdetto.out` apre con «PRE: tree sporco — STOP» del tentativo abortito: file citabile che mescola due run.
3. Banda v2-s125 a N=1 notte: SL_str crolla 2,89→0,47 (6×); per calls la soglia è fissata dal PAV_PIN (0,60>banda 0,25), cioè dominata dal rumore di rilancio su binari byte-identici. Guardie solo-regressione a 0,40–0,60 ns/iter rischiano falsi scatti inter-notte.
4. cbargs: il lettore pre-registrato (`s125-ab.sh`) ha emesso MISURA_INVALIDA (rc=3 strutturale); la REFUTAZIONE è stata dichiarata leggendo a mano il tsv. L'early-stop a segno opposto È nel criterio p.6, ma come gate, non come giudice finale: sostanza salva (2/2, |D|≈10×SL), forma no.

## Vagliate e respinte

- Cifre ≠ raw: verificato — 828,20=786,68+41,52; 168,55/173,59 derivano esatti da `cb1smoke-runs.tsv`; banda arr 1,31=149,57−148,26.
- Ricetta S-120≠S-125: stessa pair109, stesse 16 celle, stesso oracle (built Jun 2 2026, identity), criterio committato prima del run (92d5e9b 00:20 < epoch 1786314041).
- Early-stop non pre-registrato: lo è (criterio cbargs p.6).

## Azioni S-126

1. Gate di contesa per gamba in cross-ratios (involuntary ctx switches; gamba >1,5× la mediana = nulla, rieseguire).
2. Emendare GAP_TREND in «effetto full ≤~2% non risolvibile» + A/B full off-patch s123↔s124 stessa notte (già deferito dal criterio p.8).
3. Seconda notte di banda; fino a replica, guardie con max(SL s123, SL s125).
4. Rigenerare il verdetto census senza l'header del run abortito.
5. Dotare s125-ab.sh di modalità smoke R=2 con lettore early-stop interno.
