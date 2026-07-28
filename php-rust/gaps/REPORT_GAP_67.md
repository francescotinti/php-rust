# REPORT_GAP_67 — misure della SOLA sessione WP-67 (2026-07-28 notte)

Perimetro misurato: catena full serale run55 TRIPLE (coppia propria
P-2 + self-pair A-A′, prima esecuzione del protocollo KG67-1/B-66.1) +
leak-shape server. Media group NON rimisurato (riferimento resta
WP-63: CPU 2,58×, footprint ~3,0× banda da ratificare G3).

## Full-suite (30.472 test, wpdev, /usr/bin/time -l, stessa sera)

| run | binario | user CPU | fail-set |
|---|---|---|---|
| run55-new | phpr-wp67 (P-2, c0f5cfff) | 777,19 s | 88 BYTE-ID = run33 |
| run55-old | phpr-wp66 (5aa60d56) | 787,45 s | 88 BYTE-ID = run33 |
| run55-new2 | phpr-wp67 (A-A′) | 786,95 s | 88 BYTE-ID = run33 |

- Delta coppia new-vs-old **−1,30%**; spread A-A′ stesso-binario
  **1,26%** ⇒ **costo CPU di P-2 = compatibile con zero** (dentro lo
  spread della serata, misurato per la prima volta con self-pair).
- Nessun nuovo claim di rapporto vs oracle (l'oracle non è stato
  rigirato stasera; riferimento full resta 2,06-2,11×).
- Peak fisico: NON misurato da coppia census (maxrss 2.196.619.264 B ≈
  2.095 MiB informativo, MAI giudice).

## Leak-shape server (metro nuovo P-2, unità dichiarate)

- Pre-P2 (memgc66, fixture nk, N=200): +2 moduli VIVI/richiesta,
  4,84 KiB/req RITENUTI; wpdev: **1,62 MiB/richiesta** ritenuti (22
  moduli: 15 impure + 7 eval).
- Post-P2 (memgc67, N=1000): pendenza Σcommitted **+2,55 KiB/req**
  (soglia pre-registrata 50; residuo = bookkeeping census dichiarato);
  dead_units 1998/2003; Δstubs_entries=0.
- CPU residua per-richiesta dei 15 impuri: **≈21 ms/richiesta**
  (script-loader 8,4 + comment 6,1 + ProviderRegistry 1,5 = 74-75%, emend. Hejlsberg) — è
  la quota della leva WP-68.
