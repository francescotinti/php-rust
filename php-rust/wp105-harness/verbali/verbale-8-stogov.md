# Verbale sedia 8 — STOGOV (Zend engine / semantica PHP) — Concilio WP-105

Giudico S-103 e la bozza §S-104. Probe odierne: oracle 8.5.7 brew + phpr
`~/Claude/php-rust-output/release/phpr` (HEAD 56a2174); C in `php-8.5.7/Zend/`.

## VERDETTO

S-103 è solida nel METODO (censimento con controlli positivi, denominatore
riletto dal sorgente, fixture rossa fuori dai gate, assert riposizionato con
correzione di rotta documentata). Ma DUE claim di catalogo sono FALSI come
enunciati e li ho refutati OGGI con probe: §3.12 «AssignOp fallito ⇒ Zend
azzera» vale solo in UN regime su tre; §3.13 «riga della LETTURA» non porta
l'UNITÀ (include/eval divergono ADESSO). La bozza S-104 è approvata
nell'ordine, emendata sotto.

## Refutazioni

**R1 — §3.12: il censimento 4/4 è tutto dentro UN solo regime.** Meccanismo
verificato nel C: su FAILURE dell'op, `add_function_slow` scrive
`ZVAL_UNDEF(result)` (zend_operators.c:1182-1184); i helper typed
(`zend_binary_assign_op_typed_ref`/`_typed_prop`, zend_execute.c:1673/1693)
passano z_copy=UNDEF al verify. NON è un azzeramento: è UNDEF ri-coercito in
weak mode allo «zero» del tipo dichiarato. Tre regimi, provati sull'oracle:
(i) op-lancia + weak ⇒ int 0, float 0.0, `?int`→int(0) (NON null);
(ii) op-lancia + `strict_types=1` ⇒ **CONSERVA** (i==1: parità phpr GIÀ oggi);
(iii) op-riesce + verify-fallisce (`$t->i .= "x"` ⇒ "Cannot assign string…")
⇒ **CONSERVA**. Le 4 specie censite sono tutte `+= "abc"` = regime (i).
Mancano al perimetro: braccio strict, tipi non-int, `.=` su int, static
DIRETTA senza ref, incdec typed a overflow (famiglia d'errore propria).
`list()`/`??=` non sono AssignOp (assign semplice conserva): entrano come
controlli positivi, non come specie.

**R2 — §3.13: la marca porta la riga, NON l'unità.** Probe: read undef-prop
in file incluso ⇒ oracle `inc313.php on line 3`; phpr `main313.php on line 3`
(file del FLUSH, riga della lettura: mezzo-giusto = sbagliato). Eval ⇒ oracle
`main313.php(4) : eval()'d code on line 1`; phpr `main313.php on line 1`
(pseudo-file assente). La coerenza HTTP (cb2 36/41) non copre i flussi
cross-unit.

**R3 — generator: «container + descend delle catture» NON è fedele.**
`zend_generator_get_gc` (zend_generators.c:427) + `zend_generator_frame_gc`
(:399) descendono value/key/retval/values + l'INTERO frame sospeso via
`zend_unfinished_execution_gc_ex` (CV, args extra, This, closure object,
symtable, frozen call stack) + `node.parent` (yield-from); CURRENTLY_RUNNING
⇒ tabella vuota; generator chiuso ⇒ ancora value/key/retval (+closure obj).
La fixture rossa morde solo il sentiero closure-use: un fix catture-sole
andrebbe VERDE lasciando vivi i leak via CV locale attraverso yield, via
$this, via yield-from. Birth-track non è il modello Zend.

**R4 — A-ST-104-4: la mia refutazione è SCIOLTA.** `gc_check_possible_root`
(zend_gc.h:88) scartoccia ESATTAMENTE UNA volta il GC_REFERENCE poi roota il
valore se collectable: Zend stesso riceve Ref al check-root; «i Ref non si
annidano» è l'invariante giusto e i tipi dei sink lo impongono meglio
dell'assert. Riserva: il debug_assert nel descend CAMPIONA (gira solo col
GC), non impone — l'invariante va garantito ai siti di CREAZIONE dei ref.

## Emendamenti

- **A-ST-105-1**: rititolare §3.12 coi TRE regimi; estendere la probe
  (strict-arm, float/`?int`/string, `.=`, static diretta, incdec typed).
- **A-ST-105-2**: il fix fedele replica la catena UNDEF→verify-weak per
  tipo dichiarato; un fix che scrive «0» letterale è fedele solo per int.
- **A-ST-105-3**: marca diagnostica = coppia (unit, line); pseudo-file eval
  `%s(%d) : eval()'d code`; fixture include+eval prima di ridichiarare
  §3.13 chiusa.
- **A-ST-105-4**: fix generator = get_gc COMPLETO (R3), con fixture sorelle
  CV-locale, $this, yield-from; braccio running e braccio closed.
- **A-ST-105-5**: debug_assert nested-Ref anche ai make-ref.

## Kill-switch

- **KS-ST-105-1**: nessun fix §3.12 atterra senza i bracci strict e `.=` nel
  gate — altrimenti si azzera dove Zend CONSERVA (regressione di fedeltà).
- **KS-ST-105-2**: la fixture generator non entra verde nei gate finché non
  esistono le sorelle CV/$this: un verde sulla sola cattura è VOID.
- **KS-ST-105-3**: ogni estensione della disciplina-marca senza campo unit
  è respinta.

## Priorità S-104

Confermo l'ordine bozza 1→2→3 (verdetto A/B, leva H-C2, H-D SiteTag: la
spina dorsale è la performance). Punto 4: se si sceglie fedeltà, SOLO col
modello get_gc completo. §3.11/§3.12: assenza consapevole ammissibile ORA,
ma il catalogo va corretto coi regimi PRIMA (una regola falsa a catalogo è
peggio dell'assenza). Igiene: probe include/eval (R2) nei fixture-gate.
