# REPORT_GAP_110 — SOLO sessione S-110 (2026-08-07 notte)

Coppia WP full+media BIMODALE sul pin **92909544** (S-109, CON lotto-3;
criterio PRE ded4a74, banda full ON [1,81;1,88]; giudice pair110.sh, ricetta
pair109 invariata a meno del numero; rapporti macchina dai .time in
`wp110-harness/pair110-ratios-{on,off}.out`).

| metrica | modo ON (default) | modo OFF | note |
|---|---|---|---|
| full master CPU | **1,867×** (447,60/835,50 s) | 1,869× (449,10/839,16 s) | ON DENTRO banda: lotto-3 non paga icache sul WP; il divario bimodale full si chiude a rumore stasera |
| media user CPU | 2,673× (21,28/56,88 s) | 2,612× (21,26/55,54 s) | quinta lettura voce aperta: entrambe in discesa, off SOTTO il rif 2,64 |
| full peak phpr | 1843,53 MiB | 1986,75 MiB | ON in linea col minimo 1845,49 (S-109) |
| media peak ratio | 2,642 | 2,717 | metrica rumorosa |
| parità | per NOME ✓ | per NOME ✓ | solo wp_is_stream ftp (catalogo); media 0 nomi ×2 |

gate_void=0 ×4 gambe; uploads sotto guardia (backup→wipe→restore verificato ×2).
Riferimento full resta 1,842/1,911 (S-109): lettura S-110 in famiglia, nessuna
cifra promossa (dentro-banda).

**Leva (d) contatori — TESI FRONTEND FIRMATA** (criterio v2 9ff53cf, emendato
DICHIARATO pre-misura; xctrace CPU Counters, quote top-down per-processo,
mediana finestre ~1ms, R=3, giudici copie interne byte-identiche):
delivery-share phpr/oracle **arith 9,75×** (0,325 vs 0,033) · **prop 5,96×**
(0,102 vs 0,017) · controllo **arr 1,04×** (specificità ✓); discarded 30×/169×
su quote piccole. L'oracle ritira useful 96,5% su arith, phpr 67%. Firma di
FAMIGLIA frontend (causa IC/ITLB/redirect NON ripartita — limite del template
di serie, eventi grezzi infattibili da CLI). **S-111 candidata:
threaded-dispatch con A/B proprio.** Nessuna leva di codice spedita in S-110.
