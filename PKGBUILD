# Maintainer: Jonathan Donszelmann <jonabent@gmail.com>
# Maintainer: Victor Roest <victor@xirion.net>
pkgname=dspfs-git
pkgver=0.1.0
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
source=('dspfs::git+https://github.com/finitum#branch=master')

build() {
    cargo build --release --locked --all-features --target-dir=target
}

package() {
    install -Dm 755 target/release/${pkgname} -t "${pkgdir}/usr/bin"
    install -m644  $srcdir/installation/dspfs.service ${pkgdir}/usr/lib/systemd/user || return 1

    echo "To start dspfs, enable the service with 'systemd enable --now --user dspfs'"
}
