# REPORT_GAP_100 — S-100 (2026-08-05): coppia WP nei DUE MODI stessa-sera + micro del giudice post-flip

**SOLO sessione S-100** (mai cumulativo). Raw: `wp100-harness/pair-out-{off,on}/`
(+ `full-phpr-r2.time` per l'R=2 della gamba off), rapporti macchina in
`wp100-harness/pair100-ratios-{off,on}.out`, bande e verdetti in
`wp100-harness/pair100-bande.out`. Binario candidato a2772e62 (coppia),
pin finale post-flip 725a2ffad763bbc4 @ fb861e4.

## Coppia WordPress full+media, DUE MODI (bande pre-registrate KS-GR-101-1)

| metrica | modo off (`=0`) | modo on (`=1`) | on/off (gate flip) |
|---|---|---|---|
| media user CPU phpr | 55,43 s | 55,63 s | 1,004 (≤1,05 ✓) |
| media peak phpr | 1.148.716.448 B | 1.126.712.736 B | 0,981 (in [0,95,1,05] ✓) |
| full master CPU phpr | 839,23 s | 846,00 s | 1,008 (≤1,03 ✓) |
| full peak phpr | 2.095.581.320 B (1998,5 MiB); R=2: 2.085.177.480 (1988,6) | 2.022.754.392 B (1929,0 MiB) | 0,965 (fuori [0,98,1,02] in direzione FAVOREVOLE) |
| gamba oracle (fotografie) | media 21,22 s / 394.855.264 B · full 447,26 s / 728.368.352 B | media 21,16 s / 394.019.704 B · full 445,11 s / 834.585.824 B | — |

- **Parità per NOME**: media 0 failure (2 modi); full 87 vs 88 nomi, unica
  differenza = `Tests_Functions::test_wp_is_stream` ftp (catalogo), nei 2 modi.
- **Voce APERTA (attribuzione dovuta)**: full peak gamba phpr-OFF 1988,6–1998,5
  MiB vs 1892,56 di WP-99 (**+~95 MiB CROSS-ALBERO** S-98-pin → S-100; R=2
  intra-sera stabile 0,5%: non è rumore). La gamba ON (quella promossa) è
  1929,0 = +1,9% vs WP-99.
- **Fatto di strumento**: l'oracle full peak balla **+14,6% intra-sera**
  (728→835 MB) e il media oracle peak intra-sera è stabile (−0,2%) ma
  lontano −11,6% dal valore S-99: l'anomalia +28,7% di S-99 = varianza
  TRA-sere della gamba oracle, resta aperta per nome.

## Micro del giudice (post-flip, R=5, netto pavimento)

- add: off 4,76 · on 3,27 (netto 3,25; S-99: 3,53) — l'estensione BinaryAdd
  recupera il residuo del loop: **vantaggio on −31%**.
- prop: oracle 0,42 · on 5,22 (**12,4×**) · off 5,88 (14,0×; S-99 13,8 in
  banda). **Decomposizione H-C: 12,4 = conteggio 2,0× (18 vs 9 op/iter,
  census opcache) × costo/op 6,2× (9,67 vs 1,56 ns/op)**; profilo co-equale:
  ~27% del tempo phpr nel ciclo di vita Zval (drop/clone/gc_note),
  oracle su handler TAILCALL specializzati.
- Isolante H-B2: L=12,9 ns/occ (BinaryAdd vs Binary(Add), stesso albero,
  un solo op di differenza) → post-estensione L'∈[−1,0].
