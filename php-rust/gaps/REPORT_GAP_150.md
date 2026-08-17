# REPORT_GAP_150 — SOLO sessione S-150 (2026-08-17 sera), pin s150 cbbe71735effb165 + server 18c2740774336c82

Misure di QUESTA sessione (verdetti in wp150-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t4, on-only) | **1,745–1,800 (N=6 pulite) · MEDIANA 1,781 COMPATIBILE** ∈ [1,738; 1,799] | PRIMO giudizio a mediana (az.rev.3 S-149); peak MISTO; nessuna deriva |
| WP media (t4) | **2,480–2,555 user-only CANONICA** | companion 2,419–2,489 |
| doctrine/orm | **7,104–7,149 net** (da 8,370–8,427 @ s145) | **SCOMMESSA BT1 VINTA OLTRE-ATTESA: Δ +6,07/+6,70 s** (attesa 0,8–3,1 = pavimento solo-alloc dichiarato); parità 16 nomi == |
| doctrine/dbal | **7,283–7,491 net** (da 8,20–8,37 @ s145) | companion; deprecations ⇒ coerente con BT1; oracle1 SEGNALATA ictx |
| micro (R=5, promo) | arith 5,5 · prop 5,5 · calls 4,8 · str 4,3 · arr 3,3 · re 2,5 | invariate entro 1 tick |
| m-backtrace (bilaterale NETTO) | **5,50×** (B 733 vs oracle 133 ns/iter, pavimenti MISURATI) | pre-cura era 148× |

Leva spedita: **BT1 promossa** (unica leva s145→s150: direzione+meccanismo
firmati; il Δ ORM eccede l'attesa = lavoro di costruzione frame non prezzato,
dichiarato). Corpus 1414→**1412** (2 flip PASS per NOME).
