# Maintainer: fiftydinar <srbaizoki4@tuta.io>

pkgname=xfce-aero-lang-changer
pkgver=1.0.4
pkgrel=1
pkgdesc="GUI language switcher for XFCE with Aero-style theming"
arch=('x86_64')
url="https://github.com/fiftydinar/xfce-aero-lang-changer"
license=('Apache-2.0')

# LINK=static  → bundle FLTK (no system dep, bigger binary)
# LINK=dynamic → use system FLTK (default)
depends=('fltk')
makedepends=('cargo' 'make' 'cmake')
source=("xfce-aero-lang-changer-$pkgver.tar.gz::https://github.com/fiftydinar/xfce-aero-lang-changer/archive/refs/tags/v$pkgver.tar.gz")
sha512sums=('31198186b4820c123ba103e00e9f265d5c2fd7b889dc7dfc6596721704c2f27a4deb229f2777a0e07a6bca7b10eb69595749e9dbac09a347ce756482893dd6f1')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  make LINK="${LINK:-dynamic}"
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  make install PREFIX=/usr DESTDIR="$pkgdir"
}
