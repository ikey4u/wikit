#!/usr/bin/env bash

set -e -x

WORKDIR="$(cd "$(dirname "$0")"; pwd -P)"

function fn_add_utilities() {
    yes | pacman -Sy
    yes | pacman -S wget git curl clang cmake zip

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    echo 'export PATH=$HOME/.cargo/bin:$PATH' >> ~/.bashrc
    source ~/.bashrc
}

function fn_add_mac_toolchain() {
    git clone https://github.com/tpoechtrager/osxcross
    cd osxcross
    wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
    mv MacOSX10.10.sdk.tar.xz tarballs/
    UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
    echo export PATH=${PWD}/target/bin:'$PATH' >> ~/.bashrc
    source ~/.bashrc
    rustup target add x86_64-apple-darwin
}

function fn_add_win_toolchain() {
    yes | pacman -S mingw-w64-gcc
    rustup target add x86_64-pc-windows-gnu
    rustup toolchain install stable-x86_64-pc-windows-gnu
}

if [[ -e /etc/issue ]] && [[ $(cat /etc/issue) =~ "Arch Linux" ]]; then
    cd ${WORKDIR}
    echo "[+] fn_add_utilities ..."
    fn_add_utilities
    echo "[+] fn_add_mac_toolchain ..."
    fn_add_mac_toolchain
    echo "[+] fn_add_win_toolchain ..."
    fn_add_win_toolchain
else
    echo "[x] Run $0 script on arch linux"
fi