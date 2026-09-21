"""Inject diagnostic only into an explicit archive directory; never canonical source."""
from pathlib import Path
import sys
root=Path(sys.argv[1]).resolve()
assert str(root).startswith('/private/tmp/phpr-s178-investigation/src/')
h=Path(__file__).resolve().parent
assert not (root/'crates/php-runtime/src/vm/s178_probe.rs').exists(), 'Already instrumented'
def replace_once(s, old, new):
    assert s.count(old)==1, (old, s.count(old))
    return s.replace(old,new,1)
p=root/'crates/php-runtime/src/vm/mod.rs'; s=p.read_text();s=replace_once(s, 'mod arrays;', 'mod s178_probe;\nmod arrays;')
s=replace_once(s, '    // S-139 ic-stats: dump CLI-only', '    s178_probe::dump();\n    // S-139 ic-stats: dump CLI-only')
p.write_text(s)
(root/'crates/php-runtime/src/vm/s178_probe.rs').write_text((h/'probe.rs').read_text())
p=root/'crates/php-runtime/src/vm/run.rs';s=p.read_text()
start=s.index('    fn prop_set_entry<');end=s.index('    fn prop_get_fallback',start)
a=s[start:end]
a=replace_once(a, '        // A `prop_init` thunk writes defaults directly:', '        let mut probe = super::s178_probe::Probe::begin(&target, &self.frames[top].func.file, name, ic as *const _ as usize);\n        // A `prop_init` thunk writes defaults directly:')
a=replace_once(a, '            if let Some(old) = write_property(&target, &key, value.clone())?', '            probe.storage_key(&key);\n            if let Some(old) = write_property(&target, &key, value.clone())?')
a=replace_once(a, '            if fast {', '            if fast {\n                probe.private = 0; probe.readonly = 0; probe.storage_key(name);')
a=replace_once(a, '''            let key = match object_class_id(&target)''','''            probe.path = "init";
            let key = match object_class_id(&target)''')
a=replace_once(a, '''                if hit {
''','''                if hit {
                    probe.private = 0; probe.readonly = 0; probe.storage_key(name);
                    probe.path = if raw & crate::bytecode::PropIc::TY != 0 { "hit_typed" } else { "hit_plain" };
''')
a=replace_once(a, '''        // S-176 census «typed»: not served''','''        probe.path = "miss";
        // S-176 census «typed»: not served''')
a=replace_once(a, '''            if declared_slot {
                if let Some(decl) = ro_decl {''','''            probe.storage_key(&key);
            probe.private = (key.as_ref() != &name[..]) as i8;
            probe.readonly = ro_decl.is_some() as i8;
            if declared_slot {
                if let Some(decl) = ro_decl {''')
s=s[:start]+a+s[end:];p.write_text(s)
p=root/'rust-toolchain.toml';p.write_text(p.read_text().replace('1.96.0','1.98.1'))
