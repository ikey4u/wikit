#!/usr/bin/env bash

set -e

PROJDIR=$(git rev-parse --show-toplevel)

function main() {
    case "$1" in
        publish)
            if docker ps --format '{{.Names}}' | grep -q wikit; then
                docker exec -it wikit bash /wikitdev/wikit/scripts/build_on_docker.sh
            else
                echo "[x] Required docker container 'wikit' is not found"
            fi
            ;;
        *)
            echo "[x] Unknown subcommand: [$1]"
            ;;
    esac
}

main "$@"