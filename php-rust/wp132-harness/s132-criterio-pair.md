# Criterio S-132 p.1 — coppia WP full+media sul pin s131 (rimisura riferimento post-E1-KO)

COPIA DICHIARATA di `wp131-harness/s131-criterio-pair.md` coi SOLI adattamenti:
pin s131, segno atteso aggiornato, az.rev. S-131 #3 e #5 cablate (punti 4-bis e 3).

1. Oggetto: RIMISURA del riferimento sul pin s131 `ff66cb846e6cd439` + server s131
   `97ed6e0635b3f71a` (E1-KO tocca FieldAssign): nessuna soglia di promozione,
   l'esito REGISTRA il nuovo riferimento qualunque sia.
2. R: 4 gambe INTERCALATE off1→on1→off2→on2 (ricetta pair109 INVARIATA) precedute
   da UNA gamba WARM-UP dichiarata (media-only bilaterale, MAI giudicata) — riuso
   S-131: 4/4 gambe pulite col warm-up leg.
3. Giudice: `s132-pair.sh <tentativo>` → cross-ratios meccanici dai `.time`; full
   cpu=user+sys ETICHETTATO; media CANONICA user-only + companion user+sys.
   **Az.rev. S-131 #5**: verdetto = FILE NUOVO per tentativo
   (`s132-pair-verdetto-<tentativo>.out`, rifiuto se esiste), mai append su
   tentativo fallito. Questo criterio è committato PRIMA del codice
   dell'orchestratore (commit distinti).
4. Gate contesa: ictx/s per gamba e per lato, mediana PER MOTORE, soglia 1,5×med
   del proprio motore; gamba segnalata ⇒ ESCLUSA dal riferimento ma riportata.
4-bis. **Az.rev. S-131 #3**: il giudice emette (a) il riferimento anche PER
   CONFIGURAZIONE — coppie proprie e intervallo separati per le gambe ON e OFF,
   con **ON-ONLY = riferimento CANONICO** (flag-on è default dal FLIP S-100; il
   full-PULITO misto resta come companion) — e (b) la FIRMA PER GAMBA accanto al
   gate 1,5×: ictx/s in % della mediana del proprio motore + rank della CPU
   oracle della gamba (1=più veloce) — la firma S-129 delle gambe escluse era
   (ictx +17%, oracle CPU la più veloce).
5. Quiescenza: gate SEPARATO (`wp129-harness/s129-quiescenza.sh`) PRIMA di ogni
   gamba, mai nello stesso comando del lancio; l'HEADER del verdetto cita
   file+valore di OGNI rc di quiescenza.
6. Segno atteso: full ≤ 1,757–1,797 (S-131 @ s130) — direzione ↓ piccola attesa
   (E1-KO −23,3 ns/iter su objdatains R=5, morde FieldAssign non-leaf); l'esito
   registra comunque (rimisura, non leva).
7. Parità per NOME attesa: media 0 nomi ×4; full diff == SOLO `wp_is_stream data
   set #2`. Diverso ⇒ indagine PRIMA di registrare.
8. gate_void=1 su una gamba ⇒ gamba NULLA. Peak footprint phpr registrato
   (rif 1788–1887 MiB sulle gambe pulite S-131).
9. rc autoritativo = SOLO `wp132-harness/pair-out/pair132-<tentativo>.done`
   scritto dall'orchestratore; run DETACHED sequenziale col daemonizer
   (`wp52-harness/daemonize.pl`), nessun'altra run pesante in parallelo;
   rust-analyzer quiescente (gate ≤5% CPU); MAI edit `.rs` in finestra di misura;
   argv del lancio SENZA pattern del gate (lezione S-131: path neutri).
