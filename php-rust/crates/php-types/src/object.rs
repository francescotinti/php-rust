//! PHP object instances (step 19, OOP).
//!
//! A PHP object has **handle (reference) semantics**: the value is
//! `Zval::Object(Rc<RefCell<Object>>)`, so assigning an object (`$q = $p`) shares
//! the `Rc` and a mutation through any handle is visible through all the others.
//! This deliberately contrasts with arrays, which are copy-on-write value types
//! (`Rc<PhpArray>` + `Rc::make_mut`). The interior `RefCell` is what lets the
//! evaluator mutate a shared instance in place without cloning it (D-19.1).

use std::rc::Rc;

use crate::{PhpStr, Zval};

/// One object instance. `class_id` indexes the program's class table for method
/// resolution / `instanceof` (evaluator side); `class_name` is carried in the
/// value itself so `var_dump` and error messages can render it without the class
/// table (mirrors how `Closure` carries its `ClosureInfo`, D-19.2).
#[derive(Debug)]
pub struct Object {
    pub class_id: u32,
    pub class_name: Rc<PhpStr>,
    /// Declared and dynamic properties, in insertion order.
    pub props: Props,
    /// Object handle (`#N` in `var_dump`), assigned monotonically at creation.
    pub id: u32,
    /// Per-class render metadata (declared-property visibility) so `var_dump` /
    /// `print_r` can annotate `:protected` / `:"C":private` without the class
    /// table (step 19-7, D-19.20). Shared (`Rc`) by all instances of a class.
    pub info: Rc<ObjectInfo>,
    /// Names of `readonly` properties that have been **initialised** on this
    /// instance (readonly enforcement). A readonly property is write-once: the
    /// first in-scope write records its name here; any later write fatals with
    /// "Cannot modify readonly property", and a read before initialisation fatals
    /// with "must not be accessed before initialization". Empty for objects with
    /// no readonly properties, so the common case costs nothing.
    pub readonly_init: Vec<Box<[u8]>>,
    /// Readonly properties that may be **re-initialised once** right now (PHP 8.3
    /// readonly-clone amendment): populated by the `clone` operator before it runs
    /// `__clone`, one entry per readonly property, and revoked when that `__clone`
    /// frame returns. A write consumes the matching entry; a manual `__clone()`
    /// call leaves this empty, so a readonly write there still fatals. Empty for
    /// every object outside an active clone, so the common case costs nothing.
    pub readonly_clone_writable: Vec<Box<[u8]>>,
    /// Declared TYPED properties explicitly `unset()` on this instance. Zend
    /// keeps the slot UNDEF but clears its IS_PROP_UNINIT flag: the property
    /// still renders `uninitialized` (var_dump/reflection), but a read now
    /// dispatches `__get` (never-initialized reads keep the before-init
    /// fatal instead — symfony Constraint's lazy-groups idiom). Meaningful
    /// only while the slot holds `Undef`; a re-assignment makes it moot.
    /// Empty for the common object, so it costs nothing.
    pub typed_unset: Vec<Box<[u8]>>,
    /// Lazy-object marker (PHP 8.4): `Some` while the object is a lazy
    /// ghost/proxy. A **ghost** clears this to `None` on initialization (it
    /// becomes an ordinary object). A **proxy** keeps `Some(Proxy)` for life —
    /// once initialized it forwards property access to the real instance held in
    /// [`Self::proxy_instance`]. The pending initializer/factory closure lives in
    /// a VM-side table keyed by object id. Drives `var_dump`'s "lazy ghost"/"lazy
    /// proxy" rendering and the access-time init trigger.
    pub lazy: Option<LazyKind>,
    /// The real instance a **lazy proxy** forwards to, set when the proxy is
    /// initialized (its factory returns this object). `None` for every non-proxy
    /// object and for an uninitialized proxy. A proxy with this `Some` is the
    /// "initialized" state: property reads/writes redirect here and `var_dump`
    /// renders a single synthetic `["instance"]` slot.
    pub proxy_instance: Option<Box<Zval>>,
}

/// Which kind of uninitialized lazy object this is (PHP 8.4): a *ghost*
/// (initializer populates it in place) or a *proxy* (factory returns the real
/// instance the proxy forwards to).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LazyKind {
    Ghost,
    Proxy,
}

