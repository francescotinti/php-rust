# WP_SESSION_136 — coppia WP N=3 ESEGUITA (obbligo assolto) + LEVA FD1 dim-write SPEDITA (pin s136)

**In una frase**: la verifica completa di WordPress è stata rifatta tre volte per
configurazione sul motore nuovo (rapporto stabile a 1,78, memoria di picco
RIENTRATA) e scrivere in un array dentro una proprietà (`$o->prop[k]=v`, pane
quotidiano di Doctrine) ora salta il corridoio interno: −9% sulla micro dedicata.

**SCOREBOARD** (pin NUOVO **s136 1e14793e**4c0d9650c + server s136 91c4e043; micro gate promozione):
**arith 5,5 ↗ · prop 5,6 ↗ · calls 4,7 ↘ · str 4,2 = · arr 3,3 ↗ · re 2,6 ↗**
(run-to-run vs s135) · **rif WP full ON-ONLY = 1,777–1,779 (N=3 @ pin s135,
COMPATIBILE col rif 1,769** su banda off 0,041) · **leve perf spedite: 1 (FD1)**
· incidenti: 2 nuovi (LSP ~40 s + 3 probe dump in finestra, dichiarati; conteggio corretto da az.rev. #3).

## Esiti secchi
1·**Coppia WP p.1**: t4 rc=0, 6/6 gambe pulite, parità 6/6, on-only
  1,777–1,779 (spread 0,002), off 1,793–1,807, media 2,460–2,477,
  **peak 1753–1825 = bordo +80 RIENTRATO**. t1/t2/t3 rc=8 a gate quiescenza
  (mediaanalysisd, flare ~25' post-churn media): 2 emende ricetta DICHIARATE
  (assestamento a streak + retry gate ×3; COPIA-GATE rc=0), gate INVARIATO.
2·**Az.rev. S-135 5/5**: sonda fedeltà agli atti (r1 rc=2 filtro righe vuote,
  emendato → r2 rc=0) · emenda criterio AP1 p.2 (equivalenza osservabile) ·
  AP1_BUSY contatore su ramo irraggiungibile · catalogo §3.21 (a/b/c, flush
  +1/+5) · fixture v2 spezzata + s15-s22.
3·**LEVA FD1 SPEDITA** (istruttoria: sonda 2 resolve/iter → dump lowering
  `FieldAssign{[Prop,Index]}` → modello tempo chiusura 94%, arm 118,2 con
  walk_driver 37,2 dominante → criterio UB falsificabile 69,6): A/B R=5
  objdatains **D=+83,3** (soglia 13,3; riconc. smoke 1,6; **FUORI-UB +0,4
  DICHIARATO**, eccedenza +13,7 → sonda ripartizione dovuta; guardie 10/10,
  `re` morsa allo smoke rientrata col drop-1 vero) → promozione rc=0 →
  **pin s136**. Submicro: objdatains 1060→**963,3** (riconc. 13,4 ≤ 26,7) ·
  objchurn →1180 collaterale · objallocni 8,1→7,9 (osservazione S-135 rientra).
## ⭐ Lezioni (max 3)
- ⭐⭐ Un gate di quiescenza a 2 campioni è ingannabile da un daemon che OSCILLA
  (4→23% in 30 s): l'assestamento si fonda su una STREAK di quiete della stessa
  scala del gate, e un morso a un confine si ritenta prima di buttare ore di
  gambe valide — il gate resta invariato e autoritativo.
- ⭐⭐ Il riuso LETTERALE del walker del cammino pieno (`field_write_walk` +
  driver-loop) rende il fast path equivalente per costruzione sul leaf: la
  fixture (15 sezioni) è uscita cand==stash al primo colpo.
- ⭐ Serena accende rust-analyzer: in finestra di misura la navigazione simbolica sporca la misura (veto «LSP in volo» morde anche i tool).
