# Revisione S-127 — lente PROCESSO

## Reperto principale (la cosa che più indebolisce il claim)
L'attestazione finale di promozione cita una fonte che la contraddice. `s127-promozione2.sh` chiude con `echo 0 > "$OUT/rc2"` ma la riga di verdetto dice «PROMOZIONE COMPLETA rc=0 **(da promo-out/rc)**» — e `promo-out/rc` contiene **1** (la morte al corpus della catena-1). L'rc autoritativo vive in `rc2`, mai nominato nel verdetto committato; `promo-out/` è per giunta in `.gitignore`, quindi la provenienza dichiarata è insieme sbagliata e non verificabile dal repo. I gate sono realmente verdi (batteria.rc=0, corpus2-rc=0, fixture, ORM, hk), ma l'atto che sigilla «gate pieni» ha la tracciabilità rotta: chi audita `promo-out/rc` legge un fallimento.

## Reperti secondari
- «Criterio PRIMA della forma» non è provato da git: `s127-criterio-ab.md` entra in 1b6be91, lo STESSO commit che nomina la forma (e la cita); provabile solo «prima del codice» (a64721b).
- Il verdetto avverso smoke-1 è committato solo INSIEME all'emenda delle guardie (078161f): nessuna prova d'ordine che il verdetto preceda la riscrittura del giudice; idem smoke2+A/B nello stesso commit dda02e1 (§10 vuole un commit per passo).
- Nell'arbitro le SL di objchurn/objmap sono 0,80 «prestate» (categorie senza banda storica): inerti (max con 4) ma non derivate; commento stale «(wp97)» sopravvive all'emenda.
- REGOLE §5 aggiornata a 1414 (e01e605) PRIMA del rerun rc=0 della catena-2: la cifra congelata è cambiata sulla fiducia del rerun, che poi ha confermato.

## Vagliate e respinte
- Emenda guardie = giudice cambiato dopo il verdetto? No: le bande SL erano già nel dizionario di a64721b, native dei giudici scalati; l'emenda ricolloca il giudice sulla metrica della banda (diff 6 righe), dichiara, conserva smoke-1, riesegue smoke2 — conforme §3/§5; prova meccanica: arr 9150→149,8 ns/iter.
- Binario B «a mano»: l'arbitro verifica l'hash 834f5e01 a ogni run e la catena riproduce il candidato al byte via ricetta prima del pin da `pin-phpr.sh` — artefatto di lavoro, non pin.
- Soglia giudice: 10,0 = spread gamba B su R=5 (0,03 s/3M), derivata come pre-registrato; Δ +250 la supera 25×. Incidente LSP: contato, dichiarato, verdetto contaminato conservato (3d9acb2).

## Azioni S-128
1. Correggere s127-promozione2.sh: citare `promo-out/rc2` (o un rc per-catena nominato) nella riga finale; ri-emettere la riga del verdetto con la fonte giusta.
2. Regola operativa: il verdetto cita SOLO il file rc che il proprio script scrive; check testuale nello script.
3. Verdetto avverso committato PRIMA dell'emenda che ne discende (due commit distinti); vale anche smoke2 prima dell'R=5.
4. Derivare o dichiarare «default 4 ns, nessuna banda» per objchurn/objmap; ripulire il commento wp97 nell'arbitro.
5. Nei claim di chiusura usare la formula provabile da git: «criterio prima del CODICE», non «prima della forma».
