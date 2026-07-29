# REPORT_GAP_72 — misure della SOLA sessione WP-72 (2026-07-29)

Sessione fedeltà+leva-memoria: il gap classico media (CPU+footprint
vs oracle) NON è stato rimisurato. Misurato: coppia full stessa-sera
+ residuo per-request (il fronte della roadmap).

## Coppia full stessa-sera (C-72, run72-pair.sh, metro user+sys del wrapper time -l)

| Lato | Binario | master-CPU | Esito suite |
|---|---|---|---|
| old | phpr-wp71 (38af5eaa) | 821,4s | 30.472 test, 2F/86W/73S |
| new | phpr-wp72 (4fd7b2d5) | 818,4s | 30.472 test, 2F/86W/73S |

**Delta −0,36% ≤ +1% (cap lockato) = la leva mass-teardown è
CPU-NEUTRA.** Fail-set 88 nomi IDENTICO old-vs-new e == baseline
run33. ⚠️ Il metro di questa coppia (user+sys del wrapper) NON è
confrontabile alla cifra col 697,8s storico di run45 (metro/ambiente
diversi): il rapporto full-vs-oracle NON viene aggiornato da questa
coppia — riferimento resta 2,06-2,11× fino a una coppia
phpr-vs-oracle stesso-metro.

## Residuo per-request (il fronte della sessione) — CHIUSO

| Metro | pre-fix (WP-70/71) | post-fix (WP-72) |
|---|---|---|
| tripla used_n | 20,000 obj/req (spread 0,003) | **0,000 (spread 0,000)** |
| tripla used_b | 2,1109 KiB/req | **0,0010** |
| amp K=10 | +80,000 obj/req | **+0,005** |
| ladder release (4000 req) | +7,7 / +8,3 MiB | **−2,0 / −12,6 MiB** |
| two-boot esistenza (P70-0-bis) | Δ5,8 MiB/4000 | superato dal ladder |

Contatori leva: broken/req=21 costante, reg/req=680, busy=0,
cellpark/drainfails=0. C4 (160|192) CHIUSA a −0,000.

## Riferimenti invariati (non rimisurati)

media CPU 2,58× · footprint media ~3,0-3,1 (banda 2,9-3,2) · peak
FULL ~1,98-2,03GB · full CPU 2,06-2,11× · server hit_cross steady
(ora 505/505 per-req, 7 unit-load in meno dai fix di fedeltà).
