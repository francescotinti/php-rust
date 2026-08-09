# WP_SESSION_123 — METRO sanato · L-ST1/L-RE2 refutazioni DEFINITIVE · classifica-v2 FUSA · perimetro PhpStr

**In una frase**: riparato il metro di misura (ordine permutato, pavimento
pin-vs-pin, timer fine, run lunghi), le due leve sospese sono state bocciate in
via DEFINITIVA con margine vero, e il censimento rifatto sul sentiero release
vero indica la prossima leva strutturale: stringhe a UNA sola allocazione.

**SCOREBOARD** (pin s120 885d2c64 INVARIATO — micro vs oracle non rimisurate; vs S-122):
**arith 5,5 = · prop 5,5 = · calls 4,8 = · str 5,3 = · arr 3,7 = · re 2,8 =**
· rif WP **full = 1,810–1,889** (non rimisurato) · **leve perf spedite: 0**
(2 A/B a verdetto + metro: ritmo rispettato; sessioni-senza-Δ = 3).
2026-08-09 · Fable 5 · a9ef293→4498f83.

## Esiti secchi
1·**BANDA_LAYOUT v2** (criterio 767f185 PRIMA; K=5 con P0b copia-pin, quadrato
latino 5×5, timer getrusage-µs, N scalati ≥5 s, R=5): **arith 0,94 · prop 0,80 ·
calls 0,73 · str 2,89 · arr 2,49 · re 4,46** ns/iter; PAV_PIN 0,01–1,36;
posizioni NON monotone; predizioni p.8 **3/3** ⇒ il confondente S-122 era
l'ordine, il layout è REALE ma più piccolo del provvisorio.
2·**L-ST1 REFUTAZIONE ACQUISITA** (A/B alternato R=5, giudice simmetrico con
2×spread_A): str D_med **−5,08**, segni 0/5, soglia_ref −3,50 — margine VERO
(S-122 era a margine zero). Candidato resta reperto s121-st1, leva morta.
3·**L-RE2 REFUTATA e ARCHIVIATA** (smoke R=6 alternato): re D_med **−15,74**,
0/6, soglia ±10,68 — l'anti-tesi mosse Caps 176 B vince; 4ª caduta alloc-removal
sul costo sostitutivo. Reperto s122-re2.
4·**Classifica-v2 FUSA** (build SOLO mem-census, fuso VIVO — spento solo da
zval/op-census run.rs:4213; admission 7/7, R=2 deterministico): Δalloc/iter vs
oracle **re +5,00 · str +3,00 · arr +2,05 · resto 0,00**; prop zvclone **5→3**
(effetto-fusione puro; predizione ~0-1 di GRADO mancata, residui in `$s+=$o->x`);
**arr 4,08 ≈ 2 ZStr di chiave per lettura** ⇒ single-alloc → ~2,04 vs oracle 2,03.
NOTA anti-misattribuzione: v1 re 17→10 = L-RE1 (raw S-119 pre-L-RE1), non fusione.
5·**Perimetro PhpStr single-alloc** (istruttoria s123): 7 siti Rc-API (1 solo
load-bearing: get_mut del `.=` run.rs:433), funnel unico zstr.rs:54, 3 rischi non
testuali (RcEqIdent chiavi, hash in Cell, !Clone). PGO RINVIATO dichiarato.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Sanare il metro non ribalta i verdetti: li rende DICIBILI** — permutazione
  e pin-vs-pin confermano entrambe le refutazioni con soglie legittime.
- ⭐⭐ **Un census vale solo sul sentiero che il release percorre** (prop 5→3 cloni/iter fuso vs spento).
- ⭐ **Raw di census di epoche diverse vanno DATATI**: re 17→10 v1→v2 = L-RE1, non lo strumento.
