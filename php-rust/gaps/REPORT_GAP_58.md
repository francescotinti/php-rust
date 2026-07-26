# REPORT_GAP_58 — gap perf oracle↔phpr della sessione WP-58

> SOLO le misure della sessione WP-58; trend cumulativo in `GAP_TREND.md`.
> Sessione di LEVA (Fase 3 tranche 2 "arena": dieta header + scan-mode
> small hashed + block arena; release = phpr-wp58 2d7efdf8…).

## Media group (ab58, 6 round interleaved old=phpr-wp57 / new, stesso giorno)

- **CPU**: old 54,91s → new **54,84s** = **−0,13% FLAT** (r2-6 +0,2% =
  rumore; guardia checkpoint pilota ≤+2% SUPERATA). Oracle 21,04s ⇒
  **media CPU 54,84/21,04 = 2,61×** (invariato da WP-54/56).
- **Footprint (peak fisico /usr/bin/time -l)**: old 1,619G → new
  **1,602G** = **−1,05% = −17MB medi** (new stretto 1,596-1,614; old
  1,607-1,665 con outlier r2) — dentro la banda WP-57 −10..−20MB.
  Oracle 0,394G ⇒ **footprint 1,602/0,394 = 4,07×** (da 4,08×).

## Full-suite (run46 new vs run46-old phpr-wp57, STESSA SERA back-to-back)

- Parità: ENTRAMBE 30.472 test, 0E/2F/86W/73S, **fail-set 88 nomi
  BYTE-ID a run33 ×2** ⇒ fix yield_from `e9a1679` VALIDATO sul full
  (chiusa la ⚠️ di WP-57); baseline resta run33.
- Master-CPU: new ≈**714,4s** (11:54,4) vs old ≈696,6s (11:36,6) raw =
  +2,5%; il campionatore ps a 20s tronca asimmetricamente (old ~16s wall
  dopo l'ultimo campione, new ~2s) ⇒ stima onesta **+1,0..+2,5%
  SOLO-FULL** (media FLAT). TENUTA (direttiva no-revert), attribuzione
  aperta a WP-59 (candidati: TLS+RefCell del pool sul Drop path, scan
  lineare 5-8 slot). Rapporto di giornata: 714,4/339 ≈ **2,11×**
  (cumulato Fase 3 sul full resta negativo: WP-56 −2,66%).

## Mechanism-check (pin c, census su live-accounting esatto)

- Leva C alla CIFRA: hashed b1 −373.824B = 32×11.682, b2 −312.160 =
  32×9.755; arr peak census 66,43→65,58MB; riconciliazione arr
  reached==live 63.435==63.435 INTATTA con l'arena.
- Leva A invisibile al census per costruzione (ARR_OVERHEAD costante) —
  giudicata dal peak fisico esterno (−17MB ⊃ quota A ≈ −5MB al picco).
- **Ob.2**: canale obj live-ESATTO (probe: 1501==1501 oggetti,
  352.208==352.208 byte, cum_n 6500 = costruzioni esatte). Prima misura
  esatta: **obj peak 56,1MB ≈ 3,7% del fisico**; live EOR 13,9MB
  (estimatore morto: era +28% sull'EOR). Canali valore al picco:
  arr 65,6 + str 62,3 + obj 56,1 ≈ **12% del fisico** — la leva GRANDE
  di footprint resta FUORI dai canali valore (+ unit 222,6MB).
