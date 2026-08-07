# Revisione S-107 — revisore singolo, lente MISURA (REGOLE §7)

## Verdetto sul claim
La PROMOZIONE del lotto è lecita alla lettera del criterio §5 (arith 5/5 sopra soglia, nessuna regressione oltre soglia). Ma il claim «ha mosso TUTTE e sei le categorie» ECCEDE la misura: per il criterio pre-registrato (soglia = max(4; spread)), **arr NON supera** (Δ +500 su rumore A 900, 4/5 con una run negativa) e **re NON supera** (Δ 10,0 < soglia 15). Il verdetto stesso li declassa a «positivo/direzione firmata»; il claim li promuove a «mossi». Anche **str è fragile**: margine 2,5 ns su spread A=15 — una run diversa lo ribalta. Cifre difendibili: 4/6 sopra soglia; arr/re = direzione+meccanismo, magnitudine non stabilita (REGOLE §4). Le discese micro di arr/str/re (4,2→3,9 ecc.) sono letture a binario singolo, non A/B: direzione, non cifra.

## Debolezza più grave
**Il rischio icache di run_loop +16.364 B (bl 5418→5675) è dichiarato ma non misurato fuori dai micro.** La lezione H-C2 (S-104) dice che proprio questo profilo di leva cadde icache-bound; la coppia WordPress era DOVUTA in S-107 (ordine WP-108) e non è stata rimisurata: il riferimento 1,894× resta di un'altra sessione su un altro pin. Il lotto potrebbe pagare sul carico reale ciò che guadagna sui sei loop caldi.

Secondaria ma di principio: la batteria 1739 vs 1740 è liquidata per CONTEGGIO (git grep 1767=1767 sul sorgente), in contrasto con la regola «gate per NOME, mai solo conteggio» — l'inventario sorgente non prova che lo stesso insieme sia stato ESEGUITO (cfg/feature possono spegnere un test). Inoltre l'A/B è su a0543213, il pin è b4b1a87d: churn dichiarato, ma senza prova di identità del codice (.text) i Δ appartengono a una build gemella, non al pin.

## Azioni
1. Correggere claim in NEXT_SESSION/session file: «4/6 sopra soglia; arr/re direzione firmata, magnitudine non ripartita; str a margine 2,5».
2. S-108: rerun A/B mirato arr e re con rumore abbattuto (N interno maggiore, ns/inner-iter) prima di citarne cifre.
3. Nominare il −1 di batteria: diff per NOME degli eseguiti (`cargo test -- --list`) fra pin S-106 e S-107.
4. Rimisurare la coppia WordPress full sul pin 62a4df65 in apertura S-108: è il test dell'ipotesi icache, e blocca la leva successiva se fuori banda (§4).
5. Provare l'identità .text a0543213↔b4b1a87d, o marcare a verbale l'A/B come «build gemella del pin».
