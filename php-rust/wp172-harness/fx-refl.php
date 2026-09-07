<?php
// S-158 L-RF2 — fixture parità A==B con RUOLI DISTINTI (az.rev. S-156 #4):
// per ogni nome convertito a 2+ argomenti, il caso DRITTO e il caso a
// argomenti SCAMBIATI/permutati con esito visibilmente diverso — uno swap
// d'ordine nel plumbing slice cambierebbe queste righe. Nomi interni phpr:
// contratto A==B byte-id (nessuna gamba oracle).
#[Attribute] class FxRole { public function __construct(public string $tag) {} }
class FxBase { protected function baseSecret(): void {} private function basePrivate(): void {} }
class FxCls extends FxBase {
    #[FxRole('marcatore-ruolo')] public int $roleProp = 3;
    public static bool $roleStatic = true;
    public function roleMethod(string $a, int $b): bool { return true; }
    protected function roleProt(): void {}
}
echo "== method_info (2 arg): dritto vs scambiato ==\n";
$d = __reflect_method_info('FxCls', 'roleMethod');
var_dump(is_array($d) ? [$d['visibility'], $d['static'], $d['declaringClass']] : $d);
var_dump(__reflect_method_info('roleMethod', 'FxCls'));
echo "== prop_details (2 arg): dritto vs scambiato ==\n";
var_dump(__reflect_prop_details('FxCls', 'roleProp'));
var_dump(__reflect_prop_details('roleProp', 'FxCls'));
var_dump(__reflect_prop_details('FxCls', 'roleStatic'));
echo "== prop_attr_new (3 arg): dritto vs permutati ==\n";
var_dump(__reflect_prop_attr_new('FxCls', 'roleProp', 0));
var_dump(__reflect_prop_attr_new('roleProp', 'FxCls', 0));
var_dump(__reflect_prop_attr_new('FxCls', 'roleProp', 5));
echo "== 1-arg: real_name / method_names / class_loc ==\n";
var_dump(__reflect_class_real_name('fxcls'));
var_dump(__reflect_class_real_name('FxMissingNope'));
var_dump(__reflect_method_names('FxCls'));
var_dump(__reflect_class_loc('FxCls'));
