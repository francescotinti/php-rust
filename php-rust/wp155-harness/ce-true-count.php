<?php
// S-155 az.rev. #2 — sonda ce-count in forma autoload=true 1-arg (driver =
// m-classexists ridotto: FQCN mixed-case ESISTENTE 45 B ≤64, hit-path via
// resolve_class_autoload). Attesa: k=1 (solo args-Vec, 1 arg → b=16 B).
// Due N via env CEN. CONTEGGI, mai tempo.
namespace Doctrine\Tests\Models\CMS { class CmsUserLikeProbeEntity {} }
namespace {
    $name = 'Doctrine\\Tests\\Models\\CMS\\CmsUserLikeProbeEntity';
    $n = (int)(getenv('CEN') ?: 100000);
    $hit = 0;
    for ($i = 0; $i < $n; $i++) {
        if (class_exists($name)) { $hit++; }
    }
    echo "CE-TRUE-OK n=$n hit=$hit\n";
}
