#!/usr/bin/env bash

set -e -x

WORKDIR="$(cd "$(dirname "$0")"; pwd -P)"

function cmd_prx() {
    http=$http_proxy
    https=$https_proxy

    export http_proxy=$HTTP_PROXY
    export https_proxy=$HTTP_PROXY

    if [[ ! -z "${http_proxy}" ]]; then
        echo "[+] Using proxy [${http_proxy}] ..."
    fi
    bash -c -i "$*"

    export http_proxy=$http
    export https_proxy=$https
}

function fn_add_utilities() {
    yes | pacman -Sy
    yes | pacman -S wget git curl clang cmake zip

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    echo 'export PATH=$HOME/.cargo/bin:$PATH' >> ~/.bashrc
    source ~/.bashrc
}

function fn_add_mac_toolchain() {
    cmd_prx git clone https://github.com/tpoechtrager/osxcross
    cd osxcross
    wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz
    mv MacOSX10.10.sdk.tar.xz tarballs/
    cmd_prx UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
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
    echo "[+] Proxy is [$HTTP_PROXY] ..."
    case "$1" in
        add_utilities)
            echo "[+] fn_add_utilities ..."
            fn_add_utilities
            ;;
        add_win_toolchain)
            echo "[+] fn_add_win_toolchain ..."
            fn_add_win_toolchain
            ;;
        add_mac_toolchain)
            echo "[+] fn_add_mac_toolchain ..."
            fn_add_mac_toolchain
            ;;
        *)
            ;;
    esac
else
    echo "[x] Run $0 script on arch linux"
fi