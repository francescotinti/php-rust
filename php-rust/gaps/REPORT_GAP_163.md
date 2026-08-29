# REPORT_GAP_163 — SOLO sessione S-163 (2026-08-29 giorno), pin s163 fea4a2d040a0d8d0 + server 8d76d6f129bfd4af

Misure di QUESTA sessione (verdetti in wp163-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t13, on-only) | **1,749–1,779 (N=6 pulite 6/6) · MEDIANA 1,763 COMPATIBILE** ∈ [1,738; 1,799] | finestra quieta DICHIARATA (uptime/top); banda_ON companion 0,029 (fondata resta 0,018 t12); misurata @ pin s162 |
| WP media (user-only) | 2,435–2,466 | companion user+sys 2,387–2,434 |
| doctrine/orm | **[7,072; 7,114] VALIDO** (registrato s162 [7,035;7,086]: sovrapposizione parziale, dentro RES, nessun claim) | sentinella contaminazione NEGATIVA (oracle 4,85/4,84 dal lato VELOCE); assoluto MIGLIORA [+0,04;+0,21]s; attesa-AM2 COMPATIBILE (tetto ~0); attesa-AF1 aperta |
| doctrine/dbal | [7,394; 7,570] CON RISERVA | ictx-oracle SEGNALATE per la 2ª coppia consecutiva ⇒ istruttoria in aperture |
| micro (pin s163, R=5) | arith 5,5 · prop 5,5 · calls 4,7 · str 4,2 · arr 3,2 · re 2,6 | ⚠️ arith 5,5 persiste da s162 ⇒ indagine dovuta S-164 |
| m-arrload (leva L-AU1) | oracle-parità; phpr 334→292 ns/miss (**D=+42,0**) | census 3 alloc/miss ESATTE; conferma post-pin +42,0 segni 5/5 cifra piena; FUORI-UB SOPRA reperto (plumbing) |

Coppia misurata @ pin s162 (dovuta: pin nuovo); leva promossa a FINE catena ⇒ coppia @ s163 dovuta S-164.
