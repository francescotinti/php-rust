<?php
// S-154 L-CE1 — giudice micro class_exists (forma Doctrine: FQCN mixed-case
// ESISTENTE, autoload default true, hit-path). N letterale nel loop (l'awk
// generico lo estrae). Parità: stampa il conteggio dei true.
namespace Doctrine\Tests\Models\CMS { class CmsUserLikeProbeEntity {} }
namespace {
    $name = 'Doctrine\\Tests\\Models\\CMS\\CmsUserLikeProbeEntity';
    $acc = 0;
    for ($i = 0; $i < 10000000; $i++) {
        if (class_exists($name)) { $acc++; }
    }
    echo "CE-OK $acc\n";
}
