<?php
$s=0;
for($i=0;$i<20000;$i++){
  eval("class EC$i { public int \$a=0; public ?string \$b=null; public function m0(\$x){return \$x;} public function m1(int \$x): int {return \$x+1;} public function m2(){return \$this->a;} public function m3(\$x,\$y){return \$y;} public function m4(): string {return 'k';} public function m5(){return 5;} }");
  $c = "EC$i"; $o = new $c; $s += $o->m1($i & 7);
}
echo $s,"\n";
