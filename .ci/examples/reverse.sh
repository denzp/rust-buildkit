#!/bin/bash
set -e

EXAMPLE="reverse"
FRONTEND_LABEL="rust-buildkit:$EXAMPLE-frontend"
OUTPUT_LABEL="rust-buildkit:$EXAMPLE-image"

docker build -t $FRONTEND_LABEL -f .ci/examples/$EXAMPLE.dockerfile .
docker build -t $OUTPUT_LABEL   -f .ci/examples/$EXAMPLE.input .

diff --strip-trailing-cr --color=always <(cat .ci/examples/$EXAMPLE.output) <(docker run --rm $OUTPUT_LABEL)
