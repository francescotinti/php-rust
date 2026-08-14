# Revisione S-138 — lente SEMANTICA (revisore singolo)

## Reperto principale
**Le 21 fixture di fedeltà non hanno eseguito il fast path NEMMENO UNA VOLTA.** Il fill avviene solo sul ramo pieno a esito Ok dello STESSO sito (run.rs 6018-6022, 6077-6080); la PropIc è MONOMORFICA per sito (bytecode.rs: una sola Cell `(epoch,cid1,scope,slot)`). I vettori 1-19 eseguono ogni statement UNA volta (IC fredda → Miss); il v.20 chiama `bump()` una volta sola; il v.21 alterna P1/P2 sullo stesso sito → cid-mismatch PERPETUO, mai un hit. Il gate «fedeltà PRIMA del tempo» (criterio p.6, byte-id candidato==pin) ha quindi collaudato la leva ZERO volte; l'unica evidenza calda alla promozione era lo stdout banale dei due bench (`3000000`), che copre solo `+=1` Long e `++` post a valore scartato. Buco del COLLAUDO, non refutazione della semantica.

## Reperti secondari
1. Copertura calda retro-colmata in revisione: 19 vettori CON LOOP ×4 (scratchpad `attacco-rmw-hot.php`, `attacco-rmw-keys.php`): entry-Ref write-through per `+=` e `++`, valore d'uso post-inc/pre-dec, overflow `+=`/`++` a PHP_INT_MAX, aliasing `$k += $k`, `evil()` che sostituisce la prop o rebinda l'entry a ref durante il rhs, chiavi `'07' '-0' ' 7' '1e2' '9223372036854775808'` e Long negativo, `*=`/`-=` Double, basi `$this`/global/privata, COW con snapshot vivo. Esito ESATTO: `cmp` → **BYTE-IDENTICI s138-vs-oracle** su entrambi i file (rc=0/0). Nessuna divergenza → contro-verifica s136 non dovuta.
2. Asimmetria non dichiarata: il fast scrive con `field_write_walk(..., rw=false)` (arrays.rs 1727-1730), il pieno RMW con `field_set_op` (rw=true). In perimetro non ho trovato divergenza osservabile, ma l'equivalenza è ASSUNTA, non a criterio.
3. Il v.21 è etichettato «poly» come se esercitasse il sito caldo: per costruzione garantisce solo MISS.

## Vagliate e respinte (con la prova)
- «Promozione con FUORI MODELLO senza sonda»: respinta — sonda v2 eseguita e ACQUISITA (identità 177,0 vs D_AB 173,3, in banda 13,3+4).
- Conferma-smoke post-pin (emenda p.9): eseguita, |diff|=5,0 ≤ 13,3 su entrambi i giudici.
- Ref replaced-not-written-through, overflow, aliasing, chiavi non-canoniche: respinte empiricamente (byte-identici, sopra).

## Azioni S-139
1. Sezione CALDA in fixtures-rmw (loop ≥3 per vettore) + rifare byte-id candidato==pin.
2. Contatore diagnostico di hit del fast e gate che PROVI ≥1 hit per vettore caldo.
3. Provare (o allineare) l'equivalenza rw=false/rw=true in perimetro, a criterio.
4. Correggere il v.21: coda mono-classe ripetuta dopo l'alternanza.
5. Integrare i 19 vettori della batteria di revisione nell'harness.
