<?php
// Fixture 19b — BASE=1: RICEVITORE TEMPORANEO (RC-HO-104, Concilio WP-104
// ordine §2). L'audit INV-RECV-1 provava l'invariante solo sotto base=2
// (created + slot): con `(new C)->x` la base è 1 e il conteggio mid-arm
// post-move scende a 2 — la zona degli osservatori `==2` MAI esaminati
// (siti OBS-4/OBS-6) e del collector OBS-8. Qui il temp in volo entra in
// un self-cycle dentro __get e il collector gira MID-ARM: se l'handle
// mosso non contasse come holder esterno, il temp verrebbe raccolto con
// __get ancora sulla pila (dtor anticipato, output divergente).
// ATTESA (scritta PRIMA):
//   begin
//   collected:no        <- il temp in volo è VIVO (holder esterno = braccio)
//   alive-in-get
//   x=V
//   after
//   ~C                  <- dtor DENTRO il collect finale
//   final-collect:yes
//   end
// CORREZIONE POST-ORACLE (S-103, registrata, COSMETICA): l'echo
// multi-argomento stampa "x=" PRIMA di valutare (new C)->x ⇒ le righe
// reali sono «x=collected:no / alive-in-get / V». L'ordine SEMANTICO dei
// marker era esatto. phpr byte-id all'oracle nei 2 modi.
class C {
    public $peer = null;
    public function __get($name) {
        $this->peer = $this;            // self-cycle sul temp in volo
        $n = gc_collect_cycles();       // collector MID-ARM, base=1
        echo "collected:", ($n > 0 ? "yes" : "no"), "\n";
        echo "alive-in-get\n";
        return "V";
    }
    public function __destruct() { echo "~C\n"; }
}
echo "begin\n";
echo "x=", (new C)->x, "\n";
echo "after\n";
$n = gc_collect_cycles();
echo "final-collect:", ($n > 0 ? "yes" : "no"), "\n";
echo "end\n";
