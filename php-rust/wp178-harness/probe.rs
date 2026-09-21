//! Temporary S-178 diagnostic. Injected into an archived source, never production.
use std::{cell::RefCell, collections::{BTreeMap, HashMap, HashSet}, rc::{Rc, Weak}};
use php_types::{Object, Zval};
#[derive(Default)]
struct Row { n: u64, site_repeat: u64, object_repeat: u64, ns: u128 }
struct Seen { object: Weak<RefCell<Object>>, props: HashSet<Vec<u8>> }
struct State {
    mode: String, calls: u64, rows: BTreeMap<String, Row>,
    sites: HashSet<(usize, String)>, objects: HashMap<u64, Seen>,
}
impl State {
    fn new() -> Self { Self {
        mode: std::env::var("PHPR_S178_MODE").unwrap_or_default(),
        calls: 0, rows: BTreeMap::new(), sites: HashSet::new(), objects: HashMap::new(),
    } }
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::new()); }
pub struct Probe {
    row: Option<String>, storage_known: bool, prop: Vec<u8>, site: usize, object: Option<(u64, Weak<RefCell<Object>>)>,
    pub path: &'static str, pub private: i8, pub readonly: i8,
}
impl Probe {
    pub fn storage_key(&mut self, key: &[u8]) {
        self.storage_known = true;
        if self.object.is_some() { self.prop = key.to_vec(); }
    }
    pub fn begin(obj: &Zval, file: &[u8], prop: &[u8], site: usize) -> Self {
        let record = STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.calls += 1;
            s.mode == "count"
        });
        let mut p = Self { row: None, storage_known: false, prop: Vec::new(), site, object: None, path: "early", private: -1, readonly: -1 };
        if record {
            let class = if let Zval::Object(o) = obj {
                let b = o.borrow();
                p.object = Some((b.id as u64, Rc::downgrade(o)));
                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
            } else { "<nonobject-or-reference>".into() };
            p.row = Some(format!("{}\t{}", class, String::from_utf8_lossy(file)));
            p.prop = prop.to_vec();
        }
        p
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        let ns = 0; // Counts only: timing explicitly deferred by the user.
        let Some(base) = self.row.take() else { return };
        let row = format!("{}\t{}\t{}\t{}", base, self.path, self.private, self.readonly);
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let sr = if s.mode == "count" { !s.sites.insert((self.site, row.clone())) } else { false };
            let mut repeat = false;
            if let Some((id, weak)) = self.object.as_ref().filter(|_| self.storage_known) {
                let seen = s.objects.entry(*id).or_insert_with(|| Seen { object: weak.clone(), props: HashSet::new() });
                if !Weak::ptr_eq(&seen.object, weak) { seen.object = weak.clone(); seen.props.clear(); }
                repeat = !seen.props.insert(self.prop.clone());
            }
            let r = s.rows.entry(row).or_default();
            r.n += 1; r.ns += ns; r.site_repeat += sr as u64; r.object_repeat += repeat as u64;
        });
    }
}
pub fn dump() {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_S178_OUT") else { return };
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "# pid={} mode={} calls={}", std::process::id(), s.mode, s.calls);
            for (k,r) in &s.rows { let _ = writeln!(f, "{}\t{}\t{}\t{}\t{}", k,r.n,r.site_repeat,r.object_repeat,r.ns); }
        }
        s.rows.clear(); s.sites.clear(); s.objects.clear(); s.calls = 0;
    });
}
