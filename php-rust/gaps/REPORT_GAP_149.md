# REPORT_GAP S-149 (2026-08-17) — SOLO questa sessione (mai cumulativo)

Replica coppia t3 @ pin s145 (DEBITO s148), verdetto
`wp148-harness/s148-pair-verdetto-t3.out` (harness INVARIATO, criterio
`wp149-harness/s149-criterio-t3.md`):

- **WP full ON-ONLY t3: 1,786–1,802** (coppie proprie N=6, **6/6 PULITE —
  prima finestra tutta pulita**; ictx% 99–101% su ogni gamba, entrambi i
  lati) ⇒ **COMPATIBILE** col rif S-142 [1,765–1,788] su banda 0,036: il
  fuori-banda basso di t2 NON si ripete — **su 3 finestre la conferma resta
  NON piena (t2 fuori; rett. rev. S-149)**. Banda di
  finestra t3 = 0,017 (t1 0,090 · t2 0,020) ⇒ **banda_ON RIFONDATA
  multi-finestra (criterio t3 p.5): unione t1+t2+t3 = 1,722–1,823 (0,101)
  su 17 coppie proprie pulite** — d'ora in poi il confronto formale usa
  questa, mai una finestra singola.
- **media user-only t3: 2,504–2,540** (leg6 max; nessuna ricorrenza di
  gamba-massima tra finestre).
- **peak phpr t3: 1772,2–1782,0 MiB, 6/6 livello BASSO unico** (il doppio
  livello di S-142 non si riproduce); deriva test con N=6: ρ_A(leg,peak)
  = −0,886 (critico 0,829) ⇒ **deriva discendente dei peak CONFERMATA**;
  ρ_B(peak,rapporto) = −0,600 ⇒ accoppiamento col rapporto NON confermato
  (osservativo).
- **leg1**: PULITA in t3 — corrobora l'indagine
  `wp149-harness/s149-ictx-leg1-indagine.md` (firma di FINESTRA, non di
  gamba: t3 notturna con CI in mutex = finestra uniforme, nessun transitorio).
- micro/ORM/dbal: NON rimisurate (pin invariato; rif S-145 micro, S-147 ORM
  8,370–8,427 / dbal 8,20–8,37). **Scommessa BT1 pre-registrata sul
  prossimo pin: attesa ↓ 0,8–3,1 s sulla coppia ORM**
  (`wp149-harness/s149-decisione-bt1.md`).
