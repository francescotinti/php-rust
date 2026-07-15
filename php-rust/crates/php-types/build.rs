// Link the **system libgd** (Homebrew keg, the same dylib the PHP oracle links)
// for ext/gd — see src/gdio.rs. Like ext/zlib's libz-sys, using the system
// library is what makes the *generated file bytes* identical to PHP's: libgd
// carries its own libjpeg-turbo/libpng/libwebp/libavif codec choices, so a
// Rust-crate reimplementation (image/imageproc) would diverge on every encode.
fn main() {
    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/gd/lib");
    println!("cargo:rustc-link-lib=dylib=gd");
    println!("cargo:rerun-if-changed=build.rs");
}
