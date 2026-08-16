# Criterio S-149 p.3 — replica coppia WP t3 (DEBITO s148) — commit PRIMA del run

1. Harness: `wp148-harness/s148-pair.sh t3` INVARIATO (stesso file, tentativo
   nuovo — comparabilità con t1/t2 per costruzione; verdetto
   `wp148-harness/s148-pair-verdetto-t3.out`); pin s145 gate interno.
2. Confronto formale EREDITATO: on-only coppie proprie vs rif S-142
   [1,765–1,788] banda 0,036; esiti pre-registrati come t2 (dentro ⇒
   COMPATIBILE; fuori ⇒ nessun claim e si dichiara).
3. Deriva (az.rev. S-146 #3): il test ordinato Spearman si APPLICA solo con
   N=6 gambe pulite (soglia 0,829 EREDITATA, pre-dati); N<6 ⇒
   SOTTO-CAMPIONATO, dichiarato.
4. ictx leg1 (indagine s149-ictx-leg1-indagine.md): la firma % resta il gate
   VIGENTE; a CORREDO si legge l'eccesso ASSOLUTO (ictx/s − mediana MOTORE)
   per gamba, derivato dai .time già emessi — se leg1 esce di nuovo
   SEGNALATA con eccesso assoluto ≈ +50–90/s su entrambi i lati, la lettura
   «disturbo di finestra amplificato dalla baseline bassa» è CORROBORATA
   (3/3 finestre) e il criterio t4+ potrà proporre warmup esteso (mai
   retroattivo).
5. Banda_ON: da RIFONDARE multi-finestra nel verdetto di sessione — unione
   t1+t2+t3 delle coppie proprie ON pulite; MAI da finestra singola.
6. Igiene: misura di tempo ⇒ lock di finestra + quiescenza (nel harness);
   nessuna run pesante concorrente; census/sonda CHIUSE prima del lancio.
