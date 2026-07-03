//! VM oop logic, extracted from vm/mod.rs (no semantic change).
use super::*;

pub(super) fn read_property(recv: &Zval, name: &[u8], diags: &mut Diags) -> Zval {
    match recv {
        Zval::Object(o) => {
            let obj = o.borrow();
            if let Some(v) = obj.props.get(name) {
                return v.deref_clone();
            }
            let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
            drop(obj);
            // `name` may be a mangled storage key; diagnostics show the source name.
            let prop = String::from_utf8_lossy(php_types::prop_display_name(name)).into_owned();
            diags.push(Diag::Warning(format!("Undefined property: {cls}::${prop}")));
            Zval::Null
        }
        Zval::Ref(rc) => read_property(&rc.borrow(), name, diags),
        Zval::Null | Zval::Undef => {
            let prop = String::from_utf8_lossy(name).into_owned();
            diags.push(Diag::Warning(format!("Attempt to read property \"{prop}\" on null")));
            Zval::Null
        }
        other => {
            let prop = String::from_utf8_lossy(name).into_owned();
            diags.push(Diag::Warning(format!(
                "Attempt to read property \"{prop}\" on {}",
                other.type_name_for_error()
            )));
            Zval::Null
        }
    }
}

/// Write `value` into property `name` of `recv` (following a `Ref` receiver),
/// returning the value it displaced (`None` if the property was newly created).
/// Callers that track destruction timing pass the displaced value to
/// [`Vm::gc_note`]; the others drop it (unchanged behaviour).
pub(super) fn write_property(recv: &Zval, name: &[u8], value: Zval) -> Result<Option<Zval>, PhpError> {
    match recv {
        Zval::Object(o) => Ok(o.borrow_mut().props.replace(name, value)),
        Zval::Ref(rc) => write_property(&rc.borrow(), name, value),
        other => Err(PhpError::Error(format!(
            "Attempt to assign property \"{}\" on {}",
            String::from_utf8_lossy(name),
            other.type_name_for_error()
        ))),
    }
}

/// The shared storage cell for object property `name`, promoting it to a
/// `Zval::Ref` in place if it is a plain value (so a `foreach ($o as &$v)` binds
/// `$v` to the property and writes through it). The property must already exist.
pub(super) fn prop_ref_cell(o: &Rc<RefCell<Object>>, name: &[u8]) -> Rc<RefCell<Zval>> {
    let mut b = o.borrow_mut();
    if let Some(Zval::Ref(rc)) = b.props.get(name) {
        return Rc::clone(rc);
    }
    let cur = b.props.get(name).cloned().unwrap_or(Zval::Null);
    let cell = Rc::new(RefCell::new(cur));
    b.props.set(name, Zval::Ref(Rc::clone(&cell)));
    cell
}

/// `isset($o->name)`: true iff the property exists and is not null/undefined
/// (silent), following a reference receiver.
pub(super) fn prop_isset(recv: &Zval, name: &[u8]) -> bool {
    match recv {
        Zval::Object(o) => match o.borrow().props.get(name) {
            Some(v) => !matches!(v.deref_clone(), Zval::Null | Zval::Undef),
            None => false,
        },
        Zval::Ref(rc) => prop_isset(&rc.borrow(), name),
        _ => false,
    }
}

/// `unset($o->name)`: remove the property (no-op if absent or non-object).
pub(super) fn prop_unset(recv: &Zval, name: &[u8]) {
    match recv {
        Zval::Object(o) => {
            o.borrow_mut().props.remove(name);
        }
        Zval::Ref(rc) => prop_unset(&rc.borrow(), name),
        _ => {}
    }
}

/// Resolve a method by name at run time, walking the receiver class's `parent`
/// chain child→ancestor (case-insensitive). Returns the *defining* class id and
/// the method's index in [`crate::bytecode::CompiledClass::methods`].
pub(super) fn resolve_method_runtime(classes: &[&CompiledClass], start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut cid = Some(start);
    while let Some(c) = cid {
        // `.get` rather than `[c]`: defensive against a stale id; the global class
        // table (step 57, Phase 1c) keeps ids valid across modules.
        let class = classes.get(c)?;
        if let Some(i) = class.methods.iter().position(|m| m.name.eq_ignore_ascii_case(name)) {
            return Some((c, i));
        }
        cid = class.parent;
    }
    None
}