thread_local! {
    /// Freed object-handle ids, LIFO — Zend's `EG(objects_store).free_list_head`.
    /// [`Drop`] pushes here when an `Object`/`Closure`/`GenState` is released;
    /// the VM pops on allocation so `#N` handles are REUSED newest-first,
    /// exactly like `zend_objects_store_put`. Reset per program run.
    static FREED_OBJECT_IDS: std::cell::RefCell<Vec<u32>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Push a released handle id (0 = synthetic carrier, never pushed).
pub fn free_object_id(id: u32) {
    if id != 0 {
        FREED_OBJECT_IDS.with(|f| f.borrow_mut().push(id));
    }
}

/// Pop the most recently freed handle id (Zend reuses newest-first).
pub fn take_freed_object_id() -> Option<u32> {
    FREED_OBJECT_IDS.with(|f| f.borrow_mut().pop())
}

/// Clear the freed-id list (a new program run starts a fresh handle space).
pub fn reset_freed_object_ids() {
    FREED_OBJECT_IDS.with(|f| f.borrow_mut().clear());
}

impl Drop for Object {
    fn drop(&mut self) {
        free_object_id(self.id);
    }
}

impl Object {
    /// A field-by-field copy carrying the given handle id. `Object` is
    /// deliberately NOT `Clone`: an implicit copy would carry the source's id,
    /// and its eventual drop would push a LIVE id onto the freed-id list
    /// (handle reuse, see [`crate::free_object_id`]). Pass `0` for a synthetic
    /// carrier that must never release a handle.
    pub fn copy_with_id(&self, id: u32) -> Object {
        Object {
            class_id: self.class_id,
            class_name: Rc::clone(&self.class_name),
            props: self.props.clone(),
            id,
            info: Rc::clone(&self.info),
            readonly_init: self.readonly_init.clone(),
            readonly_clone_writable: self.readonly_clone_writable.clone(),
            typed_unset: self.typed_unset.clone(),
            lazy: self.lazy,
            proxy_instance: self.proxy_instance.clone(),
        }
    }

    /// Whether typed property `name` was explicitly `unset()` (see field doc).
    pub fn is_typed_unset(&self, name: &[u8]) -> bool {
        self.typed_unset.iter().any(|n| n.as_ref() == name)
    }

    /// Record that typed property `name` was explicitly `unset()` (idempotent).
    pub fn mark_typed_unset(&mut self, name: &[u8]) {
        if !self.is_typed_unset(name) {
            self.typed_unset.push(name.into());
        }
    }

    /// Whether readonly property `name` has been initialised on this instance.
    pub fn is_readonly_init(&self, name: &[u8]) -> bool {
        self.readonly_init.iter().any(|n| n.as_ref() == name)
    }

    /// Record that readonly property `name` has now been initialised (idempotent).
    pub fn mark_readonly_init(&mut self, name: &[u8]) {
        if !self.is_readonly_init(name) {
            self.readonly_init.push(name.into());
        }
    }

    /// Drop `name` from the initialised set (an `unset` during `__clone`, which
    /// returns a readonly property to the uninitialised state).
    pub fn clear_readonly_init(&mut self, name: &[u8]) {
        self.readonly_init.retain(|n| n.as_ref() != name);
    }

    /// Whether readonly property `name` may be (re-)initialised once right now —
    /// i.e. `clone` granted a write permission still pending consumption.
    pub fn readonly_clone_writable(&self, name: &[u8]) -> bool {
        self.readonly_clone_writable.iter().any(|n| n.as_ref() == name)
    }

    /// Consume the clone-write permission for `name` (a no-op if absent).
    pub fn consume_clone_writable(&mut self, name: &[u8]) {
        self.readonly_clone_writable.retain(|n| n.as_ref() != name);
    }
}

/// Split a stored property key into its display name and visibility. A `private`
/// property is stored under a *mangled* key `\0Class\0prop` (the declaring class
/// embedded), so this returns (`prop`, `Private(Class)`); a `\0*\0prop` key is
/// `Protected`. Any other key is a plain name whose visibility comes from the
/// class's [`ObjectInfo`] (`Public` for a dynamic / undeclared property). Used by
/// every consumer that iterates an object's stored slots for display
/// (`var_dump`/`print_r`/`var_export`/`json_encode`/`serialize`) or scope views.
/// Opaque internal handle classes (PHP 8 resource-object wrappers, e.g.
/// `GdImage`): no visible properties or methods, not instantiable, cloneable
/// or serializable. Shared by the VM (clone/serialize/var_dump/Reflection)
/// and the pure builtins (var_export/print_r/json) so the prelude's hidden
/// handle prop stays invisible everywhere.
pub fn is_opaque_handle_class(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"gdimage") || name.eq_ignore_ascii_case(b"finfo")
}

