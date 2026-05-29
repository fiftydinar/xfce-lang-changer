# Maintainer: fiftydinar <fiftydinar@tuta.io>

pkgname=xfce-lang-changer
pkgver=1.0.0
pkgrel=1
pkgdesc="GUI language switcher for XFCE with Aero-style theming"
arch=('x86_64')
url="https://github.com/fiftydinar/xfce-lang-changer"
license=('Apache-2.0')
depends=('fltk')
makedepends=('cargo' 'make')
source=("$pkgname-$pkgver.tar.gz::https://github.com/fiftydinar/$pkgname/archive/refs/tags/v$pkgver.tar.gz")
sha512sums=('SKIP')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  make
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  make install PREFIX=/usr DESTDIR="$pkgdir"
}
