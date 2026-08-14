<?php
error_reporting(E_ALL);
class T { public array $data = ['k' => 1]; }
$t = new T();
// scalda l'IC del sito
for ($i = 0; $i < 4; $i++) { $t->data['k'] += 1; }
var_dump($t->data['k']);
// ora la prop diventa UNINIT (typed): il sito caldo DEVE dare il fatal BP_VAR_RW
unset($t->data);
try { $t->data['k'] += 1; } catch (Error $e) { var_dump($e->getMessage()); }