/// The class id of an object value (following a reference), or `None` for a
/// non-object.
pub(super) fn object_class_id(v: &Zval) -> Option<ClassId> {
    match v {
        Zval::Object(o) => Some(o.borrow().class_id as usize),
        Zval::Ref(rc) => object_class_id(&rc.borrow()),
        _ => None,
    }
}

/// The object a value resolves to (following a reference), or `None` for a
/// non-object — for inspecting a field-path base that may be a `Ref`.
pub(super) fn deref_object(v: &Zval) -> Option<Rc<RefCell<Object>>> {
    match v {
        Zval::Object(o) => Some(o.clone()),
        Zval::Ref(rc) => deref_object(&rc.borrow()),
        _ => None,
    }
}

/// The object id of a value (following a reference), or 0 for a non-object — the
/// key the hook guard uses to mark an active property hook on an instance.
pub(super) fn object_id(v: &Zval) -> u32 {
    match v {
        Zval::Object(o) => o.borrow().id,
        Zval::Ref(rc) => object_id(&rc.borrow()),
        _ => 0,
    }
}

/// Whether class `a` is `b` or descends from it (parent chain only) — the test
/// behind forwarding `$this` propagation for `Parent::m()`-style calls.
pub(super) fn class_is_a(classes: &[&CompiledClass], a: ClassId, b: ClassId) -> bool {
    let mut cur = Some(a);
    while let Some(c) = cur {
        if c == b {
            return true;
        }
        cur = classes[c].parent;
    }
    false
}

/// Resolve a class constant at run time (for `static::CONST`): own constants and
/// parent chain first, then interfaces transitively. Returns the declaring class
/// id and the constant's index. Case-sensitive, like PHP and the compiler's
/// `find_class_const`.
pub(super) fn find_const_runtime(classes: &[&CompiledClass], start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut c = Some(start);
    while let Some(x) = c {
        if let Some(i) = classes[x].consts.iter().position(|k| k.name.as_ref() == name) {
            return Some((x, i));
        }
        c = classes[x].parent;
    }
    let mut c = Some(start);
    while let Some(x) = c {
        for &i in &classes[x].interfaces {
            if let Some(r) = find_const_runtime(classes, i, name) {
                return Some(r);
            }
        }
        c = classes[x].parent;
    }
    None
}

/// The "call to undefined method" fatal, shared by instance and static dispatch.
pub(super) fn undefined_method(classes: &[&CompiledClass], cid: ClassId, method: &[u8]) -> PhpError {
    PhpError::Error(format!(
        "Call to undefined method {}::{}()",
        String::from_utf8_lossy(&classes[cid].name),
        String::from_utf8_lossy(method)
    ))
}

/// Whether a member of visibility `vis` declared on `decl` is accessible from the
/// running frame's class `cur` (OOP-2b), mirroring the tree-walker's
/// `visible_from`: public always; private only from the declaring class;
/// protected from anywhere in the same hierarchy.
pub(super) fn visible_from(classes: &[&CompiledClass], cur: Option<ClassId>, vis: Visibility, decl: ClassId) -> bool {
    match vis {
        Visibility::Public => true,
        Visibility::Private => cur == Some(decl),
        Visibility::Protected => matches!(
            cur,
            Some(cc) if class_is_a(classes, cc, decl) || class_is_a(classes, decl, cc)
        ),
    }
}

