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
#[derive(Debug, Clone)]
pub struct Object {
    pub class_id: u32,
    pub class_name: Rc<PhpStr>,
    /// Declared and dynamic properties, in insertion order.
    pub props: Props,
    /// Object handle (`#N` in `var_dump`), assigned monotonically at creation.
    pub id: u32,
}

/// An insertion-ordered, string-keyed property map. Objects typically have only
/// a handful of properties, so a linear-scan vector is simpler than a hash index
/// and preserves declaration/assignment order exactly — which `var_dump` and
/// `print_r` reproduce (D-19.2).
#[derive(Debug, Default, Clone)]
pub struct Props {
    entries: Vec<(Box<[u8]>, Zval)>,
}

impl Props {
    pub fn new() -> Self {
        Props::default()
    }

    /// The current value of property `name`, if present.
    pub fn get(&self, name: &[u8]) -> Option<&Zval> {
        self.entries
            .iter()
            .find(|(k, _)| k.as_ref() == name)
            .map(|(_, v)| v)
    }

    /// A mutable handle to property `name`, if present.
    pub fn get_mut(&mut self, name: &[u8]) -> Option<&mut Zval> {
        self.entries
            .iter_mut()
            .find(|(k, _)| k.as_ref() == name)
            .map(|(_, v)| v)
    }

    pub fn contains(&self, name: &[u8]) -> bool {
        self.entries.iter().any(|(k, _)| k.as_ref() == name)
    }

    /// Set property `name`, updating in place (keeping its position) if it already
    /// exists, otherwise appending it at the end.
    pub fn set(&mut self, name: &[u8], value: Zval) {
        if let Some(slot) = self.get_mut(name) {
            *slot = value;
        } else {
            self.entries.push((name.into(), value));
        }
    }

    /// Remove property `name`; returns whether it was present.
    pub fn remove(&mut self, name: &[u8]) -> bool {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k.as_ref() == name) {
            self.entries.remove(pos);
            true
        } else {
            false
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
