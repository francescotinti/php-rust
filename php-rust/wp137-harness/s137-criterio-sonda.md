# s137-criterio-sonda — ripartizione eccedenza FD1 +13,7 (az.rev. S-136 #4; PRE-REGISTRATO, prima dei numeri)

1. Oggetto: attribuire per NOME l'eccedenza D_A/B 83,3 − UB 69,6 = **+13,7 ns/statement**
   su objdatains (FUORI-UB +0,4 dichiarato S-136). Ipotesi NOMINATE (da vagliare, non
   da confermare): quota di plumbing_set 17,6 bypassata al hit · leaf_index 18,9 più
   corto sul fast path (inlining/monomorfizzazione) · dispatch+push 7,0 ridotto ·
   pop+keys 4,5 · costo IC-check aggiunto (segno OPPOSTO: riduce l'eccedenza spiegabile).
2. Strumento: probe a SEGMENTI, stessa architettura s136-tempo (s136tp adattato al
   sorgente s136 CON fast path; #[path] fuori-crates = ricetta; build EMENDATA, hash
   dichiarato, MAI pinnabile) su m-dimwrite; R=3, mediana per segmento, overhead
   coppia (seg9-equivalente) sottratto. Baseline di confronto = segmenti s136-tempo
   (probe @ s135): il confronto è TRA SONDE OMOGENEE, mai cifra-leva.
3. Chiusura: arm post-leva atteso ≈ 118,2 − 83,3 = 34,9 (dall'A/B). I segmenti
   nominati (dispatch+push · IC-check · walk child · plumbing residuo · pop) devono
   chiudere ≥90% dell'arm post-leva misurato, o modello INCOMPLETO dichiarato.
4. Conteggi di controllo (run SEPARATO dal tempo): contatore hit/miss fast path su
   m-dimwrite — atteso hit≈N, miss=O(1) (fill alla prima iter). Miss ricorrenti ⇒
   la ripartizione non è interpretabile, sonda FALLITA dichiarata.
5. Esito in s137-sonda-verdetto.out: eccedenza ATTRIBUITA per NOME (somma canali
   spiegati entro ±banda spread-batch 13,3) oppure NON CHIUSA dichiarata ⇒ il blocco
   az.rev. S-136 #4 sulle leve dim-write PERSISTE.
6. Vincoli: esecuzione SOLO dopo il done della coppia t1 (misure sequenziali); lock
   misura RICREATO prima; quiescenza gate separato; NIENTE Serena/LSP in finestra;
   parità fixtures-fd1 sulla build emendata PRIMA del tempo (come s136-tempo).
