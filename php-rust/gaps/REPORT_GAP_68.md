# REPORT_GAP_68 — misure della SOLA sessione WP-68 (2026-07-28)

Binario: phpr-wp68 (8eea807e…, tree `f663765`). Oracle: PHP 8.5.7 brew.

## Full-suite (wpdev, 30.472 test — run56-triple, sandwich A-B-A′)

- Master user CPU: new **790,31u** · old (phpr-wp67) **791,10u** ·
  new′ **815,27u**. Coppia **−0,10%** ≪ spread A-A′ **3,16%** ⇒ costo
  della leva defer-always INDISTINGUIBILE DA ZERO (nessuna cifra
  citabile, G-68.1). Rapporto vs oracle: riferimento run45 resta il
  minimo storico (697,8u = 2,06×); la serata odierna colloca il full a
  **≈2,10× (790,31u)** — dentro la banda 2,06-2,11× nota (macchina
  sotto swap 7,5-8,5 GB per l'intera catena).
- Fail-set: **88 BYTE-ID = run33 in tutte e tre le run**.

## Media group (coppia singola oracle vs phpr, gap68-media)

- CPU user: 21,83u → 58,47u = **2,68×** (riferimento storico 2,58×;
  coppia singola sotto swap: cifra INDICATIVA, nessun claim di
  regressione senza self-pair).
- Footprint peak: 380,8 MB → 1.172,9 MB = **3,08×** — DENTRO la banda
  standing 2,9-3,2 (quarta coppia G-66.4: mediana ≈3,0-3,1).

## Canale server (la leva della sessione)

- CPU residua include: **21 → ~7,8 ms/req** su build census
  (Δ(l+c) whole-file = 0,00; 16 DECLARE × 490 µs) — guadagno netto
  ≈13 ms/req; hit_cross=512/512, cold=0.
- Leak-shape fixture nk: pendenza **0,586 KiB/req** (post prune
  E-68.2; era 2,549). wpdev N=1000: de-leak LETTO (dead ~16/req, vivi
  piatti ≈1.036); pendenza +62,6 KiB/req ATTRIBUITA (ipotesi
  quantificata) al census-own — banda axum NON citabile fino a L-68.1.
