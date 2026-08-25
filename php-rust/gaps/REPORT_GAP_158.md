# REPORT_GAP_158 — SOLO sessione S-158 (2026-08-25 notte), pin s158 92b0aea36573955a + server f381b3666e740cab

Misure di QUESTA sessione (verdetti in wp158-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t8, on-only) | **1,752–1,761 (N=6 pulite 6/6) · MEDIANA 1,754 COMPATIBILE** ∈ [1,738; 1,799] | bordo basso; banda_ON 0,008 (la più stretta della serie); peak 6/6 alto |
| WP media (t8) | **2,454–2,470 user-only CANONICA** | nessuna gamba segnalata |
| doctrine/orm | **6,952–7,093 net** (@ pin s157; ass. 34,90–35,25) | giudizio ORACLE-NORMALIZZATO (emenda S-157): Δ_norm [−0,22;+0,57] NON RISOLTA lato migliorativo; leg1 phpr ictx SEGNALATA |
| doctrine/dbal | **7,477–7,486 net** | companion; conteggi 3921/626 vs 3929/594 (metodo LC_ALL=C) |
| micro (R=5, promo s158) | arith 5,4 · prop 5,5 · calls 4,8 · str 4,3 · arr 3,3 · re 2,5 | vs s157: str/arr +1 tick |
| m-refl (giudice leva) | A 379→B 350 ns/iter (**D=+29,0**, −7,7%) | 2 chiamate __reflect_*/iter (method_info cache-hit + class_real_name) |

Leva spedita: **L-RF2 promossa** (tranche-2 slice `__reflect_*`: 6 nomi da
dispatcher Vec a slice, −1 args-Vec/chiamata su famiglia da 11,66M alloc/run
ORM a census). D=+29,0 ECCEDE l'UB-alloc 13,8+rumore: surplus nel cammino Vec
(free+doppio match) plausibile ma NON attribuito — **sonda conteggi post-cura
DOVUTA (S-159)**; conferma post-pin +21,0 segni 5/5 (drift-tree dichiarato).
