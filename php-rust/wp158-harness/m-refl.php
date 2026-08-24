<?php
// S-158 L-RF2 — giudice micro args-Vec della tranche-2 __reflect_*: 2 chiamate
// convertite per iter su classe ESISTENTE (niente autoload): method_info a
// CACHE-HIT (dalla seconda iter: clone Rc dal memo) + class_real_name.
// Il corpo alloca in proprio (common-mode sui due bracci): il segnale è la
// sola args-Vec (1 alloc/chiamata). N letterale nel loop. Parità: RF-OK.
class ReflJudgeCls { public function judgeMethod(int $x): string { return "j"; } }
$acc = 0;
for ($i = 0; $i < 10000000; $i++) {
    $mi = __reflect_method_info('ReflJudgeCls', 'judgeMethod');
    if ($mi !== false) { $acc++; }
    $rn = __reflect_class_real_name('refljudgecls');
    if ($rn === 'ReflJudgeCls') { $acc++; }
}
echo "RF-OK $acc\n";
