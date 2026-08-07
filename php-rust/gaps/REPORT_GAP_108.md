# REPORT_GAP_108 — SOLO S-108 (2026-08-07)

Coppia WP full+media BIMODALE sul pin 62a4df65 (s107b; raw `wp108-harness/pair-out-{on,off}/`,
criterio pre-registrato `s108-pair-criterio.out`, rapporti macchina `pair108-ratios-{on,off}.out`):

| metrica | ON (default) | OFF | riferimento |
|---|---|---|---|
| full master CPU | **1,855** | 1,885 | 1,894 (WP-102) — banda [1,86;1,93]: ON SOTTO (favorevole, direzione) |
| media user CPU | 2,677 | 2,747 | 2,64 — resta voce APERTA (alta varianza; oracle stabile su 3 sere) |
| full peak phpr | 1866,69 MiB | 1979,91 MiB | in famiglia (on 1863-1942 / off 1979-1998) |
| parità | per NOME ✓ | per NOME ✓ | solo `wp_is_stream` ftp (catalogo); conteggi 30472/4.558.029 identici |

Esito: ipotesi icache del +16,4 KB (lotto-1) NON morde sull'aggregato ⇒ leva S-108 sbloccata dal
criterio; voce S-105 full-off 1,947 CHIUSA per NOME (rerun 1,885 in famiglia). Il +14,7 KB del
lotto-2 (run_loop 288.920 B) pretende la SUA coppia in S-109.
Micro sul pin nuovo 3b3d25e2: arith 9,4 · prop 8,0 · calls 5,3 · str 6,2 · arr 3,8 · re 3,5.
