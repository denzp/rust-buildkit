#!/bin/bash
source $(dirname $0)/common.sh

FRONTEND_LABEL="rust-buildkit:download-frontend"
OUTPUT_LABEL="rust-buildkit:download-image"

set -ex

docker build -t $FRONTEND_LABEL -f $EXAMPLES_DIR/download.dockerfile $WORKSPACE_DIR
docker build -t $OUTPUT_LABEL   -f $EXAMPLES_DIR/download.input $WORKSPACE_DIR

diff --strip-trailing-cr --color=always <(cat $EXAMPLES_DIR/download.output) <(docker run --rm $OUTPUT_LABEL)
