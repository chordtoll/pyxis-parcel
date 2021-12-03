# Maintainer: chordtoll <arch@chordtoll.com>
pkgname=pyxis-parcel
pkgver=0.2.0
pkgrel=1
pkgdesc="Utilities to manipulate parcel archives for the pyxis package manager"
license=("MIT")
arch=("x86_64")
depends=("fuse3" "gcc-libs")
makedepends=("cargo")
url="https://github.com/chordtoll/pyxis-parcel"
source=("$pkgname-$pkgver.tar.gz::https://static.crates.io/crates/$pkgname/$pkgname-$pkgver.crate")
sha256sums=('c330836996984e72a1db2f7c2e2c18d6fb9d232e359320e1bc4291df83e2619b')

prepare() {
    cd "$pkgname-$pkgver"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

package() {
    cd "$pkgname-$pkgver"
    find target/release \
        -maxdepth 1 \
        -executable \
        -type f \
        -exec install -Dm0755 -t "$pkgdir/usr/bin/" {} +
    install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}