/// [`visible_from`] for a *method*, with Zend's prototype rule for `protected`:
/// the check runs against the ROOT declaration up the parent chain, so a
/// sibling subclass may call an override it inherited access to
/// (zend_check_protected on `fbc->common.prototype` — PHPUnit's UnaryOperator
/// calling IsIdentical::failureDescription, both under Constraint).
pub(super) fn method_visible_from(
    classes: &[&CompiledClass],
    cur: Option<ClassId>,
    vis: Visibility,
    decl: ClassId,
    method: &[u8],
) -> bool {
    if !matches!(vis, Visibility::Protected) {
        return visible_from(classes, cur, vis, decl);
    }
    // Walk to the outermost ancestor that still declares (a non-private)
    // `method`: that is the prototype's scope.
    let mut root = decl;
    let mut cur_cls = classes[decl].parent;
    while let Some(p) = cur_cls {
        let declares = classes[p]
            .methods
            .iter()
            .any(|m| m.func.name.eq_ignore_ascii_case(method) && m.visibility != Visibility::Private);
        if declares {
            root = p;
        }
        cur_cls = classes[p].parent;
    }
    visible_from(classes, cur, vis, root)
}

/// Look up the unified, compile-time-resolved metadata for a declared instance
/// property — a single hashmap lookup over the parent-flattened `prop_info` table
/// (the flattening, with all shadowing rules applied, happened in `compile_class`).
/// `None` for a dynamic / undeclared property. The `.get(class)?` is defensive
/// against a partially-seeded class table.
pub(super) fn prop_info<'a>(classes: &[&'a CompiledClass], class: ClassId, name: &[u8]) -> Option<&'a PropInfo> {
    classes.get(class)?.prop_info.get(name)
}

/// Outcome of resolving a property access `obj->name` from a given `scope`
/// (the property-mangling resolver). Decides both *which storage slot* an access
/// targets and *whether* it is permitted.
pub(super) enum PropAccess {
    /// An accessible declared property: read/write under this storage key (the
    /// plain name today; a mangled `\0Class\0name` for a private once mangling is on).
    Slot(Vec<u8>),
    /// No accessible declared property of this name — an undeclared (dynamic)
    /// property, or (once mangling lands) a private declared only by an ancestor and
    /// invisible to a related scope. Behaves as a dynamic property under the plain
    /// name (a read warns "Undefined property", a write creates it).
    Dynamic,
    /// Declared but not accessible from `scope`: the caller raises the visibility
    /// error (after first offering a magic accessor).
    Denied { decl: ClassId, vis: Visibility },
}

/// Resolve a property access `obj_class->name` from `scope`. Single source of truth
/// for both the storage key and the visibility decision, mirroring Zend's
/// zend_get_property_info:
///
/// 1. If the running scope itself declares a *private* `name` and the object is an
///    instance of that scope, the access targets the scope's mangled slot — even if
///    a subclass redeclared a same-name private (the dual-slot case).
/// 2. Otherwise the object's flattened table decides: an undeclared name is
///    `Dynamic`; a visible declaration resolves to its slot; a *parent's* private
///    on a child instance behaves as undeclared (Zend drops it from the child's
///    table — reads warn "Undefined property", writes create a dynamic one); only
///    an invisible private declared by the object's own class (or an invisible
///    protected) is `Denied`.
pub(super) fn resolve_prop_access(classes: &[&CompiledClass], obj_class: ClassId, name: &[u8], scope: Option<ClassId>) -> PropAccess {
    if let Some(s) = scope {
        if let Some(pi) = prop_info(classes, s, name) {
            if pi.visibility == Visibility::Private
                && pi.declaring_class == s
                && class_is_a(classes, obj_class, s)
            {
                return PropAccess::Slot(pi.storage_key.to_vec());
            }
        }
    }
    match prop_info(classes, obj_class, name) {
        None => PropAccess::Dynamic,
        Some(pi) => {
            if visible_from(classes, scope, pi.visibility, pi.declaring_class) {
                PropAccess::Slot(pi.storage_key.to_vec())
            } else if pi.visibility == Visibility::Private && pi.declaring_class != obj_class {
                PropAccess::Dynamic
            } else {
                PropAccess::Denied { decl: pi.declaring_class, vis: pi.visibility }
            }
        }
    }
}

