<?php
// S-194 N: is the object-id gap in var_dump after temporaries a namespace matter? Same statements in the global namespace.
class S { function __toString() { return " abc "; } }
class K implements Countable { function count(): int { return 7; } }
class Dbg { public $a = 1; }
echo strlen(new S), "|", trim(new S), "|", count(new K), "|", str_replace("a", "b", new S), "\n";
var_dump(new Dbg);
echo strlen(new S), "\n"; var_dump(new Dbg);
echo count(new K), "\n"; var_dump(new Dbg);
echo str_replace("a", "b", new S), "\n"; var_dump(new Dbg);
