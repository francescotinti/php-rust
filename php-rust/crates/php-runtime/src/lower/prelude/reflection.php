// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
// The Reflector interface every reflection object satisfies (it extends
// Stringable). phpr does not yet render the full PHP export format in the
// classes' __toString, but the interface membership itself is what code
// type-hints and tests via `instanceof Reflector`.
interface Reflector extends Stringable {}
// The Reflection base: a handful of static helpers, chiefly getModifierNames().
abstract class Reflection {
    public static function getModifierNames($modifiers) {
        $names = [];
        // Order fixed by ext/reflection: abstract, final, visibility, static,
        // readonly (NOT by bit value).
        if ($modifiers & 64)  { $names[] = 'abstract'; }   // IS_ABSTRACT
        if ($modifiers & 32)  { $names[] = 'final'; }      // IS_FINAL
        if ($modifiers & 1)   { $names[] = 'public'; }     // IS_PUBLIC
        if ($modifiers & 2)   { $names[] = 'protected'; }  // IS_PROTECTED
        if ($modifiers & 4)   { $names[] = 'private'; }    // IS_PRIVATE
        if ($modifiers & 16)  { $names[] = 'static'; }     // IS_STATIC
        if ($modifiers & 128) { $names[] = 'readonly'; }   // IS_READONLY
        return $names;
    }
}
class ReflectionException extends Exception {}
// A loaded Zend (C) extension. phpr has no C extensions, so this reflects the
// bare name it is constructed with: enough for `new ReflectionZendExtension($n)`
// and its accessors to exist.
class ReflectionZendExtension implements Reflector {
    public $name;
    public function __construct($name) { $this->name = $name; }
    public function __toString() { return ''; }
    public function getName() { return $this->name; }
    public function getVersion() { return ''; }
    public function getAuthor() { return ''; }
    public function getURL() { return ''; }
    public function getCopyright() { return ''; }
}
class ReflectionAttribute {
    const IS_INSTANCEOF = 2;
    // Validate the $flags argument and apply the IS_INSTANCEOF filter. `$label` is
    // the Reflection class reported in the invalid-flag error (PHP reports the
    // declaring scope, e.g. ReflectionFunctionAbstract for func/method). When the
    // flag is set, `$all` was fetched unfiltered (host name = null) and we keep
    // only attributes whose class is `$name` or a subclass/implementor of it; the
    // filter class must exist (else PHP throws "Class X not found"). Without the
    // flag the host already applied the exact-name filter, so `$all` is as-is.
    public static function __filter($all, $name, $flags, $label) {
        if ($flags !== 0 && $flags !== self::IS_INSTANCEOF) {
            throw new Error($label . '::getAttributes(): Argument #2 ($flags) must be a valid attribute filter flag');
        }
        if (($flags & self::IS_INSTANCEOF) === 0 || $name === null) { return $all; }
        if (!class_exists($name) && !interface_exists($name)) {
            throw new Error('Class "' . $name . '" not found');
        }
        $want = strtolower($name);
        $out = [];
        foreach ($all as $a) {
            $x = $a->getName();
            $isa = strcasecmp($x, $name) === 0;
            // Only walk the hierarchy for a class that actually exists; an
            // unresolved attribute class is simply not an instance of anything but
            // its own name (mirrors PHP, which emits no warning here).
            if (!$isa && (class_exists($x) || interface_exists($x))) {
                foreach (class_parents($x) as $p) { if (strtolower($p) === $want) { $isa = true; break; } }
                if (!$isa) {
                    foreach (class_implements($x) as $i) { if (strtolower($i) === $want) { $isa = true; break; } }
                }
            }
            if ($isa) { $out[] = $a; }
        }
        return $out;
    }
    public $name;
    // Private handle to the owning class + the attribute's position in it, used by
    // the host builtins to materialise the attribute lazily. `__prop` is set for an
    // attribute that decorates a property (vs the class itself).
    public $__class;
    public $__index;
    public $__prop;
    public $__func;
    public $__method;
    public $__const;
    public $__classconst;
    public $__paramfunc;
    public $__paramclass;
    public $__parampos;
    public $__closure_val;
    public function getName() { return $this->name; }
    public function getArguments() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_args($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_args($this->__class, $this->__classconst, $this->__index);
        }
        if (isset($this->__prop)) {
            return __reflect_prop_attr_args($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_args($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_args($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_args($this->__const, $this->__index);
        }
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_args($this->__closure_val, $this->__index);
        }
        return __reflect_attr_arguments($this->__class, $this->__index);
    }
    public function newInstance() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_new($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_new($this->__class, $this->__classconst, $this->__index);
        }
        if (isset($this->__prop)) {
            return __reflect_prop_attr_new($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_new($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_new($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_new($this->__const, $this->__index);
        }
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_new($this->__closure_val, $this->__index);
        }
        return __reflect_attr_newinstance($this->__class, $this->__index);
    }
}
class ReflectionClass implements Reflector {
    const SKIP_INITIALIZATION_ON_SERIALIZE = 8;
    const SKIP_DESTRUCTOR = 16;
    public $name;
    public function __construct($objectOrClass) {
        $this->name = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        // An *object* argument is always reflectable (engine values like a
        // Closure included); only a class-name string is checked for existence.
        if (!is_object($objectOrClass) && !class_exists($this->name) && !interface_exists($this->name) && !trait_exists($this->name)) {
            throw new ReflectionException(sprintf('Class "%s" does not exist', $this->name));
        }
        // Class names resolve case-insensitively but ReflectionClass::$name carries
        // the CANONICAL declared casing (a `new ReflectionClass('MY\CLASS')` reports
        // `My\Class`): normalize a string argument to it.
        if (!is_object($objectOrClass)) {
            $real = __reflect_class_real_name($this->name);
            if ($real !== false) { $this->name = $real; }
        }
    }
    public function getFileName() { $l = __reflect_class_loc($this->name); return $l[0]; }
    public function isInternal() { return $this->getFileName() === false; }
    public function isUserDefined() { return $this->getFileName() !== false; }
    public function getDocComment() { return __reflect_class_doc($this->name); }
    public function isReadOnly() { return __reflect_class_modifiers($this->name)['readonly'] ?? false; }
    public function getStartLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[1]; }
    public function getEndLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[2]; }
    // phpr mangles anonymous classes exactly like PHP: `class@anonymous\0N`.
    public function isAnonymous() { return strpos($this->name, 'class@anonymous') === 0; }
    public function getName() { return $this->name; }
    // Indent every non-empty line of a stringable child (a nested Method block) by
    // $n spaces for the export format.
    private function __indent($obj, $n) {
        $pad = str_repeat(' ', $n);
        $out = '';
        foreach (explode("\n", rtrim((string) $obj, "\n")) as $l) {
            $out .= ($l === '' ? '' : $pad . $l) . "\n";
        }
        return $out;
    }
    // The PHP `Reflection::export` string for a class (ext/reflection golden output).
    public function __toString() {
        return $this->__exportString(false, []);
    }
    // Shared class/object export body. In object mode the header word is
    // "Object of class" and a "- Dynamic properties [N]" section (from $dynNames)
    // is inserted between the declared Properties and the Methods.
    protected function __exportString($objectMode, $dynNames) {
        $src = $this->isUserDefined() ? '<user>' : '<internal>';
        if ($objectMode) {
            $head = 'Object of class';
            $kind = ($this->isAbstract() ? 'abstract ' : '') . ($this->isFinal() ? 'final ' : '') . 'class';
        }
        elseif ($this->isInterface()) { $head = 'Interface'; $kind = 'interface'; }
        elseif ($this->isTrait()) { $head = 'Class'; $kind = 'trait'; }
        else {
            $head = 'Class';
            $kind = ($this->isAbstract() ? 'abstract ' : '') . ($this->isFinal() ? 'final ' : '') . 'class';
        }
        $hdr = "$head [ $src $kind {$this->name}";
        $parent = $this->getParentClass();
        if ($parent !== false) { $hdr .= " extends " . $parent->getName(); }
        $ifaces = $this->getInterfaceNames();
        if (!empty($ifaces)) { $hdr .= " implements " . implode(', ', $ifaces); }
        $s = $hdr . " ] {\n";
        $file = $this->getFileName();
        if ($file !== false) {
            $s .= "  @@ $file " . $this->getStartLine() . "-" . $this->getEndLine() . "\n";
        }
        $consts = $this->getReflectionConstants();
        $props = $this->getProperties();
        $sprops = []; $iprops = [];
        foreach ($props as $p) { if ($p->isStatic()) { $sprops[] = $p; } else { $iprops[] = $p; } }
        $methods = $this->getMethods();
        $smeth = []; $imeth = [];
        foreach ($methods as $m) { if ($m->isStatic()) { $smeth[] = $m; } else { $imeth[] = $m; } }
        $s .= "\n  - Constants [" . count($consts) . "] {\n";
        foreach ($consts as $c) { $s .= "    " . rtrim((string) $c, "\n") . "\n"; }
        $s .= "  }\n";
        $s .= "\n  - Static properties [" . count($sprops) . "] {\n";
        foreach ($sprops as $p) { $s .= "    " . rtrim((string) $p, "\n") . "\n"; }
        $s .= "  }\n";
        $s .= "\n  - Static methods [" . count($smeth) . "] {\n";
        foreach ($smeth as $m) { $s .= $this->__indent($m, 4); }
        $s .= "  }\n";
        $s .= "\n  - Properties [" . count($iprops) . "] {\n";
        foreach ($iprops as $p) {
            $doc = $p->getDocComment();
            if ($doc !== false) { $s .= "    " . $doc . "\n"; }
            $s .= "    " . rtrim((string) $p, "\n") . "\n";
        }
        $s .= "  }\n";
        if ($objectMode) {
            $s .= "\n  - Dynamic properties [" . count($dynNames) . "] {\n";
            foreach ($dynNames as $dn) { $s .= "    Property [ <dynamic> public \$$dn ]\n"; }
            $s .= "  }\n";
        }
        $s .= "\n  - Methods [" . count($imeth) . "] {\n";
        foreach ($imeth as $m) { $s .= $this->__indent($m, 4); }
        $s .= "  }\n";
        return $s . "}\n";
    }
    public function getShortName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? $this->name : substr($this->name, $p + 1);
    }
    // Attributes are retained at lowering; the host builds one ReflectionAttribute
    // per attribute declared on the class (optionally filtered by name).
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_class_attributes($this->name, $hostName), $name, $flags, 'ReflectionClass');
    }
    public function newInstance(...$args) { $this->__checkCtorPublic(); return new $this->name(...$args); }
    public function newInstanceArgs($args = []) { $this->__checkCtorPublic(); return new $this->name(...$args); }
    private function __checkCtorPublic() {
        // Zend: reflection_class_new_instance refuses a non-public constructor
        // with ReflectionException (the `new` below would otherwise raise the
        // scope-based Error naming ReflectionClass as the calling scope).
        $ctor = $this->getConstructor();
        if ($ctor !== null && !$ctor->isPublic()) {
            throw new ReflectionException('Access to non-public constructor of class ' . $this->name);
        }
    }
    public function newInstanceWithoutConstructor() {
        // Internal final classes cannot skip their constructor (Zend rejects
        // it; doctrine/instantiator's PDORow probe relies on the refusal).
        if ($this->isInternal() && $this->isFinal()) {
            throw new ReflectionException('Class ' . $this->name . ' is an internal class marked as final that cannot be instantiated without invoking its constructor');
        }
        return __reflect_new_no_ctor($this->name);
    }
    const SKIP_INITIALIZATION_ON_SERIALIZE = 8;
    const SKIP_DESTRUCTOR = 16;
    private function __lazyOptionsCheck($method, $argno, $options, $valid) {
        if (($options & ~$valid) !== 0) {
            // SKIP_DESTRUCTOR alone gets the dedicated wording; any other
            // stray bit is "invalid flags" (Zend's order).
            if (($options & ~($valid | self::SKIP_DESTRUCTOR)) === 0 && ($options & self::SKIP_DESTRUCTOR)) {
                throw new ReflectionException(sprintf('ReflectionClass::%s(): Argument #%d ($options) does not accept ReflectionClass::SKIP_DESTRUCTOR', $method, $argno));
            }
            throw new ReflectionException(sprintf('ReflectionClass::%s(): Argument #%d ($options) contains invalid flags', $method, $argno));
        }
    }
    public function newLazyGhost(callable $initializer, int $options = 0) {
        $this->__lazyOptionsCheck('newLazyGhost', 2, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE);
        return __reflect_new_lazy_ghost($this->name, $initializer, $options);
    }
    public function newLazyProxy(callable $factory, int $options = 0) {
        $this->__lazyOptionsCheck('newLazyProxy', 2, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE);
        return __reflect_new_lazy_proxy($this->name, $factory, $options);
    }
    public function resetAsLazyGhost($object, callable $initializer, int $options = 0) {
        $this->__lazyOptionsCheck('resetAsLazyGhost', 3, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE | self::SKIP_DESTRUCTOR);
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, false, $initializer, $options);
    }
    public function resetAsLazyProxy($object, callable $factory, int $options = 0) {
        $this->__lazyOptionsCheck('resetAsLazyProxy', 3, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE | self::SKIP_DESTRUCTOR);
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, true, $factory, $options);
    }
    public function isUninitializedLazyObject($object) { return __lazy_is_uninitialized($object); }
    public function getLazyInitializer($object) { return __lazy_get_initializer($object); }
    public function initializeLazyObject($object) { return __lazy_initialize($object); }
    public function markLazyObjectAsInitialized($object) { return __lazy_mark_initialized($object); }
    public function isInstantiable() { return class_exists($this->name); }
    public function isCloneable() {
        if ($this->isInterface() || $this->isAbstract() || $this->isEnum()) { return false; }
        if ($this->hasMethod('__clone')) {
            return $this->getMethod('__clone')->isPublic();
        }
        return true;
    }
    public function isInterface() { return interface_exists($this->name); }
    public function isTrait() { return trait_exists($this->name); }
    public function isEnum() { return in_array('UnitEnum', class_implements($this->name)); }
    public function isFinal() { return __reflect_class_modifiers($this->name)['final']; }
    public function isAbstract() { return __reflect_class_modifiers($this->name)['abstract']; }
    // Bitmask of class modifiers. Only an *explicit* `abstract class` sets 64;
    // interfaces/traits (whose methods are implicitly abstract) return 0, as does
    // a plain class. `final` is 32.
    public function getModifiers() {
        $m = 0;
        if (!$this->isInterface() && !$this->isTrait() && $this->isAbstract()) { $m |= 64; }
        if ($this->isFinal()) { $m |= 32; }
        return $m;
    }
    public function isInstance($object) { return $object instanceof $this->name; }
    // A class is iterable when it implements Traversable (via Iterator or
    // IteratorAggregate); class_implements includes inherited interfaces.
    public function isIterateable() {
        return in_array('Traversable', class_implements($this->name), true)
            || strcasecmp($this->name, 'Traversable') === 0;
    }
    public function isIterable() { return $this->isIterateable(); }
    // Static properties as a `name => value` map (own + inherited), values current.
    public function getStaticProperties() { return __reflect_static_props($this->name); }
    public function getStaticPropertyValue($name, $default = null) {
        $props = $this->getStaticProperties();
        if (array_key_exists($name, $props)) { return $props[$name]; }
        if (func_num_args() >= 2) { return $default; }
        throw new ReflectionException(sprintf('Property %s::$%s does not exist', $this->name, $name));
    }
    public function setStaticPropertyValue($name, $value) {
        if (!__reflect_static_prop_set($this->name, $name, $value)) {
            throw new ReflectionException(sprintf('Class %s does not have a property named %s', $this->name, $name));
        }
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
    public function hasProperty($name) {
        if (!property_exists($this->name, $name)) { return false; }
        // An ancestor's PRIVATE property is invisible to a subclass' reflection
        // surface: hasProperty() must agree with getProperty()/new
        // ReflectionProperty() (which throw for it), else a caller that guards a
        // getProperty() with hasProperty() still hits the exception
        // (Doctrine ORM ClassMetadata::isTypedProperty, GH11199).
        $decl = __reflect_prop_declaring_class($this->name, $name);
        if ($decl !== false && strcasecmp($decl, $this->name) !== 0) {
            $info = __reflect_prop_details($decl, $name);
            if (is_array($info) && ($info['visibility'] ?? null) === 'private') {
                return false;
            }
        }
        return true;
    }
    public function getProperty($name) { return new ReflectionProperty($this->name, $name); }
    public function getProperties($filter = null) {
        $out = [];
        foreach (__reflect_prop_names($this->name) as $n) {
            // Construct each with the DECLARING class as scope so the ancestor-
            // private guard in the ctor does not fire (it would otherwise abort
            // enumeration). Zend then OMITS an ancestor's private property from a
            // subclass' getProperties() (_addproperty: PRIVATE && prop_info->ce !=
            // ce), matching hasProperty()/getProperty() which reject it.
            $decl = __reflect_prop_declaring_class($this->name, $n);
            $rp = new ReflectionProperty($decl === false ? $this->name : $decl, $n);
            if ($decl !== false && strcasecmp($decl, $this->name) !== 0 && $rp->isPrivate()) {
                continue;
            }
            $out[] = $rp;
        }
        return $out;
    }
    public function hasConstant($name) { return defined($this->name . '::' . $name); }
    public function getConstant($name) { return constant($this->name . '::' . $name); }
    public function getConstants($filter = null) {
        return $filter === null
            ? __reflect_class_constants($this->name)
            : __reflect_class_constants($this->name, $filter);
    }
    public function getReflectionConstant($name) {
        try { return new ReflectionClassConstant($this->name, $name); }
        catch (ReflectionException $e) { return false; }
    }
    public function getReflectionConstants($filter = null) {
        $out = [];
        foreach (__reflect_class_const_names($this->name) as $n) {
            $rc = new ReflectionClassConstant($this->name, $n);
            if ($filter === null || ($rc->getModifiers() & $filter)) { $out[] = $rc; }
        }
        return $out;
    }
    public function implementsInterface($interface) {
        return in_array($interface, class_implements($this->name), true);
    }
    public function isSubclassOf($class) {
        return in_array($class, class_parents($this->name), true)
            || in_array($class, class_implements($this->name), true);
    }
    public function getParentClass() {
        $p = get_parent_class($this->name);
        return $p === false ? false : new ReflectionClass($p);
    }
    public function getInterfaceNames() { return array_values(class_implements($this->name)); }
    public function getInterfaces() {
        $out = [];
        foreach (class_implements($this->name) as $i) { $out[$i] = new ReflectionClass($i); }
        return $out;
    }
    public function getTraitNames() { return array_values(class_uses($this->name)); }
    public function getTraits() {
        $out = [];
        foreach (class_uses($this->name) as $t) { $out[$t] = new ReflectionClass($t); }
        return $out;
    }
    public function getTraitAliases() { return []; }
    public function getDefaultProperties() { return __reflect_prop_defaults($this->name); }
    public function getNamespaceName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? '' : substr($this->name, 0, $p);
    }
    public function inNamespace() { return strpos($this->name, '\\') !== false; }
    public function getMethod($name) { return new ReflectionMethod($this->name, $name); }
    public function getConstructor() {
        return method_exists($this->name, '__construct')
            ? new ReflectionMethod($this->name, '__construct') : null;
    }
    public function getMethods($filter = null) {
        $out = [];
        // All visibilities, parent chain included (get_class_methods filters
        // to public outside the class -- PHPUnit's #[Before] hooks are protected).
        foreach (__reflect_method_names($this->name) as $m) {
            $rm = new ReflectionMethod($this->name, $m);
            if ($filter !== null && ($rm->getModifiers() & $filter) === 0) { continue; }
            $out[] = $rm;
        }
        return $out;
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
}
abstract class ReflectionType {
    abstract public function allowsNull(): bool;
    abstract public function __toString(): string;
}
class ReflectionUnionType extends ReflectionType {
    public $__types; public $__nullable;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('|', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionUnionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionIntersectionType extends ReflectionType {
    public $__types;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return false; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('&', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionIntersectionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        return $r;
    }
}
class ReflectionNamedType extends ReflectionType {
    public $__name; public $__builtin; public $__nullable;
    public function getName() { return $this->__name; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function isBuiltin() { return $this->__builtin; }
    public function __toString(): string {
        $q = ($this->__nullable && $this->__name !== 'mixed' && $this->__name !== 'null') ? '?' : '';
        return $q . $this->__name;
    }
    public static function __fromInfo($t) {
        if ($t === false) { return null; }
        if (isset($t['kind'])) {
            return $t['kind'] === 'intersection'
                ? ReflectionIntersectionType::__fromInfo($t)
                : ReflectionUnionType::__fromInfo($t);
        }
        $r = new ReflectionNamedType();
        $r->__name = $t['name']; $r->__builtin = $t['builtin']; $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionParameter implements Reflector {
    public $name;
    public $__pos; public $__optional; public $__variadic; public $__byref;
    public $__type; public $__hasDefault; public $__default;
    public $__declClass; public $__declFunc; public $__defaultConst; public $__defaultError; public $__promoted;
    public function __construct($function = null, $param = null) {
        if ($function === null) { return; } // internal factory path (__fromInfo)
        $info = is_array($function)
            ? __reflect_method_info($function[0], $function[1])
            : __reflect_func_info($function);
        if ($info === false) { throw new ReflectionException('The function does not exist'); }
        foreach ($info['params'] as $p) {
            if ((is_int($param) && $p['position'] === $param) || $p['name'] === $param) {
                $this->__init($p); return;
            }
        }
        throw new ReflectionException('The parameter specified does not exist');
    }
    public function __init($p) {
        $this->name = $p['name']; $this->__pos = $p['position'];
        $this->__optional = $p['optional']; $this->__variadic = $p['variadic'];
        $this->__byref = $p['byref']; $this->__type = $p['type'];
        $this->__hasDefault = $p['hasDefault']; $this->__default = $p['default'];
        $this->__declClass = $p['declClass'] ?? ''; $this->__declFunc = $p['declFunc'] ?? '';
        $this->__defaultConst = $p['defaultConstant'] ?? false;
        $this->__defaultError = $p['defaultError'] ?? false;
        $this->__promoted = $p['promoted'] ?? false;
    }
    public function isPromoted() { return $this->__promoted; }
    public function isDefaultValueConstant() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        return $this->__defaultConst !== false;
    }
    public function getDefaultValueConstantName() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        return $this->__defaultConst === false ? null : $this->__defaultConst;
    }
    public static function __fromInfo($p) { $r = new ReflectionParameter(); $r->__init($p); return $r; }
    public function getName() { return $this->name; }
    public function getPosition() { return $this->__pos; }
    public function isOptional() { return $this->__optional; }
    public function isVariadic() { return $this->__variadic; }
    public function isPassedByReference() { return $this->__byref; }
    public function canBePassedByValue() { return !$this->__byref; }
    public function hasType() { return $this->__type !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo($this->__type); }
    public function allowsNull() { return $this->__type === false ? true : $this->__type['nullable']; }
    public function isDefaultValueAvailable() { return $this->__hasDefault; }
    public function getDefaultValue() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        // A default that could not be evaluated surfaces its error here (lazily),
        // never at ReflectionParameter construction (PHP evaluates on demand).
        if ($this->__defaultError !== false) {
            throw new Error($this->__defaultError);
        }
        return $this->__default;
    }
    // The declaring function/method of this parameter.
    public function getDeclaringFunction() {
        return $this->__declClass !== ''
            ? new ReflectionMethod($this->__declClass, $this->__declFunc)
            : new ReflectionFunction($this->__declFunc);
    }
    // The class the parameter is declared in, or null for a plain function.
    public function getDeclaringClass() {
        return $this->__declClass !== '' ? new ReflectionClass($this->__declClass) : null;
    }
    // Deprecated (pre-8.0): the ReflectionClass of a class-typed parameter, else
    // null. `self`/`parent`/`static` resolve against the declaring class.
    public function getClass() {
        $t = $this->__type;
        if ($t === false || isset($t['kind']) || $t['builtin']) { return null; }
        $n = $t['name'];
        if (strcasecmp($n, 'self') === 0 || strcasecmp($n, 'static') === 0) { $n = $this->__declClass; }
        elseif (strcasecmp($n, 'parent') === 0) { $n = get_parent_class($this->__declClass); }
        return $n === '' || $n === false ? null : new ReflectionClass($n);
    }
    // Deprecated type predicates (pre-8.0): true for a bare `array`/`callable` hint.
    public function isArray() {
        return $this->__type !== false && !isset($this->__type['kind'])
            && strcasecmp($this->__type['name'], 'array') === 0;
    }
    public function isCallable() {
        return $this->__type !== false && !isset($this->__type['kind'])
            && strcasecmp($this->__type['name'], 'callable') === 0;
    }
    // `Parameter #N [ <optional> Type $name = DEFAULT ]` (oracle format).
    // PHPUnit's mock generator parses the piece after ' = ' as *source code*
    // for an object default, so enum cases render `\FQCN::CASE`.
    public function __toString() {
        // A variadic is <optional> without a default; the ` = DEFAULT` suffix is
        // driven separately by whether a default value is available.
        $opt = $this->isOptional();
        $hasDefault = $this->isDefaultValueAvailable();
        $s = 'Parameter #' . $this->getPosition() . ' [ <' . ($opt ? 'optional' : 'required') . '> ';
        $t = $this->getType();
        $ts = '';
        if ($t !== null) {
            if ($t instanceof ReflectionUnionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('|', $parts);
            } elseif ($t instanceof ReflectionIntersectionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('&', $parts);
            } else {
                $ts = $t->getName();
                if ($t->allowsNull() && $ts !== 'null' && $ts !== 'mixed') { $ts = '?' . $ts; }
            }
        }
        $s .= ($ts !== '' ? $ts . ' ' : '') . ($this->isPassedByReference() ? '&' : '') . ($this->isVariadic() ? '...' : '') . '$' . $this->getName();
        if ($hasDefault && !$this->isVariadic()) {
            $v = $this->__default;
            if ($v === null) { $d = 'NULL'; }
            elseif (is_object($v)) {
                $d = ($v instanceof UnitEnum) ? '\\' . get_class($v) . '::' . $v->name : 'new \\' . get_class($v) . '(...)';
            }
            elseif (is_array($v)) { $d = str_replace("\n", '', var_export($v, true)); }
            else { $d = var_export($v, true); }
            $s .= ' = ' . $d;
        }
        return $s . ' ]';
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_param_attributes($this->__declClass, $this->__declFunc, $this->__pos, $hostName), $name, $flags, 'ReflectionParameter');
    }
}
class ReflectionObject extends ReflectionClass {
    // The reflected instance is held host-side (keyed by this object's id) rather
    // than as a property, so a var_dump of a ReflectionObject shows only `name`
    // (matching Zend, which keeps the pointer internally).
    public function __construct($object) {
        parent::__construct($object);
        if (is_object($object)) { __reflect_object_bind($this, $object); }
    }
    // Full export format with the object's dynamic properties. The dynamic-prop
    // names come from a host that reads the property table directly, so a lazy
    // object is NOT initialized (init_trigger_reflection_object_toString).
    public function __toString() {
        return $this->__exportString(true, __reflect_object_dynprops($this));
    }
    // Unlike ReflectionClass, the instance's *dynamic* properties are part of
    // the surface (PHPUnit's assertObjectHasProperty polyfill keys off this).
    public function hasProperty($name) {
        return parent::hasProperty($name)
            || in_array($name, __reflect_object_dynprops($this), true);
    }
    public function getProperty($name) {
        if (!parent::hasProperty($name)
            && in_array($name, __reflect_object_dynprops($this), true)) {
            return new ReflectionProperty(__reflect_object_instance($this), $name);
        }
        return parent::getProperty($name);
    }
}
// A reference an array element holds (7.4). `fromArrayElement` returns null when
// the element is not a `&`-reference; `getId` yields the same string for two
// elements aliasing the same reference (Symfony var-exporter / deepclone rely on
// this to preserve shared references across a clone).
final class ReflectionReference {
    private $id;
    private function __construct() {}
    public static function fromArrayElement(array $array, int|string $key): ?ReflectionReference {
        if (!array_key_exists($key, $array)) {
            throw new ReflectionException(sprintf('Key "%s" does not exist', $key));
        }
        $id = __reflect_ref_id($array, $key);
        if ($id === false) { return null; }
        $ref = new ReflectionReference();
        $ref->id = $id;
        return $ref;
    }
    public function getId(): string { return $this->id; }
}
// Reflects a suspended Generator: its current execution point, bound $this and
// the underlying function. Backed by the host, which reads the parked frame.
final class ReflectionGenerator {
    private $__gen;
    public function __construct(Generator $generator) { $this->__gen = $generator; }
    public function getExecutingLine(): int { return __reflect_gen_info($this->__gen)[0]; }
    public function getExecutingFile() { return __reflect_gen_info($this->__gen)[1]; }
    public function getThis() { return __reflect_gen_info($this->__gen)[2]; }
    public function getFunction(): ReflectionFunctionAbstract {
        $name = __reflect_gen_info($this->__gen)[3];
        if (strpos($name, '::') !== false) {
            [$c, $m] = explode('::', $name, 2);
            return new ReflectionMethod($c, $m);
        }
        return new ReflectionFunction($name);
    }
    public function getExecutingGenerator(): Generator { return $this->__gen; }
    public function isClosed(): bool { return __reflect_gen_info($this->__gen)[4]; }
    // The suspended call stack is not modelled; return an empty trace rather than
    // erroring (best-effort; the other accessors are exact).
    public function getTrace($options = 1): array { return []; }
}
// Reflects a suspended Fiber: its current execution point and the callable it
// runs. Backed by the host, which reads the fiber's parked frame.
final class ReflectionFiber {
    private $__fiber;
    public function __construct(Fiber $fiber) { $this->__fiber = $fiber; }
    public function getFiber(): Fiber { return $this->__fiber; }
    public function getExecutingLine(): int { return __reflect_fiber_info($this->__fiber)[0]; }
    public function getExecutingFile() { return __reflect_fiber_info($this->__fiber)[1]; }
    public function getCallable(): callable { return __reflect_fiber_info($this->__fiber)[2]; }
    // The suspended call stack is not modelled (see ReflectionGenerator::getTrace).
    public function getTrace($options = 1): array { return []; }
}
class ReflectionConstant {
    public $name;
    public function __construct($name) {
        if (!defined($name)) {
            throw new ReflectionException(sprintf('Constant "%s" does not exist', $name));
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getValue() { return constant($this->name); }
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_const_attributes($this->name, $hostName), $name, $flags, 'ReflectionConstant');
    }
    public function __toString() { return sprintf("Constant [ %s ]\n", $this->name); }
}
class ReflectionClassConstant implements Reflector {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_FINAL = 32;
    public $name;
    public $class;
    public $__info;
    public function __construct($class, $constant) {
        $cls = is_object($class) ? get_class($class) : $class;
        $info = __reflect_class_const_info($cls, $constant);
        if ($info === false) {
            throw new ReflectionException(sprintf('Constant %s::%s does not exist', $cls, $constant));
        }
        $this->name = $constant;
        $this->class = $info['declaringClass'];
        $this->__info = $info;
    }
    public function getName() { return $this->name; }
    public function getValue() { return $this->__info['value']; }
    public function getDeclaringClass() { return new ReflectionClass($this->class); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isFinal() { return $this->__info['final']; }
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    // phpr does not retain a class-constant's doc comment; false (as PHP for one
    // without a `/** */` block).
    public function getDocComment() { return false; }
    public function isEnumCase() { return $this->__info['enumCase']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['final']) { $m |= self::IS_FINAL; }
        return $m;
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_classconst_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionClassConstant');
    }
    // `Constant [ VIS TYPE NAME ] { VALUE }` (ext/reflection golden output). The
    // type shown matches the declared type for a typed constant and the value's
    // type for an untyped one; for scalars they coincide, so derive from the value.
    public function __toString() {
        $vis = $this->__info['visibility'];
        $v = $this->getValue();
        $tmap = ['integer' => 'int', 'double' => 'float', 'boolean' => 'bool', 'NULL' => 'null'];
        $t = gettype($v);
        $type = $tmap[$t] ?? $t;
        if (is_bool($v)) { $val = $v ? 'true' : 'false'; }
        elseif ($v === null) { $val = ''; }
        elseif (is_string($v)) { $val = $v; }
        else { $val = (string) $v; }
        return "Constant [ $vis $type {$this->name} ] { $val }\n";
    }
}
class ReflectionEnumUnitCase extends ReflectionClassConstant {
    // getValue() is inherited: __reflect_class_const_info returns the case
    // singleton as the constant's value.
    public function getEnum() { return new ReflectionEnum($this->class); }
    public function getDocComment() { return false; }
}
class ReflectionEnumBackedCase extends ReflectionEnumUnitCase {
    public function getBackingValue() { return $this->getValue()->value; }
}
class ReflectionEnum extends ReflectionClass {
    public function isBacked() { return in_array('BackedEnum', class_implements($this->name)); }
    public function getBackingType() { return ReflectionNamedType::__fromInfo(__reflect_enum_backing($this->name)); }
    public function hasCase($name) {
        $cls = $this->name;
        foreach ($cls::cases() as $c) { if ($c->name === $name) { return true; } }
        return false;
    }
    public function getCase($name) {
        if (!$this->hasCase($name)) {
            throw new ReflectionException(sprintf('Case %s::%s does not exist', $this->name, $name));
        }
        return $this->isBacked()
            ? new ReflectionEnumBackedCase($this->name, $name)
            : new ReflectionEnumUnitCase($this->name, $name);
    }
    public function getCases() {
        $out = [];
        $cls = $this->name;
        $backed = $this->isBacked();
        foreach ($cls::cases() as $c) {
            $out[] = $backed
                ? new ReflectionEnumBackedCase($this->name, $c->name)
                : new ReflectionEnumUnitCase($this->name, $c->name);
        }
        return $out;
    }
}
abstract class ReflectionFunctionAbstract implements Reflector {}
class ReflectionFunction extends ReflectionFunctionAbstract {
    public $name;
    public $__info;
    public $__closure;
    public function __construct($name) {
        if ($name instanceof Closure) {
            $this->__closure = $name;
            $this->__info = __reflect_closure_info($name);
            // Zend reports the wrapped callable's name: the bare method name for
            // a method-backed closure, the function name for a first-class
            // callable, `{closure:file:line}` for a genuinely anonymous one.
            $this->name = ($this->__info !== false && isset($this->__info['name']))
                ? $this->__info['name'] : '{closure}';
        } else {
            $this->name = is_string($name) ? $name : '{closure}';
            $this->__info = __reflect_func_info($this->name);
            // A callable INTERNAL builtin (registry/host) has no lowered body
            // to introspect: reflect it as an internal function with an empty
            // parameter list (declared residue: real param metadata absent —
            // the reflection corpus' check_all only needs the walk to not
            // throw, WP-17).
            if ($this->__info === false && function_exists($this->name)) {
                $this->__info = array('params' => array(), 'returnType' => false);
            }
        }
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Function %s() does not exist', $this->name));
        }
    }
    public function getName() { return $this->name; }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function getFileName() { return $this->__info['file'] ?? false; }
    public function getStartLine() { return $this->__info['startLine'] ?? false; }
    public function getEndLine() { return $this->__info['endLine'] ?? false; }
    public function isUserDefined() { return ($this->__info['file'] ?? false) !== false; }
    public function isInternal() { return ($this->__info['file'] ?? false) === false; }
    public function isClosure() { return $this->__closure !== null; }
    // Anonymous = a genuine `function () {}` literal; a first-class callable
    // or method-backed closure reports the named function (Zend checks the
    // ZEND_ACC_ANON_FUNCTION flag, visible here as the `{closure:…}` name).
    public function isAnonymous() {
        return $this->__closure !== null && strncmp($this->name, '{closure', 8) === 0;
    }
    // Closure binding surface: null for a named-function reflection.
    public function getClosureThis() {
        return $this->__closure !== null ? __reflect_closure_bind($this->__closure)[0] : null;
    }
    public function getClosureScopeClass() {
        if ($this->__closure === null) { return null; }
        $s = __reflect_closure_bind($this->__closure)[1];
        return $s === null ? null : new ReflectionClass($s);
    }
    // The called scope: the bound $this's class when bound, else the scope
    // class (static closures, Closure::bind(…, null, Class)) — oracle-pinned.
    public function getClosureCalledClass() {
        if ($this->__closure === null) { return null; }
        $b = __reflect_closure_bind($this->__closure);
        if (is_object($b[0])) { return new ReflectionClass(get_class($b[0])); }
        return $b[1] === null ? null : new ReflectionClass($b[1]);
    }
    public function isStatic() {
        return $this->__closure !== null && __reflect_closure_bind($this->__closure)[2];
    }
    public function getClosureUsedVariables() {
        return $this->__closure !== null ? __reflect_closure_uses($this->__closure) : [];
    }
    public function getStaticVariables() { return __reflect_static_vars(null, $this->name); }
    public function isGenerator() { return $this->__info['isGenerator'] ?? false; }
    public function returnsReference() { return $this->__info['byRef'] ?? false; }
    // Deprecated via the #[\Deprecated] attribute (8.4).
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function invoke(...$args) { return call_user_func_array($this->name, $args); }
    public function invokeArgs($args) { return call_user_func_array($this->name, $args); }
    // The underlying closure (for a Closure-reflection) or the named function
    // wrapped as one.
    public function getClosure() {
        return $this->__closure !== null ? $this->__closure : Closure::fromCallable($this->name);
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        $all = $this->__closure !== null
            ? __reflect_closure_attributes($this->__closure, $hostName)
            : __reflect_func_attributes($this->name, $hostName);
        return ReflectionAttribute::__filter($all, $name, $flags, 'ReflectionFunctionAbstract');
    }
    // The PHP `Reflection::export` string for a function (ext/reflection golden output).
    public function __toString() {
        $src = ($this->__info['file'] ?? false) !== false ? '<user>' : '<internal>';
        $kind = $this->__closure !== null ? 'Closure' : 'Function';
        $s = "$kind [ $src function {$this->name} ] {\n";
        if (($this->__info['file'] ?? false) !== false) {
            $s .= "  @@ " . $this->__info['file'] . " " . ($this->__info['startLine'] ?? 0) . " - " . ($this->__info['endLine'] ?? 0) . "\n";
        }
        $params = $this->getParameters();
        if (count($params) > 0 || $this->hasReturnType()) {
            $s .= "\n  - Parameters [" . count($params) . "] {\n";
            foreach ($params as $p) { $s .= "    " . $p . "\n"; }
            $s .= "  }\n";
            if ($this->hasReturnType()) { $s .= "  - Return [ " . $this->getReturnType() . " ]\n"; }
        }
        return $s . "}\n";
    }
}
class ReflectionMethod extends ReflectionFunctionAbstract {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_STATIC = 16;
    const IS_FINAL = 32;
    const IS_ABSTRACT = 64;
    public $name;
    public $class;
    public $__info;
    public function getName() { return $this->name; }
    // A declared method is never a closure (ReflectionFunctionAbstract).
    public function isClosure() { return false; }
    public function getModifiers() {
        $bits = 0;
        if ($this->isPublic()) { $bits |= 1; }
        if ($this->isProtected()) { $bits |= 2; }
        if ($this->isPrivate()) { $bits |= 4; }
        if ($this->isStatic()) { $bits |= 16; }
        if ($this->isFinal()) { $bits |= 32; }
        if ($this->isAbstract()) { $bits |= 64; }
        return $bits;
    }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    // The hierarchy annotation Zend appends inside `<user...>` (see
    // ext/reflection _function_string): `inherits X` when the body lives in an
    // ancestor, `overwrites X` when this class redeclares a parent method, and
    // `prototype X` for the topmost interface/abstract origin. Order matches Zend:
    // inherits|overwrites, then prototype, then ctor.
    private function __hierAnnot() {
        $scope = $this->class;
        $declaring = $this->__info['declaringClass'];
        $out = '';
        if (strcasecmp($declaring, $scope) !== 0) {
            $out .= ", inherits $declaring";
        } else {
            $parent = get_parent_class($scope);
            if ($parent !== false && method_exists($parent, $this->name)) {
                $pm = new ReflectionMethod($parent, $this->name);
                $ps = $pm->getDeclaringClass()->getName();
                if (strcasecmp($ps, $scope) !== 0 && !$pm->isPrivate()) {
                    $out .= ", overwrites $ps";
                }
            }
        }
        $proto = $this->__protoScope();
        if ($proto !== null && strcasecmp($proto, $scope) !== 0) { $out .= ", prototype $proto"; }
        if ($this->isConstructor()) { $out .= ', ctor'; }
        return $out;
    }
    // Zend's `fptr->common.prototype->common.scope`: the topmost class or
    // interface that declares this method (independent of which subclass reflects
    // it, so computed from the declaring class). Interfaces are the ultimate
    // origin; among ancestor classes the furthest one wins. Returns null when the
    // method has no inherited counterpart (its prototype pointer is NULL in Zend).
    private function __protoScope() {
        $name = $this->name;
        $declaring = $this->__info['declaringClass'];
        // A root interface method that declares this name wins as the prototype.
        foreach (class_implements($declaring) as $iface) {
            if (method_exists($iface, $name)) {
                $rm = new ReflectionMethod($iface, $name);
                $root = $rm->getDeclaringClass()->getName();
                // topmost interface: one whose own super-interfaces do not declare it
                $topmost = true;
                foreach (class_implements($root) as $sup) {
                    if (strcasecmp($sup, $root) !== 0 && method_exists($sup, $name)) { $topmost = false; break; }
                }
                if ($topmost) { return $root; }
            }
        }
        // Otherwise the furthest ancestor class that declares it.
        $best = null;
        for ($p = get_parent_class($declaring); $p !== false; $p = get_parent_class($p)) {
            if (method_exists($p, $name)) {
                $rm = new ReflectionMethod($p, $name);
                if ($rm->isPrivate()) { break; }
                $best = $rm->getDeclaringClass()->getName();
            }
        }
        return $best;
    }
    // The PHP `Reflection::export` string for a method (ext/reflection golden output).
    public function __toString() {
        $src = $this->isUserDefined() ? ('<user' . $this->__hierAnnot() . '>') : ('<internal' . $this->__hierAnnot() . '>');
        $mods = '';
        if ($this->isAbstract()) { $mods .= 'abstract '; }
        if ($this->isFinal()) { $mods .= 'final '; }
        if ($this->isStatic()) { $mods .= 'static '; }
        $vis = $this->isPublic() ? 'public' : ($this->isProtected() ? 'protected' : 'private');
        $s = "Method [ $src {$mods}{$vis} method {$this->name} ] {\n";
        // An internal (prelude) method has no source location, so PHP omits the `@@`.
        if ($this->isUserDefined()) {
            $s .= "  @@ " . $this->getFileName() . " " . $this->getStartLine() . " - " . $this->getEndLine() . "\n";
        }
        if (!$this->isAbstract()) {
            $params = $this->getParameters();
            // The parameters/return block appears only when there is something in it.
            if (count($params) > 0 || $this->hasReturnType()) {
                $s .= "\n  - Parameters [" . count($params) . "] {\n";
                foreach ($params as $p) { $s .= "    " . $p . "\n"; }
                $s .= "  }\n";
                if ($this->hasReturnType()) { $s .= "  - Return [ " . $this->getReturnType() . " ]\n"; }
            }
        }
        return $s . "}\n";
    }
    public function __construct($objectOrClass, $method = null) {
        // A class-name string autoloads (Zend does; the info lookup below is
        // autoload-blind and would report "does not exist" for a not-yet-loaded
        // class).
        if (is_string($objectOrClass)) { class_exists(strpos($objectOrClass, '::') !== false ? explode('::', $objectOrClass, 2)[0] : $objectOrClass); }
        if ($method === null && is_string($objectOrClass) && strpos($objectOrClass, '::') !== false) {
            $parts = explode('::', $objectOrClass, 2);
            $objectOrClass = $parts[0]; $method = $parts[1];
        }
        $this->class = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        $this->name = $method;
        $this->__info = __reflect_method_info($this->class, $method);
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Method %s::%s() does not exist', $this->class, $method));
        }
    }
    public function isConstructor() { return strcasecmp($this->name, '__construct') === 0; }
    // Named constructor (8.3): build from a "Class::method" string.
    public static function createFromMethodName(string $method): static {
        $parts = explode('::', $method, 2);
        if (count($parts) !== 2) {
            throw new ReflectionException(sprintf('%s is not a valid method name', $method));
        }
        return new static($parts[0], $parts[1]);
    }
    public function hasPrototype() { return $this->__protoScope() !== null; }
    public function getPrototype() {
        $proto = $this->__protoScope();
        if ($proto === null) {
            throw new ReflectionException(sprintf('Method %s::%s does not have a prototype', $this->class, $this->name));
        }
        return new ReflectionMethod($proto, $this->name);
    }
    public function returnsReference() { return $this->__info['byRef'] ?? false; }
    // A method is deprecated when it carries the #[\Deprecated] attribute (8.4);
    // not inherited (an override without the attribute is not deprecated).
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function getStaticVariables() { return __reflect_static_vars($this->class, $this->name); }
    public function isGenerator() { return $this->__info['isGenerator'] ?? false; }
    public function hasTentativeReturnType() { return false; }
    public function getTentativeReturnType() { return null; }
    public function isUserDefined() { return ($this->__info['file'] ?? false) !== false; }
    public function isInternal() { return ($this->__info['file'] ?? false) === false; }
    public function isDestructor() { return strcasecmp($this->name, '__destruct') === 0; }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function getFileName() { return $this->__info['file']; }
    public function getStartLine() { return $this->__info['startLine']; }
    public function getEndLine() { return $this->__info['endLine']; }
    public function isStatic() { return $this->__info['static']; }
    public function isFinal() { return $this->__info['final']; }
    public function isAbstract() { return $this->__info['abstract']; }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function setAccessible($accessible) {}
    public function invoke($object, ...$args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    public function invokeArgs($object, $args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    // A Closure bound to the method: a static method ignores $object, an instance
    // method binds it (8.0+ allows null for a static method).
    public function getClosure($object = null) {
        return $this->isStatic()
            ? Closure::fromCallable($this->class . '::' . $this->name)
            : Closure::fromCallable([$object, $this->name]);
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_method_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionFunctionAbstract');
    }
}
class ReflectionProperty implements Reflector {
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function isFinal() { return false; }
    public function isAbstract() { return false; }
    // Asymmetric visibility (8.4): phpr does not model aviz declarations, so
    // every property reports symmetric get/set (ORM's RuntimeReflectionService
    // probes these on each mapped field).
    public function isProtectedSet() { return false; }
    public function isPrivateSet() { return false; }
    public function isVirtual() { return false; }
    public function hasHooks() { return false; }
    public function getHooks() { return []; }
    public function hasHook($type) { return false; }
    public function getHook($type) { return null; }
    public function isDynamic() { return $this->__info['dynamic'] ?? false; }
    // A declared property is a "default" property; a dynamic one is not.
    public function isDefault() { return !$this->isDynamic(); }
    // Constructor-promoted iff the declaring class's constructor has a promoted
    // parameter of the same name (derived from ReflectionParameter::isPromoted).
    public function isPromoted() {
        $ctor = (new ReflectionClass($this->class))->getConstructor();
        if ($ctor === null) { return false; }
        foreach ($ctor->getParameters() as $p) {
            if (strcasecmp($p->getName(), $this->name) === 0 && $p->isPromoted()) { return true; }
        }
        return false;
    }
    // The property's storage key in the object's property table: public keeps its
    // name, protected is `\0*\0name`, private is `\0DeclaringClass\0name` (matches
    // a `(array)` cast of the instance).
    public function getMangledName() {
        if ($this->isPublic()) { return $this->name; }
        if ($this->isProtected()) { return "\0*\0" . $this->name; }
        return "\0" . $this->__info['declaringClass'] . "\0" . $this->name;
    }
    // Without asymmetric visibility modelled, the settable type equals the type.
    public function getSettableType() { return $this->getType(); }
    public function setAccessible($accessible) {}
    public function isLazy($object = null) {
        return $object !== null && __lazy_prop_is_lazy($object, $this->class, $this->name);
    }
    const IS_STATIC = 16;
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_READONLY = 128;
    const IS_PROTECTED_SET = 2048;
    const IS_PRIVATE_SET = 4096;
    public $name;
    public $class;
    public $__info;
    public function __construct($class, $property) {
        $cls = is_object($class) ? get_class($class) : $class;
        // A dynamic property is reflectable when the INSTANCE is given.
        if (!property_exists($cls, $property)
            && !(is_object($class) && property_exists($class, $property))) {
            throw new ReflectionException(sprintf('Property %s::$%s does not exist', $cls, $property));
        }
        // The declaring class is the most-derived class that declares the property
        // itself (a child redeclaration shadows the parent's); mirrors
        // ReflectionProperty::$class. The host resolves it from the per-class
        // declared-property lists, which `property_exists` (inherited too) can't.
        $this->name = $property;
        $decl = __reflect_prop_declaring_class($cls, $property);
        $this->class = $decl === false ? $cls : $decl;
        $this->__info = __reflect_prop_details($this->class, $this->name);
        // An ancestor's PRIVATE property is invisible to the subclass's
        // reflection surface (Zend: getProperty throws; realize_skipped).
        if (is_array($this->__info) && isset($this->__info['visibility'])
            && $this->__info['visibility'] === 'private'
            && strcasecmp($this->class, $cls) !== 0) {
            throw new ReflectionException(sprintf('Property %s::$%s does not exist', $cls, $property));
        }
        if (!is_array($this->__info)) {
            // A dynamic property: public, untyped, no default.
            $this->__info = ['visibility' => 'public', 'static' => false, 'readonly' => false,
                'hasDefault' => false, 'default' => null, 'declaringClass' => $this->class,
                'dynamic' => true];
        }
    }
    public function getName() { return $this->name; }
    public function getValue($object = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            return __reflect_static_prop_get($this->class, $this->name);
        }
        return __reflect_prop_get($this->class, $this->name, $object);
    }
    public function setValue($object, $value = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            // Static form: setValue($value) and setValue(null, $value) both write
            // the class-level slot (Composer pokes InstalledVersions::$selfDir).
            if (func_num_args() === 1) { $value = $object; }
            __reflect_static_prop_set($this->class, $this->name, $value);
            return;
        }
        __reflect_prop_set($this->class, $this->name, $object, $value);
    }
    // PHP 8.4 raw accessors bypass property hooks; phpr's reflection reads the
    // backing slots directly already, so they alias get/setValue.
    public function getRawValue($object) { return $this->getValue($object); }
    public function setRawValue($object, $value) { $this->setValue($object, $value); }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_prop_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionProperty');
    }
    public function isStatic() { return __reflect_prop_is_static($this->class, $this->name); }
    public function hasType() { return __reflect_prop_type($this->class, $this->name) !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo(__reflect_prop_type($this->class, $this->name)); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isReadOnly() { return $this->__info['readonly']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['static']) { $m |= self::IS_STATIC; }
        if ($this->__info['readonly']) {
            $m |= self::IS_READONLY;
            // PHP 8.4: a `public readonly` property's implicit set-visibility is
            // downgraded to protected (asymmetric visibility), adding IS_PROTECTED_SET.
            if ($this->__info['visibility'] === 'public') { $m |= self::IS_PROTECTED_SET; }
        }
        return $m;
    }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function hasDefaultValue() { return $this->__info['hasDefault']; }
    public function getDefaultValue() { return $this->__info['default']; }
    // `Property [ VIS [static ][readonly ]TYPE $name[ = DEFAULT] ]` (export format).
    // Asymmetric set-visibility (`protected(set)`) is not modelled, so a readonly
    // property renders without it.
    public function __toString() {
        $mods = $this->isPublic() ? 'public' : ($this->isProtected() ? 'protected' : 'private');
        if ($this->isStatic()) { $mods .= ' static'; }
        if ($this->isReadOnly()) { $mods .= ' readonly'; }
        $type = '';
        if ($this->hasType()) {
            $t = $this->getType();
            if ($t instanceof ReflectionNamedType) {
                $tn = $t->getName();
                if ($t->allowsNull() && $tn !== 'null' && $tn !== 'mixed') { $tn = '?' . $tn; }
            } else {
                $tn = (string) $t;
            }
            $type = $tn . ' ';
        }
        $s = "Property [ $mods {$type}\$" . $this->name;
        if ($this->hasDefaultValue()) {
            $v = $this->getDefaultValue();
            if ($v === null) { $d = 'NULL'; }
            elseif (is_array($v)) { $d = str_replace("\n", '', var_export($v, true)); }
            else { $d = var_export($v, true); }
            $s .= ' = ' . $d;
        }
        return $s . " ]\n";
    }
    public function isInitialized($object = null) { return __reflect_prop_initialized($this->class, $this->name, $object); }
    public function skipLazyInitialization($object) {
        $msg = __lazy_skip_init($object, $this->class, $this->name);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
    public function setRawValueWithoutLazyInitialization($object, $value) {
        $msg = __lazy_set_raw($object, $this->class, $this->name, $value);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
}
class ReflectionExtension implements Reflector {
    public $name;
    public function __construct($name) {
        if (!extension_loaded($name)) {
            throw new ReflectionException(sprintf('Extension "%s" does not exist', $name));
        }
        // Canonical casing as get_loaded_extensions reports it, mirroring
        // ReflectionExtension::getName().
        foreach (get_loaded_extensions() as $ext) {
            if (strcasecmp($ext, $name) === 0) { $name = $ext; break; }
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getVersion() { return phpversion($this->name); }
    // phpr's registered extensions are all statically built in, never runtime-loaded.
    public function isPersistent() { return true; }
    public function isTemporary() { return false; }
    public function getDependencies() { return []; }
    // info() prints the phpinfo() block for the extension. phpr models these
    // extensions with Rust crates, not the C internals whose text this reports,
    // so it emits nothing (the OpenSSL-text-rendering class of rabbit hole);
    // callers parse it defensively (e.g. Composer regexes for an optional
    // sub-library version and simply skips it when absent).
    public function info() {}
    public function __toString() { return ''; }
    // Constants per extension. pcntl's table is what real consumers read
    // (monolog's SignalHandler maps signo -> "SIG*" name through it); values
    // are the macOS oracle's. Other extensions read as empty for now.
    public function getConstants() {
        if (strcasecmp($this->name, 'pcntl') === 0) {
            return [
                'WNOHANG' => 1, 'WUNTRACED' => 2, 'WCONTINUED' => 16, 'WEXITED' => 4,
                'WSTOPPED' => 8, 'WNOWAIT' => 32, 'P_ALL' => 0, 'P_PID' => 1, 'P_PGID' => 2,
                'SIG_IGN' => 1, 'SIG_DFL' => 0, 'SIG_ERR' => -1, 'SIGHUP' => 1, 'SIGINT' => 2,
                'SIGQUIT' => 3, 'SIGILL' => 4, 'SIGTRAP' => 5, 'SIGABRT' => 6, 'SIGIOT' => 6,
                'SIGBUS' => 10, 'SIGFPE' => 8, 'SIGKILL' => 9, 'SIGUSR1' => 30, 'SIGSEGV' => 11,
                'SIGUSR2' => 31, 'SIGPIPE' => 13, 'SIGALRM' => 14, 'SIGTERM' => 15,
                'SIGCHLD' => 20, 'SIGCONT' => 19, 'SIGSTOP' => 17, 'SIGTSTP' => 18,
                'SIGTTIN' => 21, 'SIGTTOU' => 22, 'SIGURG' => 16, 'SIGXCPU' => 24,
                'SIGXFSZ' => 25, 'SIGVTALRM' => 26, 'SIGPROF' => 27, 'SIGWINCH' => 28,
                'SIGIO' => 23, 'SIGINFO' => 29, 'SIGSYS' => 12, 'SIGBABY' => 12,
                'PRIO_PGRP' => 1, 'PRIO_USER' => 2, 'PRIO_PROCESS' => 0,
                'PRIO_DARWIN_BG' => 4096, 'PRIO_DARWIN_THREAD' => 3, 'SIG_BLOCK' => 1,
                'SIG_UNBLOCK' => 2, 'SIG_SETMASK' => 3, 'PCNTL_EINTR' => 4,
                'PCNTL_ECHILD' => 10, 'PCNTL_EINVAL' => 22, 'PCNTL_EAGAIN' => 35,
                'PCNTL_ESRCH' => 3, 'PCNTL_EACCES' => 13, 'PCNTL_EPERM' => 1,
                'PCNTL_ENOMEM' => 12, 'PCNTL_E2BIG' => 7, 'PCNTL_EFAULT' => 14,
                'PCNTL_EIO' => 5, 'PCNTL_EISDIR' => 21, 'PCNTL_ELOOP' => 62,
                'PCNTL_EMFILE' => 24, 'PCNTL_ENAMETOOLONG' => 63, 'PCNTL_ENFILE' => 23,
                'PCNTL_ENOENT' => 2, 'PCNTL_ENOEXEC' => 8, 'PCNTL_ENOTDIR' => 20,
                'PCNTL_ETXTBSY' => 26, 'PCNTL_ENOSPC' => 28, 'PCNTL_EUSERS' => 68,
            ];
        }
        return [];
    }
}