/// The property-resolution context a mixed field path (the recursive
/// `field_write` / `field_get` / `field_unset` / `field_cell` walkers) drills
/// with: the loaded classes plus the scope every `Prop` step resolves from.
/// `Copy`, so the walkers thread it for free; the storage key is re-resolved at
/// each `Prop` step against the class of the object actually encountered there,
/// under the *same* scope — exactly Zend, where every FETCH_OBJ resolves with
/// the executing function's scope.
#[derive(Clone, Copy)]
pub(super) struct FieldScope<'a> {
    pub(super) classes: &'a [&'a CompiledClass],
    pub(super) scope: Option<ClassId>,
}

impl FieldScope<'_> {
    /// The storage key a `Prop` step addresses on an instance of `ocid`: the
    /// resolved slot key for an accessible declared property, the plain name for
    /// a dynamic one. A `Denied` step keeps addressing the *declared* slot — the
    /// mixed-path walkers have never enforced visibility (pre-existing gap: no
    /// `__get`/`__set` protocol on intermediate steps either), so an inaccessible
    /// private must keep hitting its real storage rather than autovivifying a
    /// parallel dynamic property (Bug #34893's `$a->p->t = 'bar'` through `__get`).
    /// Like [`Self::prop_key`], but for the SILENT read walker (`field_get`,
    /// backing isset/empty/`??`): an inaccessible declared property reads as
    /// absent (`None`) — PHP's `isset($o->private)` from outside is false and
    /// `$o->private ?? $d` yields `$d`, with no error (mirrors Op::PropIsset).
    pub(super) fn prop_key_read<'n>(&self, ocid: ClassId, name: &'n [u8]) -> Option<std::borrow::Cow<'n, [u8]>> {
        match resolve_prop_access(self.classes, ocid, name, self.scope) {
            PropAccess::Slot(k) => Some(std::borrow::Cow::Owned(k)),
            PropAccess::Dynamic => Some(std::borrow::Cow::Borrowed(name)),
            PropAccess::Denied { .. } => None,
        }
    }

    pub(super) fn prop_key<'n>(&self, ocid: ClassId, name: &'n [u8]) -> std::borrow::Cow<'n, [u8]> {
        match resolve_prop_access(self.classes, ocid, name, self.scope) {
            PropAccess::Slot(k) => std::borrow::Cow::Owned(k),
            PropAccess::Denied { .. } => match prop_info(self.classes, ocid, name) {
                Some(pi) => std::borrow::Cow::Owned(pi.storage_key.to_vec()),
                None => std::borrow::Cow::Borrowed(name),
            },
            PropAccess::Dynamic => std::borrow::Cow::Borrowed(name),
        }
    }
}

/// The declaring class of a `readonly` instance property, or `None` if
/// non-readonly / dynamic. The shadowing (a more-derived non-readonly
/// redeclaration cancels) is already baked into `PropInfo.readonly`.
pub(super) fn prop_readonly_decl(classes: &[&CompiledClass], class: ClassId, name: &[u8]) -> Option<ClassId> {
    prop_info(classes, class, name).filter(|pi| pi.readonly).map(|pi| pi.declaring_class)
}

/// A typed instance property's declaring class and declared type, or `None` if
/// untyped / dynamic. The untyped-redeclaration-cancels-type shadowing is already
/// baked into `PropInfo.type_hint`.
pub(super) fn prop_type_decl(classes: &[&CompiledClass], class: ClassId, name: &[u8]) -> Option<(ClassId, TypeHint)> {
    let pi = prop_info(classes, class, name)?;
    pi.type_hint.clone().map(|h| (pi.declaring_class, h))
}

/// A declared instance property's visibility and declaring class, or `None` if
/// dynamic / undeclared.
pub(super) fn prop_vis_decl(classes: &[&CompiledClass], class: ClassId, name: &[u8]) -> Option<(Visibility, ClassId)> {
    prop_info(classes, class, name).map(|pi| (pi.visibility, pi.declaring_class))
}

