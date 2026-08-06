<?php
// Fixture 17 — LAZY-INIT CHE DROPPA L'ULTIMA REF ESTERNA (A-MA-103-3,
// Matsakis). L'inizializzatore lazy e' PHP SINCRONO in mezzo al braccio
// Prop-op: qui dentro (a) droppa l'ultima referenza esterna a un ALTRO
// oggetto (dtor mid-arm), (b) chiama gc_collect_cycles() — il collector
// gira col ricevitore in volo tenuto SOLO dall'handle mosso + slot.
// Le fixture 04/12/13 misurano finestre a conteggi old/new IDENTICI; la
// finestra -1 e' QUESTA: re-entrancy sincrona intra-arm.
// ATTESA (scritta PRIMA):
//   before
//   init-start
//   ~victim              <- l'ultima ref esterna cade DENTRO l'init
//   collected:yes        <- il ciclo orfano e' raccolto MID-ARM
//   init-end
//   x=42                 <- la lettura che ha innescato tutto completa
//   ghost-alive           <- il ghost sopravvive al collect in volo
//   end (+ dtor di L a fine script)
// CORREZIONE POST-ORACLE (S-102, registrata — l'attesa sopra era monca in
// due dettagli, la POLARITA' regge): (1) "x=" esce PRIMA dell'init (echo
// valuta gli argomenti in ordine: la riga vera e' "x=init-start...42");
// (2) ~victim compare 3 volte: 1 dall'unset + 2 dal collect (anche i nodi
// del ciclo sono Victim e i loro dtor girano DENTRO gc_collect_cycles,
// prima di "collected:yes"). phpr byte-identico all'oracle nei 2 modi.
class Victim {
    public $peer;
    public function __destruct() { echo "~victim\n"; }
}
class L {
    public $x = 0;
    public function __destruct() { echo "dtor(L)\n"; }
}
$GLOBALS['victim'] = new Victim();
$cycA = new Victim(); $cycB = new Victim();
$cycA->peer = $cycB; $cycB->peer = $cycA;   // ciclo orfano dopo l'unset
unset($cycA, $cycB);
$r = new ReflectionClass(L::class);
$o = $r->newLazyGhost(function ($obj) {
    echo "init-start\n";
    unset($GLOBALS['victim']);              // (a) ultima ref esterna: dtor QUI
    $n = gc_collect_cycles();               // (b) collector MID-ARM
    echo "collected:", ($n > 0 ? "yes" : "no"), "\n";
    $obj->x = 42;
    echo "init-end\n";
});
echo "before\n";
echo "x=", $o->x, "\n";                     // la LETTURA innesca l'init
echo "ghost-alive\n";
echo "end\n";
