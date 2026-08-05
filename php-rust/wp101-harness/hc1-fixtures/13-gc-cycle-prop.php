<?php
// Fixture CICLO GC VIA PROPRIETA' (Matsakis R2 + Stogov «under-noting
// delays a destructor»): un ciclo o->p->o raccolto dal cycle collector.
// Se H-C1x sotto-nota il ricevitore, il ciclo sopravvive al collect.
// ATTESA: prima del collect 0 distruzioni; il collect raccoglie ENTRAMBI.
class N {
    public $peer;
    public function __construct(public string $tag) {}
    public function __destruct() { echo "~", $this->tag, "\n"; }
}
$a = new N("a"); $b = new N("b");
$a->peer = $b; $b->peer = $a;      // ciclo via proprieta'
unset($a, $b);
echo "pre-collect\n";              // nessun ~ prima
$n = gc_collect_cycles();
echo "collected>0:", ($n > 0 ? "yes" : "no"), "\n";
echo "post-collect\n";
// Un ciclo VIVO (handle esterna) NON deve essere raccolto:
$c = new N("c"); $c->peer = $c;    // self-cycle, handle viva in $c
gc_collect_cycles();
echo "c-alive:", $c->tag, "\n";    // c ancora vivo
$c->peer = null;                   // spezza il ciclo
unset($c);                         // ~c qui
echo "end\n";
