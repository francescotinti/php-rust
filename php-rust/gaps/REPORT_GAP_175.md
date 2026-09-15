# REPORT_GAP_175 — gap perf oracle↔phpr al pin s175 (SOLO sessione S-175, 2026-09-15 notte)

Pin: phpr 5de14d6856d760a8 + server 9b9179d4dd3d95bd (promozione braccio C «Sweep-in-op», wp174-harness/s174-promo-verdetto.out; identità a contenuto col candidato b6c4b5876971d76d: 48 B in 2 cluster LC_UUID/firma).

## Micro (run-micro.sh R=5, pavimenti sottratti per binario) — rapporto phpr/oracle
| categoria | s173 | **s175** | Δ |
|---|---|---|---|
| arith | 2,8 | **2,3** | −0,5 (leva: Sweep inerte scavalcato dopo BinarySCSCDst/BinarySTDst + back-edge fuso) |
| prop | 3,0 | **2,6** | −0,4 (leva: 2 Sweep/iter fusi ai siti P3/P1-P4) |
| calls | 4,8 | 4,6 | −0,2 (for canonici nei driver: back-edge fuso) |
| str | 4,1 | 4,0 | −0,1 (tick) |
| arr | 3,1 | 3,1 | = |
| re | 2,5 | 2,5 | = |
⚠ Igiene (incidente #1 S-175): micro e conferma post-pin misurate 02:50-02:55 con l'updater di un'app utente attivo (ShipIt, ucciso 02:51 e 02:53) e Data a 0,5-2G: non registrato nel .out ⇒ i rapporti micro sopra sono a GUARDIA (direzione coerente con l'A/B S-174) e vanno RIMISURATI in S-176 in finestra pulita (R=5, ~5 min).

## Giudici propri della leva (ns/iter)
- Conferma post-pin arith-dq (pin s175 vs stash s173, R=5): 23,76 → 20,20, D=+3,56, rumore 0,16, segni 5/5 = SOLO SEGNO (az.rev. S-174 (c): D < pavimento 4); coerente con l'A/B S-174 (+3,64/+3,56 e +3,52/+3,48).
- Mock (finestra pulita 05:59-06:04, wp174-harness/s175-mock-verdetto.out): A = sorgente del pin 20,08 (arith-dq, oracle 8,64 ⇒ **2,32×**) · 36,13 (prop-dq, oracle 14,00 ⇒ **2,58×**); Z = pin s175 20,32/36,07 (controllo nullo |A−Z| 0,24/0,07).
- Tetto del predicato Sweep (B = MS, predicato mai letto ai siti fusi): arith −2,36 (direzione, <4) · **prop −4,67 = CIFRA** (entrambi gli stimatori; rumore ≤0,93); 3ª clausola sola (C = M3): −0,80 / −1,93.

## Suite (coppia t20 al pin s175, DOVUTA)
- WP full/media t20: mediana **1,765** in banda [1,738;1,799] = COMPATIBILE (t19 1,772: direzione ≤0, nessun claim); 6/6 gambe pulite, banda_ON 0,026, nessuna deriva (s175-pair-verdetto-t20.out rc=0).
- ORM (tentativo 3 rc=0; t1/t2 rc=8 quiescenza per flare mediaanalysisd): phpr net [34,47;34,51] vs rif [34,27;34,35] ⇒ Δ [−0,24;−0,12] s nel rumore ±0,293; rapporto net **[6,936;7,014]** (t19 [6,988;7,012]; resta ≤7,05); **sentinella oracle 4,97 FUORI banda [4,83;4,94]** ⇒ Delta_norm [+0,01;+0,47] NON giudicante (E2); parità ORM 16 nomi · dbal 10 stabile; ictx oracle1 segnalata a verbale.

## Residuo nominato (direzione; le quote per-op sono IPOTESI da mock/mutanti, non cifre di leva)
arith-dq 20,08 vs 8,64: residuo 11,4 ns/iter = predicato Sweep 2,36 (mock B: tetto; 3ª clausola 0,80) + dispatch di 3 op/iter (CmpJmpSC fuso nel back-edge, BinarySCSCDst, IncDecSlotJmp: ~5,25 a 1,75/op) + corpi (~3,8). prop-dq 36,13 vs 14,00: residuo 22,1 = predicato ×2 = 4,67 (cifra) + dispatch 6 op/iter (~10,5) + corpi prop (~7). Prossima leva: flag «gc idle» (1 load al posto di 3 load + somma + 2 confronti; tetto 2,36/4,67) con criterio proprio su prop-dq a nomina; portata reale sui workload da census «op in place + Sweep» (WP/ORM).
