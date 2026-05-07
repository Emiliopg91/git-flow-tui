pkgname=git-flow-tui
pkgver=1.0.2
pkgrel=1
pkgdesc='Terminal UI for gitflow'
arch=('x86_64')
url="https://github.com/Emiliopg91/${pkgname}"
license=('GPL-2')
depends=(
  'glibc'
  'libgcc'
)
makedepends=(
  'rust'
)
source=(
  "git+$url.git#tag=$pkgver-$pkgrel"
)
sha256sums=(
  'SKIP'
)

build() {
  cd "$srcdir/${pkgname}"
  cargo build --release
}

package() {
  install -Dm755 "$srcdir/${pkgname}/target/release/${pkgname}" "$pkgdir/usr/bin/${pkgname}"
}