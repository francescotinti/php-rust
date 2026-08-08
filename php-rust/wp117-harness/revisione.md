# Revisione S-117 (revisore singolo, lente: SEMANTICA)

## Verifiche fatte
- Letti criteri PRE, i 3 verdetti, gli script emittenti, banda-v2, session file, REGOLE.
- Raw promo-out: batteria.log (93 KB) esiste; awk dal log dà 1742/0/2, rc=0 da file; corpus off/on.fails = 1415/1415; tutti gli rc-file = 0; fixture-chain rc=0; micro pin prop 5,9 / calls 4,8 confermate nel raw.
- Script confrontati: `s117-aprime-gates.sh` esegue la macchina s102 (chunk off↔on byte-id, carve-out 3 settype-NaN); `s117-promozione(.2).sh` NO — confronta solo INSIEMI di nomi (off vs congelato, on vs congelato, off==on per nomi).
- **Check post-hoc mio**: rieseguita la macchina chunk sui raw esistenti, aprime-out (A′ senza L-A) vs promo-out (pin con L-A), entrambi i modi: solo-A 0, solo-B 0, chunk diversi fuori carve-out **0** (3 nel carve-out, attesi). Per transitività il pin ha anche off↔on a livello di contenuto.
- la-build.sh: admission 5 casi con parità output+rc, {main} A↔B via cmp, bigramma anche nei 4 miss (guardie bail esercitate). Candidato B in target dedicato = 67d74c70; pin canonico = 1656580e; A′ = 8135dcf8 — stessa sorgente+ricetta, hash path-dipendenti.

## Il punto che ridimensiona
La promozione ha DECLASSATO il gate semantico del corpus: sul tree effettivamente promosso (A′+L-A) il contenuto dei 1415 fail non è stato giudicato da nessuno script — solo i nomi. Una leva che cambia il DETTAGLIO di un fail restando nello stesso insieme sarebbe passata; l'admission a 5 casi non copre 1415 programmi. Il claim sopravvive solo perché il mio chunk-diff post-hoc dà zero differenze: vero nei fatti, non garantito dal processo.

## Verdetti
- A′ con gate pieni rc=0: **REGGE** (raw verificati, chunk-machine inclusa).
- «off↔on zero differenze» sul pin: **RIDIMENSIONATO** — stabilito su A′ senza L-A; sul pin solo nomi, contenuto pareggiato post-hoc dal revisore.
- Batteria/churn (9595e59a→ricetta 1656580e==H1): **REGGE** — la batteria giudica il sorgente, il determinismo riproduce il binario; ma vale solo intra-target: cross-target gli hash divergono.
- Attribuzione A/B (verdetto su 67d74c70, pin 1656580e): **RIDIMENSIONATO lieve** — guardie fini (0,50 ns) giudicate su un layout diverso dal pin; magnitudine però confermata dalle micro sul pin (prop 8,2→5,9, calls 4,8 invariato).
- «Parità semantica invariata» tout court: **RIDIMENSIONATO in perimetro** — server/per-request (WP full/media, census) non rimisurati, debito dichiarato: il claim vale per la superficie CLI.

## Azioni (S-118)
1. Portare la macchina chunk (perl s102) dentro lo script di promozione/pin: corpus-gate = nomi E contenuto, off↔on e vs baseline.
2. Congelare in wp109-harness/corpus-gate un golden dei CONTENUTI dei fail per modo, accanto ai nomi.
3. Misurare una volta la banda della path-dipendenza (2 build stessa sorgente, target diversi) o eseguire gli A/B futuri nel target canonico, prima della prossima guardia sub-nanosecondo.
4. Rimisurare WP full/media + pin-server (tripla census per-request) sotto A′; fino ad allora scrivere «parità CLI» nei report.