/// Resolve a static property to its declaring class and index, walking the parent
/// chain (OOP-2b).
pub(super) fn find_static_prop(classes: &[&CompiledClass], start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut cid = Some(start);
    while let Some(c) = cid {
        if let Some(i) = classes[c].static_props.iter().position(|p| p.name.as_ref() == name) {
            return Some((c, i));
        }
        cid = classes[c].parent;
    }
    None
}

/// Enforce instance-property visibility for an access from frame class `cur` on an
/// object of `obj_class`. A dynamic / undeclared property is always accessible;
/// the error cases are exactly [`resolve_prop_access`]'s `Denied` outcomes.
pub(super) fn check_prop_access(
    classes: &[&CompiledClass],
    cur: Option<ClassId>,
    obj_class: ClassId,
    name: &[u8],
) -> Result<(), PhpError> {
    match resolve_prop_access(classes, obj_class, name, cur) {
        PropAccess::Denied { decl, vis } => Err(prop_access_error(classes, decl, name, vis)),
        _ => Ok(()),
    }
}

/// The "Cannot access {private,protected} property C::$p" fatal.
pub(super) fn prop_access_error(classes: &[&CompiledClass], decl: ClassId, name: &[u8], vis: Visibility) -> PhpError {
    let kind = if matches!(vis, Visibility::Private) { "private" } else { "protected" };
    PhpError::Error(format!(
        "Cannot access {kind} property {}::${}",
        String::from_utf8_lossy(&classes[decl].name),
        String::from_utf8_lossy(name)
    ))
}

/// The "Call to {private,protected} method C::m() from <scope>" fatal.
pub(super) fn method_access_error(
    classes: &[&CompiledClass],
    decl: ClassId,
    method: &[u8],
    cur: Option<ClassId>,
    vis: Visibility,
) -> PhpError {
    let kind = if matches!(vis, Visibility::Private) { "private" } else { "protected" };
    let scope = match cur {
        Some(c) => format!("scope {}", String::from_utf8_lossy(&classes[c].name)),
        None => "global scope".to_string(),
    };
    PhpError::Error(format!(
        "Call to {kind} method {}::{}() from {scope}",
        String::from_utf8_lossy(&classes[decl].name),
        String::from_utf8_lossy(method)
    ))
}

/// Decide whether writing readonly property `decl::$name` from scope `cur` is
/// allowed, given whether the property is already initialised on the instance.
/// Returns the fatal to raise, or `None` if the write is a permitted first
/// initialisation. Mirrors PHP 8.4: an already-initialised readonly property
/// cannot be modified from *any* scope; an uninitialised one carries implicit
/// `protected(set)` visibility, so it may only be initialised from within the
/// declaring class hierarchy (else "from <scope>").
pub(super) fn readonly_write_error(
    classes: &[&CompiledClass],
    cur: Option<ClassId>,
    decl: ClassId,
    name: &[u8],
    initialized: bool,
) -> Option<PhpError> {
    let cls = String::from_utf8_lossy(&classes[decl].name);
    let prop = String::from_utf8_lossy(name);
    if initialized {
        return Some(PhpError::Error(format!("Cannot modify readonly property {cls}::${prop}")));
    }
    // Uninitialised: allowed only from the declaring class or a subclass
    // (protected(set) semantics).
    if visible_from(classes, cur, Visibility::Protected, decl) {
        return None;
    }
    let scope = match cur {
        Some(c) => format!("scope {}", String::from_utf8_lossy(&classes[c].name)),
        None => "global scope".to_string(),
    };
    Some(PhpError::Error(format!(
        "Cannot modify protected(set) readonly property {cls}::${prop} from {scope}"
    )))
}

/// Whether an object of `class_id` is an instance of `target`: the class itself,
/// any ancestor, or any implemented interface (transitively), mirroring the
/// tree-walker's `is_instance_of` (OOP-1 omits the `Stringable` auto-impl).
pub(super) fn is_instance_of(
    classes: &[&CompiledClass],
    stringable: Option<ClassId>,
    class_id: ClassId,
    target: ClassId,
) -> bool {
    // `Stringable` is auto-implemented (step 24-1): any class with a resolvable
    // `__toString` satisfies it, even without an explicit `implements Stringable`.
    if stringable == Some(target)
        && resolve_method_runtime(classes, class_id, b"__toString").is_some()
    {
        return true;
    }
    let mut cur = Some(class_id);
    while let Some(c) = cur {
        if c == target {
            return true;
        }
        if classes[c].interfaces.iter().any(|&i| iface_is_a(classes, i, target)) {
            return true;
        }
        cur = classes[c].parent;
    }
    false
}

