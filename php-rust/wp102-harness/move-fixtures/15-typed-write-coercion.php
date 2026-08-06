<?php
// Fixture 15 — TYPED-WRITE-COERCION (A-ST-103-2, Stogov; aggancio §3.12).
// Scrittura su proprieta' TIPIZZATA con coercion RIUSCITA e FALLITA mentre
// l'handle del ricevitore e' quello MOSSO: il TypeError deve unwindare il
// braccio senza perdere il ricevitore ne' toccare il vecchio valore.
// ATTESA (scritta PRIMA):
//   n=42 (int)            <- "42" coerce a int
//   TypeError catturato   <- "abc" NON coercibile
//   n dopo il fallimento: 42 (il vecchio valore INTATTO)
//   TypeError array       <- array mai coercibile
//   n=1 (bool->int ok)
//   receiver vivo fino a fine script; dtor UNA volta, alla fine.
class T {
    public int $n = 0;
    public function __destruct() { echo "dtor(T) n={$this->n}\n"; }
}
$t = new T();
$t->n = "42";
echo "n=", $t->n, " (", gettype($t->n), ")\n";
try {
    $t->n = "abc";
    echo "NO-THROW (inatteso)\n";
} catch (TypeError $e) {
    echo "TypeError:string\n";
}
echo "after-fail n=", $t->n, "\n";
try {
    $t->n = [1, 2];
    echo "NO-THROW (inatteso)\n";
} catch (TypeError $e) {
    echo "TypeError:array\n";
}
$t->n = true;
echo "n=", $t->n, "\n";
echo "end\n";
