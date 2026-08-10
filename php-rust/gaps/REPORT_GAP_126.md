# REPORT_GAP_126 — SOLO S-126 (2026-08-10). Sessione di MAPPA+ISTRUTTORIA sul pin s125 002e6cc1: nessuna rimisura della coppia WP vs oracle (riferimento resta 1,815–1,896 @ s124); misurati 4 workload nuovi + micro-ORM + A/B off-patch s123↔s124 (notturno).

## Cifre mappa2 (verdetto: wp126-harness/s126-mappa2-verdetto.out; N=2 per lato, user CPU, pavimenti per-binario, oracle memory_limit=-1)
| workload | ratio raw (leg1/leg2) | validità |
|---|---|---|
| doctrine/dbal (3929, sqlite) | **8,291 / 8,325** | CANONICA (fail-set stabile 10 nomi = 0,25% ≤1%, §3.20; ictx phpr alto ma gambe concordi 0,4%) |
| symfony/http-foundation (1854) | **2,554 / 2,565** | CANONICA (diff 17 nomi = famiglia `php -S`/session-server, 0,92% ≤1%) |
| doctrine/collections (242) | 6,200 / 6,200 | INDICATIVA (oracle netto 0,09 s: denominatore sotto-scala) |
| composer install OFFLINE | phpr NULLA run1 | phar/`__halt_compiler` (§3.19); rimisura con composer estratto → s126-compoff-verdetto.out |

## Cifre istruttoria ORM (verdetti: s126-orm-micro{,2}-verdetto.out, R=5 netto-pavimento)
evalcls **316,9** (2,38 ms/classe vs 7,5 µs) · refl **42,4** · objchurn **10,3**
= objalloc **9,9** (1220 vs 123,3 ns; ~67%) + objmap 17,3 (~10%) + residuo.
Profilo ORM phpr (indizio unilaterale): churn multi-%, compile ≤~1% leaf, reflection <0,5%.

## Lettura
- **La mappa ora ha 8 righe e una forma**: WP 1,85 ≪ hf 2,6 ≪ hk 4,3 ≪ dbal 8,3 ≈ ORM 8,5.
  dbal (mock-leggera) conferma che il driver dell'8,5× è il LAVORO-OGGETTI, non il compile dei mock.
- **LEVA NOMINATA L-OL1** (ciclo-di-vita oggetto) con criterio A/B pre-scritto → s126-leva-nominata.md.
- Aperture per NOME: evalcls 316,9× (cliff compile-per-classe; strumento di densità prima della leva) · refl 42,4×.
- A/B off-patch s123↔s124 (az. rev. S-125): verdetto in s126-aboff-verdetto.out (appendice sotto).

## Appendice notturna (verdetti: s126-aboff-verdetto.out; compoff2 abortita)
- **aboff s123-p0b↔s124, 4 gambe intercalate stessa notte, gate contesa**: cpu full
  A 823,32/809,28 (med 816,30, spread 1,72%) · B 820,39/790,02 (med 805,20, spread 3,77%);
  ictx 103–195k, mediana 140k, **nessuna gamba nulla**; fail-set stabili A e B.
  **Δ B−A = −1,36%** (segno atteso) < soglia 3,77% ⇒ **NON RISOLVIBILE anche same-night —
  VOCE CHIUSA per criterio p.5** (l'effetto full di PhpStr resta «≤~2% non risolvibile»,
  senza ulteriori notti; deriva monotona della notte assorbita dall'intercalata).
- **compoff2**: abortita dal SUO smoke (rc=8): anche composer 2.10 ESTRATTO muore su phpr
  **rc=255 silente** (§3.19 aggravata). Nessuna cifra; bisezione a S-127.
