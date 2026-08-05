# Verbale sedia 7 — Daan Leijen (allocatore, footprint fisico, layout) — Concilio WP-102

## VERDETTO

S-100 regge sui gate di parità; la **contabilità del footprint NO**. Due
refutazioni capitali. La bozza §S-101 punto 4 va riformulata prima
dell'esecuzione.

## REFUTAZIONI

**R1 (capitale) — «cross-albero» è un'etichetta non provata.** L'R=2
intra-sera (0,5%) prova solo che il +95 MiB della gamba OFF **non è rumore
di stasera**; non prova che sia dell'ALBERO. Tra WP-99 e S-100 è cambiato
anche l'AMBIENTE (corpus WP, uploads, stato MySQL, macchina/disco a ~13G).
La contro-prova che discrimina esiste ed è economica: **un solo full run
del pin S-99 sigillato (stash `phpr-s99-sigillo`) stasera-stessa accanto
al pin S-100**. Senza questo A/B, un bisect (≥5 full run, con pin storici
che sappiamo non sempre riproducibili — WP-94) rischia di inseguire un
fantasma d'ambiente.

**R2 (capitale) — il peak del DEFAULT è un numero del CANDIDATO.** La
coppia WP è girata su a2772e62; il pin promosso 725a2ffad763bbc4 nasce
DOPO, col flip fb861e4 che tocca il funnel di produzione
(`ProgramCtx.reg_lower`, `compile_program_with_mode`, `lowered()`
eliminato). Il corpus per NOME non gatea il footprint: una regressione di
peak passa la parità. Eppure la rotazione pubblica «full peak ora 1929,0
MiB nel modo default»: è lo stato di un binario **mai misurato**.

**R3 — la voce aperta è mal nominata: non è «gamba OFF».** ON è a
1929,0 vs 1892,56 di WP-99 = **+36,5 MiB**; OFF +95÷106. La crescita
tocca ENTRAMBE le gambe; l'ON ne mostra meno forse perché il lowering ne
maschera una parte. Attribuire «OFF +95» pre-seleziona la spiegazione.

**R4 — «in storia (+1,9%)» per la gamba ON è indulgente**: la storia
dichiarata è −0,45% su WP-94→99; +1,9% è ~4× quel movimento, benedetto
senza banda.

**R5 — banda onesta sul full-peak**: la ±2% simmetrica era doppiamente
sbagliata — sotto il rumore (+14,6% intra-sera oracle) e simmetrica per
un gate anti-regressione. Il peak è una statistica di MASSIMO (coda
destra pesante): R=2 non è una calibrazione. E il rumore è **per-motore**
(phpr 0,5%, oracle 14,6%): una banda unica per i due motori è vuota.

## EMENDAMENTI

- **A-LE-102-1**: prima mossa dell'attribuzione = **A/B stessa-sera dei
  due pin sigillati (S-99 vs S-100), stesso ambiente**; solo se il delta
  si riproduce ⇒ **census allocatore PRIMA del bisect** (vmmap physical
  footprint per fase + stats mimalloc, `MIMALLOC_PURGE_DELAY=0`): il
  census dà il CANALE, il bisect solo il commit, a 5× il costo.
- **A-LE-102-2**: la voce si rinomina «crescita d'albero: OFF +95 / ON
  +36,5 MiB»; l'attribuzione misura entrambe le gambe.
- **A-LE-102-3**: calibrazione del rumore PRIMA di ogni banda futura sul
  full-peak: R≥5 sul pin, per motore; statistica = **mediana** (lezione
  WP-91), spread pubblicato; banda **unilaterale** (solo tetto
  anti-regressione), larghezza ≥ spread misurato.
- **A-LE-102-4**: ogni numero di peak pubblicato in rotazione porta
  l'HASH del binario misurato; smoke peak media sul pin promosso SUBITO
  (costa ~1 min), full alla prossima coppia.

## KILL-SWITCH

- **KS-LE-102-1**: se l'A/B dei due pin non riproduce il +95 (delta <
  2× lo spread intra-sera), la voce si riclassifica AMBIENTE e il bisect
  è VIETATO.
- **KS-LE-102-2**: banda sul full-peak senza spread R≥5 pubblicato prima
  = VOID.
- **KS-LE-102-3**: se lo smoke peak sul pin promosso esce dallo spread
  calibrato del candidato, il «1929,0» si RITIRA dalla rotazione e il
  flip si ricollauda sul footprint.
