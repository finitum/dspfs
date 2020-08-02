# Maintainer: Jonathan Donszelmann <jonabent@gmail.com>
# Maintainer: Victor Roest <victor@xirion.net>
pkgname=dspfs
pkgver=0.1.0
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')

build() {
    return 0
}

package() {
    cargo install --root="$pkgdir" dspfs
    install -m644  $srcdir/installation/dspfs.service ${pkgdir}/usr/lib/systemd/user || return 1

    echo "To start dspfs, enable the service with 'systemd enable --now --user dspfs'"
}
