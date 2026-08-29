# Criterio S-162 p.1a — coppia WP t12 @ pin s161 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s162-pair.sh` = COPIA DICHIARATA di s161-pair.sh (copia-gate a
   verifica POSITIVA dei path adattati + manifest s162-pair-copia.diff);
   TAG=t12, pin s161 (phpr ec0a636ad0c42005 + server cad5cf83fa2309c9, pena
   rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE.
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795 ·
   t8=1,754 · t9=1,767 · t10=1,760 · t11=1,765; banda INVARIATA [1,738; 1,799]
   (t11 interno): dentro ⇒ COMPATIBILE (nessun claim) · sotto ⇒ direzione ↓
   SEGNALATA · sopra ⇒ REGRESSIONE SEGNALATA ⇒ indagine PRIMA di ogni altra
   leva.
3. Attesa L-AL2 su WP DICHIARATA: SOTTO-risoluzione (quota loader-closure WP
   mai censita; coeff sito-autoload dalla rimisura s162 su stash fermi —
   tabella PER-SITO, veto S-161 sul trasporto tra siti) — un ↓ marcato NON
   attribuibile senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150%
   annotata); N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak:
   replica senza inserzione (riferimento t11: esito MISTO dichiarato, nessuna
   firma); deriva ρ a N=6. **Tentativo integrativo banda_ON (t11 N=4<5): se
   t12 esce con ≥5 gambe pulite, banda_ON fondabile (p.4 criterio t11) —
   6/6 pulite ⇒ fondare e chiudere il tentativo (NEXT_SESSION p.4).**
5. EMENDE EREDITATE nella copia (az.rev. S-157 #3-#4): (a) header del verdetto
   coi pin dall'HASH MISURATO nel run, mai da stringa fissa; (b) sentinella
   language-server registrata nel `.out` a inizio E fine finestra.
6. FINESTRA QUIETA (lezione S-161, NEXT_SESSION p.1): lancio SOLO dalla catena
   s162-lancio-pair.sh che DICHIARA uptime+top pre-lancio
   (pair-out/quiet-decl-t12.txt); contaminazione sistemica si giudica ANCHE
   col confronto oracle-vs-riferimento storico, non col solo flag per-gamba.
   Gate a rischio morso PRE-DICHIARATI: quiescenza su flare mediaanalysisd
   (retry ×3 ereditato); rc SOLO da pair-out/pair162-t12.done. CI in
   quiet_wait a lock presente (Data 14G al pre-flight, requeue <7G a verbale).
