# REPORT_GAP_109 — SOLO sessione S-109 (2026-08-07 sera)

Coppia WP full+media BIMODALE sul pin **3b3d25e2** (S-108, pre-lotto-3;
criterio PRE 5b7ea5b; giudice pair109.sh, ricetta pair94/102/108 invariata;
rapporti macchina dai .time in `wp109-harness/pair109-ratios-{on,off}.out`).

| metrica | modo ON (default) | modo OFF | note |
|---|---|---|---|
| full master CPU | **1,842×** (449,26/827,58 s) | 1,911× (447,66/855,63 s) | ON = MINIMO STORICO, sotto banda [1,86;1,93]: debito icache lotto-2 assolto |
| media user CPU | 2,707× (21,38/57,87 s) | 2,634× (21,08/55,52 s) | off AL riferimento 2,64; voce aperta perde monotonia (4ª/5ª lettura) |
| full peak phpr | 1845,49 MiB | 1872,00 MiB | ON sotto la famiglia storica 1863-1998 |
| media peak ratio | 3,169 | 3,396 | metrica rumorosa ~10% |
| parità | per NOME ✓ | per NOME ✓ | solo wp_is_stream ftp (catalogo); 30472 test / 4.558.029 assertion identici |

gate_void=0 ×2; uploads sotto guardia (backup→wipe→restore verificato).

**Micro sul pin NUOVO 92909544** (R=5, dopo lotto-3): arith 9,3 · prop 7,9 ·
calls 5,1 · **str 5,3** (S-108: 6,2; leva A/B Δ=+37,5 ns/iter 5/5) · arr 3,9 ·
re 3,5. Nessun A/B WP sul lotto-3 in questa sessione: la coppia S-109 misurava
il pin S-108 (debito lotto-2); il collaudo aggregato del lotto-3 è la coppia
DOVUTA in S-110 (run_loop 287.944 B, −976: attesa nulla o favorevole).

Server: pin **443ae42f GRADATO PIENO ×2** (S-109; post-lotti 1-2, pre-lotto-3
— binario php-server non contiene il lotto-3): cifre server attribuibili.
