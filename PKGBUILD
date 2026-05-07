pkgname=git-flow-tui
pkgver=0.1.0
pkgrel=1
pkgdesc='Terminal UI for gitflow'
arch=('x86_64')
url="https://github.com/Emiliopg91/${pkgname}"
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
  'rust'
  'pkg-config' 
)
source=(
  "git+$url.git#tag=$pkgver-$pkgrel"
)
sha256sums=(
  'SKIP'
)

build() {
  export LIBSSH2_SYS_USE_PKG_CONFIG=1
  export OPENSSL_NO_VENDOR=1         

  cd "$srcdir/${pkgname}"
  cargo build --release
}

package() {
  install -Dm755 "$srcdir/${pkgname}/target/release/${pkgname}" "$pkgdir/usr/bin/${pkgname}"
}