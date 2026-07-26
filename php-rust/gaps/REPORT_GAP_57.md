# REPORT_GAP_57 — gap perf oracle↔phpr della sessione WP-57

> SOLO le misure della sessione WP-57; trend cumulativo in `GAP_TREND.md`.
> Sessione di QUOTA: **zero modifiche ai binari di parità** (release
> verificato = stash `phpr-wp56` sha256 65466c64…) ⇒ media CPU/footprint e
> full-suite NON rimisurati — restano validi i riferimenti WP-56
> (media 2,61× · footprint 4,08× · full run45 2,06×).

## Misure della sessione (quota, non gap)

- **Ob.1 frequenza `.=` non-locali sul FULL** (prima run census di sempre
  sul full, phpr-op57, 30.472 test): ArrElem 30.633 ev / 0,34MB lhs ·
  Prop 8.338 / 0,20MB · Field 9.963 / 0,16MB · StaticProp/Dyn/ArrayAccess
  0 ev → **TOTALE 48.934 eventi, Σ lhs 0,70MB ≈ 23µs alla banda calibrata
  30GB/s** (probe56: 12,8GB/0,43s); media group: 221 ev / 2,5KB. Nessun
  evento con lhs >1KB. **Canale morto: l'estensione del fuso (Ob.1c) non
  si apre** — il gap full resta attribuito a corpi+dispatch 41,9% ·
  gc-walk 10% · crypt onesto (WP-54), con lo str-copy residuo ora
  ricondotto alle copie generiche, non ai siti compound non-locali.
- **Ob.2 metro non-biased canale arr** (phpr-memgc57, live-accounting
  esatto validato: riconciliazione reached==live alla unità, 63.435):
  **arr peak ESATTO 66,4MB = ~4,3% del peak fisico 1.536MB** (il "arr ~39%
  del proxy" del checkpoint WP-55 era l'artefatto dell'estimatore, 6,0×
  over alla cifra); str peak 62,3MB (~4,1%); unit standing 222,6MB; obj
  live EOR ~17,8MB (ultimo canale death-accounted). Istogramma per-repr
  (EOR master): packed 36,7k arr / 4,2MB (dominante 1-2 elementi), hashed
  26,7k / 9,2MB (21,4k da 1-4 el); overhead fisso 64B/arr ≈ 30% del canale.
- **Ri-quota tranche 2 Fase 3**: arena handle-based ≈ **−10..−20MB peak
  (~−1% fisico)**; exact-fit hashed piccoli ≈ −1..−2MB. La leva GRANDE di
  footprint residua è FUORI dal canale arr (fisico ~1,15GB oltre i canali
  valore + unit 222,6MB).

## Stato riferimenti (invariati da WP-56)

- media CPU 54,665/20,945 = **2,61×** · footprint 1,536/0,376G = **4,08×**
  · full run45 697,8/339 = **2,06×** · fail-set byte-id a run33 (88 nomi).
