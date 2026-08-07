<?php // held-out 2 (S-111): error-path e coercizioni con diagnostica — CONGELATO
$caught = 0; $sum = 0; $data = ["a"=>1, "b"=>2];
for($i=0;$i<6000000;$i++){
  $k = ($i % 5 < 3) ? "a" : "z$i";
  $sum += $data[$k] ?? 0;
  $sum += @("12x" + 1);
  if ($i % 997 === 0) {
    try { throw new RuntimeException("x"); }
    catch (RuntimeException $e) { $caught++; }
  }
  if ($i % 1000 === 0) {
    try { $sum += intdiv(4, ($i % 2000 === 0) ? 0 : 2); }
    catch (DivisionByZeroError $e) { $caught++; }
  }
}
echo $sum, " ", $caught, "\n";
