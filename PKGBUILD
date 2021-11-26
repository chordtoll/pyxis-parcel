# Maintainer: chordtoll <git@chordtoll.com>
pkgname=pyxis-parcel
pkgver=0.1.1
pkgrel=1
pkgdesc="Utilities to manipulate parcel archives for the pyxis package manager"
license=("MIT")
arch=("x86_64")
makedepends=("cargo")
url="https://github.com/chordtoll/pyxis-parcel"
source=("$pkgname-$pkgver.tar.gz::https://static.crates.io/crates/$pkgname/$pkgname-$pkgver.crate")
sha256sums=('3931982678eba123104f04e175500672213f311a0e9a61e819575b43dc12420d')

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
}

