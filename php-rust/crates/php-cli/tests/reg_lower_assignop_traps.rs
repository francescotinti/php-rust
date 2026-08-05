//! A-ST-101-3 / KS-ST-101-1 (Concilio WP-101): le SETTE fixture-trappola
//! AssignOp (A-ST-99-3 a–g) come GATE della promozione flag-on — VIETATO il
//! flip del default finché non esistono e non passano BYTE-IDENTICHE nei
//! DUE modi. Fixture IN-TREE (A-ST-100-3) condivise col gate oracle di
//! harness: `wp100-harness/assignop-traps/*.php`.
//!
//! Il giudice QUI è il confronto per-modo (off↔on, valori ESPLICITI della
//! lista chiusa del contratto S-100). La gamba a DUE MOTORI (KS-ST-100-2)
//! vive in `wp100-harness/s100-assignop-oracle.sh` con la lista NOMINATA
//! delle divergenze oracle pre-esistenti (catalogo S-100: undef-lhs senza
//! warning; typed-ref azzerato da Zend dopo AssignOp fallito).
use std::process::Command;

/// (fixture, marker di controllo positivo: la fixture DEVE stamparlo — una
/// fixture muta che passa per parità è una forgia silenziosa).
const TRAPS: &[(&str, &str)] = &[
    ("a-rhs-first.php", "byref:"),
    ("b-typed-ref.php", "coerced:"),
    ("c-concat-nonoverlap.php", "a01234"),
    ("d-overflow.php", "float(9.223372036854776E+18)"),
    ("e-undef-warning-order.php", "eff-ran"),
    ("f-destruct-timing.php", "destruct:displaced"),
    ("g-never-commute.php", "g9:"),
];

fn traps_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../wp100-harness/assignop-traps")
        .canonicalize()
        .expect("wp100-harness/assignop-traps esiste in-tree")
}

fn run_mode(reg: &str, file: &std::path::Path) -> (String, String, bool) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_phpr"));
    c.env_remove("PHPR_REG_LOWER");
    c.env_remove("PHPR_DUMP_OPS");
    c.env("PHPR_REG_LOWER", reg);
    let out = c.arg(file).output().expect("spawn phpr");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

#[test]
fn the_seven_assignop_traps_are_byte_identical_across_modes() {
    let dir = traps_dir();
    // Il dente conta le trappole: una fixture rimossa/rinominata non
    // sparisce in silenzio dal gate (KS-ST-101-1 esige tutte e sette).
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .expect("read traps dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        // I resource-fork AppleDouble (`._*.php`) del volume non sono fixture.
        .filter(|n| n.ends_with(".php") && !n.starts_with("._"))
        .collect();
    found.sort();
    assert_eq!(
        found,
        TRAPS.iter().map(|(n, _)| n.to_string()).collect::<Vec<_>>(),
        "le fixture su disco non sono le sette trappole nominate"
    );
    for (name, marker) in TRAPS {
        let f = dir.join(name);
        let (off_out, off_err, off_ok) = run_mode("0", &f);
        let (on_out, on_err, on_ok) = run_mode("1", &f);
        assert!(
            off_out.contains(marker),
            "{name}: fixture muta — manca il marker `{marker}`\n{off_out}\n{off_err}"
        );
        assert_eq!(off_ok, on_ok, "{name}: exit status diverge tra i modi");
        assert_eq!(off_out, on_out, "{name}: stdout diverge tra flag-off e flag-on");
        assert_eq!(off_err, on_err, "{name}: stderr diverge tra flag-off e flag-on");
    }
}
