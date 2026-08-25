# Criterio S-159 p.1a — coppia WP t9 @ pin s158 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s159-pair.sh` = COPIA DICHIARATA di s158-pair.sh (copia-gate rc=0,
   manifest s159-pair-copia.diff); TAG=t9, pin s158 (phpr 92b0aea36573955a +
   server f381b3666e740cab, pena rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE.
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795 ·
   t8=1,754; banda INVARIATA [1,738; 1,799] (t8 interno): dentro ⇒ COMPATIBILE
   (nessun claim) · sotto ⇒ direzione ↓ SEGNALATA · sopra ⇒ REGRESSIONE
   SEGNALATA ⇒ indagine PRIMA di ogni altra leva.
3. Attesa L-RF2 su WP DICHIARATA: piccola/nulla (la famiglia `__reflect_*` non è
   testa di profilo WP; prezzo meccanismo ~7 ns/chiamata — az.rev. S-158 #2 —
   per conteggi WP mai censiti) — un ↓ marcato NON attribuibile senza census WP
   proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t8: 6/6 ALTE ≥1830, livello unico dichiarato);
   deriva ρ a N=6.
5. EMENDE S-158 EREDITATE nella copia (az.rev. S-157 #3-#4): (a) header del
   verdetto coi pin dall'HASH MISURATO nel run, mai da stringa fissa;
   (b) sentinella language-server registrata nel `.out` a inizio E fine
   finestra — presenza ≠ esclusione (misure a user-CPU), si DICHIARA.
6. Gate a rischio morso PRE-DICHIARATI: quiescenza su flare mediaanalysisd
   (retry ×3 ereditato t4) · nessun gate d'identità in finestra (nessuna
   build); rc SOLO da pair-out/pair159-t9.done (mai da monitor). La CI resta
   in quiet_wait a lock presente: il lancio attende la quiete dei job in volo
   (coda notturna S-158 in smaltimento: un job REQUEUE disk-low a verbale).
