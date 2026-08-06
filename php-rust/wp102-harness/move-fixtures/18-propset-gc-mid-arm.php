<?php
// Fixture 18 — PROPSET CON COLLECTOR ATTRAVERSATO MID-ARM SU OLD CICLICO
// (A-MA-103-3 variante, Matsakis). Il vecchio valore sovrascritto e'
// CICLICO e il suo dtor (che parte dentro il braccio PropSet) chiama
// gc_collect_cycles(): il collector cammina il grafo mentre il ricevitore
// e' in volo con l'handle MOSSO. L'osservatore -2>in_edges deve vedere
// l'handle del braccio come holder ESTERNO (ricevitore MAI raccolto).
// ATTESA (scritta PRIMA):
//   begin
//   ~trigger sees new    <- garbage-poi-dtor anche qui
//   collected:yes        <- il ciclo orfano raccolto DENTRO il braccio
//   holder-alive         <- il ricevitore in volo sopravvive al collect
//   slot=replacement
//   end (+ dtor residui a fine script)
// CORREZIONE POST-ORACLE (S-102, registrata): i due ~cyc del ciclo raccolto
// escono DENTRO gc_collect_cycles, PRIMA di "collected:yes" (i dtor girano
// al collect, non dopo). phpr byte-identico all'oracle nei 2 modi.
class Cyc {
    public $peer;
    public function __destruct() { echo "~cyc\n"; }
}
class Trigger {
    public $h;
    public function __construct($h) { $this->h = $h; }
    public function __destruct() {
        echo "~trigger sees " . (is_string($this->h->slot) ? $this->h->slot : "?") . "\n";
        $n = gc_collect_cycles();           // collector MID-ARM
        echo "collected:", ($n > 0 ? "yes" : "no"), "\n";
        echo "holder-alive\n";
    }
}
class Holder {
    public $slot = null;
    public function __destruct() { echo "dtor(holder)\n"; }
}
// ciclo orfano pronto per il collect
$a = new Cyc(); $b = new Cyc();
$a->peer = $b; $b->peer = $a;
unset($a, $b);
$h = new Holder();
$h->slot = new Trigger($h);
echo "begin\n";
$h->slot = "replacement";                   // dtor(Trigger) DENTRO il braccio
echo "slot=", $h->slot, "\n";
$h->slot = null;
echo "end\n";
