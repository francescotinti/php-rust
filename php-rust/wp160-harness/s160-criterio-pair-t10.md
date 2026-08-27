# Criterio S-160 p.2a — coppia WP t10 @ pin s159 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s160-pair.sh` = COPIA DICHIARATA di s159-pair.sh (copia-gate rc=0,
   manifest s160-pair-copia.diff); TAG=t10, pin s159 (phpr f2d17f18c00a4049 +
   server c8e43b585c0a4c74, pena rc=9); criteri s132/s146/s148 EREDITATI; lock
   della SESSIONE.
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795 ·
   t8=1,754 · t9=1,767; banda INVARIATA [1,738; 1,799] (t9 interno): dentro ⇒
   COMPATIBILE (nessun claim) · sotto ⇒ direzione ↓ SEGNALATA · sopra ⇒
   REGRESSIONE SEGNALATA ⇒ indagine PRIMA di ogni altra leva.
3. Attesa L-AM1 su WP DICHIARATA: piccola/nulla (conteggi array_map WP mai
   censiti; prezzo 12,0 ns/passaggio-dispatcher solo su forme ammesse
   closure-anonima-semplice arità-1 1-array) — un ↓ marcato NON attribuibile
   senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t9: 6/6 BASSE ≤1825, livello unico dichiarato);
   deriva ρ a N=6.
5. EMENDE EREDITATE nella copia (az.rev. S-157 #3-#4): (a) header del verdetto
   coi pin dall'HASH MISURATO nel run, mai da stringa fissa; (b) sentinella
   language-server registrata nel `.out` a inizio E fine finestra — presenza ≠
   esclusione (misure a user-CPU), si DICHIARA.
6. Gate a rischio morso PRE-DICHIARATI: quiescenza su flare mediaanalysisd
   (retry ×3 ereditato) · nessun gate d'identità in finestra (nessuna build);
   rc SOLO da pair-out/pair160-t10.done. CI in quiet_wait a lock presente
   (coda REQUEUE disk-low a verbale, Data 10G dichiarato al pre-flight).