pub fn unmangle_prop_key<'a>(key: &'a [u8], info: &ObjectInfo) -> (&'a [u8], PropVis) {
    if let Some(rest) = key.strip_prefix(b"\0") {
        if let Some(sep) = rest.iter().position(|&b| b == 0) {
            let class = &rest[..sep];
            let prop = &rest[sep + 1..];
            if class == b"*" {
                return (prop, PropVis::Protected);
            }
            return (prop, PropVis::Private(PhpStr::new(class.to_vec())));
        }
    }
    (key, info.vis_of(key))
}

/// Build the *mangled* storage key of a private property: `\0Class\0prop`
/// (Zend's zend_mangle_property_name), the key its slot lives under in
/// [`Props`] so a parent's private and a subclass's same-name redeclaration
/// coexist. The inverse of [`unmangle_prop_key`].
pub fn mangle_prop_key(class: &[u8], prop: &[u8]) -> Vec<u8> {
    let mut k = Vec::with_capacity(class.len() + prop.len() + 2);
    k.push(0);
    k.extend_from_slice(class);
    k.push(0);
    k.extend_from_slice(prop);
    k
}

/// The display name of a stored property key: the `prop` part of a mangled
/// `\0Class\0prop` / `\0*\0prop`, the key itself when plain. For diagnostics
/// that must never leak NUL-mangled storage keys.
pub fn prop_display_name(key: &[u8]) -> &[u8] {
    if let Some(rest) = key.strip_prefix(b"\0") {
        if let Some(sep) = rest.iter().position(|&b| b == 0) {
            return &rest[sep + 1..];
        }
    }
    key
}

/// Visibility of a declared property as rendered by `var_dump` / `print_r`
/// (step 19-7). A dynamic (undeclared) property is treated as `Public`.
#[derive(Debug, Clone, PartialEq)]
pub enum PropVis {
    Public,
    Protected,
    /// `private`, carrying the *declaring* class name (var_dump prints it).
    Private(Rc<PhpStr>),
}

/// Per-class property-visibility table for object dumping (step 19-7, D-19.20).
#[derive(Debug, Default, PartialEq)]
pub struct ObjectInfo {
    /// Declared property name → visibility, in declaration order. Dynamic
    /// properties are absent and default to `Public`.
    entries: Vec<(Box<[u8]>, PropVis)>,
    /// Declared property name → its type as displayed (`int`, `?Foo`, …), for the
    /// typed properties that have one. Used by `var_dump`/`print_r` to render an
    /// uninitialized typed property as `uninitialized(type)`. Empty when the class
    /// has no typed properties.
    types: Vec<(Box<[u8]>, Box<[u8]>)>,
    /// `true` when this instance is an enum case singleton, so `var_dump` /
    /// `print_r` render it as `enum(Name::Case)` rather than `object(...)`
    /// (step 23, D-23.5).
    pub is_enum_case: bool,
    /// `true` on the synthetic carrier built for an object's `__serialize()`
    /// payload: its "property" keys are actually array keys, so a canonical
    /// integer key serializes as `i:N` (array semantics) rather than `s:…`.
    pub opaque_array_keys: bool,
}

impl ObjectInfo {
    pub fn from_entries(entries: Vec<(Box<[u8]>, PropVis)>) -> Self {
        ObjectInfo { entries, types: Vec::new(), is_enum_case: false, opaque_array_keys: false }
    }

