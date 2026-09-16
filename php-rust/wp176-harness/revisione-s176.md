# Revisione S-176 — lente MISURA (revisore singolo, sola lettura)

## Verdetto: REGGE CON RILIEVI

## Ciò che regge
- Cifre A/B ricalcolate dal TSV (mediane, floors per binario, appaiate 2,67/4,20/1,04): tornano. rc=3 coerente con p.4 (il copione aggiunge SL al KILL: più severo, lecito).
- MF prova che il flag è LETTO (stuck-true ⇒ 4 blocchi rotti, fx-sw1 diverge). Ogni sito dirty passa da `gc_idle_set`; `collect_cycles` fuori sweep non crea stale-true.
- Z è controllo nullo: |A−Z| 0,04–0,20 contro D_B 2,7. Census a parità per NOME.

## Rilievi
1. **«63 % del tetto» è cifra indebita**: D_B «NON nominato (vale 0)» viene diviso per un tetto nominato per 0,20 con rumore 0,93; col tetto S-175 (4,67) sarebbe 57 %. Solo direzione.
2. **Banda tra run su binari diversi**: PREV 20,08/36,13 è l'A di S-175 = phpr-C (b6c4b587); oggi A = pin. Same-binary regge; l'etichetta viola §3.
3. **«Rimandata a rerun» è irraggiungibile**: pavimento 4 contro tetto 4,2–4,67 ⇒ il flag da solo non nomina mai; la via è la composizione.
4. **Il census non dice che «la coppia non vede il flag»**: il predicato è letto anche dal handler `Op::Sweep` (vm/run riga 7013): portata su ORM = 10,88 % degli op, non il 2 % fuso.
5. **«~63 ns/op ⇒ Sweep ≈0,5 %»** (WP_SESSION_176.md:10 «ipotesi», :16 lezione ⭐⭐) è cifra da componenti su binario census: vietata da §3 e dal criterio census p.4.
6. **E4 non è fondata sulla sua tabella**: mediana delle 18 gambe S-162…S-175 = 4,94, non 4,96 ⇒ banda [4,84;5,04]; REF2 rinormalizzata slitta di 0,14 s, più della sua larghezza (0,08).
7. **«Senza regressione» copre solo arith-dq**, che non esercita i siti dirty; ORM non misurato; HEAD porta il flag senza batteria/corpus (CI ferma dal 15/09).
8. Igiene micro: sentinella FINE con mediaanalysisd al 220 %; drop-1 A'=0,47 nasce da un pareggio float (chiave dichiarata ⇒ 0,33).

## Azioni S-177
1. Nel NEXT: flag = direzione «oltre metà del tetto»; rotta = composizione (A = pin, B = flag+leva, terzo braccio flag-solo).
2. Correggere E4 a 4,94 o motivare 4,96 come emenda; poi rieseguire il criterio.
3. Coppia ORM anche sul braccio flag-solo (attesa ≤0); batteria+corpus sul tree prima di usarlo come base A/B.
4. Declassare la lezione «0,5 %» a direzione; cifra solo da A/B proprio su ORM col mutante MS.
5. PREV same-binary; streak anti-flare nel lanciatore micro; tie-break del drop-1 esplicito.
