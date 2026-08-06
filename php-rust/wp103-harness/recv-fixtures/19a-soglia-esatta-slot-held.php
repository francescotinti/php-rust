<?php
// Fixture 19a — SOGLIA ESATTA MID-ARM SU SLOT-HELD (RC-MA-104 / KS-MA-104-2,
// Concilio WP-104 ordine §2). Le fixture 17-18 giravano a distanza ≥2 dalla
// soglia: il −1 del MOVE non era MAI arbitrato da un conteggio esatto.
// Qui lo slot muore DENTRO il braccio (__get azzera $h): mid-arm il
// ricevitore vive di created + handle del braccio + self-cycle —
// l'osservatore del collector (sito OBS-8, «count − 2 > in_edges») decide
// VIVO/MORTO esattamente sull'handle mosso: post-move 3−2=1 > 1 è FALSO
// se l'handle del braccio non fosse contato come holder esterno ⇒ il
// ricevitore in volo verrebbe raccolto DENTRO __get (dtor anticipato,
// output divergente). L'oracle lo tiene VIVO: la fixture MORDE il −1.
// ATTESA (scritta PRIMA):
//   begin
//   slot-dropped
//   collected:no      <- il temp in volo è holder-esterno: ciclo VIVO
//   y=W
//   after
//   ~S                <- dtor DENTRO il collect finale (lezione fixture 18)
//   final:yes
//   end
// CORREZIONE POST-ORACLE (S-103, registrata, COSMETICA): l'echo
// multi-argomento stampa "y=" PRIMA di valutare $h->x ⇒ le righe reali
// sono «y=slot-dropped / collected:no / W». L'ordine SEMANTICO dei
// marker (slot-dropped → collected:no mid-arm → W → ~S dentro il collect
// finale → final:yes) era esatto. phpr byte-id all'oracle nei 2 modi.
class S {
    public $peer = null;
    public function __get($n) {
        global $h;
        $h = null;                       // lo slot muore MID-ARM
        echo "slot-dropped\n";
        $c = gc_collect_cycles();        // collector col braccio in volo
        echo "collected:", ($c > 0 ? "yes" : "no"), "\n";
        return "W";
    }
    public function __destruct() { echo "~S\n"; }
}
$h = new S();
$h->peer = $h;          // self-cycle: il refcount non lo libera al drop dello slot
echo "begin\n";
echo "y=", $h->x, "\n";
echo "after\n";
$c = gc_collect_cycles();
echo "final:", ($c > 0 ? "yes" : "no"), "\n";
echo "end\n";
