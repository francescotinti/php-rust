# BANDA TRA-SERE del giudice (A-GR-104-1) — protocollo + registro punti

**Mandato** (Gregg, Concilio WP-104): la banda tra-sere del giudice va
NOMINATA come NUMERO da ≥3 sere — finché non esiste, ogni Δ tra sere
diverse è INDIZIO (KS-GR-104-1: mai narrato come effetto-leva). Può
chiudersi su sere successive.

## Protocollo (fissato PRIMA dei numeri)

- Strumento: `wp97-harness/micro/run-micro.sh` (R=5, mediana+spread,
  netto pavimenti — feedback-one-sided-profile), modo DEFAULT flag-on.
- **Stesso pin** su tutte le sere (un binario diverso = serie diversa).
- Sera valida solo con spread phpr ≤ 0,10 s su ogni categoria (regola
  KS-GR-102-2: banda<rumore ⇒ VOID, come s101-r1).
- Banda per categoria = max−min del RAPPORTO sulle ≥3 sere valide;
  banda del giudice = il vettore delle sei (mai un aggregato: un solo
  numero DILUIREBBE, feedback-one-sided-profile).
- Macchina quieta; misura MAI concorrente a build o run pesanti.

## Registro punti STESSO-PIN (serie d0b01362433b3039)

| sera | finestra | arith | prop | calls | str | arr | re | fonte |
|---|---|---|---|---|---|---|---|---|
| 1 | 2026-08-06 mattina (S-102) | 12,3 | 11,5 | 7,7 | 6,6 | 4,6 | 3,6 | `wp102-harness/micro-baseline-s102.out` |
| 2 | 2026-08-06 pomeriggio (S-103, 16:1x) | 12,3 | 11,5 | 7,4 | 6,6 | 4,4 | 3,7 | `wp103-harness/micro-baseline-s103.out` (spread phpr ≤0,06 su tutte: sera VALIDA) |
| 3 | sera successiva | — | — | — | — | — | — | — |

**BANDA: NON ANCORA NOMINABILE** (2 punti su ≥3). Delta stesso-pin
sera1→sera2: arith 0,0 · prop 0,0 · calls −0,3 · str 0,0 · arr −0,2 ·
re +0,1 — coerenti con l'indizio cross-binario (±0,2-0,4). Il terzo
punto chiude la banda (sera successiva). Fino ad allora vige
KS-GR-104-1 in forma piena.

## Indizi cross-binario (NON banda: binari diversi, leve in mezzo)

Solo per calibrare l'attesa, mai per giudicare: sulle categorie MAI
toccate da leve tra S-99→S-102 i rapporti hanno ballato re 3,8→3,5→3,6 ·
str 6,9→7,1→7,0→6,6 · arr 4,9→4,6→4,3→4,6 ⇒ ordine di grandezza atteso
della banda ~±0,2-0,4 per categoria. È un INDIZIO di taratura del
protocollo (soglie spread), non un numero citabile.
