# S-140 — criterio COPPIA WP @ pin s140 (pre-registrato PRIMA del run)

Obbligo: pin NUOVO s140 (phpr f2708b75 + server c7a03e2a) ⇒ coppia dovuta
(regola utente 2026-08-12). Harness: `s140-pair.sh` = COPIA DICHIARATA di
s139-pair.sh (manifest s140-pair-copia.diff), SOLI adattamenti: pin s140,
nomi s140/pair140/wp140-harness, giudice p.3-p.4 qui sotto.

1. Config: 6 gambe TUTTE ON (canone S-139); warm-up on-config; emende
   S-136 (streak+retry ×3) e gate ictx/s per motore EREDITATE INVARIATE.
2. Attesa: la leva HC1 rimuove ~1-2 ns/check tipizzato (canale ~0,1% su
   suite typed-dense) ⇒ su WP attesa = FERMO dentro banda.
3. **Confronto formale**: proprio on-only vs rif S-139 [1,752–1,785] su
   **banda_ON 0,033 — PRIMO uso cross-finestra** (az.rev. S-139 #1): questa
   coppia è la SECONDA finestra; esito COMPATIBILE ⇒ banda CONFERMATA
   cross-finestra e resta canonica; esito FUORI BANDA ⇒ NON dichiara delta
   da solo (banda intra-finestra): tentativo ripetuto prima di ogni claim,
   e se persiste la banda si RIFONDA come max-min cross-finestra.
4. Banda_ON aggiornata: max-min delle coppie proprie ON pulite di QUESTO
   run (N≥5) riportata a verbale; la banda canonica post-S-140 = max-min
   sull'UNIONE delle finestre S-139+S-140 (dichiarata nel verdetto).
5. Peak per gamba riportati (obiettivo §S-140 p.2): confronto OSSERVATIVO
   con 1831–1849 (@s138) e banda oss. s136/s137 1743–1825 — nessun gate.
6. Finestra: lock sessione presente (verify-only); niente push a finestra
   aperta salvo atti di pin (runner CI in quiet_wait col lock: contesa
   esclusa per costruzione); pgrep rust-analyzer prima del lancio.
