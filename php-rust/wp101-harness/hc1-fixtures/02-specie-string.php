<?php
// Fixture specie STRING CONDIVISA (Bak): la cura giusta per int puo' essere
// sbagliata per string — qui il valore letto DEVE essere una copia logica
// (COW): mutare la proprieta' DOPO la lettura non cambia il letto.
// ATTESA: a=abc (immutato dopo la scrittura), prop=abcXYZ, b=abc, len 3/6.
class S { public $p = "abc"; }
$o = new S;
$a = $o->p;            // lettura condivisa
$o->p .= "XYZ";        // muta la proprieta'
echo $a, "\n";         // abc  (il letto NON segue la mutazione)
echo $o->p, "\n";      // abcXYZ
$b = $o->p;
$o->p = "abc";
echo $b, "\n";         // abcXYZ (idem, direzione opposta)
echo strlen($a), " ", strlen($b), "\n";  // 3 6
// Interning/condivisione: due proprieta' che puntano alla stessa stringa.
$o2 = new S; $o2->p = $o->p;
$o->p .= "!";
echo $o2->p, "\n";     // abc (non segue)
echo $o->p, "\n";      // abc!
