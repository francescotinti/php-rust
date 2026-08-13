# Revisione S-135 — lente SEMANTICA (revisore singolo)

## Reperto principale
Nessun controesempio: il claim di equivalenza REGGE su tutti i probe eseguiti.
Il reperto più forte è di collaudo, non di codice: la sonda di fedeltà
(criterio p.7) NON ha verdetto agli atti — nessun file registra il run di
`fixtures-ap1.php` («BYTE-ID al pin» è dichiarato solo nel session file) — e la
forma «Busy-cycle» promessa da p.7 NON esiste nella fixture. Il ramo
`LeafWrite::Busy` replicato nel fast path (run.rs:2424-2435) non è mai stato
esercitato; analisi: verosimilmente IRRAGGIUNGIBILE (nel fast path nessun guard
di walk è vivo e il borrow del base cell termina prima del drain, quindi
`try_borrow_mut` non può fallire senza rientranza) — replica letterale
corretta, ma codice difensivo morto e non collaudato.

## Reperti secondari
1. p.2 «stessi passi, stesso ordine» è FALSO alla lettera: il pieno fa
   `make_mut` PRIMA di `coerce_key_diag` (mod.rs:17233-17238), il fast il
   contrario (run.rs:2407-2416). Osservabilmente equivalente (`make_mut` è
   muto; su chiave illegale il pieno de-condivide l'array prima del TypeError,
   il fast no — CoW non osservabile). Emendare la lettera, non il codice.
2. La fixture s8 sta su UNA riga: maschera il difetto pre-esistente di
   attribuzione riga dei diag — probe: phpr attribuisce il Deprecated
   float-key alla riga dello statement SUCCESSIVO (28 vs 27 oracle).
3. Divergenze vs oracle confermate identiche su pin E stash (pre-esistenti,
   non-leva): TypeError «Illegal offset type» vs «Cannot access offset of type
   array on array»; deprecation 8.5 «null as array offset» non emessa; «false
   to array» non emessa. Non tutte a catalogo.
4. Emenda rev. S-112: VERIFICATO che r2 ha riapplicato TUTTO il criterio
   (giudice R=5 rieseguito, riconciliazioni, guardie a formula nuova; r1 rc=5
   agli atti). Nota: D=+56,7 vs UB 57,7 — dentro banda per 1,0 ns.

## Vagliate e respinte
Probe ESEGUITI su oracle + pin s135 + stash s134 (pin==stash BYTE-ID ovunque;
scratchpad rev135-p1/p2.php): distruttore dell'elemento spostato (timing ==
oracle); spostamento attraverso elemento Ref; valore d'espressione con RHS
alias; chiave illegale su array condiviso (stato dopo == oracle); next-key dopo
set int alta + append e PHP_INT_MAX; auto-assegnazione `$a['k']=$a`; chiavi
stringa di bordo ("08","-0"," 5"); superglobali `$_GET`/`$GLOBALS`; ciclo
`$a[0]=&$a; $a[0]=7` (base diventa 7 == oracle). Fixture 14 sezioni rieseguita
sui tre binari: pin==stash byte-id, divergenze oracle tutte pre-esistenti.

## Azioni S-136
1. Registrare la sonda di fedeltà con script + verdetto + rc agli atti;
   assorbire i probe del revisore come sezioni nuove della fixture.
2. Emendare a verbale s135-criterio-ap1.md p.2 (ordine make_mut↔coerce
   invertito; equivalenza osservabile argomentata, non «stessi passi»).
3. Ramo Busy del fast path: o commento d'irraggiungibilità + contatore che
   denuncia se mai morde, o collaudo costruito.
4. Catalogare in PHPR_DIVERGENCES_FROM_PHP.md: messaggio TypeError offset,
   deprecation null-offset 8.5, attribuzione riga diag allo statement dopo.
5. Spezzare le fixture multi-statement su righe separate (espone
   l'attribuzione di riga dei diag).
