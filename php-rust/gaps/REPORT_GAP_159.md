# REPORT_GAP_159 — SOLO sessione S-159 (2026-08-25/26 notte), pin s159 f2d17f18c00a4049 + server c8e43b585c0a4c74

Misure di QUESTA sessione (verdetti in wp159-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t9, on-only) | **1,757–1,814 (N=6 pulite 6/6) · MEDIANA 1,767 COMPATIBILE** ∈ [1,738; 1,799] | @ pin s158; banda_ON 0,058; peak 6/6 BASSE (doppio livello non riprodotto); deriva assente |
| WP media (user-only CANONICA) | **2,441–2,462** | companion user+sys 2,391–2,421; parità 6/6 (solo wp_is_stream #2) |
| doctrine/orm (net, @ s158) | **7,090–7,141** | attesa-RF2 COMPATIBILE (Δ_norm [−0,77;−0,16], scaletta dal solo dn_max: lato peggiorativo a cavallo DICHIARATO; RIF s158 CONTESO); replica-AL1 APERTA (phpr1 ictx 160,5 segnalata, 3ª finestra consecutiva); oracle di giornata più veloce (4,89/4,91) |
| doctrine/dbal (net, companion) | **7,440–7,630** | fail-set stabile 10 nomi |
| micro (promo @ pin s159) | arith 5,4 · prop 5,5 · calls 4,7 · str 4,3 · **arr 3,1** · re 2,5 | arr 3,3→3,1: companion della leva L-AM1 (direzione firmata, magnitudine non ripartita) |
| m-arrmap (giudice L-AM1) | phpr 126,0→115,0 ns/elem (D proprio +11,0) | bilaterale companion ~3,9× (oracle ~33 ns, run grezzo) |
| m-refl (sonda, stash fermi) | D=+24,0±5,0 (registro L-RF2 risolto) | coeff vec![args] TARATO 12,0 ns/alloc-sito (±2,5) |

Nessuna nuova voce oltre-3× aperta; obiettivo tappa: arr SOTTO 3,2 per la prima volta (3,1).
