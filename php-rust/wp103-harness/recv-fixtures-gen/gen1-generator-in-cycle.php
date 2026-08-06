<?php
// Fixture gen1 — GENERATOR-IN-CYCLE, fixture di MORSO (A-HO-104-1 ≡
// A-ST-104-3, Concilio WP-104: TERZA DEROGA VIETATA).
// Il buco A-HO-103-2: in `Zval::is_gc_container` il Generator non è
// trattato da container. Se il collector non attraversa il Generator,
// un ciclo che passa PER il generator (holder → gen → closure che
// cattura holder) non viene mai raccolto: dtor MAI stampato e
// collect=0. L'oracle lo raccoglie. La fixture decide: MORDE (phpr
// diverge ⇒ il buco è reale e va chiuso: container o birth-track) o
// NON morde (phpr già equivalente su questo canale).
// ATTESA (scritta PRIMA, dall'oracle atteso):
//   begin
//   step:1
//   after-unset          <- il refcount NON libera: è un ciclo
//   ~H                   <- dtor DENTRO il collect (lezione fixture 18)
//   collected:yes
//   end
// ESITO S-103 (registrato, pin d0b01362, 2 modi IDENTICI): **MORDE** —
// phpr stampa «collected:no» e ~H solo a shutdown: il ciclo via
// Generator NON è mai raccolto (leak fino a fine richiesta). Il buco
// A-HO-103-2 è REALE. Fixture ROSSA per costruzione fino al fix
// (container+descend o birth-track): NON entra in nessun gate verde;
// verdetto in gen1-verdetto.out.
class H {
    public $gen = null;
    public function __destruct() { echo "~H\n"; }
}
$h = new H();
$h->gen = (function () use ($h) {
    yield 1;
    yield 2;
})();
// ciclo: $h --prop gen--> Generator --closure use--> $h
echo "begin\n";
echo "step:", $h->gen->current(), "\n";
unset($h);
echo "after-unset\n";
$n = gc_collect_cycles();
echo "collected:", ($n > 0 ? "yes" : "no"), "\n";
echo "end\n";
