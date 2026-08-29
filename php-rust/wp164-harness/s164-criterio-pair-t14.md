# Criterio S-164 p.1a — coppia WP t14 @ pin s163 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s164-pair.sh` = COPIA DICHIARATA di s163-pair.sh (copia-gate a
   verifica POSITIVA dei path adattati + manifest s164-pair-copia.diff);
   TAG=t14, pin s163 (phpr fea4a2d040a0d8d0 + server 8d76d6f129bfd4af, pena
   rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE (creato
   all'apertura S-164, solo VERIFICATO qui).
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795 ·
   t8=1,754 · t9=1,767 · t10=1,760 · t11=1,765 · t12=1,767 · t13=1,763; banda
   INVARIATA [1,738; 1,799] (t13 interno): dentro ⇒ COMPATIBILE (nessun claim)
   · sotto ⇒ direzione ↓ SEGNALATA · sopra ⇒ REGRESSIONE SEGNALATA ⇒ indagine
   PRIMA di ogni altra leva.
3. Attesa L-AU1 su WP DICHIARATA (NEXT_SESSION p.1): il criterio CLASSIFICA le
   forme — il fast path morde SOLO al MISS autoload con loader [obj,metodo]
   k=1; a REGIME Composer risolve da classmap (hit), i miss sono RARI e la
   quota miss WP NON è censita ⇒ attesa SOTTO-risoluzione/~0, NESSUN claim
   direzionale. Un ↓ o ↑ marcato NON attribuibile senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t13: 6/6 pulite). banda_ON FONDATA=0,018 (t12):
   max-min REPORTATO come companion di continuità.
5. EMENDE EREDITATE nella copia (az.rev. S-157 #3-#4): (a) header del verdetto
   coi pin dall'HASH MISURATO nel run, mai da stringa fissa; (b) sentinella
   language-server registrata nel `.out` a inizio E fine finestra.
6. FINESTRA QUIETA + PREDICATO ANTI-FLARE PRE-REGISTRATO (az.rev. S-163 #5, il
   flare mediaanalysisd è noto dalla p.1b s163): lancio SOLO dalla catena
   s164-lancio-pair.sh che (a) attende la fine della run CI 4754d04ab2cf in
   volo (lock S-164 creato all'apertura: la coda CI resta ferma); (b) esige
   quiete CONTINUA 6 campioni ×30s con mediaanalysisd <5% (predicato del
   riaggancio-r5b s163, ora NEL criterio, azzerato a ogni campione ≥5%);
   (c) DICHIARA uptime+top pre-lancio (pair-out/quiet-decl-t14.txt); poi exec.
   Contaminazione sistemica giudicata ANCHE col confronto oracle-vs-storico.
   rc SOLO da pair-out/pair164-t14.done. Data 11G al pre-flight (requeue CI
   <7G a verbale).
