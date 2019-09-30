#!/bin/bash
source $(dirname $0)/common.sh

FRONTEND_LABEL="rust-buildkit:reverse-frontend"
OUTPUT_LABEL="rust-buildkit:reverse-image"

set -ex

docker build -t $FRONTEND_LABEL -f $EXAMPLES_DIR/reverse.dockerfile $WORKSPACE_DIR
docker build -t $OUTPUT_LABEL   -f $EXAMPLES_DIR/reverse.input $WORKSPACE_DIR

diff --strip-trailing-cr --color=always <(cat $EXAMPLES_DIR/reverse.output) <(docker run --rm $OUTPUT_LABEL)
