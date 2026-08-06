<?php
/**
 * cb1.php — dente CAPTURE-BOUNDARY (A-PE-103-2b, Concilio WP-103).
 *
 * Scopo: l'output emesso da __destruct/shutdown alla request_end deve
 * cadere DENTRO la finestra di capture (binding rule Pedersen/Stogov:
 * output capture PRIMA di request_end). H-C1b (MOVE dell'handle
 * ricevitore) accorcia la vita dei riferimenti forti temporanei: un
 * distruttore che scatta un tick prima emette dentro o fuori la
 * finestra — la CLI non può giudicarlo per costruzione, questo dente sì.
 *
 * Gate (nel launcher): tutti i marker presenti + byte-id tra 1ª/2ª/3ª
 * richiesta sullo STESSO worker (workers=1) + byte-id tra i due modi
 * + corpo identico all'oracle (php -S).
 *
 * Il fixture è DETERMINISTICO: nessun oid, nessuna entropia, nessun
 * warning atteso.
 */

class Probe {
    public int $x = 0;
    public string $tag;
    public function __construct(string $t) { $this->tag = $t; }
    public function __destruct() { echo "DTOR:{$this->tag}:x={$this->x}\n"; }
}

echo "CB1:begin\n";

// R0: loop scalare puro — le forme registro compaiono nel dump {main}
// (serve al mode-probe del launcher: il modo si PROVA, non si presume).
$s = 0;
for ($i = 0; $i < 100; $i++) { $s = $s + $i; }
echo "CB1:sum=$s\n";

// A: morte MID-REQUEST — l'ultima referenza cade dopo una raffica di
// Prop-op (il canale del MOVE H-C1b): il DTOR deve stampare QUI.
$a = new Probe("mid");
for ($i = 0; $i < 50; $i++) { $a->x = $a->x + 1; }
$a = null;

// B: oggetto raggiungibile SOLO via proprietà di un altro oggetto —
// teardown a catena alla request_end.
class Bag {
    public ?Probe $p = null;
    public function __destruct() { echo "DTOR:bag\n"; }
}
$bag = new Bag();
$bag->p = new Probe("inbag");
$bag->p->x = 9;

// C: oggetto vivo fino alla request_end — il suo DTOR è l'ultimo output
// del lifecycle: se la capture chiude prima, questa riga SPARISCE.
$end = new Probe("end");
$end->x = 3;

register_shutdown_function(function () {
    echo "SHUTDOWN:1\n";
    $sh = new Probe("shut");
    $sh->x = 7;
    // DTOR:shut:x=7 alla fine della closure, dentro la fase shutdown.
});

// D: reset-invariante per-richiesta (byte-id tra richieste sullo stesso
// worker: se lo static persiste, la 2ª richiesta diverge).
function cb1_counter(): int { static $n = 0; return ++$n; }
echo "CB1:req=" . cb1_counter() . "\n";

echo "CB1:body-done\n";
