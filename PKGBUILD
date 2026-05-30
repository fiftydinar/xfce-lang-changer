# Maintainer: fiftydinar <srbaizoki4@tuta.io>

pkgname=xfce-aero-lang-changer
pkgver=1.0.4
pkgrel=1
pkgdesc="GUI language switcher for XFCE with Aero-style theming"
arch=('x86_64')
url="https://github.com/fiftydinar/xfce-aero-lang-changer"
license=('Apache-2.0')
depends=('fltk')
makedepends=('cargo' 'make' 'cmake')
source=("xfce-aero-lang-changer-$pkgver.tar.gz::https://github.com/fiftydinar/xfce-aero-lang-changer/archive/refs/tags/v$pkgver.tar.gz")
sha512sums=('62c5e66501fa90c250347eb8dd42816395e93e2cf000900a165fc4010295cdf9bacbd48df94db8c767ba5a7de09f89742de54a994f906dbd9a4d9d9caf3130bb')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  make  
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  make install PREFIX=/usr DESTDIR="$pkgdir"
}
