# Verbale Sedia 9 — Gregg (metodologia di misura e attribuzione + mandato inverso)

## VERDETTO: CON EMENDAMENTI

La coppia stessa-sera R=3 sullo stesso binario è il metodo giusto e il −30,7%
su `arith` (2,40 s contro spread 0,04) è fuori rumore di due ordini. L'abbandono
sul criterio scritto è esemplare. Ma la sessione ha già in mano il dato che
LIMITA H-B1 e non ne ha tratto l'inferenza.

## Refutazione capitale (sì)

**«Il divario vive nel costo per opcode → quindi H-B1 preambolo» è un
non-sequitur refutato dai dati della sessione stessa.** `ha2-sweep.out` dà il
costo MARGINALE di uno Sweep noop: 0,07 s / 50M dispatch = **1,4 ns** — e uno
Sweep noop paga per intero il preambolo (fetch, bounds check, len). Quindi 1,4 ns
è un TETTO sull'overhead uniforme per dispatch. Su `arith` flag-off:
19 × 1,4 = 26,6 ns su 156,6 ns/iter = **tetto ~17%** (banda ~8–27%: il 0,07 s è
appena 2× lo spread). Un tetto del 17-27% non chiude un fattore 8 per opcode:
gli ~8,5 ns residui del residuo stanno nei CORPI (discriminazione di tipo, lavoro
Zval) — cioè H-B2, non H-B1. La lezione «il costo non è uniforme» dice la stessa
cosa e la sessione l'ha scritta senza applicarla all'ordine delle ipotesi.

## Risposte alle domande di perimetro

- **Collaterali**: sorretti. prop 6,27→5,50 (Δ 0,77 s vs spread 0,04-0,05, ~15×),
  calls 4,02→3,39 (Δ 0,63 vs 0,05), arr Δ 0,17 vs 0,01. Ma sono numeri SENZA
  attribuzione: nessun census flag-on/off su prop/calls — quanti Binary/CmpJmp
  per iterazione spiegano il −12,3%? Un calo senza meccanismo contato non è
  conoscenza dell'oggetto.
- **«str/re = rumore»**: classificazione DISONESTA per omissione. str: Δ 0,04 s
  con spread 0,08 sul lato off → NON RISOLTO, banda ~[−11%, +4%] — che è diverso
  da «zero». re: Δ 0,02 s con risoluzione di `time -p` 0,01 s → al limite dello
  strumento. Va scritta la banda, non l'etichetta.
- **7,83 vs 7,88/7,95**: binari DIVERSI (0dd98eb vs 2f6c1a/d5ce86e); la deriva
  inter-build (0,6%, stessa scala dello spread) NON è nominata in
  `ha1-registers.out` («coerente con 7.88» la usa come conferma senza dichiarare
  il caveat). Qui è innocua — anzi è la prova timing dello zero-delta flag-off —
  ma va nominata come termine.

## Ricetta ESATTA per attribuire il preambolo PRIMA di H-B1 (misura M1, zero codice VM)

1. `noop.php`: `for($i=0;$i<200000000;$i++){}` — census → ops/iter (attesi
   ~4-5: IncDecSlot, CmpJmpSC, Jump, Sweep[, Pop]); tempo con la ricetta
   run-micro.sh → **D = ns/dispatch dei soli op economici** con braccio di leva
   4 miliardi di dispatch (statistica stretta, non il Δ 0,07 s di ha2).
2. ASM del binario CORRENTE: contare i bounds check/len davvero emessi nel
   run_loop (prof95-media §PREAMBOLO è di un'ALTRA build; LLVM può già eliderne).
3. Predizione scritta nel .out PRIMA del codice: **P = 19·D/156,6 ns** (quota
   massima di `arith` flag-off attribuibile al preambolo).

**Criterio di caduta che ne segue**: H-B1 cade A TAVOLINO se P < 10%; se
scritta, cade se il calo misurato < max(P/2, 3× lo spread relativo della coppia
≈ 1,5%). Caveat da scrivere: D è marginale in pipeline out-of-order, quindi
tetto SOFFICE verso il basso.

## Mandato inverso — l'OGGETTO

**Sappiamo oggi**: (1) il fattore conteggio è chiudibile (19→11 vs 7, shape in
albero); (2) il residuo è ~8× PER OPCODE, pinnato: 11 op a 9,87 ns vs 7 a 1,23;
(3) il dispatch nudo costa ~1,4 ns → il grosso è nei corpi; (4) la v3 vince la
sua categoria (−30,7%) pur avendo perso l'aggregato; (5) il fold commutativo è
un buco osservabile (ordine operandi in "Unsupported operand types").
**Non misurato e dovuto**: la decomposizione per-opcode degli 11 residui (quale
dei corpi porta i ~8,5 ns?); il census dei collaterali; la banda di str; il
preambolo sull'ASM corrente; prop resta 13,1 flag-on (seconda peggiore) senza
un solo numero nuovo sul suo meccanismo.

## Emendamenti

- **A-GR-99-1**: str/re riscritti come BANDE in ha1-registers.out; vietata
  l'etichetta «rumore» senza `delta/spread/banda`.
- **A-GR-99-2**: eseguire M1 (noop micro + census + ASM) e scrivere P prima di
  ogni riga di codice H-B1; ordine H-B1/H-B2 deciso da P, non dal piano.
- **A-GR-99-3**: census flag-on/off su prop e calls per attribuire i collaterali.
- **A-GR-99-4**: ogni confronto cross-binario porta la riga
  `deriva_inter_build=` esplicita.

## Kill-switch

- **KS-GR-99-1**: H-B1 non scrive codice VM finché P non è nel .out; P < 10% ⇒
  H-B1 cade a tavolino e si passa a H-B2.
- **KS-GR-99-2**: nullo qualsiasi claim di miglioramento < 3× lo spread relativo
  della propria coppia stesso-binario.
- **KS-GR-99-3**: vietato «rumore»/«coerente» su numeri di binari diversi senza
  la deriva inter-build dichiarata.
