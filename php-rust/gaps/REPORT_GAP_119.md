# REPORT_GAP_119 — SOLO S-119 (2026-08-09). Coppia WP sul pin s119 350582e5 + server s119 b7bd6744 (ricetta A′), pair109 ×2 modi stessa sera, rc=0 entrambi.

## Cifre (raw: wp109-harness/pair-out-{off,on}/, ratio .out committati)
| voce | OFF | ON |
|---|---|---|
| full master CPU (oracle → phpr) | 449,63 → 826,16 · **1,837** | 481,97 → 827,07 · **1,716** |
| media user CPU | 21,18 → 53,35 · **2,519** | 21,33 → 56,23 · **2,636** |
| full peak footprint | 2092,5 MiB phpr · 2,874 | **1933,6 MiB** phpr · 2,619 |
| parità | full == solo wp_is_stream; media 0 fail | idem |

## Lettura (direzione-solo: serie A′ su pin s119 = N=1)
- **full scende in ENTRAMBI i modi** rispetto al riferimento s118 (1,913/1,955 →
  1,716/1,837). Cautela dichiarata: l'oracle si è mosso ~7% TRA LE GAMBE stanotte
  (449,63 off vs 481,97 on a phpr quasi fermo 826/827) — la magnitudine del calo
  non è ripartibile tra leva (V3-V5 mordono MethodCall su WP reale, direzione
  attesa dal manifest) e rumore oracle tra-sere. Niente ranking storici.
- media ON 2,636: sopra la lettura s118 (2,506) — dentro il rumore tra-sere della
  voce (aperta da S-113); direzione non firmata.
- peak ON 1933,6 MiB vs 1839,2 (s118): +94 MiB, serie N=1, voce da riosservare.

## Metodo
Riferimento nuovo della famiglia = QUESTA coppia (pin s119+server s119); il
riferimento s118 (pin s117+server s118) decade come pre-registrato dalla regola
«il riferimento segue il pin». Prossima coppia: stesso pin ⇒ N=2, prime bande.
