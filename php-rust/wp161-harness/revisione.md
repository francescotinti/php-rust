# Revisione S-161 — lente MISURA

## VERDETTO: REGGE CON RILIEVI
La promozione L-AL2 regge come DIREZIONE+MECCANISMO (segni 5/5 R=5, 5/5 post-pin, census Δ=1 esatto, guardie 17/17, catena gate piena); la MAGNITUDINE ~5,0 è al bordo della soglia e senza taratura propria, e il modello di famiglia esce dalla sessione senza potere predittivo.

## Rilievi
1. **Cifra AL2 al bordo, conferma sotto-rumore.** D_R5=+5,0 vs soglia 4,0: margine 1,0 ns = 1 tick del giudice (tick=soglia/4, il minimo ammesso da §3); rumore drop-1 B'=2,0 > margine. La conferma post-pin D=+3,0 con rumore 5,0 vincola solo il SEGNO: non discrimina D_vero=3 (sotto soglia) da 5. Lo standard che S-160 applicò a L-AF1 («+16,0 fuori di 0,5 ⇒ sonda dovuta») pretende qui una rimisura su stash fermi prima di scrivere «~5,0» a registro.
2. **Doppia pre-registrazione contraddittoria dell'UB AL2.** Sonda AF1 p.6(b): «autoload k=1 → hostcall» ⇒ [9,5;14,5]; criterio AL2 p.0/p.4: closure-vec ⇒ [15;19]. Riclassificazione mai dichiarata. Il verdetto SOTTO-MODELLO è invariante (5,0 sotto entrambe), ma con l'arbitrato census che decide «coeff PER-SITO» post-hoc (tre siti, tre coefficienti: 12,0 · 17,0 · ~5,0) il bivio UB p.4 non falsifica più nulla per le leve future.
3. **Sonda AF1: bivio (b) al bordo e già superato.** |17,0−12,0|=5,0 vs gate 4,5: margine 0,5 ns (mezzo tick). Lo sdoppiamento a DUE famiglie è demolito in-sessione dal «per-sito» di AL2: a registro deve andare la tabella per-sito, non le due famiglie. La rimisura +17,0 è indipendente in finestra ma sugli STESSI binari stash del A/B s160 (esclude rumore di finestra, non idiosincrasia di build); drift +1,0/+2,0 in banda: solida.
4. **ORM: la LETTURA non nomina l'assoluto NEGATIVO.** Il blocco meccanico (riga 24) dà Delta assoluto=[-2,12;-0,17] s — phpr PEGGIORA in assoluto mentre il canonico normalizzato dice GIÙ [+0,72;+1,85]. La lettura chiama il GIÙ «artefatto» ma non mette a verbale questa tensione. Il non-chiudere è corretto.
5. **Pair t11 al minimo esatto e stesso punto cieco.** N=4 = bordo del criterio p.4; banda_ON dichiarata da integrare. L'argomento della lettura ORM («flag per-gamba cieco al sistemico») vale anche per le gambe 1-4 (oracle ictx 1145-1152 ovunque); il COMPATIBILE regge solo perché il rapporto stessa-gamba elide contaminazione simmetrica.
6. **Smoke AL2: «GUARDIA MORDE» (re: −4,6 vs −4,5) superato senza arbitrato dichiarato** — il census arbitrò solo la banda; il R=5 (re −3,3 ok) assolve ex post.

## Azioni
1. S-162: rimisura AL2 su stash FERMI (pin s160 vs s161), stile sonda AF1 → cifra 5,0±IC propria; fino ad allora PERF_MAP: «direzione firmata, magnitudine non tarata».
2. Registro famiglia → tabella PER-SITO; classificazione del cammino fissata in UN solo posto; conflitto sonda-p.6/criterio-AL2 a verbale.
3. Rimisura ORM in finestra quieta (dovuta) col companion ASSOLUTO nominato nella lettura + tentativo integrativo pair a ≥5 gambe pulite.
4. Emendare lo smoke: guardia che morde a R=2 pretende arbitrato dichiarato come la banda.
5. Conferma post-pin: pretendere rumore ≤ attesa/2 o dichiararla «solo segno».
