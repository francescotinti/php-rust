# REPORT_GAP_102 — S-102 (2026-08-06, SOLO questa sessione)

Binario phpr **d0b01362433b3039** @ HEAD b6f8098 (is_gc_container + fix
§3.13 + denti; default flag-on); oracle brew 8.5.7. Strumenti:
run-micro.sh (R=5, mediane, netto pavimento); pair102.sh
(`/usr/bin/time -l`, guardia uploads, reset DB, oracle prima);
s102-peak-noise.sh (R=5, banda rumore).

## Giudice (sei micro-categorie, modo DEFAULT, R=5, spread ≤0,10)

| categoria | S-101 chiusura | S-102 | lettura |
|---|---|---|---|
| proprietà | 11,5 | **11,5** | INVARIATO — is_gc_container perf-neutro sul guard caldo |
| aritmetica | 12,2 | 12,3 | tra-sere |
| chiamate | 7,3 | 7,7 | tra-sere (+0,4; spread 0,07) |
| stringhe | 7,0 | 6,6 | tra-sere (−0,4) |
| array | 4,3 | 4,6 | tra-sere |
| regex | 3,5 | 3,6 | tra-sere |

Nessuna leva perf spedita in S-102: il ri-baseline è di PARITÀ (movimenti
±0,4 bidirezionali, finestra pomeridiana vs notturna di S-101).

## Coppia WordPress bimodale (pair102, stessa sera)

- media CPU: off 2,640 · on 2,641 (S-101: 2,589/2,604 — banda tra-sere)
- full CPU: off **1,891** · on **1,894** (S-100 1,873 · S-101 1,890/1,894)
- full peak: off 1989,88 MiB · on 1942,05 MiB (riferimento)
- Parità: media 0 failnames ×2; full = SOLO delta pre-esistente
  `wp_is_stream`, IDENTICO nei 2 modi.

## Peak — la novità di metodo

- **Banda rumore full-peak PHPR (prima misura)**: R=5 pin S-100 off —
  mediana 1896,91 MiB, spread **34,64 MiB (~1,8%)**; l'oracle balla ~10%.
  La gamba phpr È bisecabile sul peak (spread < 48, KS-LE-103-3).
- **A/B pin S-99↔S-100 (off/off, ABAB R=5) IN VOLO** a fine sessione:
  verdetto meccanico dalla regola pre-registrata in
  `wp102-harness/s102-peak-criterio.out` → `peak-ab-out/ab-verdetto.out`.

## Corpus

**1418 → 1417 per NOME nei 2 modi**: il fix §3.13 (riga del warning
all'accodamento) fa passare `nullsafe_operator/015.phpt` (byte-id oracle
verificato nei 2 modi); riferimento canonico aggiornato con la miglioria
documentata. Diff per-test off↔on: ZERO.