/// Whether interface `i` is, or transitively extends, `target`.
pub(super) fn iface_is_a(classes: &[&CompiledClass], i: ClassId, target: ClassId) -> bool {
    if i == target {
        return true;
    }
    classes[i].interfaces.iter().any(|&p| iface_is_a(classes, p, target))
}

impl<'m> Vm<'m> {
    /// `(object)` cast (PAR): an object passes through; an array becomes a
    /// stdClass with one property per element (int keys stringified); null/unset
    /// is an empty stdClass; a scalar becomes `stdClass { scalar: v }`. Mirrors
    /// `eval::object_cast`.
    pub(super) fn object_cast(&mut self, v: Zval) -> Result<Zval, PhpError> {
        match v.deref_clone() {
            obj @ Zval::Object(_) => Ok(obj),
            Zval::Array(a) => {
                let obj = self.alloc_stdclass()?;
                if let Zval::Object(o) = &obj {
                    let mut b = o.borrow_mut();
                    for (k, val) in a.iter() {
                        let name = match k {
                            Key::Int(i) => i.to_string().into_bytes(),
                            Key::Str(s) => s.as_bytes().to_vec(),
                        };
                        b.props.set(&name, val.deref_clone());
                    }
                }
                Ok(obj)
            }
            Zval::Null | Zval::Undef => self.alloc_stdclass(),
            scalar => {
                let obj = self.alloc_stdclass()?;
                if let Zval::Object(o) = &obj {
                    o.borrow_mut().props.set(b"scalar", scalar);
                }
                Ok(obj)
            }
        }
    }

    /// Allocate a fresh empty `stdClass` instance (PAR), for `(object)` casts.
    pub(super) fn alloc_stdclass(&mut self) -> Result<Zval, PhpError> {
        let cid = self
            .module
            .class_index
            .get(&b"stdclass"[..])
            .copied()
            .ok_or_else(|| PhpError::Error("VM: stdClass is not available".to_string()))?;
        self.alloc_object(cid)
    }

    /// Resolve a runtime class-reference value to its class id (PAR, dynamic
    /// class): an object reuses its class; a string is looked up
    /// case-insensitively with a leading `\` stripped; anything else (or an
    /// unknown name) yields `None`. Used by `instanceof $cls` (where `None` means
    /// `false`); `new $cls` resolves inline so it can distinguish the error kinds.
    pub(super) fn class_id_from_value(&self, v: &Zval) -> Option<ClassId> {
        match v {
            Zval::Object(o) => Some(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                self.class_index.get(&name.to_ascii_lowercase()).copied()
            }
            Zval::Ref(r) => self.class_id_from_value(&r.borrow()),
            _ => None,
        }
    }

    pub(super) fn magic_applies(
        &self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
        cur_class: Option<ClassId>,
        kind: MagicKind,
        magic_name: &[u8],
    ) -> Option<(ClassId, usize, u32)> {
        let (cid, oid, present, accessible) = {
            let obj = o.borrow();
            let cid = obj.class_id as usize;
            // Presence is judged at the *resolved* slot: a declared-but-unset
            // private (mangled key) triggers magic from its own scope; a parent's
            // private resolved `Dynamic` from a child scope is "present" only if a
            // plain dynamic slot exists.
            let (present, accessible) = match resolve_prop_access(&self.classes, cid, name, cur_class) {
                PropAccess::Slot(k) => (obj.props.contains(k.as_slice()), true),
                PropAccess::Dynamic => (obj.props.contains(name), true),
                PropAccess::Denied { .. } => (obj.props.contains(name), false),
            };
            (cid, obj.id, present, accessible)
        };
        if present && accessible {
            return None;
        }
        if self.magic_guard.contains(&(oid, kind, name.to_vec())) {
            return None;
        }
        let (defc, midx) = resolve_method_runtime(&self.classes, cid, magic_name)?;
        Some((defc, midx, oid))
    }

