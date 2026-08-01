# SUPPLEMENT_NORM.md — A-SK29 (Council WP-84): supplementi di campagna NORMATI

Il supplemento fase-R di S-82.0 (phaseR-supplement.sh) era legittimo nel
merito e "precedente pericoloso" nella forma (Klabnik Q2): non replicava le
precondizioni di campagna, cambiava ARM vendendolo come re-run, e nulla
limitava i re-run finché la cifra non piace. Da qui in poi un supplemento è
legale SOLO dentro questa norma; fuori norma ⇒ **cifra VOID, quarantena mai
rm** (KS-SK-84-2).

## Lista CHIUSA dei tipi ammessi

1. **strumento-non-armato**: la fase è corsa con lo strumento compilato ma
   non armato (feature ≠ env ≠ allocatore, lezione ⭐⭐ S-82.0). Il re-run
   arma SOLO l'interruttore mancante; stesso binario per hash (ENFORCE),
   stesso arm, stesse fixture.
2. **analisi-only**: ri-lettura di raw GIÀ committati con uno script nuovo
   (nessuna run nuova). Nessuna precondizione di campagna, ma lo script e
   l'output vanno committati insieme (KG-83-3).

Ogni altro tipo (cambio d'arm, cambio di fixture, cambio di N/R, "il numero
non mi piace") NON è un supplemento: è una CAMPAGNA nuova, con battery,
matrix ULTIMO e precondizioni piene.

## Precondizioni (tipo 1)

- Le stesse della campagna madre, RICONTROLLATE dallo script del
  supplemento: battery/.done a rev pinnata (o equivalenza LEGALE via
  battery-equivalence.sh), tree/harness porcelain pulito, driver_sha.
- Hash del binario di fase == hash registrato dalla campagna madre
  (ENFORCE, mai "compatibile").
- Full-body vs oracolo PASS PRIMA della cifra (KS-AH-83-1) dove la fase
  madre lo prevedeva.

## Ledger (obbligatorio, per campagna)

File: `wp<N>-harness/evidence/supplements.ledger`, una riga per supplemento:

    campaign=<script> phase=<X> type=<1|2> reason=<...> bin=<hash> date=<ISO>

- **Max 1 supplemento per fase per campagna.** Il secondo tentativo sulla
  stessa fase ⇒ fase VOID d'ufficio: si riapre come campagna nuova.
- La run sostituita va in quarantena con manifest (mai rm, KS-AH-83-2) e il
  contatore void della campagna si aggiorna PER-MANIFEST (A-BG29).

## Vincoli di onestà

- Il supplemento non può MAI cambiare l'arm dichiarato della cifra: se la
  fase madre era CLI-server, la cifra resta "per-ARM CLI-server" (A-PP25).
- L'header del documento MEASURE che cita il supplemento nomina il ledger
  e il manifest della run sostituita (KG-84-1: ogni numero ricomputabile).
