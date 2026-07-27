# REPORT_GAP_65 — misure della SOLA sessione WP-65 (2026-07-27 pomeriggio)

Binario nuovo = leva slot_names `04d559b` (release 778f8ead…);
old di coppia = stash phpr-wp64 (522e0f61…) e, per l'attribuzione,
igiene-only phpr-hyg65 (b93a6e91…, commit 6619b7a).

## Media (gruppo --group media, coppia stessa-sera, orchestrate65)

| metro | oracle | phpr (leva) | rapporto |
|---|---|---|---|
| user CPU | 20,89u | 53,77u | **2,57×** (replica del riferimento 2,58×) |
| peak footprint | 382,2MB | 1.150,6MB | **3,01×** |

Terza coppia G3: {WP-63 2,98×, WP-64 3,01×, WP-65 3,01×} — mediana
3,01×, tutte DENTRO la banda 2,9-3,2× ⇒ KG63-A ha le sue ≥3 coppie
phpr (ratifica della banda consolidata ~3,0× rimessa al concilio).

## Full-suite (30.472 test, wpdev, tre coppie stessa-sera)

- **Fail-set: 88 BYTE-ID ×2 in TUTTE e tre le coppie (run52, run53,
  run54) = run33.**
- CPU: run52 new 808,51u / old(wp64) 791,44u (+2,16%); run53 (ordine
  invertito) 802,66 / 792,91 (+1,23%); **run54 (build adiacenti:
  igiene-only vs leva) 804,23 / 802,52 = −0,21% ⇒ il costo CPU della
  LEVA è zero; il +1,2-2,2% vs stash è spread build-vs-stash** (classe
  run51 −1,5%, segno opposto — non si cita). **Riferimento full CPU
  resta 2,06-2,11×.**
- Peak fisico: run54 (coppia pulita) 2.028,7→1.976,9MB = **−51,8MB ≈
  counted ×1,00**; vs stash: −86,3 (run52) / −15,8 (run53) = rumore
  ±35MB attorno alla stessa media. **Peak full ora ~1,98-2,03GB**
  (da ~2,0-2,1GB WP-64).
- Compile-side counted (census65/65b, memgc a parità di workload):
  net_tot **499,9 → 448,3MB (−51,63MB, −10,3%)**; cumulato da WP-62:
  1.973,3 → 448,3 = **−77,3%**. slotnames_tot 51,6MB → **0**.
