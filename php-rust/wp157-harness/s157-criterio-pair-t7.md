# Criterio S-157 p.1a — coppia WP t7 @ pin s156 (DOVUTA: pin nuovo) — scritto PRIMA del run

1. Harness: `s157-pair.sh` = COPIA DICHIARATA di s155-pair.sh (copia-gate rc=0,
   manifest s157-pair-copia.diff); TAG=t7, pin s156 (phpr 42efea3e34feb390 +
   server ef89630f9c7408c3, pena rc=9); criteri s132/s146/s148 EREDITATI; lock della SESSIONE.
2. GIUDIZIO CANONICO a MEDIANA per finestra (az.rev.3 S-149): mediane storiche
   t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771; banda
   INVARIATA [1,738; 1,799]: dentro ⇒ COMPATIBILE (nessun claim) · sotto ⇒
   direzione ↓ SEGNALATA · sopra ⇒ REGRESSIONE SEGNALATA ⇒ indagine PRIMA di
   ogni altra leva.
3. Attesa HD2-hostcall su WP DICHIARATA: piccola/nulla (i 6 nomi convertiti non
   sono testa di profilo WP nei census s148/s149) — un ↓ marcato NON
   attribuibile senza census WP proprio.
4. Gambe 6; firma pre-registrata S-136 (>150% med = sporca; 120–150% annotata);
   N<4 pulite ⇒ finestra SOTTO-CAMPIONATA, nessun giudizio. Peak: replica senza
   inserzione (riferimento t6 6/6 da leggere nel verdetto); deriva ρ a N=6.
5. Gate a rischio morso PRE-DICHIARATI: quiescenza su flare mediaanalysisd
   (retry ×3 ereditato t4) · nessun gate d'identità in finestra (nessuna
   build); rc SOLO da pair-out/pair157-t7.done (mai da monitor). La CI resta
   in quiet_wait a lock presente: il lancio attende la quiete dei job in volo.
