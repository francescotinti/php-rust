# S-142 — criterio COPPIA WP @ pin s142 + PEAK PER POSIZIONE (pre-registrato PRIMA del run)

Obbligo: pin NUOVO s142 (phpr bba8a734 + server <hash da pin-server s142, scritto
nel harness PRIMA del lancio>) ⇒ coppia dovuta (regola utente 2026-08-12).
Harness: `s142-pair.sh` = COPIA DICHIARATA di s140-pair.sh (manifest
s142-pair-copia.diff), adattamenti: pin s142, nomi s142/pair142/wp142-harness,
**fix del header stantio «s139…pin s138» (chiude l'apertura “fix echo
s140-pair.sh”; il rif del confronto resta quello del criterio qui sotto)**,
inserzione peak-bisezione p.5.

1. Config: 6 gambe TUTTE ON (canone S-139); warm-up on-config; emende S-136
   (streak+retry ×3) e gate ictx/s PER MOTORE EREDITATE INVARIATE.
2. Attesa: L-RD1 rimuove ~0,8 ns/elem al teardown array; su WP (diluita I/O,
   rapporto ~1,77) attesa = **FERMO dentro banda**.
3. **Confronto formale**: proprio on-only vs rif S-140 **[1,765–1,777]** su
   **banda_ON 0,033** (TERZA finestra; banda confermata cross-finestra S-140).
   COMPATIBILE ⇒ banda resta canonica; FUORI BANDA ⇒ nessun claim dal singolo
   tentativo: replica PRIMA di ogni delta (canone S-140 p.3), e la replica è
   SENZA l'inserzione p.5 (per escludere l'inserzione come causa).
4. Banda_ON aggiornata: max-min delle coppie ON pulite di questo run (N≥5) a
   verbale; canonica post-S-142 = max-min sull'UNIONE S-139+S-140+S-142.
5. **PEAK PER POSIZIONE (bisezione osservativa, NESSUN gate)**: reperto S-140 =
   leg1 1807 MiB (bassa, dentro banda oss. 1743–1825) vs leg2–6 1838–1853
   (alte). Ipotesi in gara: (A) STATO dell'ambiente lasciato dal warm-up media
   (leg1 segue il warm-up; leg2–6 seguono una gamba full) · (B) POSIZIONE nella
   finestra (cumulo). **Bisezione: tra leg3 e leg4 si REINSERISCE il warm-up
   media bilaterale (dichiarato, MAI giudicato), stessa finestra, quiescenza
   gate invariata per ogni gamba.** Lettura pre-registrata: leg4 torna ~leg1
   (≤1825) e leg5–6 risalgono ⇒ firma STATO-post-media; leg4–6 restano alte
   (≥1830) ⇒ firma POSIZIONE/cumulo; esito misto ⇒ nessuna firma, si dichiara.
   I rapporti CPU delle 6 gambe restano giudicati dal canone p.3 (l'inserzione
   è tra le gambe e dietro gate di quiescenza; rischio dichiarato al p.3).
6. Finestra: lock misura PRESENTE (runner CI in quiet_wait); pgrep
   rust-analyzer prima del lancio; peak per gamba a verbale vs banda oss.
   1743–1825 (s136/s137) e 1807–1853 (S-140).
