#!/usr/bin/env bash
# Build the C libraries phpr links (gd, tidy, libxslt/libexslt, libxml2 and gd's
# codecs) as static, position-independent musl archives, so that
#   cargo build --release --target x86_64-unknown-linux-musl -p php-cli
# produces a fully static phpr. Distributions ship these libraries only as
# glibc-built shared objects, which cannot go into a static musl binary.
#
# Everything lands in $PHPR_MUSL_PREFIX (default ~/.local/phpr-musl), outside the
# repo; crates/php-types/build.rs reads the same variable on musl targets.
#
# Requirements (Linux x86_64 host):
#   - musl C toolchain providing `musl-gcc` (Arch: `pacman -S musl`,
#     Debian/Ubuntu: `apt install musl-tools`)
#   - the Rust target: `rustup target add x86_64-unknown-linux-musl` (for the
#     toolchain pinned in rust-toolchain.toml)
#   - cmake, make, nasm (libaom's x86 assembly), curl, tar with xz support
#
# Process:
#   1. scripts/build-musl-deps.sh            # once; download + build what is missing
#   2. cargo build --release --target x86_64-unknown-linux-musl -p php-cli
#      -> target/x86_64-unknown-linux-musl/release/phpr (static-pie)
#   scripts/build-musl-deps.sh --clean       # wipe the prefix and rebuild everything
set -euo pipefail

P="${PHPR_MUSL_PREFIX:-$HOME/.local/phpr-musl}"
SRC="$P/src"
JOBS="$(nproc)"

if [[ "${1:-}" == "--clean" ]]; then
  rm -rf "$P"
fi
mkdir -p "$SRC" "$P/lib" "$P/include"

for tool in musl-gcc cmake nasm curl make; do
  command -v "$tool" >/dev/null || { echo "missing tool: $tool" >&2; exit 1; }
done

export CC=musl-gcc
export CFLAGS="-O2 -fPIC"
export CPPFLAGS="-I$P/include"
export LDFLAGS="-L$P/lib"
# Only the prefix's .pc files: never let a glibc system library leak in.
export PKG_CONFIG_LIBDIR="$P/lib/pkgconfig"
export PKG_CONFIG_PATH=""

CMAKE_COMMON=(
  -DCMAKE_BUILD_TYPE=Release
  -DCMAKE_C_COMPILER=musl-gcc
  -DCMAKE_INSTALL_PREFIX="$P"
  -DCMAKE_INSTALL_LIBDIR=lib
  -DCMAKE_PREFIX_PATH="$P"
  -DCMAKE_FIND_ROOT_PATH="$P"
  -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY
  -DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY
  -DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=ONLY
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON
  -DBUILD_SHARED_LIBS=OFF
)

# fetch <url> <dir-name>: download and unpack into $SRC/<dir-name>
fetch() {
  local url="$1" dir="$2" tarball="$SRC/${2}.tar"
  [[ -d "$SRC/$dir" ]] && return 0
  echo ">> fetch $dir"
  curl -fsSL "$url" -o "$tarball"
  mkdir -p "$SRC/$dir"
  tar -xf "$tarball" -C "$SRC/$dir" --strip-components=1
  rm -f "$tarball"
}

# step <name> <marker-lib>: skip a library already installed in the prefix
step() {
  if [[ -f "$P/lib/$2" ]]; then
    echo ">> $1: already built"
    return 1
  fi
  echo ">> build $1"
}

if step zlib libz.a; then
  fetch https://github.com/madler/zlib/releases/download/v1.3.1/zlib-1.3.1.tar.gz zlib
  (cd "$SRC/zlib" && ./configure --prefix="$P" --static && make -j"$JOBS" && make install)
fi

if step libpng libpng16.a; then
  fetch https://github.com/pnggroup/libpng/archive/refs/tags/v1.6.47.tar.gz libpng
  cmake -S "$SRC/libpng" -B "$SRC/libpng/_build" "${CMAKE_COMMON[@]}" \
    -DPNG_SHARED=OFF -DPNG_STATIC=ON -DPNG_TESTS=OFF -DPNG_TOOLS=OFF
  cmake --build "$SRC/libpng/_build" -j"$JOBS" && cmake --install "$SRC/libpng/_build"
fi

if step libjpeg-turbo libjpeg.a; then
  fetch https://github.com/libjpeg-turbo/libjpeg-turbo/releases/download/3.1.0/libjpeg-turbo-3.1.0.tar.gz libjpeg-turbo
  cmake -S "$SRC/libjpeg-turbo" -B "$SRC/libjpeg-turbo/_build" "${CMAKE_COMMON[@]}" \
    -DENABLE_SHARED=OFF -DENABLE_STATIC=ON -DWITH_TURBOJPEG=OFF
  cmake --build "$SRC/libjpeg-turbo/_build" -j"$JOBS" && cmake --install "$SRC/libjpeg-turbo/_build"
fi

