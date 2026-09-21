"""New S-179 injection, anchors adapted from S-178, originals preserved."""
from pathlib import Path
import sys
root=Path(sys.argv[1]).resolve();assert root==Path('/Users/francescotinti/Claude/phpr-s179-cost-resume-results/src/php-rust')
h=Path(__file__).resolve().parent
assert not (root/'crates/php-runtime/src/vm/s179_probe.rs').exists()
def once(s,a,b):
 assert s.count(a)==1,(a,s.count(a))
 return s.replace(a,b,1)
p=root/'crates/php-runtime/src/vm/mod.rs';s=p.read_text();s=once(s,'mod arrays;','mod s179_probe;\nmod arrays;');s=once(s,'    // S-139 ic-stats: dump CLI-only','    s179_probe::dump();\n    // S-139 ic-stats: dump CLI-only');p.write_text(s)
(root/'crates/php-runtime/src/vm/s179_probe.rs').write_text((h/'probe.rs').read_text())
p=root/'crates/php-runtime/src/vm/run.rs';s=p.read_text();start=s.index('    fn prop_set_entry<');end=s.index('    fn prop_get_fallback',start);a=s[start:end]
a=once(a,'        // FAST PATH (WP-25): overwrite','        let mut probe = super::s179_probe::Probe::begin(&target, &self.frames[top].func.file);\n        // FAST PATH (WP-25): overwrite')
a=once(a,'            if fast {','            if fast {\n                probe.private = 0; probe.readonly = 0;')
a=once(a,'            if declared_slot {\n                if let Some(decl) = ro_decl {','''            probe.private = (key.as_ref() != &name[..]) as i8;
            probe.readonly = ro_decl.is_some() as i8;
            if probe.count && probe.private == 1 && probe.readonly == 0 {
                let b = o.borrow();
                // Independent bits, not mutually exclusive rejection reasons.
                let scope_ok = cur == Some(ocid);
                let slot_ok = slot_idx.is_some();
                let no_magic = resolve_method_runtime(&self.classes, ocid, b"__set").is_none();
                let instance_ok = b.lazy.is_none() && !b.info.is_enum_case;
                let present = slot_idx.and_then(|i| b.props.get_slot(i)).is_some();
                let ref_ok = match slot_idx.and_then(|i| b.props.get_slot(i)) {
                    Some(Zval::Ref(_)) => self.typed_refs.is_empty(),
                    Some(_) => true, None => false,
                };
                let typed = prop_type_decl(&self.classes, ocid, name).is_some();
                probe.guards = (scope_ok as u32) | ((slot_ok as u32)<<1)
                    | ((np_fillable as u32)<<2) | ((no_magic as u32)<<3)
                    | ((instance_ok as u32)<<4) | ((present as u32)<<5)
                    | ((ref_ok as u32)<<6) | ((typed as u32)<<7);
            }
            if declared_slot {
                if let Some(decl) = ro_decl {''')
# Only final full slow-path success; early paths remain success=false and are not private candidates.
assert a.count('        Ok(())\n    }')==1
a=once(a,'        Ok(())\n    }','        probe.success = true;\n        Ok(())\n    }')
s=s[:start]+a+s[end:];p.write_text(s)
