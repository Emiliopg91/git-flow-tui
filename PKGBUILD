pkgname=git-flow-tui
pkgver=0.1.0
pkgrel=1
pkgdesc='Terminal UI for gitflow'
arch=('x86_64')
url='https://github.com/Emiliopg91/aur-check-rebuild'
license=('GPL-2')
depends=(
  'brotli'
  'glibc'
  'libgcc'
  'libgit2'
  'libssh2'
  'llhttp' 
  'openssl' 
  'pcre2' 
  'zlib-ng-compat' 
  'zstd'
)
makedepends=(
  rust
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