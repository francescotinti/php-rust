# REPORT_GAP_157 — SOLO sessione S-157 (2026-08-23/24 notte), pin s157 76787303716acd4e + server bdd32a989ae1f492

Misure di QUESTA sessione (verdetti in wp157-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t7, on-only) | **1,767–1,825 (N=5 pulite) · MEDIANA 1,795 COMPATIBILE** ∈ [1,738; 1,799] | bordo alto; leg3 sporca esclusa; banda_ON 0,058; peak MISTO |
| WP media (t7) | **2,459–2,579 user-only CANONICA** | leg3 SEGNALATA 2,600 esclusa dal canone |
| doctrine/orm | **7,028–7,067 net** (@ pin s156; ass. 34,82–34,91) | Δ ass. vs s154-window = falso allarme da drift di giornata (indagine + replica3: ratio DENTRO [6,972;7,053]) |
| doctrine/dbal | **7,701–7,853 net** | companion; leg2 contesa ictx SEGNALATA; estrazione emendata LC_ALL=C: phpr 3921/626 vs oracle 3929/594 A VERBALE |
| micro (R=5, promo s157) | arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 · re 2,5 | vs s156: ±1 tick |
| m-missload (giudice leva) | A 306→B 284 ns/iter (**D=+22,0**, −7,2%) | 1 class_exists MISS/iter con loader no-op |

Leva spedita: **L-AL1 promossa** (miss/autoload plumbing 0-alloc: −3 alloc/miss
su cammino comune; UB 20,7 rispettato; conferma post-pin +25,0 5/5). La fetta
census miss/autoload E ∈ [4,82;6,05]M alloc è ora in parte pagata; residuo
dichiarato: vec![arg.clone()] per loader (1 alloc/iter loader).
