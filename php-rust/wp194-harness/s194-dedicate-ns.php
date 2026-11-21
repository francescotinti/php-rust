<?php
namespace Shadow { function strlen($x) { return "shadow:" . $x; } function count($x) { return "shadowc"; }
  function probe() { echo strlen("abc"), "|", count([1]), "|", \strlen("abc"), "|", \count([1, 2]), "\n"; } }
namespace Plain { function probe() { echo strlen("abcd"), "|", count([1, 2, 3]), "|", sizeof([1]), "|", \strlen("x"), "\n"; } }
