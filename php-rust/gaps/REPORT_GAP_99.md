# REPORT_GAP_99 — S-99.0 (2026-08-05) — SOLO questa sessione

Sessione di SOLE MISURE (ordine Concilio WP-100). Coppia full+media
stessa-sera RIMISURATA (prima dopo WP-94) + ri-baseline delle sei
categorie su ENTRAMBI i motori.

## Coppia full+media (raw: `/Volumes/Extreme Pro/Claude/wp99-harness/pair-out/`, rapporti macchina `wp99-harness/pair99-ratios.out`)

| metrica | rapporto phpr/oracle | nota |
|---|---|---|
| media group, user CPU | **2,630×** | WP-94: 2,639× — piatta |
| media group, peak footprint | **2,698×** | WP-94: 3,381× — la gamba ORACLE si muove; phpr 1202,7 MB |
| full suite, master CPU | **1,893×** | WP-94: 1,873× — piatta |
| full suite, peak footprint | **2,374×** (phpr 1984497752 B = 1892,56 MiB) | WP-94: 2,673×, 1901,11 MiB — gamba phpr PIATTA (−0,45%) |

Parità: media 0 failure sui due lati; full conteggi IDENTICI
(30472/4558029), unica divergenza `test_wp_is_stream` ftp (catalogo).
Binari: phpr 4e268c3f (S-98), oracle brew 8.5.7. Nessun claim di
movimento: fotografie stessa-sera; per il movimento fanno fede i raw.

## Sei categorie, ENTRAMBI i motori stessa finestra (`wp99-harness/micro-rebaseline99.out`, pin 4e268c3f flag-off)

arith **17,5** · prop **13,8** · calls **8,6** · str **6,9** · arr **4,9**
· re **3,8** (S-97.0: 18,5 · 15,3 · 9,5 · 6,9 · 5,2 · 3,8 — la gamba
oracle stantia è SANATA; H-C e H-D rianimate, entrambe ≫ 5×).

## Giudice registro, stessa finestra (`wp99-harness/premisura-rollout99.out`)

add off 4,66 / on 3,53 · arith off 7,44 / on 5,44 (flag-on INVARIATO da
S-97.1; vantaggio flag-on arith −26,9%; rapporto arith flag-on 12,7×).
Decomposizione D (build INT1): 6,27 ns/occ = 3,60 call/marshalling (57%)
+ 2,67 traffico Vec (43%).