    /// Run `__destruct` on every object still tracked at the end of the script,
    /// in reverse creation order (PHP shutdown is LIFO), step OOP-3d. The frame
    /// stack is cleared first so this works even after a fatal unwound `main`.
    /// Run `register_shutdown_function` callbacks in registration order at script
    /// end — after the main run (and any uncaught-fatal banner), before object
    /// destructors. A synthetic `main` frame gives `call_callable` a caller frame
    /// for value-builtin results. A throw inside a callback is swallowed (PHP turns
    /// it into a separate shutdown-time fatal, not modelled here).
    pub(super) fn run_shutdown_functions(&mut self) {
        if self.shutdown_fns.is_empty() {
            return;
        }
        self.frames.clear();
        self.frames.push(Frame::new(&self.module.main, self.module));
        for (cb, args) in std::mem::take(&mut self.shutdown_fns) {
            // `exit`/`die` in a shutdown callback aborts the remaining callbacks
            // (PHP). Other throws are swallowed (a separate fatal is not modelled).
            if let Err(PhpError::Exit(_)) = self.call_callable(cb, args) {
                break;
            }
        }
        self.frames.clear();
    }

    pub(super) fn run_shutdown_destructors(&mut self) {
        self.frames.clear();
        let survivors = std::mem::take(&mut self.created);
        for o in survivors.into_iter().rev() {
            let (cid, id) = {
                let b = o.borrow();
                (b.class_id as usize, b.id)
            };
            if self.destructed.contains(&id) {
                continue;
            }
            // A lazy *wrapper* (uninitialized ghost, or a proxy) never runs its own
            // `__destruct` (PHP 8.4) — mirrors the `gc_sweep` rule for objects that
            // survive to shutdown. The real instance behind a proxy is itself a
            // tracked survivor and runs its destructor on its own turn.
            if o.borrow().lazy.is_some() {
                continue;
            }
            if let Some((defc, midx)) = resolve_method_runtime(&self.classes, cid, b"__destruct") {
                self.destructed.insert(id);
                let callee = &self.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee, self.class_mod(defc));
                frame.this = Some(Zval::Object(Rc::clone(&o)));
                frame.class = Some(defc);
                frame.static_class = Some(cid);
                self.frames.push(frame);
                // Drive the destructor to completion; swallow any fatal it raises
                // (PHP turns a shutdown-time throw into a separate fatal).
                let _ = self.run();
            }
        }
    }

    /// Dispatch an instance method call `obj->method(args)` where the receiver's
    /// class `cid` and bound `$this` are already resolved (OOP). A missing or
    /// inaccessible target routes to `__call`, otherwise raises the visibility /
    /// Dispatch an instance method call `$this->method(args)` with `$this` already
    /// deref-cloned. A `Generator`/`Fiber` receiver routes to the native built-in
    /// methods (their result is pushed directly); any other object resolves the
    /// method at run time via [`Self::dispatch_instance_call`]; a non-object is the
    /// "Call to a member function …() on …" fatal. Shared by [`Op::MethodCall`] and
    /// the spread variant [`Op::MethodCallArgs`].
    pub(super) fn method_call(
        &mut self,
        top: usize,
        this: Zval,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<(), PhpError> {
        // A `Generator` is not a user object: dispatch its built-in methods
        // (current/key/next/valid/rewind/…) directly (GEN).
        if let Zval::Generator(gs) = &this {
            let gs = Rc::clone(gs);
            // Native dispatch — no `bind_params` step — so a reference pushed by a
            // dynamic call (SEND_VAR_EX) is decayed to its value first.
            let result = self.generator_method(gs, method, decay_args(args))?;
            self.frames[top].stack.push(result);
            return Ok(());
        }
        // A `Fiber` instance's methods (start/resume/getReturn/is*) are dispatched
        // natively, except `__construct` which runs the prelude body (GEN-4).
        if let (Zval::Object(o), Some(fcid)) = (&this, self.fiber_class_id) {
            let cid = o.borrow().class_id as usize;
            if is_instance_of(&self.classes, self.stringable_id, cid, fcid) {
                let result = self.fiber_method(&this, method, decay_args(args))?;
                self.frames[top].stack.push(result);
                return Ok(());
            }
        }
        // A closure value's built-in methods (`$c->bindTo(...)`, `$c->call(...)`)
        // are dispatched natively — a closure is not a user object (step 19-6).
        if let Zval::Closure(cl) = &this {
            let cl = Rc::clone(cl);
            let result = self.closure_instance_method(&cl, method, decay_args(args))?;
            self.frames[top].stack.push(result);
            return Ok(());
        }
        let cid = match &this {
            Zval::Object(o) => o.borrow().class_id as usize,
            other => {
                return Err(PhpError::Error(format!(
                    "Call to a member function {}() on {}",
                    String::from_utf8_lossy(method),
                    other.type_name_for_error()
                )))
            }
        };
        self.dispatch_instance_call(top, cid, this, method, args)
    }

    /// Dispatch an instance method call `$this->method(positional…, named…)` whose
    /// **named arguments** are bound at run time against the callee's `param_names`
    /// (Session A). A non-object receiver is the "Call to a member function …" fatal
    /// (a `Generator`/`Fiber`'s native methods take no named arguments — the first
    /// name is reported as unknown). A resolved-and-visible user method binds via
    /// [`build_named_frame`]; a missing/inaccessible one routes to `__call` (the
    /// named args ride in the `$args` array as string keys, like the evaluator),
    /// else the visibility / undefined-method error.
    pub(super) fn dispatch_instance_call_named(
        &mut self,
        top: usize,
        this: Zval,
        method: &[u8],
        positional: Vec<Zval>,
        named: Vec<(Box<[u8]>, Zval)>,
    ) -> Result<(), PhpError> {
        let cid = match &this {
            // A `Generator`/`Fiber`'s native methods take no named arguments.
            Zval::Generator(_) => return Err(unknown_named_param(&named)),
            Zval::Object(o) => o.borrow().class_id as usize,
            other => {
                return Err(PhpError::Error(format!(
                    "Call to a member function {}() on {}",
                    String::from_utf8_lossy(method),
                    other.type_name_for_error()
                )))
            }
        };
        if let Some(fcid) = self.fiber_class_id {
            if is_instance_of(&self.classes, self.stringable_id, cid, fcid) {
                return Err(unknown_named_param(&named));
            }
        }
        let module = self.module;
        let resolved = resolve_method_runtime(&self.classes, cid, method);
        let usable = resolved.filter(|&(defc, midx)| {
            method_visible_from(&self.classes, self.frames[top].class, self.classes[defc].methods[midx].visibility, defc, method)
        });
        match usable {
            Some((defc, midx)) => {
                let callee = &self.classes[defc].methods[midx].func;
                let qn = format!(
                    "{}::{}",
                    String::from_utf8_lossy(&self.classes[defc].name),
                    String::from_utf8_lossy(method)
                );
                let mut frame = build_named_frame(callee, module, &qn, positional, named)?;
                frame.this = Some(this);
                frame.class = Some(defc);
                frame.static_class = Some(cid); // LSB = receiver's actual class
                self.enter_callee(frame)?;
            }
            None => match resolve_method_runtime(&self.classes, cid, b"__call") {
                Some((cdefc, cmidx)) => {
                    self.push_magic_call_named(cdefc, cmidx, Some(this), cid, method, positional, named);
                }
                None => {
                    return Err(match resolved {
                        Some((defc, midx)) => method_access_error(
                            &self.classes,
                            defc,
                            method,
                            self.frames[top].class,
                            self.classes[defc].methods[midx].visibility,
                        ),
                        None => undefined_method(&self.classes, cid, method),
                    })
                }
            },
        }
        Ok(())
    }
}
