<?php
/**
 * cb2.php — braccio WARNING-LINE §3.13 (KS-PE-104-1, Concilio WP-104).
 *
 * Scopo: il fix §3.13 (diag_line_marks: il warning di lettura proprietà
 * si timbra con la riga dell'op che LEGGE all'accodamento; flush_diags
 * preferisce la marca) è un cambio di RUNTIME — il collaudo S-102 era
 * CIECO a questo canale (refutazione Pedersen). Questo braccio serve la
 * famiglia PropGet-undef via HTTP e giudica la RIGA del warning al
 * confine: byte-id con l'oracle php -S + parità esplicita dei numeri
 * «on line N» + conteggio esatto dei warning (2).
 *
 * W2 è il caso discriminante: espressione multi-riga — la lettura è
 * sulla riga della proprietà, il flush può cadere sulla riga dopo.
 * Pre-fix §3.13 la riga sarebbe quella del flush: il braccio MORDE.
 *
 * DETERMINISTICO: nessun oid, nessuna entropia; html_errors=0 esplicito
 * (la parità si giudica sul testo nudo, non sul markup del SAPI).
 */
ini_set('display_errors', '1');
ini_set('html_errors', '0');
error_reporting(E_ALL);

class W {}

echo "CB2:begin\n";

// R0: loop scalare puro — forme registro nel dump {main} (mode-probe).
$s = 0;
for ($i = 0; $i < 100; $i++) { $s = $s + $i; }
echo "CB2:sum=$s\n";

$w = new W();

// W1: lettura undef su riga singola — il warning porta QUESTA riga.
$v1 = $w->missing_a + 1;
echo "CB2:v1=$v1\n";

// W2: lettura undef in espressione MULTI-RIGA (caso §3.13): la lettura
// è sulla riga di missing_b, il flush può cadere sulla riga del + 2.
$v2 = $w->missing_b
    + 2;
echo "CB2:v2=$v2\n";

echo "CB2:body-done\n";
