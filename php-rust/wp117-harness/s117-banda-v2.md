# BANDA v2 — misurata su pipeline A′ (phpr-s117-aprime 8135dcf8), N=2 zavorre RICOSTRUITE (s114 + s115-2), committata PRIMA del primo A/B di leva (R2 concilio)

| cat | banda vecchia (N=2, pre-A′) | nulla-1 | nulla-2 | **banda_new = max** |
|---|---|---|---|---|
| arith | 0,40 | 0,40 | 0,80 | **0,80** |
| prop | 4,33 | 3,33 | 0,67 | **3,33** |
| calls | 5,50 | 0,50 | 0,50 | **0,50** |
| str | 5,00 | 7,50 | 0,00 | **7,50** |
| arr | 6,67 | 6,67 | 1,67 | **6,67** |
| re | 10,00 | 0,00 | 0,00 | **0,00** |

Globale: **7,50** (vecchia 10,00). Held-out banda N=2 (|Dnet| max): poly **0,01** · err **0,06** · wploop **0,08** s.

DICHIARAZIONI (senza deroghe):
- **«Metro riparato» NON passa la soglia pre-registrata** (max 7,50 > 5): A′ resta giudicabile come sola leva velocità (distinzione R2-Gregg, due claim mai fusi). KS-A comunque NON scatta (uplift mediano +2,17% ≥ 2%, congiunzione).
- Il metro è riparato PER CATEGORIA dove serviva: calls 5,50→0,50 · re 10,00→0,00 · prop 4,33→3,33. str (7,50) e arr (6,67) restano larghi: guardie str/arr poco diagnostiche, dichiarato nei verdetti che le usano.
- Quanto del cronometro: 0,01 s/N — su calls (N=2·10⁷) = 0,50 ns/iter = banda_new(calls): ogni guardia usa anche il pavimento 2×quanto (emendamento (f), az.3 rev. S-116).
- Admission zavorre anti-forgia LTO: dump ON {main} ×6 e OFF al byte identici ad A′; run_loop +3.644 B (nulla-1) / +768 B (nulla-2): la zavorra NON è svanita sotto fat-LTO.
