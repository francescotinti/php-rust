<?php
// S-156 HD2-hostcall — giudice micro args-Vec dei host-builtin: 2 hostcall
// per iter a corpo 0-alloc PROVATO (sonde fe/ce-true S-155: k=1 = solo
// args-Vec): class_exists 2-arg autoload=false (hit ≤64 B) + function_exists
// 1-arg. N letterale nel loop. Parità: stampa il conteggio dei true.
namespace Doctrine\Tests\Models\CMS { class HostArgsProbeEntity {} }
namespace {
    $cn = 'Doctrine\\Tests\\Models\\CMS\\HostArgsProbeEntity';
    $acc = 0;
    for ($i = 0; $i < 10000000; $i++) {
        if (class_exists($cn, false)) { $acc++; }
        if (function_exists('strlen')) { $acc++; }
    }
    echo "HA-OK $acc\n";
}
