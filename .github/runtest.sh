#!/bin/bash -ex

EXEC_PATH=$(dirname "$(realpath "$0")")
PROJECT_PATH="$(dirname $EXEC_PATH)"

TEST_ARTIFACT_FOLDER="test_artifacts/"
CONTAINER_WORKSPACE="/workspace/rabc"
CONTAINER_TEST_ARTIFACT_FOLDER="/test_artifacts"

CONTAINER_ID=""

function cleanup {
    if [ -n "$CONTAINER_ID" ];then
        podman rm $CONTAINER_ID
    fi
}

trap run_exit ERR EXIT

CONTAINER_ID=$(podman run -d \
    -v $PROJECT_PATH:$CONTAINER_WORKSPACE \
    -v $TEST_ARTIFACT_FOLDER:$CONTAINER_TEST_ARTIFACT_FOLDER \
    $1
)

podman exec -i $CONTAINER_ID \
    /bin/bash -c "cd $CONTAINER_WORKSPACE; make install"

podman exec -i $CONTAINER_ID \
    /bin/bash -c "cd $CONTAINER_WORKSPACE; \
        env TEST_ARTIFACTS_FOLDER=$CONTAINER_TEST_ARTIFACT_FOLDER \
            tests/runtest.sh"
