# Criterio S-131 p.1 — coppia WP full+media sul pin s130 (rimisura riferimento post-F4)

1. Oggetto: RIMISURA del riferimento sul pin s130 `0fdf1c49b16c24ba` + server s130 (F4 tocca FieldAssign): nessuna soglia di promozione, l'esito REGISTRA il nuovo riferimento qualunque sia.
2. R: 4 gambe INTERCALATE off1→on1→off2→on2 (ricetta pair109 INVARIATA) precedute da UNA gamba WARM-UP dichiarata (media-only bilaterale, MAI giudicata): le 2 gambe escluse in S-129 erano PRIME di sequenza (indizio warm-up, apertura per NOME).
3. Giudice: `s131-pair.sh` → cross-ratios meccanici dai `.time`; full cpu=user+sys ETICHETTATO; media CANONICA user-only + companion user+sys. Questo criterio è committato PRIMA del codice dell'orchestratore (commit distinti).
4. Gate contesa: ictx/s per gamba e per lato, mediana PER MOTORE (addendum rev. S-129), soglia 1,5×med del proprio motore; gamba segnalata ⇒ ESCLUSA dal riferimento ma riportata.
5. Quiescenza: gate SEPARATO (`wp129-harness/s129-quiescenza.sh`) PRIMA di ogni gamba, mai nello stesso comando del lancio; l'HEADER del verdetto cita file+valore di OGNI rc di quiescenza (az.rev. S-130 #3).
6. Segno atteso: full ≤ 1,758–1,805 (S-129 @ s127b) — direzione ↓ piccola attesa (F4 morde FieldAssign prop-rooted, micro −80 ns su objdatains); l'esito registra comunque (rimisura, non leva).
7. Parità per NOME attesa: media 0 nomi ×4; full diff == SOLO `wp_is_stream data set #2` (S-119/S-120). Diverso ⇒ indagine PRIMA di registrare.
8. gate_void=1 su una gamba ⇒ gamba NULLA (KS-KL-101-3). Peak footprint phpr registrato (rif 1828–1880 MiB sulle gambe pulite S-129).
9. rc autoritativo = SOLO `wp131-harness/pair-out/pair131.done` scritto dall'orchestratore; run DETACHED sequenziale col daemonizer, nessun'altra run pesante in parallelo; rust-analyzer quiescente; MAI edit `.rs` in finestra di misura.
