# Criterio S-158 p.1a — coppia WP t8 @ pin s157 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s158-pair.sh` = COPIA DICHIARATA di s157-pair.sh (copia-gate rc=0,
   manifest s158-pair-copia.diff); TAG=t8, pin s157 (phpr 76787303716acd4e +
   server bdd32a989ae1f492, pena rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE.
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771 · t7=1,795;
   banda INVARIATA [1,738; 1,799] (t7 interno): dentro ⇒ COMPATIBILE (nessun
   claim) · sotto ⇒ direzione ↓ SEGNALATA · sopra ⇒ REGRESSIONE SEGNALATA ⇒
   indagine PRIMA di ogni altra leva.
3. Attesa L-AL1 su WP DICHIARATA: piccola/nulla (miss di classe/autoload non
   sono testa di profilo WP; Composer cachea i miss — reperto S-157) — un ↓
   marcato NON attribuibile senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t7 MISTO, dichiarato); deriva ρ a N=6.
5. EMENDE S-158 (az.rev. S-157 #3-#4, DICHIARATE nella copia): (a) header del
   verdetto stampa i pin dall'HASH MISURATO nel run, mai da stringa fissa;
   (b) sentinella language-server (pgrep pattern IDE/LSP) registrata nel `.out`
   del verdetto a inizio E fine finestra — presenza ≠ esclusione (misure a
   user-CPU), si DICHIARA.
6. Gate a rischio morso PRE-DICHIARATI: quiescenza su flare mediaanalysisd
   (retry ×3 ereditato t4) · nessun gate d'identità in finestra (nessuna
   build); rc SOLO da pair-out/pair158-t8.done (mai da monitor). La CI resta
   in quiet_wait a lock presente: il lancio attende la quiete dei job in volo.
