<?php
// S-140 — fixture bilaterali leva HC1 (hint-check): i cammini toccati dal
// borrow-first devono restare byte-identici all'oracle. Dry-run col catalogo
// divergenze PRIMA del gate (az.rev. S-139 #4): qui nessun cammino a
// catalogo (niente undefined-key/float-key/str-increment).
class P { public int $v = 0; }
class Q extends P {}
class S1 { public function __toString(): string { return "sss"; } }
function t_class(P $p): P { return $p; }
function t_nullable(?P $p): ?P { return $p; }
function t_union(int|string $x): int|string { return $x; }
function t_weakstr(string $s): string { return $s; }
function t_int(int $i): int { return $i; }
function t_iter(iterable $it): int { $n=0; foreach($it as $_) $n++; return $n; }
function t_byref(P &$p): void { $p->v++; }

// 1. classe esatta + sottoclasse
$p = new P; $q = new Q;
var_dump(get_class(t_class($p)), get_class(t_class($q)));
// 2. nullable
var_dump(t_nullable(null));
// 3. union exact + weak preference (int resta int; "7" resta string)
var_dump(t_union(7), t_union("7"));
// 4. weak: Stringable -> string
var_dump(t_weakstr(new S1));
// 5. weak: int "42" -> 42
var_dump(t_int("42"));
// 6. iterable array
var_dump(t_iter([1,2,3]));
// 7. by-ref (cammino Ref: deref_clone conservato)
$r = new P; t_byref($r); var_dump($r->v);
// 8. TypeError su classe sbagliata (messaggio byte-id)
try { t_class(new S1); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
// 9. TypeError su null non-nullable
try { t_class(null); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
// 10. RIMOSSO dal gate (dry-run az.rev. #4): deprecation float->int diverge
//     per vie PRE-esistenti fuori perimetro HC1 (riga doppia CLI oracle +
//     attribuzione linea decl-callee vs call-site) — apertura per NOME.
// 11. return-hint violato
function t_badret(): P { return 42; }
try { t_badret(); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
