# REPORT_GAP_164 — SOLO sessione S-164 (2026-08-30 notte), pin s164 == s163 fea4a2d040a0d8d0 + server 8d76d6f129bfd4af (nessuna promozione)

Misure di QUESTA sessione (verdetti in wp164-harness/):

| workload | rapporto phpr/oracle | note |
|---|---|---|
| WP full (t14, on-only) | **1,757–1,768 (N=6 pulite 6/6) · MEDIANA 1,761 COMPATIBILE** ∈ [1,738; 1,799] | PRIMA coppia @ pin s163; predicato anti-flare PRE-registrato (6×30s <5%); **banda_ON 0,011 = record** (prec. 0,018 t12) |
| WP media (user-only) | 2,341–2,450 | companion user+sys 2,307–2,412 |
| doctrine/orm | **[7,066; 7,111] VALIDO** (registrato s162 [7,035;7,086]: sovrapposizione, nessun claim; 2 gambe NON aggiornano il registrato) | sentinella NEGATIVA (oracle 4,86/4,85 lato veloce, 3ª coppia così: da tenere d'occhio); attesa-AU1 COMPATIBILE tetto ~0; AF1 pool +2 gambe |
| doctrine/dbal | [7,459; 7,491] CON RISERVA | ictx-oracle segnalata (3ª coppia consecutiva) ⇒ istruttoria in aperture con 3 punti dati |
| micro (pin s163, invariato) | arith 5,5 · prop 5,5 · calls 4,7 · str 4,2 · arr 3,2 · re 2,6 | **arith de-quantizzato: 5,426 (s161) · 5,454 (s162) · 5,417 (s163)** — tick 5,3→5,5 = quantizzazione, indagine CHIUSA |
| m-missload (leva L-AL3, caduta) | A=282,0 B=282,0 ns/miss **D=+0,0** | census: il pool RIMUOVE il Box (Δ=199998/200000) ma non paga ⇒ STOP p.3b, revert al byte |

Coppia misurata @ pin s163 (dovuta da S-163) ⇒ ASSOLTA; nessun pin nuovo ⇒ NESSUNA coppia dovuta a S-165.
