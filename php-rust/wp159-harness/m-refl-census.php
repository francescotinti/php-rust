<?php
// S-159 sonda — driver CONTEGGI census: COPIA di wp158-harness/m-refl.php con
// N=200000 (adattamento DICHIARATO, criterio s159-criterio-sonda.md p.3).
// Parità attesa: RF-OK 400000 (2 incrementi/iter).
class ReflJudgeCls { public function judgeMethod(int $x): string { return "j"; } }
$acc = 0;
for ($i = 0; $i < 200000; $i++) {
    $mi = __reflect_method_info('ReflJudgeCls', 'judgeMethod');
    if ($mi !== false) { $acc++; }
    $rn = __reflect_class_real_name('refljudgecls');
    if ($rn === 'ReflJudgeCls') { $acc++; }
}
echo "RF-OK $acc\n";
