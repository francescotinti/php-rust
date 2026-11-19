# Revisione S-192 (lente: semantica) — passo 2 della leva K0, commit 0ac0b35b. Generato con assistenza AI (Claude Fable 5.2).
**Mandato**: invalidare il claim «quando `gc_note_this_tail` risponde `true`, lo stato lasciato (buffer, cursore, flag per oggetto, idle, radici di ciclo, refcount) è IDENTICO a quello della catena `gc_note_this → gc_note_receiver → gc_note → gc_note_slow` ramo (iv-b)». Revisore indipendente, sola lettura statica (nessuna build, nessun test).
**Verdetto: REGGE** — nessuna differenza di stato trovata.
1. Effetti saltati: nessuno. `is_gc_container` è puro; il ramo `destructed && !created` è irraggiungibile (il cammino corto esce con `false` se `destructed`); `take`/rimessa di `this` è una move senza effetto sul conteggio; sul ramo (iv-b) l'originale non tocca idle, pos, birth, radici né il buffer.
2. Riceventi non-Object (`Ref`, `Closure`, …): il pattern fallisce, catena originale. Per un Object non distrutto i tre test (destructed, buffered, coda per `ptr_eq`) sono gli stessi; l'ordine è irrilevante (letture pure).
3. Borrow: un prestito condiviso al posto di due, rilasciato prima di leggere il buffer; un `borrow_mut` pendente farebbe panic in entrambi.
4. Condizione sul cursore: identica token per token.
**Riserve (nessuna è una differenza di stato)**:
- a. I contatori `gc-census`/`zval-census` non vedono queste note (già dichiarato nel commento e nel criterio).
- b. L'equivalenza regge per DUPLICAZIONE del ramo (iv-b): se `gc_note_slow` cambia quel ramo, il cammino corto diverge in silenzio e nessun test lega le due copie. **Aperta per nome**: un dente che confronti i due cammini, o un commento-ancora nel ramo (iv-b) a saldo di righe.
- c. `gc_note_frame` (Ret generale) non passa dal cammino corto: lì nulla cambia, e nulla guadagna.
**Non coperto dalla revisione**: le cifre (sono direzione, non verdetto: gemelli da archivio, Data 8G), e il censimento h2 (soli conteggi).

## Revisione di CHIUSURA (lente: misura) — 19/11, revisore indipendente, sola lettura
**Mandato**: invalidare il claim «sei leve nel tree; la più grossa (registro `IdMap`, debbca2f) −4,84 % user su una gamba ORM a gemelli; tutto direzione, notte a gruppo dovuta». **Verdetto: REGGE CON RISERVE.**
1. REGGE — il costruttore dei gemelli è sano: tre volte l'hash di L coincide con lo Z della corsa successiva (ad2d48ec, 51f754b2, 945ef024) da commit diversi solo per documenti: né l'ordine di build né la cache entrano nel binario.
2. REGGE — il −4,84 % come direzione: tutti gli L (33,59–34,19 s) sotto tutti gli Z (35,35–35,65 s), dispersione 0,85 %, righe `Tests:` identiche; conferma indipendente nei campioni totali dei profili (38.415 → 36.519, −4,9 %).
3. INDEBOLISCE — magnitudine non ripartita (la spiegazione «cache dei chiamanti» non è misurata): SANATA, p.8 del criterio GC riscritto.
4. INDEBOLISCE — «36,5 → 33,8 s» è un contrasto fra corse e binari diversi; la somma degli appaiati (−10,2 %) supera l'osservato (−7,7 %), quindi i «non risolti» sono sovrastimati: SANATA nel verbale (mediane 36,66 → 33,82, «direzione fra corse diverse»).
5. INDEBOLISCE — «memo + `array_diff` ≈ −1,4 %» nel NEXT era la corsa `s192abc` (con la leva A poi ritirata, senza 0eca36da): SANATA («tempo NON risolto»).
6. INDEBOLISCE — il giudice di meccanismo non ha un rumore misurato: la quota di una voce oscilla ~0,5 punti fra profili (la leva A «sale» pur togliendo lavoro); reggono solo i salti grandi (`BTreeMap` → 0, 911 → 330, 811 → 324), non hasher (−0,5 punti) né la leva C. **APERTA**: due profili dello stesso binario come banda.
7. INDEBOLISCE — igiene solo in prosa (nessun `.out` dei gemelli registra disco, carico, language server) e nessuna guardia non-bersaglio rieseguita dopo `IdMap` (micro, K0/mcalls, WordPress). **APERTA**: riga d'igiene e giro Z-contro-Z nello script dei gemelli; guardie e micro nella notte sul canonico PRIMA della promozione.

