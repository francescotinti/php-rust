# Criterio S-148 p.2 — replica coppia WP t2 @ s145 (az.rev. S-146 #1) + test ordinato deriva peak↔rapporto (az.rev. S-146 #3) — commit PRIMA del run

1. Harness: `s148-pair.sh` = COPIA DICHIARATA di `wp146-harness/s146-pair.sh`
   (manifest `s148-pair-copia.diff`, collaudo copia-gate) coi SOLI adattamenti:
   tag t2/s148/wp148-harness + blocco giudice DERIVA (p.4) appeso al giudice;
   pin INVARIATO s145 (gate in testa); criterio S-146 EREDITATO INVARIATO
   (6 gambe tutte ON, quiescenza per gamba, gate ictx/s per MOTORE, parità per
   NOME, replica peak-only SENZA inserzione).
2. Scopo az #1: t1 era COMPATIBILE MARGINALE (leg1 1,823 vs limite 1,824).
   Confronto formale t2: coppie proprie on-only vs rif 1,765–1,788 su banda
   0,036. Esiti: COMPATIBILE ⇒ rif+banda restano canonici, debito replica
   SALDATO; FUORI BANDA di nuovo sul bordo ALTO ⇒ «marginale, replicato»: la
   finestra 0,090 si dichiara APERTURA STRUTTURALE da spiegare (nessun
   aggiornamento di banda da finestra singola, canone rev. S-146 az.2); FUORI
   BANDA altrove ⇒ nessun claim dal tentativo, replica t3 dovuta.
3. Banda: la banda_ON canonica resta 0,036 (UNIONE S-139+S-140+S-142);
   l'eventuale estensione dell'unione con S-146+t2 si valuta SOLO se t2
   compatibile e si dichiara nel verdetto di sessione, mai retro-fit.
4. Test ordinato DERIVA (az #3; soglie PRIMA dei dati t2; giudice python DENTRO
   lo script; OSSERVATIVO, nessun gate): sulle 6 gambe pulite ordinate,
   ρ_A = Spearman(indice gamba, peak_phpr) e ρ_B = Spearman(peak_phpr, rapporto
   proprio). Soglie N=6 (α≈0,05 unilaterale, critico 0,829): ρ_A ≤ −0,829 ⇒
   DERIVA DISCENDENTE confermata su t2; ρ_B ≥ +0,829 ⇒ ACCOPPIAMENTO
   peak↔rapporto confermato; entrambe ⇒ firma «stato discendente correlato al
   rapporto» (osservativa); altro ⇒ esito per componente, si dichiara. Gambe
   pulite <6 ⇒ test SOTTO-CAMPIONATO, nessuna soglia applicata.
5. Finestra: lock misura `/private/tmp/phpr-measure.lock` PRESENTE; CI in
   quiet_wait e run in volo ESAURITA prima del lancio; run DETACHED,
   sequenziale, DOPO il census p.1 (unica finestra pesante, ordine dichiarato).
6. Cifre citabili SOLO da `s148-pair-verdetto-t2.out`.
