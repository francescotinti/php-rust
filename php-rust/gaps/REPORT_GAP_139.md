# REPORT_GAP_139 — S-139 (2026-08-15) — SOLO questa sessione

Coppia full+media @ pin s138 (obbligo pin nuovo assolto) + rimisura dbal/ORM
(richiesta utente). Cifre SOLO dai verdetti:
`wp139-harness/s139-pair-verdetto-t1.out` · `wp139-harness/s139-rimisura-verdetto.out`.

## Coppia full+media (6 gambe TUTTE ON; 5 pulite, leg6 sporca esclusa)

| metrica | rapporto phpr/oracle | nota |
|---|---|---|
| full, cpu user+sys, coppie proprie ON | **1,752–1,785 (N=5) CANONICO** | rif S-137 1,767–1,781: COMPATIBILE su banda 0,041 ULTIMO USO — RMW non muove WP (atteso pre-registrato) |
| **banda_ON fondata** | **0,033** | max−min delle N=5 coppie pulite; da S-140 sostituisce la banda off 0,041 (az.rev. S-137 #4 assolta) |
| media group, user-only CANONICA | **2,470–2,486** | companion 2,428–2,444; dentro S-137 2,445–2,529, bordo alto rientrato |
| peak phpr full | **1831–1849 MiB** | OSSERVAZIONE: sopra banda oss. s136/s137 (1743–1825) su TUTTE le gambe; causa NON attribuita (candidate per NOME: celle IC per-sito RMW, warmup on-config) |

Parità 6/6: media diff vuoto; full solo `wp_is_stream #2`. Quiescenza 7/7
rc=0 senza retry. Gambe a verbale nel verdetto (matrice 5×5 inclusa).

## Rimisura dbal/ORM @ s138 (ricetta S-135; test delle TRE leve dim-write)

| workload | net S-139 | net S-135 @ s134 | lettura |
|---|---|---|---|
| doctrine/dbal | **8,15–8,23** | 8,36–8,45 | direzione ↓ lieve (−2,5%; nessun A/B proprio ⇒ direzione sola) |
| doctrine/orm | **8,59–8,71** | 8,43–8,56 | **FERMO/lieve ↑ — REPERTO: AP1+FD1+RMW non parlano alla suite; l'attesa ↓ pre-registrata è FALSIFICATA** ⇒ la prossima leva si sceglie dal profilo SUITE (churn clone/drop, insert/lookup) |

Parità: ORM fail-set 16 nomi == baseline (2 gambe); dbal 10 nomi stabili ==
baseline (0,25% ≤1% ⇒ canonica). Gambe segnalate al gate ictx (orm-phpr1,
dbal-oracle×2 su run corti ~1,6 s) con coerenza stesso-lato <1% ⇒ valide
(precedente S-135). Summary phpr dbal vuota = classe S-126 #3 (fail-set dai
.failnames). Floors per-workspace 0,06/0,07 · 0,06/0,19.
