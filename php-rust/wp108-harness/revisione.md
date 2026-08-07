# Revisione S-108 — revisore singolo, lente SEMANTICA (REGOLE §7)

## Verdetto sul claim
Il claim di esattezza semantica REGGE sulle sei piste d'attacco, verificate sul codice: (1) `PropGetSlotRecv` (run.rs:4117) replica l'ordine originale — push silente del ricevitore, POI warning del secondo read via `reg_load_slot` (run.rs:280, parità garantita dalla guardia `fold_slot`); (2) `BinaryTCPropSetPop` (run.rs:4269) computa il funnel PRIMA di poppare l'obj: su TypeError il `?` propaga con l'obj ancora in pila, e la guardia fast-path Undef/Ref è identica a `BinaryTC`; (3) `BinarySCSCDst` (run.rs:1749) valuta a→b→combine, poi `read_slot` silente e `binary_value_ab` senza fast path come `BinarySTDst`: su TypeError dell'albero il lhs non viene letto, come nell'originale; (5) le quattro finestre passano ogni op assorbita da `free()`, che esclude `blocked` (bersagli di salto + exc_table) — il bersaglio può stare solo in testa; (6) il transiente eliso da W9b non aveva `gc_note` nell'originale (il pop di PropSetPop è un `expect`, non un `Op::Pop`). Nessuna divergenza PHP-osservabile trovata.

## Debolezza più grave
La pista 4 morde: la guardia anti-interferenza di W13 controlla solo che `ops[i+2]` non sia un Binary (protezione W5), ma NON protegge il fold specchio `[PushConst, LoadVar, Cmp]`. Caso concreto: `f($a, 3 < $b);` — prima del lotto-2 emetteva `LoadVar(a); BinarySC{specchiato}`; ora W13 ruba il PushConst(3) e l'emissione diventa `LoadVarPushConst(a,3); LoadVar(b); Binary(Lt)`. Il VALORE è identico (lo specchio è order-free), quindi non è una violazione semantica — ma è una de-ottimizzazione che FALSIFICA il sub-claim scritto nel criterio e nel commento («il lotto-2 AGGIUNGE fusioni, non cambia mai l'emissione del lotto-1»). L'admission «emissione ESATTA» ha perimetro = sei giudici; questo pattern non vi compare, quindi il gate non poteva vederlo.

## Azioni
1. Estendere la guardia W13: non fondere se `ops[i+2]` è un LoadVar foldabile e `ops[i+3]` un confronto specchiabile sulla stessa riga — oppure dichiarare a verbale la nuova emissione come emendamento.
2. Dente in batteria su stream sintetico `[LoadVar, PushConst, LoadVar, Cmp]`: asserire quale emissione è quella intesa (oggi il comportamento è non pre-registrato).
3. Fixture bilaterale per W9b: `$o->p = $x . "s";` con `$x` oggetto senza `__toString` — TypeError con obj in pila, confronto con oracle.
4. Fixture W9a con slot undef e `__get` che lancia: ordine warning/eccezione vs oracle.
5. Mantenere la coppia WP S-109 (collaudo icache, +31,1 KB cumulativi) già dovuta.

---
## Recepimento (S-108, stessa sessione)
Azione-1: presa la SECONDA via — il sub-claim «mai cambiare l'emissione lotto-1» si EMENDA a verbale:
la guardia W13 protegge il braccio W5 ma NON il fold specchio; sul pattern `[LoadVar, PushConst, LoadVar, Cmp-specchiabile]`
l'emissione cambia (valore identico, de-ottimizzazione rara, assente dai sei giudici e dal corpus per NOME — corpus 1415×2
IDENTICO lo prova sul perimetro Zend). L'estensione della guardia + il dente (azione-2) + le due fixture (azioni 3-4)
sono NOMINATE nelle aperture S-109. Azione-5 già in ordine S-109 punto 1.
