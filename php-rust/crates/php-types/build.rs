fn main() {
    // macOS: gd lives in a Homebrew keg, not the default library path.
    // Linux: libgd is in the system library path; no explicit search needed.
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/gd/lib");
    }
    println!("cargo:rustc-link-lib=dylib=gd");
    // ext/xsl (src/xsltio.rs): the **system** libxslt/libexslt/libxml2 — the
    // same /usr/lib dylibs the PHP oracle links (they live in the dyld shared
    // cache; the linker resolves them through the SDK .tbd stubs).
    // ext/tidy (src/tidyio.rs): the **system** libtidy — the same Homebrew
    // keg the PHP oracle was configured against (--with-tidy=tidy-html5).
    // macOS: tidy lives in a Homebrew keg. Linux: system library path.
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/tidy-html5/lib");
    }
    println!("cargo:rustc-link-lib=dylib=tidy");
    println!("cargo:rustc-link-lib=dylib=xslt");
    println!("cargo:rustc-link-lib=dylib=exslt");
    println!("cargo:rustc-link-lib=dylib=xml2");
    // libxml2 >= 2.12 renamed the XPath value-stack entry points `valuePop`/
    // `valuePush` to `xmlXPathValuePop`/`xmlXPathValuePush`, and the old names
    // are gone by 2.15 (PR #3: Arch, libxml2 2.15.4). The macOS system libxml2
    // is 2.9.13 and exports only the old names; Linux distributions ship both
    // eras (Debian/Ubuntu 2.9.x, Fedora/Arch >= 2.12), so the choice follows the
    // installed version, not the target OS. src/xsltio.rs picks the symbol
    // names with `cfg(libxml_xpath_new_names)`.
    println!("cargo:rustc-check-cfg=cfg(libxml_xpath_new_names)");
    println!("cargo:rerun-if-env-changed=PHPR_LIBXML_NEW_XPATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    if libxml_uses_new_xpath_names() {
        println!("cargo:rustc-cfg=libxml_xpath_new_names");
    }
    println!("cargo:rerun-if-changed=build.rs");
}


/// `PHPR_LIBXML_NEW_XPATH=1|0` overrides the probe. Otherwise ask `pkg-config`,
/// then `xml2-config`, for the installed libxml2 version: new names from 2.12.
/// macOS never probes (system libxml2 2.9.x); an unknown version keeps the old
/// names, the pre-2.12 ABI that every libxml2 up to 2.14 still exports.
fn libxml_uses_new_xpath_names() -> bool {
    if let Ok(v) = std::env::var("PHPR_LIBXML_NEW_XPATH") {
        return v == "1";
    }
    if cfg!(target_os = "macos") {
        return false;
    }
    let probes: [(&str, &[&str]); 2] = [
        ("pkg-config", &["--modversion", "libxml-2.0"]),
        ("xml2-config", &["--version"]),
    ];
    for (cmd, args) in probes {
        let Ok(out) = std::process::Command::new(cmd).args(args).output() else { continue };
        if !out.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let mut parts = text.trim().split('.').map(|p| p.trim().parse::<u32>().unwrap_or(0));
        let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
        if major == 0 {
            continue;
        }
        return (major, minor) >= (2, 12);
    }
    false
}