if step libwebp libwebp.a; then
  fetch https://storage.googleapis.com/downloads.webmproject.org/releases/webp/libwebp-1.5.0.tar.gz libwebp
  cmake -S "$SRC/libwebp" -B "$SRC/libwebp/_build" "${CMAKE_COMMON[@]}" \
    -DWEBP_BUILD_ANIM_UTILS=OFF -DWEBP_BUILD_CWEBP=OFF -DWEBP_BUILD_DWEBP=OFF \
    -DWEBP_BUILD_GIF2WEBP=OFF -DWEBP_BUILD_IMG2WEBP=OFF -DWEBP_BUILD_VWEBP=OFF \
    -DWEBP_BUILD_WEBPINFO=OFF -DWEBP_BUILD_WEBPMUX=OFF -DWEBP_BUILD_EXTRAS=OFF
  cmake --build "$SRC/libwebp/_build" -j"$JOBS" && cmake --install "$SRC/libwebp/_build"
fi

# AVIF: libaom serves as both decoder and encoder (gdImageCreateFromAvifPtr and
# gdImageAvifPtrEx), so one codec covers what the system libavif provides.
if step libaom libaom.a; then
  fetch https://storage.googleapis.com/aom-releases/libaom-3.12.1.tar.gz libaom
  # NASM 3.x reworded `nasm -hf`, so aom's probe for "-Ox" fails although the
  # flag still works: relax the probe to the bare "-O" option.
  sed -i 's/MATCHES "-Ox"/MATCHES "-O"/' "$SRC/libaom/build/cmake/aom_optimization.cmake"
  cmake -S "$SRC/libaom" -B "$SRC/libaom/_build" "${CMAKE_COMMON[@]}" \
    -DENABLE_DOCS=OFF -DENABLE_EXAMPLES=OFF -DENABLE_TESTS=OFF -DENABLE_TOOLS=OFF \
    -DENABLE_TESTDATA=OFF -DCONFIG_TUNE_VMAF=0 -DCONFIG_TUNE_BUTTERAUGLI=0
  cmake --build "$SRC/libaom/_build" -j"$JOBS" && cmake --install "$SRC/libaom/_build"
fi

if step libavif libavif.a; then
  fetch https://github.com/AOMediaCodec/libavif/archive/refs/tags/v1.3.0.tar.gz libavif
  cmake -S "$SRC/libavif" -B "$SRC/libavif/_build" "${CMAKE_COMMON[@]}" \
    -DAVIF_CODEC_AOM=SYSTEM -DAVIF_LIBYUV=OFF -DAVIF_LIBSHARPYUV=OFF \
    -DAVIF_LIBXML2=OFF -DAVIF_BUILD_APPS=OFF -DAVIF_BUILD_TESTS=OFF
  cmake --build "$SRC/libavif/_build" -j"$JOBS" && cmake --install "$SRC/libavif/_build"
fi

if step libgd libgd.a; then
  fetch https://github.com/libgd/libgd/releases/download/gd-2.3.3/libgd-2.3.3.tar.xz libgd
  # Only the library and headers: gd's sample tools do not link statically
  # (libtool puts -lz ahead of libpng16.a) and phpr does not need them.
  (cd "$SRC/libgd" && ./configure --prefix="$P" --disable-shared --enable-static \
     --with-zlib="$P" --with-png="$P" --with-jpeg="$P" --with-webp="$P" --with-avif="$P" \
     --without-freetype --without-fontconfig --without-xpm --without-tiff \
     --without-heif --without-liq --without-raqm --without-x \
   && make -j"$JOBS" -C src libgd.la && make -C src install-libLTLIBRARIES install-includeHEADERS)
fi

if step libxml2 libxml2.a; then
  fetch https://download.gnome.org/sources/libxml2/2.15/libxml2-2.15.1.tar.xz libxml2
  (cd "$SRC/libxml2" && ./configure --prefix="$P" --disable-shared --enable-static \
     --without-python --without-icu --without-lzma --with-zlib="$P" \
   && make -j"$JOBS" && make install)
fi

if step libxslt libxslt.a; then
  fetch https://download.gnome.org/sources/libxslt/1.1/libxslt-1.1.45.tar.xz libxslt
  (cd "$SRC/libxslt" && ./configure --prefix="$P" --disable-shared --enable-static \
     --without-python --without-crypto --with-libxml-prefix="$P" \
   && make -j"$JOBS" && make install)
fi

if step tidy libtidy.a; then
  fetch https://github.com/htacg/tidy-html5/archive/refs/tags/5.8.0.tar.gz tidy
  cmake -S "$SRC/tidy" -B "$SRC/tidy/_build" "${CMAKE_COMMON[@]}" \
    -DBUILD_SHARED_LIB=OFF -DSUPPORT_CONSOLE_APP=OFF -DBUILD_DOCUMENTATION=OFF \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5
  cmake --build "$SRC/tidy/_build" -j"$JOBS" && cmake --install "$SRC/tidy/_build"
  # tidy-html5 names its static archive libtidy_static.a on some versions.
  [[ -f "$P/lib/libtidy.a" ]] || cp "$P/lib/libtidy_static.a" "$P/lib/libtidy.a"
fi

echo ">> done: static musl libraries in $P/lib"
