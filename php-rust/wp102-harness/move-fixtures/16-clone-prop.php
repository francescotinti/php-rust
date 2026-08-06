<?php
// Fixture 16 — CLONE (A-ST-103-3, Stogov).
// `clone` fa addref della tabella + CoW per gli array; __clone tocca le
// proprieta' del CLONE. L'handle MOSSO dell'originale non deve mai
// aliasare la tabella del clone (una mutazione post-clone su uno dei due
// non deve trasparire nell'altro per gli array; DEVE trasparire per gli
// oggetti condivisi shallow).
// ATTESA (scritta PRIMA):
//   __clone hook vede tag=orig, poi lo cambia
//   $c->arr non vede il 99 del clone; $d->arr non vede il 100 dell'orig
//   $c->obj->v == 2 (shallow: stesso oggetto)
//   dtor: uno per c e uno per d a fine script
class C {
    public $arr = [1, 2, 3];
    public $obj = null;
    public $tag = "orig";
    public function __clone() {
        echo "__clone sees tag={$this->tag}\n";
        $this->tag = "copy";
        $this->arr[] = 99;         // CoW: separa QUI la tabella del clone
    }
    public function __destruct() { echo "dtor({$this->tag})\n"; }
}
$c = new C();
$c->obj = new stdClass();
$c->obj->v = 1;
$d = clone $c;
$c->arr[0] = 100;                  // post-clone: solo nell'originale
$d->obj->v = 2;                    // shallow: condiviso
echo "c.tag=", $c->tag, " d.tag=", $d->tag, "\n";
echo "c.arr=", implode(",", $c->arr), "\n";
echo "d.arr=", implode(",", $d->arr), "\n";
echo "c.obj.v=", $c->obj->v, "\n";
echo "end\n";
