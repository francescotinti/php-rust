# REPORT_GAP_64 — misure della SOLA sessione WP-64 (2026-07-27 mattina)

Binari: new = debiti WP-64 (`522e0f61…`), old = phpr-wp63
(`1666e1b4…`). Catena orchestrate64 stessa-mattina, uploads-guard
attorno, reset DB per ogni run.

## Media (coppia singola oracle↔phpr, spread G3 dichiarato)

- CPU: oracle 20,92u / phpr 53,85u = **2,57×** (riferimento 2,58×
  replicato alla cifra).
- Footprint (peak phys time -l): oracle 393,7MB / phpr 1.186,9MB =
  **3,01×** — DENTRO la banda provvisoria 2,9-3,2× (KG63-A); seconda
  coppia utile verso il protocollo ≥3+3 (cifra puntuale ancora vietata).

## Full (coppia run51 stessa-mattina, new vs old)

- Fail-set: **88 nomi BYTE-ID ×2 = run33** (normalizzazione
  `s/^\d+\) //`).
- Master-CPU: 788,19u (new) vs 800,07u (old) = −1,5% — dentro lo
  spread; NON si cita come guadagno (i debiti sono arm freddi).
- Peak fisico: 2,035GB (new) vs 2,049GB (old) = −0,7% (rumore).
- Riferimenti invariati: full CPU 2,06-2,11×; peak full ~2,0-2,1GB.

## Compile-side (census64-full, memgc64 — quota tranche 2)

- net_tot master **499,9MB** = replica WP-63 alla cifra.
- Canali O(seed) ritenuti: slotnames_tot 51,6MB + fnvec_tot 36,0MB
  (+ fnshare 12,0MB) = quota leva WP-65 ~90-100MB counted.
- CPU O(seed): map 1,66s + remap 1,15s = **0,36% del full** (canale
  morto; B4 sample su parità concorde ≈0,08%).
- Δcoda riconciliata: 639,2MB vs ~623 attesi (+2,6%, ±15%) ⇒
  **−74,7% SPIEGATO**.
