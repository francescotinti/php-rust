# REPORT_GAP_59 — gap perf oracle↔phpr della sessione WP-59

> SOLO le misure della sessione WP-59; trend cumulativo in `GAP_TREND.md`.
> Sessione di MISURA (binari di parità INTATTI = phpr-wp58 2d7efdf8…):
> media CPU/footprint NON rimisurati — restano 2,61× / 4,07× (WP-58).

## Full-suite (Ob.3, tre run STESSA-SERA back-to-back, metro NUOVO:
## user CPU esatta da /usr/bin/time -l — niente troncamento campionatore)

- **new (phpr-wp58)**: user tree **790,83s**, peak footprint 3,89GB,
  fail-set **88 nomi BYTE-ID a run33** (ennesima validazione).
- **pool-off**: user tree **786,44s** = **−0,56%**; footprint 3,90GB
  (±0); fail-set 88 IDENTICI + 1 flake DB ambientale (wp_install
  Duplicate entry su test filesystem — estraneo al pool).
- **scan4**: user tree **796,05s** = **+0,66%** (più LENTO: scan-mode
  ASSOLTO — lo scan 5-8 slot batte l'indice); fail-set **88 BYTE-ID** ✓.
- Spread serata a metro esatto = ±0,6% ⇒ la banda WP-58 +1..+2,5%
  (campionatore troncato) era in buona parte rumore; unico costo
  identificato: pool ≈0,6% ⇒ ⚖️ verbale revert B (racc.: revert).
- Rapporto di giornata non ri-quotato (metro tree-user ≠ master-CPU
  storico): riferimento resta run46 **2,11×**.

## Attribuzione footprint (Ob.0+Ob.1 — la MAPPA, census media)

| componente | MB al picco | % fisico (1.436 census-run) |
|---|---|---|
| canali valore (str+arr+obj live) | ~150 | 10% |
| unit counted (canale) | ~212 | 15% |
| **compile-side NON censito (HIR seeds ~2/3, payload op ~1/3)** | **~800** | **~55%** |
| runtime engine — ⚠️ PER DIFFERENZA, non attribuito (veto Gregg V8: da strumentare in WP-60 o etichettare così) | ~100-140 | 7-10% |
| frammentazione mimalloc | 29,8 | 2% |
| mimalloc non-visitato (104,3 dentro commit; non-mimalloc misurato a win10 = 4,1MB — i "FFI ~90-120MB dirty" sono di Ob.0, run diversa: NON sommabili qui, veto Gregg V8) | ~108 | 8% |

- Identità pre-registrata verificata alla cifra (win10): phys 1436,2 =
  Σused 1298,0 + frag 29,8 + non-visitato 104,3 + nonmi 4,1.
- `--list-tests` (0 test): 818MB non-censiti già a fine
  bootstrap+discovery ⇒ il footprint è ~90% costruzione della suite.
- **Leak template-include confermato** (unit.cum_n=200 su 200 re-include
  dello stesso file) ⇒ Fase 0.5 si apre.
- **obj de-fantasmato (Ob.2)**: peak 56,1 → **48,7MB** (=3,4% del
  fisico 1.436); walk_recon obj 22.141==22.141 ESATTO.

## Verdetti di concilio

- Leijen/ipotesi condivisa (frammentazione ≥400MB): **FALSIFICATA** (2%).
- Gregg ipotesi 1 (artefatto next_id): **CONFERMATA e fixata**.
- Stogov: ramo "standing non censito ⇒ dieta di quelle strutture" —
  la unit diet si RIDIMENSIONA in su: il canale unit vero ≈ 1,0GB
  (channel 222MB = 1/5); leve WP-60: compile-cache (leak) + seed HIR
  signature-only, da quotare ex-ante.
