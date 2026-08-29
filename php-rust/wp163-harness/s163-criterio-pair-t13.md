# Criterio S-163 p.1a — coppia WP t13 @ pin s162 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s163-pair.sh` = COPIA DICHIARATA di s162-pair.sh (copia-gate a
   verifica POSITIVA dei path adattati + manifest s163-pair-copia.diff);
   TAG=t13, pin s162 (phpr 20c63af44bfd077a + server f6d4a63b23b963da, pena
   rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE (creato
   all'apertura S-163, solo VERIFICATO qui).
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795 ·
   t8=1,754 · t9=1,767 · t10=1,760 · t11=1,765 · t12=1,767; banda INVARIATA
   [1,738; 1,799] (t12 interno): dentro ⇒ COMPATIBILE (nessun claim) · sotto ⇒
   direzione ↓ SEGNALATA · sopra ⇒ REGRESSIONE SEGNALATA ⇒ indagine PRIMA di
   ogni altra leva.
3. Attesa L-AM2 su WP DICHIARATA (NEXT_SESSION p.1): il criterio CLASSIFICA le
   forme — il fast path ammette SOLO array_map con stringa di funzione UTENTE
   semplice arità-1; WP usa in prevalenza closures e builtin-string (NON
   ammessi); quota utente-string WP mai censita ⇒ attesa SOTTO-risoluzione/~0.
   Un ↓ o ↑ marcato NON attribuibile senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t12: 6/6 pulite). banda_ON GIÀ FONDATA=0,018 (t12):
   qui si REPORTA max-min come companion di continuità, non è più tentativo.
5. EMENDE EREDITATE nella copia (az.rev. S-157 #3-#4): (a) header del verdetto
   coi pin dall'HASH MISURATO nel run, mai da stringa fissa; (b) sentinella
   language-server registrata nel `.out` a inizio E fine finestra.
6. FINESTRA QUIETA (lezione S-161): lancio SOLO dalla catena s163-lancio-pair.sh
   che DICHIARA uptime+top pre-lancio (pair-out/quiet-decl-t13.txt); attende la
   fine della run CI 38383a776a49 in volo (lock S-163 già presente: la coda CI
   resta ferma). Adattamento DICHIARATO del lancio: NESSUNA attesa rim.done
   (p.0 rimisura non esiste a S-163). Contaminazione sistemica giudicata ANCHE
   col confronto oracle-vs-riferimento storico. rc SOLO da
   pair-out/pair163-t13.done. Data 12G al pre-flight (requeue CI <7G a verbale).
