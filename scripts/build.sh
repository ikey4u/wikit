#!/usr/bin/env bash

set -e

PROJDIR=$(git rev-parse --show-toplevel)

function main() {
    case "$1" in
        publish)
            if docker ps --format '{{.Names}}' | grep -q wikit; then
                docker exec -it wikit bash /wikitdev/wikit/docker/build.sh
            else
                echo "[x] Required docker container 'wikit' is not found"
            fi
            ;;
        image)
            cd ${PROJDIR}/docker
	        docker build --build-arg HTTP_PROXY=${http_proxy} -t wikitdev:v0.0.1 --progress plain -f Dockerfile .
            ;;
        container)
            docker run --name wikit -dit -v ${PROJDIR}:/wikitdev/wikit wikitdev:v0.0.1 bash
            docker start wikit
            ;;
        *)
            echo "[x] Unknown subcommand: [$1]"
            ;;
    esac
}

main "$@"
