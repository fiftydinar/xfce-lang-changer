# Maintainer: fiftydinar <srbaizoki4@tuta.io>

pkgname=xfce-aero-lang-changer
pkgver=1.0.3
pkgrel=1
pkgdesc="GUI language switcher for XFCE with Aero-style theming"
arch=('x86_64')
url="https://github.com/fiftydinar/xfce-aero-lang-changer"
license=('Apache-2.0')
depends=('fltk')
makedepends=('cargo' 'make' 'cmake')
source=("xfce-aero-lang-changer-$pkgver.tar.gz::https://github.com/fiftydinar/xfce-aero-lang-changer/archive/refs/tags/v$pkgver.tar.gz")
sha512sums=('77c51bd62d2534adbd83b93668a4223c65da949147fa7cf034d8abc1537e10de91699d0a4560aa18ae8d0a4f0bbbc307891a89b0a0a15d33a2e31a08145e1bc2')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  make  
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  make install PREFIX=/usr DESTDIR="$pkgdir"
}
