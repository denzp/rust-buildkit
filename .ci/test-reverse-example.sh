#!/bin/bash
set -e

FRONTEND_LABEL="rust-buildkit:reverse-frontend"
OUTPUT_LABEL="rust-buildkit:reverse-image"

docker build -t $FRONTEND_LABEL -f $EXAMPLES_DIR/reverse.dockerfile .
docker build -t $OUTPUT_LABEL   -f $EXAMPLES_DIR/reverse.input .

diff --strip-trailing-cr --color=always <(cat $EXAMPLES_DIR/reverse.output) <(docker run --rm $OUTPUT_LABEL)