    /// Like [`Self::from_entries`] but carrying the declared property type displays
    /// (for uninitialized-property rendering).
    pub fn from_entries_typed(
        entries: Vec<(Box<[u8]>, PropVis)>,
        types: Vec<(Box<[u8]>, Box<[u8]>)>,
    ) -> Self {
        ObjectInfo { entries, types, is_enum_case: false, opaque_array_keys: false }
    }

    /// `ObjectInfo` for an enum case singleton (step 23, D-23.5). The synthetic
    /// `name`/`value` properties are public.
    pub fn enum_case(entries: Vec<(Box<[u8]>, PropVis)>) -> Self {
        ObjectInfo { entries, types: Vec::new(), is_enum_case: true, opaque_array_keys: false }
    }

    /// `ObjectInfo` for the synthetic carrier of an object's `__serialize()`
    /// payload, whose keys serialize with array (not property) semantics.
    pub fn opaque() -> Self {
        ObjectInfo {
            entries: Vec::new(),
            types: Vec::new(),
            is_enum_case: false,
            opaque_array_keys: true,
        }
    }

    /// The visibility of property `name`, defaulting to `Public` for a dynamic
    /// (undeclared) property.
    pub fn vis_of(&self, name: &[u8]) -> PropVis {
        self.entries
            .iter()
            .find(|(k, _)| k.as_ref() == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(PropVis::Public)
    }

    /// The displayed type of declared property `name` (`int`, `?Foo`), if it is a
    /// typed property — used to render `uninitialized(type)`.
    pub fn type_of(&self, name: &[u8]) -> Option<&[u8]> {
        self.types.iter().find(|(k, _)| k.as_ref() == name).map(|(_, t)| t.as_ref())
    }
}

/// An insertion-ordered, string-keyed property map. Objects typically have only
/// a handful of properties, so a linear-scan vector is simpler than a hash index
/// and preserves declaration/assignment order exactly — which `var_dump` and
/// `print_r` reproduce (D-19.2).
#[derive(Debug, Default, Clone)]
pub struct Props {
    entries: Vec<(Box<[u8]>, Zval)>,
    /// FxHash of each key, parallel to `entries` — but LAZY: kept empty while
    /// the table is small (so cloning a typical object clones an empty Vec —
    /// no allocation) and built on the first lookup past `HASH_SCAN_MIN`.
    /// Invariant: either empty, or exactly parallel to `entries`. Lookups on
    /// large tables compare hashes first, so a miss never touches key bytes.
    hashes: Vec<u64>,
}

/// Below this many entries a plain byte scan beats hash-then-scan.
const HASH_SCAN_MIN: usize = 8;

#[inline]
fn prop_key_hash(name: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut h = rustc_hash::FxHasher::default();
    h.write(name);
    h.finish()
}

impl Props {
    pub fn new() -> Self {
        Props::default()
    }

    #[inline]
    fn position(&mut self, name: &[u8]) -> Option<usize> {
        if self.entries.len() < HASH_SCAN_MIN {
            return self.entries.iter().position(|(k, _)| k.as_ref() == name);
        }
        if self.hashes.len() != self.entries.len() {
            self.hashes = self.entries.iter().map(|(k, _)| prop_key_hash(k)).collect();
        }
        let h = prop_key_hash(name);
        self.hashes
            .iter()
            .enumerate()
            .find_map(|(i, &eh)| (eh == h && self.entries[i].0.as_ref() == name).then_some(i))
    }

    /// The read-only lookup past `HASH_SCAN_MIN` — out of line so the hot
    /// small-table scan in [`Props::get`]/[`Props::contains`] stays a tight
    /// inlined loop at every call site (the WP-23 A/B showed the merged
    /// function stopped inlining and cost ~4% media CPU as a call leaf).
    fn position_large(&self, name: &[u8]) -> Option<usize> {
        if self.hashes.len() == self.entries.len() {
            let h = prop_key_hash(name);
            return self
                .hashes
                .iter()
                .enumerate()
                .find_map(|(i, &eh)| (eh == h && self.entries[i].0.as_ref() == name).then_some(i));
        }
        self.entries.iter().position(|(k, _)| k.as_ref() == name)
    }

