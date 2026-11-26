//! S-194 N2: the class fingerprint (`Vm::class_index_fp`) is maintained
//! incrementally by the ONLY writer of `Vm::class_index`,
//! `Vm::class_index_put` in `vm/class_fp`. The drift check in `unit_fp` is a
//! debug assertion and no build of this project runs it, so a second writer
//! would leave the fingerprint stale in silence (a compiled unit reused
//! against the wrong class set). This tooth scans the VM sources and fails
//! on any mutation of `class_index` outside that file.
//! Limit: it is textual; a write through an alias it cannot see is not caught.

use std::path::{Path, PathBuf};

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

const FIELD: &str = "class_index";
const MUTATORS: &[&str] = &[
    "insert(", "remove(", "clear(", "entry(", "extend(", "retain(", "drain(",
    "get_mut(", "iter_mut(", "values_mut(", "shift_remove(", "swap_remove(",
];

fn ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// True when the line mutates (or mutably borrows) the `class_index` field.
fn writes_class_index(line: &str) -> bool {
    let code = line.split("//").next().unwrap_or("");
    let bytes = code.as_bytes();
    let mut from = 0;
    while let Some(off) = code[from..].find(FIELD) {
        let at = from + off;
        let end = at + FIELD.len();
        from = end;
        if at > 0 && ident(bytes[at - 1]) {
            continue;
        }
        if end < bytes.len() && ident(bytes[end]) {
            continue;
        }
        let before = code[..at].trim_end();
        if before.ends_with('.') && before[..before.len() - 1].contains("&mut ") {
            let head = before[..before.len() - 1].trim_end();
            let recv = head.rsplit(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.')).next().unwrap_or("");
            if head[..head.len() - recv.len()].trim_end().ends_with("&mut") {
                return true;
            }
        }
        let after = code[end..].trim_start();
        if let Some(call) = after.strip_prefix('.') {
            let call = call.trim_start();
            if MUTATORS.iter().any(|m| call.starts_with(m)) {
                return true;
            }
        }
        if after.starts_with('=') && !after.starts_with("==") && !after.starts_with("=>") {
            return true;
        }
    }
    false
}

#[test]
fn class_index_has_one_writer() {
    // The detector bites on the shapes a second writer would take...
    for red in [
        "self.class_index.insert(key, cid);",
        "vm.class_index . remove(&key);",
        "self.class_index = Default::default();",
        "let m = &mut self.class_index;",
        "self.class_index.entry(k).or_insert(0);",
        "self.class_index.clear();",
    ] {
        assert!(writes_class_index(red), "detector misses: {red}");
    }
    // ...and stays silent on reads, the fingerprint field and comments.
    for green in [
        "if let Some(&cid) = self.class_index.get(key) {",
        "self.class_index_fp ^= class_fp(&key, &cid);",
        "class_index: Default::default(),",
        "if self.class_index == other {",
        "// self.class_index.insert(key, cid) lives in class_fp",
        "for (k, v) in &self.class_index {",
        "let n = self.class_index.len();",
    ] {
        assert!(!writes_class_index(green), "detector over-fires: {green}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/vm");
    let mut files = Vec::new();
    rs_files(&root, &mut files);
    assert!(files.len() > 10, "VM sources not found under {}", root.display());
    let mut writers_in_funnel = 0;
    let mut stray = Vec::new();
    for f in &files {
        let funnel = f.file_name().is_some_and(|n| n == "class_fp.rs");
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        for (i, line) in text.lines().enumerate() {
            if writes_class_index(line) {
                if funnel {
                    writers_in_funnel += 1;
                } else {
                    stray.push(format!("{}:{}: {}", f.display(), i + 1, line.trim()));
                }
            }
        }
    }
    assert_eq!(writers_in_funnel, 1, "class_fp must hold exactly one write of class_index");
    assert!(
        stray.is_empty(),
        "class_index written outside Vm::class_index_put (class_index_fp would drift):\n{}",
        stray.join("\n")
    );
}
