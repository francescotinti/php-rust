<?php
// S-154 fx-ce — fedeltà class_exists/interface_exists/trait_exists post L-CE1
// (criterio ce1 p.7): case-insensitive, leading-\, interface, trait,
// autoload=false, nome >64 B (heap path), autoloader invocato una volta.
namespace Fx\Ce { class MixedCaseKlass {} interface IfaceX {} trait TraitX {} }
namespace {
    var_dump(class_exists('Fx\Ce\MixedCaseKlass'));
    var_dump(class_exists('FX\CE\MIXEDCASEKLASS'));
    var_dump(class_exists('\Fx\Ce\MixedCaseKlass'));
    var_dump(class_exists('Fx\Ce\IfaceX'));
    var_dump(interface_exists('Fx\Ce\IfaceX'));
    var_dump(class_exists('Fx\Ce\TraitX'));
    var_dump(trait_exists('Fx\Ce\TraitX'));
    var_dump(class_exists('Fx\Ce\Nope', false));
    spl_autoload_register(function ($n) { echo "AL:", $n, "\n"; });
    var_dump(class_exists('Fx\Ce\NopeTwo'));
    var_dump(class_exists(str_repeat('A', 70)));
    var_dump(class_exists('Fx\Ce\MixedCaseKlass', false));
}
