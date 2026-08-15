# s144-criterio-B — REGOLA PRE-REGISTRATA (firmata PRIMA di sonda-B e profilo oracle; progettazione in s144-progettazione-B.md)

1. Oggetto: apertura della via B (deliberato S-143). NESSUNA riga di codice B prima di: sonda-B (ripartizione churn) + profilo oracle per famiglia + quota_obj_max (tranche-2).
2. Sonda-B: monobinaria classe S-138, SOLO conteggi+prezzi, ×2 repliche, r1≈r2 ≤1% per chiave; ripartisce il churn in memcpy / inc-dec / nota (denominatore dal sorgente della sonda).
3. REGOLA fette: inc-dec+nota ≥60% del churn ⇒ ordine B1→B2 · memcpy ≥60% ⇒ B1/B2 NON si aprono, filone conteggi (B3) torna al concilio · nessuno ≥60% ⇒ ordine per contributo assoluto misurato, fetta maggiore prima.
4. Ogni fetta: criterio proprio ≤10 righe (segno, soglia = max(4 ns, rumore, banda), R=5 ABAB, giudice micro churn + famiglia); fette su run_loop pretendono disasm bl-count prima/dopo.
5. Promozione prima fetta SOLO DOPO il profilo oracle: un canale che Zend paga in quota ≥50% della quota phpr esce dal budget (Gregg K3) e la fetta che lo bersaglia si ferma.
6. Scommessa B giudicata sulla SUITE: coppia ORM 2/lato net fuori banda ±0,7% entro ≤3 sessioni dalla prima fetta spedita E churn_zval+memops −25% relativo (profilo campionario, 2 repliche); altrimenti B FALSIFICATA (KS-B1) ⇒ revert al pin.
7. Orizzonte duro: 4 sessioni di fette con ORM fermo in banda ⇒ revert + riconvoca (KS-B2; si applica il più severo tra Klabnik-4 e Gregg-5).
8. quota_obj_max (tranche-2) in 25–40% ⇒ RICONVOCA prima di aprire B (la regola S-143 resta arbitra).
9. Esiti pre-registrati sonda: muta allo smoke ⇒ STOP rc=8, niente run · r1≠r2 >1% ⇒ dichiara e replica · esito fuori dalle tre bande p.3 impossibile per costruzione (partizione).
10. Gate semantici invariati (batteria · corpus 1414×2 per NOME · ORM 3E/13F · fixture bilaterali); fail nuovo per NOME in weakrefs/destructor ⇒ STOP fetta.
