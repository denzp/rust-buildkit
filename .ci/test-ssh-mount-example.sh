#!/bin/bash
source $(dirname $0)/common.sh

FRONTEND_LABEL="rust-buildkit:ssh-mount-frontend"
OUTPUT_LABEL="rust-buildkit:ssh-mount-image"

set -ex

docker build -t $FRONTEND_LABEL -f $EXAMPLES_DIR/ssh-mount.dockerfile $WORKSPACE_DIR
docker build -t $OUTPUT_LABEL   -f $EXAMPLES_DIR/ssh-mount.input $WORKSPACE_DIR --ssh=default

diff --strip-trailing-cr --color=always <(cat $EXAMPLES_DIR/ssh-mount.output) <(docker run --rm $OUTPUT_LABEL)
