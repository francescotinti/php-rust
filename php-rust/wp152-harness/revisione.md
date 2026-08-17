# Revisione adversariale S-152 — lente MISURA (revisore singolo, REGOLE.md §7)

Claim: «NO-GO ROBUSTO, A3c CHIUSA». Aritmetica RIFATTA a mano dai 4 file prezzi
+ conteggi census: bande, quote (residui C2=48.682.514 e C5=51.595.524 esatti
contro il top-10), banda_netta [1,109;1,324] e S3 [3,627;3,995] riprodotte.
Soglie pre-registrate s151 §6, giudizio meccanico conforme al §6 s152.

**Piste vagliate.**
1. **Buco hot simmetrico: FONDATO e NON dichiarato.** Il §7 dichiara HOT=LB il
   solo lato mock; ma anche i prezzi CORRENTI (c1_pair, c2_borrow, note, mv_obj)
   sono su UN handle caldo. In-suite entrambi i mondi pagano miss: il netto regge
   solo per cancellazione common-mode (1 miss per accesso in entrambi), MAI
   misurata né dichiarata. S2 manca di 0,21 s e S3 di 0,59 s: il «a fortiori»
   vale nel modello hot-hot, non in assoluto.
2. **Cuscinetto pro-GO che lo compensa:** c1_pair/mv_obj prezzano la COPPIA
   clone+drop ma C1/C5 contano clone e drop come eventi SEPARATI ⇒ S3_hi gonfiato
   ~2× su C1 (1,40 vs ~0,70 s) e C5 (0,86 vs ~0,43 s), C5-netto idem; più clamp
   a zero, C5 tutta a prezzo obj, mock_decq MAI addebitato al lato mock: tutte
   inflazioni pro-GO. S3 «reale hot» ≈2,9 s: il buco cache dovrebbe valere +60%
   per flippare. Verdetto quindi regge, l'etichetta «robusto» va qualificata.
3. **c2_borrow_mut 1,68 < borrow 4,27:** plausibile (store di costanti vs catena
   inc/dec dipendente); controprova: con borrow_mut=borrow, bn_hi≈1,355 <1,53 —
   non flippa.
4. **S3 lorda con C1:** il §6 s151 non specifica; la lettura scelta è la PIÙ
   generosa possibile ⇒ ogni lettura alternativa dà S3 minore: NO-GO robusto
   all'interpretazione. Clamp: gonfia, pro-GO, safe.
5. **p.3 t3:** dovuta (mv_obj 14,6%, mock_dup_rel ~14%, c2_borrow 3,4%) ed
   eseguita ×2; pooling min/max su 4 repliche allarga le bande pro-GO. Onorata.
6. **Terzo difetto di forgia:** header del verdetto gonogo dice «price-r{1,2}»
   ma i file usati sono QUATTRO (rec incluse); mst/mar/mq/mh letti e mai usati.
   Cosmetico, non aritmetico.

**Azioni (3-5):**
1. Emenda a verbale al §7 s152: dichiarare il lato prezzo-corrente ANCH'ESSO hot;
   il NO-GO poggia su common-mode cache + cuscinetto pair-per-evento (senza
   toccare soglie né rerun).
2. Dichiarare la convenzione pair-per-evento (C1/C5 ~2× pro-GO) nel verbale.
3. Correggere la riga «prezzi:» del verdetto gonogo: 4 file, rec comprese.
4. Annotare mock_decq non addebitato (pro-GO, safe).
5. Vincolo per ogni eventuale riapertura A3c: sonde working-set (~180k slot) su
   ENTRAMBI i lati, non solo mock.

REVISIONE: REGGE CON RETTIFICA — il NO-GO sta, ma «ROBUSTO a fortiori» va qualificato: l'estremo hi è un maggiorante solo nel modello hot-hot (il buco cache è simmetrico e NON dichiarato al §7); la robustezza reale la fornisce il cuscinetto pro-GO pair-per-evento (~1,1 s), da dichiarare.
