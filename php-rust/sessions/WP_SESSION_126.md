# WP_SESSION_126 — istruttoria ORM: leva L-OL1 nominata · mappa v2 (dbal conferma 8,3) · aboff same-night

**In una frase**: cercando il perché phpr è 8,5× più lento sulle suite a oggetti,
l'istruttoria ha prezzato i tre indiziati: il costo dominante è il ciclo di vita degli
oggetti (crearne+distruggerne uno costa ~10× l'oracle) — due precipizi enormi ma rari
(eval di classi 317×, reflection 42×) restano aperture; DBAL (8,3×) conferma il quadro
e l'A/B notturno chiude la questione PhpStr sul carico WP.

**SCOREBOARD** (pin **s125 002e6cc1** INVARIATO; micro NON rimisurate = S-125):
**arith 5,5 = · prop 5,6 = · calls 4,7 = · str 4,2 = · arr 3,2 = · re 2,6 =** · rif WP
full = 1,815–1,896 (S-125 @ s124) · **leve spedite: 0 — ANOMALIA DICHIARATA** (istruttoria/
mappa per mandato §S-126; L-OL1 nominata, esecuzione S-127). 2026-08-10 · Fable 5.
## Esiti secchi
1·**p.1 istruttoria ORM** (criterio b8fcb4a PRIMA; R=5 netto-pavimento): evalcls **316,9**
(2,38 ms/classe eval'd vs 7,5 µs) · refl **42,4** · objchurn **10,3**; profilo ORM (indizio
unilaterale): churn multi-%, compile ≤~1% leaf ⇒ regola p.6: **NOMINA objchurn**; emenda p.8
+ decomposizione: **objalloc 9,9 = ~67% del churn (1220 vs 123,3 ns/oggetto)** · objmap 17,3.
⇒ **L-OL1 «ciclo-di-vita oggetto»** nominata con criterio A/B (s126-leva-nominata.md).
2·**p.2 mappa v2** (criterio bd74d79 PRIMA; workspace congelati, smoke bilaterale):
**dbal 8,291/8,325 CANONICA** (10 nomi = 0,25%, §3.20) · **hf 2,554/2,565 CANONICA**
(17 nomi = famiglia php -S) · coll 6,200 INDICATIVA (denominatore 0,09 s) · **compoff
phpr NULLA** (phar §3.19); rimisura con ESTRATTO 2.10 ABORTITA dal suo smoke:
**rc=255 silente** (§3.19 aggravata; bisezione S-127).
3·**p.3 aboff s123↔s124 same-night** (criterio bd74d79; 4 gambe intercalate, gate contesa
ictx): 4/4 valide (ictx med 140k, nessuna nulla; fail-set stabili); **Δ B−A = −1,36%**
(segno atteso) < soglia 3,77% (spread intra-B) ⇒ **NON RISOLVIBILE anche same-night;
VOCE CHIUSA per criterio p.5** — niente altre notti su PhpStr-full.
4·PERF_MAP v2: WP 1,85 ≪ hf 2,6 ≪ hk 4,3 ≪ dbal 8,3 ≈ ORM 8,5 — il gap cresce col
lavoro-oggetti; dbal (mock-leggera) scagiona il compile come driver.
5·Incidente apparato (1): verdetto micro IN SCRITTURA dentro il commit bd74d79 —
ricommittato a run concluso. 3 iterazioni forgia rumorose sanate PRIMA di ogni misura.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Un cliff enorme può pesare poco**: evalcls 316,9× ma ≤~1% del run reale — la nomina
  va alla categoria che il profilo PESA, non al rapporto più spettacolare (regola p.6 scritta
  PRIMA ha deciso al posto dell'entusiasmo).
- ⭐⭐ **Una suite gemella mock-leggera è un esperimento di controllo gratuito**: dbal 8,3 ≈
  ORM 8,5 scagiona mock-eval e reflection come driver del gap senza strumentazione.
- ⭐ **La validità di una gamba è un predicato meccanico pre-registrato**: compoff morta
  a t=0 ⇒ il verdetto l'ha dichiarata NULLA da solo (fail-set stabile + 1% + vendor_ok).
