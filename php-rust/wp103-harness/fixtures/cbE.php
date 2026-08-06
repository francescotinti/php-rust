<?php
/**
 * cbE.php — cella ERRORE-POI-SUCCESSO (Concilio WP-104, ordine §1).
 *
 * Scopo: una richiesta che ACCODA una marca diagnostica (warning
 * undef-prop, canale §3.13) e poi MUORE su un errore fatale non deve
 * sporcare la richiesta successiva sullo stesso worker: diag_line_marks
 * è stato aggiunto allo stato VM in S-102 e lo stato pendente va
 * azzerato al confine richiesta. Giudizio nel launcher: cbE byte-id
 * all'oracle php -S, POI cb2 sullo STESSO worker (workers=1) byte-id
 * alla sua referenza — se una marca sopravvive, la riga del warning di
 * cb2 diverge.
 *
 * DETERMINISTICO: nessun oid, nessuna entropia; html_errors=0.
 */
ini_set('display_errors', '1');
ini_set('html_errors', '0');
error_reporting(E_ALL);

class E {}

echo "CBE:begin\n";
$e = new E();
$x = $e->gone + 1;
echo "CBE:x=$x\n";
cbE_undefined_function_deliberata();
echo "CBE:never\n";
