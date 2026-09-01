# Sedia KLABNIK — bozza indipendente (chiarezza, spec dei gate, testabilità)

## VERDETTO: GO-CONDIZIONATO

## ROTTA (una riga)
Campagna a FETTE PROMOVIBILI (ogni fetta muore o promuove su pin canonico con gate pieni), aperta NON da codice ma da una fetta di PREZZATURA: decomposizione arith a registro + quota dispatch sopravvissuta sotto PHPR_REG_LOWER; budget di campagna pre-registrato PRIMA della fetta 1.

## REFUTAZIONE (perché non GO secco)
1. Una «campagna strutturale» big-bang è ingovernabile col processo attuale: il gemello A/B presuppone diff piccoli; un diff strutturale sposta la banda-layout (MC1: +45 bl ⇒ ±5-8 ns su cammini NON toccati; H-C2 icache-bound) e la soglia da 4 ns finisce SOTTO il rumore del diff — il verdetto diventa non refutabile.
2. Manca il prerequisito di misura: arith (46,8, la categoria peggiore) NON ha decomposizione a registro. Deliberare una rotta su un costo non ripartito ripete l'errore che REGOLE §4 vieta.
3. Il quesito aperto del dossier (quanto del dispatch 36,3 sopravvive col reg-lowering?) è ESATTAMENTE ciò che prezza R1: va risolto prima di ordinare R1/R2/R3.

## EMENDAMENTI
1. **Nessun branch lungo-vivente**: ogni fetta termina in stato verde promovibile (batteria, corpus per NOME ×2, coppia, micro R=5) o revert al byte. Una rotta non affettabile in stati verdi è NO-GO *di rotta* (questo tiene R2/NaN-boxing a veto).
2. **Criterio-prima esteso alla campagna**: prima della fetta 1 si pre-registra in un `.out` il budget: obiettivo aggredibile prezzato (arith ≤3× richiede −21 ns/iter), fette massime, condizione di morte della campagna.
3. **Diff strutturale ⇒ disasm prima/dopo (bl-count) nel criterio** e soglia mai sotto la banda-layout fondata del diff stesso.
4. **Anti S-91/92**: i modelli di decomposizione ALLOCANO le fette, non promuovono MAI; arbitro di record resta micro R=5 + coppia WP sul pin, con copia-gate su ogni derivato. Lo strumento di decomposizione arith si collauda con un mutante che DEVE mordere (dente noto) prima di fidarsene.
5. Veti BOLT/PGO/NaN-boxing CONFERMATI finché la prezzatura non li rende affettabili.

## KILL-SWITCH
- **Di fetta**: D sotto max(4 ns, rumore, banda-layout) al verdetto pre-registrato ⇒ revert al byte, meccanismo nominato, la campagna VIVE.
- **Di campagna** (pre-registrati): (a) la fetta 1 prezza il costo aggredibile su arith < 21 ns/iter ⇒ R1-arith morto, si delibera R4 (ridefinizione onesta); (b) 3 fette di trasformazione consecutive cadute ⇒ campagna sospesa, concilio.

## PRIMA FETTA (S-168, sola misura)
Decomposizione arith a registro (chiusura ≥85%, standard S-129/136) + A/B PHPR_REG_LOWER on/off su arith R=5. **Giudice**: `wp97-harness/micro/run-micro.sh`, arith, R=5, criterio pre-registrato. **Soglia/verdetto**: quota dispatch+operandi prezzata e firmata; se aggredibile < 21 ns/iter ⇒ kill di campagna (a). Nessuna riga di codice VM in questa fetta.
