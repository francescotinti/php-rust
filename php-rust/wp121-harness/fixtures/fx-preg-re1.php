<?php
// fx-preg-re1 — piste calde L-RE1 congelate per NOME (az. rev. S-120 #1):
// nomi (dup (?J), prefisso sintetico __phprbg), NULL, offset, latin1, (?|),
// backref demix. Output a righe numerate: 1 caso = 1 riga.
$r = preg_match('/(?<a>\d+)-(?<b>\w+)/', 'x 42-abc', $m);           echo "1:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(?J)(?<x>a)|(?<x>b)/', 'zb', $m);                 echo "2:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(?|(a)|(b))(c)/', 'zbc', $m);                     echo "3:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(a)(b)?(c)?/', 'a', $m);                          echo "4:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(a)(b)?(c)?/', 'a', $m, PREG_UNMATCHED_AS_NULL);  echo "5:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/b/', 'abc', $m, PREG_OFFSET_CAPTURE);             echo "6:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/\d+/', '12 34 56', $m, 0, 4);                     echo "7:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match("/caf(.)-(\\d)/", "caf\xE9-9", $m);                 echo "8:", implode(',', array_map('bin2hex', $m)), "|rc=", $r, "\n";
$r = preg_match('/(a)\1/', 'aa', $m);                               echo "9:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(?<g>a)(b)\2/', 'abb', $m);                       echo "10:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(?<__phprbg1>z)/', 'z', $m);                      echo "11:", json_encode($m), "|rc=", $r, "\n";
$r = preg_match('/(?<a>q)/', 'x', $m);                              echo "12:", json_encode($m), "|rc=", $r, "\n";
