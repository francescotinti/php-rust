# Revisione S-149 — revisore singolo, lente MISURA (verbale ≤400 parole)

VERDETTO: **REGGE CON RETTIFICA**. Il nucleo regge: identità census ESATTA ×2
(335.837.200), testa 5,2× soglia; D=+19000 ns/iter è 95× la soglia e la
quantizzazione 0,01 s incide su D per ~0,35%. Ma tre pre-registrazioni non
sono state onorate alla lettera.

RILIEVI

1. **Guardie A/B: R=3, non R=5 pre-registrato.** Il criterio bt1 p.5 dice
   «Guardie (R=5)»; il verdetto mostra 3 valori per guardia. Con 4/6 guardie
   a +1 tick B-lento (ARR 0,95→0,96; PROP 2,36→2,37; CALLS 2,15→2,16; STR
   0,66→0,67), un sistematico ~0,5–1,5% non è escluso né escludibile a R=3
   con quantizzazione 0,01 s (binomiale segni: p≈0,19). Il disasm bl-count
   di run_loop, pre-registrato (p.5), è ASSENTE dal .out.
2. **Scarto +3,2% «path del workdir»: nominato, non verificato.** Nessun run
   di controllo con path lungo; la corroborazione è il solo shift
   d'istogramma (a bytes), lo scarto è di CONTEGGI (+10,4M): meccanismo
   congetturale. Non tocca la testa (130,15M ≫ 24,8M).
3. **Cifre B a precisione non posseduta.** B netto = 0,11±0,01 s ⇒ 733
   ns/iter è in realtà 667–800 (±9%); il bilaterale «1,86×» usa oracle
   0,07±0,01 RAW senza pavimento ⇒ intervallo reale ~1,6–2,3×. Companion
   dichiarato (corretto), ma le cifre puntuali non vanno propagate.
4. **«Rif S-142 CONFERMATO» è un sovra-claim.** t2 (1,722–1,742) resta FUORI
   dal rif; la stessa sessione rifonda la banda a 1,722–1,823 proprio per
   contenerlo. Legittimo: «t3 COMPATIBILE; su 3 finestre il rif NON è
   confermato». La banda-unione 0,101 (≈5,7% del rapporto) rischia la
   non-falsificabilità. La deriva ρ_A=−0,886 > critico 0,829 regge.

AZIONI S-150
1. Gate promozione BT1: guardie a R=5 come pre-registrato + disasm bl-count
   nel .out di record.
2. Run di controllo census con workdir ≥100 char: attesa hostcall.n ≈
   325,4M; altrimenti la spiegazione cade e lo scarto va re-istruito.
3. Rettificare t3: niente «CONFERMATO»; per t4+ pre-registrare una
   falsificazione a mediana per finestra, non a intervallo-unione.
4. Citare B solo come intervallo (667–800 ns/iter); misurare il pavimento
   oracle nel prossimo giudice bilaterale.
5. La scommessa ORM pre-registrata (direzione ↓, denominatore 0,293 s) resta
   l'unico arbitro della leva.

## Recepimento (stessa sessione)
- Rettifica «CONFERMATO» applicata a WP_SESSION_149/NEXT_SESSION/PERF_MAP/
  REPORT_GAP_149; formula nuova: «t3 COMPATIBILE (su 3 finestre conferma NON
  piena: t2 resta fuori)».
- Cifra B qualificata a intervallo 667–800 ns/iter nei documenti di rotazione.
- **Incidente 17 CONTATO**: guardie eseguite a R=3 contro R=5 pre-registrato
  + disasm assente dal .out (esecuzione difforme dal criterio); riparazione
  = azione 1 al gate di promozione.
- Azioni 1–5 inserite nell'ordine §S-150 di NEXT_SESSION.