    /// The current value of property `name`, if present.
    #[inline]
    pub fn get(&self, name: &[u8]) -> Option<&Zval> {
        if self.entries.len() < HASH_SCAN_MIN {
            return self.entries.iter().find(|(k, _)| k.as_ref() == name).map(|(_, v)| v);
        }
        self.position_large(name).map(|i| &self.entries[i].1)
    }

    /// A mutable handle to property `name`, if present.
    #[inline]
    pub fn get_mut(&mut self, name: &[u8]) -> Option<&mut Zval> {
        if self.entries.len() < HASH_SCAN_MIN {
            return self
                .entries
                .iter_mut()
                .find(|(k, _)| k.as_ref() == name)
                .map(|(_, v)| v);
        }
        self.position(name).map(|i| &mut self.entries[i].1)
    }

    #[inline]
    pub fn contains(&self, name: &[u8]) -> bool {
        if self.entries.len() < HASH_SCAN_MIN {
            return self.entries.iter().any(|(k, _)| k.as_ref() == name);
        }
        self.position_large(name).is_some()
    }

    /// Set property `name`, updating in place (keeping its position) if it already
    /// exists, otherwise appending it at the end.
    #[inline]
    pub fn set(&mut self, name: &[u8], value: Zval) {
        // A push may desync `hashes` — the next large lookup detects the
        // length mismatch and rebuilds (see `position`).
        if self.entries.len() < HASH_SCAN_MIN {
            match self.entries.iter_mut().find(|(k, _)| k.as_ref() == name) {
                Some((_, slot)) => *slot = value,
                None => self.entries.push((name.into(), value)),
            }
            return;
        }
        match self.position(name) {
            Some(i) => self.entries[i].1 = value,
            None => self.entries.push((name.into(), value)),
        }
    }

    /// Set property `name` (updating in place / appending, like [`Props::set`]),
    /// returning the value it displaced — or `None` when newly inserted. Used by
    /// the property write path to hand the dropped value to the GC's
    /// possible-roots tracking.
    #[inline]
    pub fn replace(&mut self, name: &[u8], value: Zval) -> Option<Zval> {
        if self.entries.len() < HASH_SCAN_MIN {
            match self.entries.iter_mut().find(|(k, _)| k.as_ref() == name) {
                Some((_, slot)) => return Some(std::mem::replace(slot, value)),
                None => {
                    self.entries.push((name.into(), value));
                    return None;
                }
            }
        }
        match self.position(name) {
            Some(i) => Some(std::mem::replace(&mut self.entries[i].1, value)),
            None => {
                self.entries.push((name.into(), value));
                None
            }
        }
    }

    /// Remove property `name`; returns whether it was present.
    pub fn remove(&mut self, name: &[u8]) -> bool {
        if let Some(pos) = self.position(name) {
            self.entries.remove(pos);
            // Desyncs `hashes` (self-healing on the next &mut lookup).
            self.hashes.clear();
            true
        } else {
            false
        }
    }

    /// After this table has been `clone`d for an object `clone`, break the
    /// sharing of any property reference that no *live external* variable
    /// aliases (Zend `zend_objects_clone_members`: bug27268, bug68262).
    ///
    /// A property that became `IS_REFERENCE` only because of the object itself
    /// (e.g. a by-reference return of `$this->p`, then a rebind that leaves the
    /// object as the sole holder) must NOT be shared with the clone — otherwise
    /// a write through the clone's slot would leak back into the source. A
    /// reference genuinely shared with an outside variable stays shared.
    ///
    /// Detection: right after the shallow `clone`, the source and this clone
    /// both hold each shared cell, so a cell whose only two owners are those
    /// two tables has `strong_count == 2`. Give the clone its own independent
    /// reference containing a copy of the value; higher counts (a live `=&`
    /// alias, or an intra-object alias shared by several properties) are left
    /// untouched.
    pub fn separate_cloned_internal_refs(&mut self) {
        for (_, v) in self.entries.iter_mut() {
            if let Zval::Ref(cell) = v {
                if Rc::strong_count(cell) == 2 {
                    let inner = cell.borrow().clone();
                    *v = Zval::Ref(Rc::new(std::cell::RefCell::new(inner)));
                }
            }
        }
    }

    /// Iterate properties in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &Zval)> {
        self.entries.iter().map(|(k, v)| (k.as_ref(), v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
