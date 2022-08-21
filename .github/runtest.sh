#!/bin/bash -ex

EXEC_PATH=$(dirname "$(realpath "$0")")
PROJECT_PATH="$(dirname $EXEC_PATH)"

TEST_ARTIFACTS_FOLDER="./test_artifacts/"
CONTAINER_WORKSPACE="/workspace/rabc"
CONTAINER_TEST_ARTIFACTS_FOLDER="/test_artifacts"

if [ -n "$1" ];then
    CONTAINER_IMAGE="quay.io/librabc/$1"
else
    if [ -z "$CONTAINER_IMAGE" ];then
        CONTAINER_IMAGE="quay.io/librabc/c9s-rabc-ci"
    fi
fi

CONTAINER_ID=""

function cleanup {
    if [ -n "$CONTAINER_ID" ];then
        podman rm -f $CONTAINER_ID || true
        CONTAINER_ID=""
    fi
}

trap cleanup ERR EXIT

mkdir $TEST_ARTIFACTS_FOLDER || true

CONTAINER_ID=$(podman run -it -d \
    -v $PROJECT_PATH:$CONTAINER_WORKSPACE \
    -v $TEST_ARTIFACTS_FOLDER:$CONTAINER_TEST_ARTIFACTS_FOLDER \
    $CONTAINER_IMAGE /bin/bash
)

podman exec -i $CONTAINER_ID \
    /bin/bash -c "cd $CONTAINER_WORKSPACE; make install"

podman exec -i $CONTAINER_ID \
    /bin/bash -c "cd $CONTAINER_WORKSPACE; \
        env TEST_ARTIFACTS_FOLDER=$CONTAINER_TEST_ARTIFACTS_FOLDER \
            tests/runtest.sh"
