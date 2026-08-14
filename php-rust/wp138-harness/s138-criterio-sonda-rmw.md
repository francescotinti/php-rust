# s138-criterio-sonda-rmw — attribuzione eccedenza fuori-modello D_rmw (PRE-REGISTRATO, prima dei numeri)

1. Obbligo dal criterio-rmw p.4: D m-dimrmw (smoke +170,0) > 63,3+100 = FUORI
   MODELLO ⇒ sonda PRIMA di promuovere. Oggetto: mostrare che l'eccedenza
   vive nei canali DICHIARATI (read-walk `field_value` + secondo preludio +
   write-walk `field_set_op`) con un contrasto OMOGENEO PER COSTRUZIONE.
2. Strumento: probe arm-only (tecnica VALIDATA in s138-sonda-v2: inerzia
   0,000, inline mantenuto) sul sorgente DELLA LEVA (worktree @ HEAD con
   FD1-ext): seg0 = arm intero di `FieldAssignOp`, seg9 = calibrazione;
   call-site intatto; modulo `s138tp.rs` (copia dichiarata di s137tp + UNA
   aggiunta: kill-switch `PHPR_TP_FULL=1` che forza MISS del fast path).
   L'arm PIENO e l'arm FAST vengono dallo STESSO binario: niente banda di
   non-omogeneità tra sonde.
3. Misura su m-dimrmw: R=3 per (seg0, fast) · (seg0 con PHPR_TP_FULL=1,
   pieno) · (seg9); mediana ns/span; overhead seg9 sottratto da entrambi.
4. IDENTITÀ: (arm_full − arm_fast) vs D_A/B R=5 su m-dimrmw, banda 13,3 +
   ε DICHIARATO NON PREZZATO: l'arm_full del candidato include
   `field_prelude_skip`+`field_assign_fill` che il pieno del pin s136 NON
   ha (scarto atteso spostato verso l'alto di ε ≥ 0). |scarto| ≤ 13,3+10
   (tetto ε dichiarato 10 ns, ordine del predicato+IC-write) → eccedenza
   ATTRIBUITA per NOME (i due walk + preludio pesano arm_full − arm_fast);
   fuori → NON CHIUSA, promozione FERMA.
5. Gate PRIMA del tempo: nm probe: `field_rmw_fast`+`field_assign_fast`
   ASSENTI (inline) · parità fixtures-rmw probe==candidato · parità stdout
   bench vs oracle a ogni run · inerzia: probe senza env vs candidato su
   m-dimrmw R=3 user, delta ≤ max(0,012 s, drop-1).
6. Vincoli: lock misura ATTIVO · quiescenza gate separato · pgrep
   rust-analyzer pre-finestra · build SOLO dopo il done dell'A/B R=5 ·
   verdetto `s138-sonda-rmw-verdetto.out`, rc da file.
