# Verbale Sedia 8 — Stogov (Zend/opcache, semantica engine) — Concilio WP-103

## VERDETTO

**APPROVATO CON RISERVA.** Le due leve H-C1a/b stanno in piedi sui loro A/B e
sul census che le ha arbitrate (la mia predizione — gc_note mai su
non-refcounted in Zend — è confermata al 100%: 120.000.008/120.000.010
scalari, `hc-census-s101.out`). Ma la RETE SEMANTICA attorno alla famiglia
toccata ha **tre buchi nominabili**: le 13 fixture non contengono né il
`__destruct` che RIENTRA nel VM durante un PropSet, né la coercion al write su
proprietà tipizzata, né il `clone`. La bozza §S-102 è accettabile con gli
emendamenti sotto.

## Refutazioni capitali

**NO.** Nessuna misura promossa cade. Una riserva forte (non capitale): il
claim «fixture semantiche della famiglia» è SOVRADIMENSIONATO rispetto al
contenuto — la famiglia receiver-handle-MOVE è stata collaudata senza i tre
casi più taglienti per un handle mosso (rientranza, coercion, clone). Il gate
era necessario ma non sufficiente; si sana retro-attivamente, non si riapre
la promozione.

## Emendamenti

- **A-ST-103-1 (fixture 14, destruct-reenter-propset)**: `$o->x = $nuovo`
  dove il vecchio valore è un oggetto il cui `__destruct` rilegge/riscrive
  `$o->x`. Semantica Zend: `zend_assign_to_variable` mette il vecchio in
  `garbage` e lo distrugge DOPO che il nuovo è in sede — il `__destruct` vede
  il valore NUOVO e può ri-assegnare o `unset` la stessa proprietà. È il
  punto esatto dove l'ordine MOVE-handle + gc_note di H-C1b può divergere.
  Fixture 12 copre l'ORDINE dei destruct, non la rientranza a metà PropSet.
- **A-ST-103-2 (fixture 15, typed-write-coercion)**: `public int $i`;
  write con coercion riuscita (`"5"`→5) e fallita (il vecchio valore resta
  intatto — aggancio a §3.12, dove sul typed-REF Zend azzera). Fixture 10
  copre solo typed-uninit in lettura.
- **A-ST-103-3 (fixture 16, clone)**: `clone $o` con proprietà object/array
  (tabella copiata con addref, array condivisi CoW) + hook `__clone` che
  tocca le proprietà. L'handle mosso non deve mai aliasare la tabella del
  clone.
- **A-ST-103-4 (§3.13, fix fedele)**: Zend attribuisce la riga perché
  `zend_error` in `zend_std_read_property` legge
  `EG(current_execute_data)->opline->lineno` — l'opline della **FETCH_OBJ_R
  in esecuzione**: la riga si lega all'atto della LETTURA, mai al flush. Fix
  phpr: timbrare la riga (span del PropGet) nel diag **al momento
  dell'accodamento** in `diags`; il flush stampa la riga timbrata. È la
  stessa meccanica che `Op::LoadVar` già fa per le variabili.
- **A-ST-103-5 (forma Zend-fedele della guardia H-C1a)**: in Zend è a DUE
  livelli — (1) `Z_REFCOUNTED_P`, un bit-test su type_info
  (`i_zval_ptr_dtor`, `zend_variables.h:40-50`: gli scalari pagano UN
  branch); (2) `GC_MAY_LEAK` = `GC_INFO_MASK | GC_NOT_COLLECTABLE`
  (`zend_gc.h:84-101`): **stringhe e resource sono refcounted ma NON
  collectable** — non entrano MAI nel root buffer; le reference si
  scartocciano (`GC_TYPE_INFO==GC_REFERENCE` → `Z_COLLECTABLE_P` del val
  interno). Verificare che `gc_note` phpr escluda stringhe/resource al
  secondo livello: bufferizzarle è churn puro e in Zend non esiste.
- **A-ST-103-6 (specie per H-C1c, da pinnare PRIMA)**: (a) string INTERNED —
  rc-free, copia = puntatore, dtor no-op (`zval_ptr_dtor_str` asserisce
  `!ZSTR_IS_INTERNED`); (b) string heap — addref, mai CoW-separation;
  (c) array — addref + COW: la separazione (`zend_array_dup`) avviene alla
  SCRITTURA, osservabile con `$a=$o->arr; $o->arr[]=1;` ($a immutato);
  (d) array IS_ARRAY_IMMUTABLE — rc-free, escluso perfino dal get_gc
  (`zend_gc.h:148`); (e) object — addref puro, identità condivisa, mai
  separato. Un «copy+addref condizionale» uniforme sbaglierebbe (a) e (d).

## Kill-switch

- **KS-ST-103-1**: nessuna NUOVA leva H-C sul percorso ricevitore/proprietà
  si promuove finché le fixture 14-16 non esistono (attese scritte PRIMA,
  verdi nei 2 modi sul binario corrente).
- **KS-ST-103-2**: H-C1c NON si apre senza le fixture per specie di
  A-ST-103-6 con attese verificate sull'oracle.
- **KS-ST-103-3**: il fix §3.13 deve CANCELLARE nello stesso commit il
  carve-out `09-unset-during-read.expected-divergence.diff` — un carve-out
  che sopravvive al suo fix diventa una maschera per regressioni.
