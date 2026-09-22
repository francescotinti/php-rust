# Revisione S-181 — lente MISURA

## Verdetto: REGGE CON RILIEVI

## Ciò che regge
- Ricalcolo dai TSV riproduce i verdetti: calls-dq D colonna/appaiata +8,50/+8,17 (cr1), +8,33/+8,50 (cr1b), +8,00/+8,17 (cr1c); rumore drop-1 ≤0,67; segni 15/15; floor 0,02 s su ~6 s: N=60M basta.
- Parità: `fxcr1-B.out` == `fxcr1-oracle.out`; mutante morde (ACE 5→2).
- cr1c è pulita anche sotto la guardia ORIGINALE (0 campioni updater): la cifra di record non dipende da E3.

## Rilievi
1. **Guardie fuori attesa, mute**: criterio p.4 attende |D|<1; misurato prop-dq +2,53/+2,87/+2,40 (7-8 %) e arith-dq +1,40/+0,76/+0,72, segni 15/15, su loop SENZA Call/Ret. `s181-ab-cr1.sh` testa solo la regressione: l'attesa violata passa in silenzio. B è più veloce dove la leva non tocca ⇒ layout/regalloc di run_loop (disasm: istr +153, sp_refs 11616→10685, −8 %).
2. **|A−Z| non stima la banda-layout di B**: Z ricostruisce il sorgente di A (7,68M byte diversi, tempi uguali al tick). Se l'effetto di layout è proporzionale (7,5 % di 99 ≈ 7 ns) il meccanismo vale ~1 ns; se additivo (~2,5) ~5,5. La cifra [8,00;8,17] è «binario B vs pin», NON «la leva».
3. **Meccanismo (a) contraddetto dal disasm**: `enter_callee` resta bersaglio `bl` 7 volte in pin E B; B aggiunge `Frame::with_buffers` (10→11) e `Vec<Frame>::push_mut` (3→4): il Frame viaggia ancora per valore (sret + copia nel push). Risparmio reale = una chiamata in meno. bl +9 (atteso −1) registrato senza interpretazione (s181-leva-build-verdetto.out).
4. **Nessuna scomposizione**: nessun braccio per (a)/(b)/(c); l'attesa [4;12] (larghezza 8 = 2× soglia) accoglie quasi ogni esito nominabile.
5. **E3 post hoc**: emendata dopo due verdetti concordi (7042bfe1 14:32, cr1c 14:36); innocua qui; resta allentamento mai servito.
6. **Promozione**: pin s181 = 19a2faa8 ≠ B 5ed39690: la cifra vale per un layout che il pin non ha; Data 8G prima della batteria (<10G del pre-flight).
7. Il mutante presidia (b) solo sul cammino lento delle FUNZIONI; nessun mutante per `methodcall_fast` a ip=1 né per l'ordine push-prima-bind di (a).

## Azioni S-182
- Leggere la conferma post-pin calls-dq+prop-dq (s181-promozione.sh:215-216): se prop-dq deriva ancora +2,5 il layout è provato.
- Braccio placebo: B con la sola (c) (nessun Ret in prop-dq): se sposta prop-dq, la banda-layout di B è misurata; poi (a) e (b) separate.
- Nello script, esito esplicito dell'attesa |D|<1 sulle guardie.
- Riscrivere (a) a verbale; mutante ip=1 sul methodcall lento; togliere E3 o portarla nel lanciatore-modello.
