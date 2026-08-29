# REPORT_GAP_162 — SOLO sessione S-162 (2026-08-29 notte/mattina), pin s162 20c63af44bfd077a + server f6d4a63b23b963da

Misure di QUESTA sessione (verdetti in wp162-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t12, on-only) | **1,760–1,778 (N=6 pulite 6/6) · MEDIANA 1,767 COMPATIBILE** ∈ [1,738; 1,799] | finestra quieta DICHIARATA (uptime/top); **banda_ON FONDATA=0,018**; misurata @ pin s161 |
| WP media (user-only) | 2,426–2,445 | companion user+sys 2,387–2,405 |
| doctrine/orm | **[7,035; 7,086] VALIDO** (rif s160 [7,077;7,097]) | sentinella contaminazione NEGATIVA (oracle 4,87/4,90 = −1,2% dal SUO rif); assoluto MIGLIORA [+0,45;+0,73]s; attesa-AF1 NON risolta per ampiezza (a cavallo RES 0,293) |
| doctrine/dbal | [7,391; 7,440] CON RISERVA | gambe oracle ictx-segnalate (denominatore corto ~1,15s) |
| micro (pin s162, R=5) | arith 5,5 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,1 · re 2,5 | tick arith ↑0,2 da sorvegliare (guardia A/B verde) |
| m-strmap (leva L-AM2) | oracle-parità; phpr 167→102 ns/elemento (**D=+65,0**) | coeff sito strmap 65,0±1,0 a TABELLA; conferma post-pin +68,0 cifra piena |

Coppia misurata @ pin s161 (dovuta: pin nuovo); leva promossa a FINE catena ⇒ coppia @ s162 dovuta S-163.
