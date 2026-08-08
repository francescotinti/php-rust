# REPORT_GAP_118 — S-118 (2026-08-08) — SOLO questa sessione

Coppia full+media BIMODALE rimisurata sotto A′ (debito R3-Pedersen, prima
misura WP dopo la pipeline nuova). Binari: phpr = pin s117 1656580e
(A′+L-A), server = pin s118 20411ba0 (A′). NB: il pin s118 phpr (15dfb6b3,
+H-P1) è nato DOPO la coppia: il suo WP è debito della prossima misura.

## Coppia full+media (raw: `wp109-harness/pair-out-{off,on}/`, rapporti macchina `wp109-harness/pair109-ratios-{off,on}.out`)

| metrica | ON (default) | OFF | nota |
|---|---|---|---|
| media group, user CPU | **2,506×** (20,73/51,94) | 2,580× (20,83/53,75) | serie A′ N=1: direzione-solo vs vecchia pipeline (az. rev. S-118) |
| full suite, master CPU | **1,913×** (400,38/766,01) | 1,955× (398,20/778,58) | rif VECCHIA pipeline 1,867/1,869 DECADE (pre-registrato); oracle stasera −11% assoluto: tra-sere = solo direzione |
| full peak footprint phpr | **1839,16 MiB** | 1917,52 MiB | serie A′ N=1: nessun ranking cross-pipeline (az. rev. S-118) |
| media peak ratio | 3,197 | 2,626 | metrica rumorosa (nota storica) |

Parità: media 0 fail ×4 gambe; full failnames diff == SOLO
`Tests_Functions::test_wp_is_stream` ftp (catalogo) ×2 modi; gate_void=0 ×2.

## Per-request (gate 4 R3): tripla census `wp118-harness/census-out/`
obj/req mediana 0,000 spread 0,000 · KiB/req 0,0000 · T-72.a/b PASS ·
uc-steady 3/3 leg — il residuo per-request CHIUSO in WP-72 resta chiuso sotto A′.

## Micro sul pin NUOVO s118 (promozione, `wp118-harness/promo-out/micro-pin-s118.out`)
arith 5,5 · prop 5,5 · calls 4,8 · str 5,3 · arr 3,8 · re 3,3 · held-out 6,3·2,5·5,4.
