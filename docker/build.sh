#!/usr/bin/env bash
# Note: you should only run this script in docker

set -e

WORKDIR="$(cd "$(dirname "$0")"; pwd -P)"
cd ${WORKDIR}

# this is necessary since docker does not load .bashrc when execute command directly from docker
source ${HOME}/.bashrc

PROJDIR=$(git rev-parse --show-toplevel)
VERSION=$(cat ${PROJDIR}/src/main.rs | grep version| grep -oE "[0-9]{1,2}.[0-9]{1,2}.[0-9]")
LINUX_CARGO_CONFIG=$(cat << 'EOF'
[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin14-clang"
ar = "x86_64-apple-darwin14-ar"

[target.x86_64-unknown-linux-gnu]
linker = "x86_64-pc-linux-gnu-g++"
ar = "x86_64-pc-linux-gnu-ar"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-g++"
ar = "x86_64-w64-mingw32-ar"
EOF
)

mkdir -p ${PROJDIR}/.cargo ${PROJDIR}/release
echo "${LINUX_CARGO_CONFIG}" > ${PROJDIR}/.cargo/config

CC=o64-clang CXX=o64-clang++ cargo build --release --target x86_64-apple-darwin
cd ${PROJDIR}/target/x86_64-apple-darwin/release && \
    zip -r wikit-mac-v${VERSION}.zip wikit && \
    mv wikit-mac-v${VERSION}.zip ${PROJDIR}/release

cargo build --release --target x86_64-unknown-linux-gnu
cd ${PROJDIR}/target/x86_64-unknown-linux-gnu/release && \
    zip -r wikit-linux-v${VERSION}.zip wikit && \
    mv wikit-linux-v${VERSION}.zip ${PROJDIR}/release

cargo build --release --target x86_64-pc-windows-gnu
cd ${PROJDIR}/target/x86_64-pc-windows-gnu/release && \
    zip -r wikit-win-v${VERSION}.zip wikit.exe && \
    mv wikit-win-v${VERSION}.zip ${PROJDIR}/release

rm -rf ${PROJDIR}/.cargo/config