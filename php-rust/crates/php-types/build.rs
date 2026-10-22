// Link the **system libgd** (Homebrew keg, the same dylib the PHP oracle links)
// for ext/gd — see src/gdio.rs. Like ext/zlib's libz-sys, using the system
// library is what makes the *generated file bytes* identical to PHP's: libgd
// carries its own libjpeg-turbo/libpng/libwebp/libavif codec choices, so a
// Rust-crate reimplementation (image/imageproc) would diverge on every encode.
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
    println!("cargo:rerun-if-changed=build.rs");
}
