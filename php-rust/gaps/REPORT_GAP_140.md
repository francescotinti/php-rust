# REPORT_GAP_140 (SOLO S-140 — mai cumulativo)

Pin: phpr s140 f2708b75660803a7 + server c7a03e2aaa7c7cba (leva HC1).
Misure di QUESTA sessione (verdetto s140-pair-verdetto-t1.out, rc=0, 6/6 pulite):
- **WP full ON-ONLY: 1,765-1,777** (coppie proprie N=6) — COMPATIBILE rif S-139
  [1,752-1,785] su banda_ON 0,033 ⇒ HC1 non muove WP (atteso); banda_ON
  CONFERMATA su 2ª finestra (canonica resta 0,033; intra-finestra 0,012).
- **media user-only: 2,462-2,479** (companion 2,406-2,429) — nel range S-139.
- **peak phpr: 1807,3-1852,7 MiB** — leg1 1807 DENTRO la banda oss. s136/s137
  (1743-1825), gambe 2-6 1838-1853 sopra: la firma «tutte le gambe alte» di
  S-139 NON si ripete ⇒ candidato binario (celle IC RMW) indebolito,
  componente stato/ordine-finestra (warmup/accumulo) indiziata per NOME.
- **micro-categoria NUOVA: hintcall 7,3×** (oracle ~33 vs phpr ~243 ns/iter,
  rif bilaterale R=5) — prima cifra.
- ORM/dbal: NON rimisurate (riferimento resta S-139: dbal 8,15-8,23 ind.,
  ORM 8,59-8,71); census monobinario: canale hint-check ~0,13% della suite.
