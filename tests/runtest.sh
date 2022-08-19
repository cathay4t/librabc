#!/bin/bash -x
# SPDX-License-Identifier: Apache-2.0

PROJECT_PATH=$(dirname $(dirname "$(realpath "$0")"))

cd $PROJECT_PATH

: ${TEST_ARTIFACTS_FOLDER:=/tmp}

python3 -c 'import rabc' 1>/dev/null 2>/dev/null && which rabcd
if [ $? -ne 0 ];then
    # We are running in developing environment without rabc installed
    make debug
    export PYTHONPATH=${PYTHONPATH}:${PROJECT_PATH}/src/python
    export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:${PROJECT_PATH}/target/debug
    export PATH=${PATH}:${PROJECT_PATH}/target/debug
fi

pytest -vvv --log-file-level=DEBUG \
    --log-file-date-format='%Y-%m-%d %H:%M:%S' \
    --log-file-format='%(asctime)s %(filename)s:%(lineno)d %(levelname)s %(message)s' \
    --log-file=${TEST_ARTIFACTS_FOLDER}/rabc_test.log \
    tests/integration
