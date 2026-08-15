<?php
// S-142 az.rev. #4 — parità teardown Hashed/tombstoni/annidati vs oracle (byte-compare).
// Ordine dei __destruct = sonda OSSERVABILE dell'ordine di drop (Key prima del valore
// non è osservabile da PHP; l'ordine tra ELEMENTI sì).
class D { public $n; function __construct($n){$this->n=$n;} function __destruct(){ echo "d:", $this->n, "\n"; } }

function packed_scalars() {
  $a = [1, 2.5, "x", "long-".str_repeat("y",40), true, null];
  $a[] = 7;
  return count($a);
}
function hashed_mix() {
  $a = [];
  $a["alpha"] = 1; $a["beta"] = "s"; $a[42] = new D("h42"); $a["gamma"] = 3.14;
  unset($a["beta"]);                 // tombstone su str
  $a["delta"] = str_repeat("z", 100);
  unset($a[42]);                     // tombstone su oggetto: destruct QUI
  $a["eps"] = new D("heps");
  echo "count:", count($a), "\n";
  echo serialize(array_keys($a)), "\n";
}                                    // teardown Hashed con 2 tombstoni: d:heps
function nested() {
  $a = ["k1" => [1,2,[3,4,["deep"=>new D("n1")]]], "k2" => ["x"=>["y"=>"v"]]];
  $b = [ ["h" => ["a"=>1]], [5, 6] ];
  unset($a["k1"]);                   // drop di subtree annidato con oggetto
  echo serialize($b), "\n";
}                                    // teardown misto packed/hashed annidati
function shared_rc() {
  $a = ["p"=>[1,2,3], "q"=>new D("sh")];
  $c = $a;                            // refcount>1: il primo unset NON droppa
  unset($a);
  echo "alive:", $c["q"]->n, "\n";
}                                    // drop reale al ritorno: d:sh
echo packed_scalars(), "\n";
hashed_mix();
nested();
shared_rc();
$g = ["s1"=>new D("g1"), "s2"=>["t"=>new D("g2")]];
unset($g["s1"]);
echo "end\n";
// teardown di shutdown: d:g2
