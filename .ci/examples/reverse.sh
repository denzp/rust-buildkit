#!/bin/bash
set -e

EXAMPLE="reverse"
FRONTEND_LABEL="rust-buildkit:$EXAMPLE-frontend"
OUTPUT_LABEL="rust-buildkit:$EXAMPLE-image"

docker build -t $FRONTEND_LABEL -f .ci/examples/$EXAMPLE.dockerfile .
docker build -t $OUTPUT_LABEL   -f .ci/examples/$EXAMPLE.input .

OUTPUT=$(docker run --rm -it $OUTPUT_LABEL)
REFERENCE=$(cat .ci/examples/$EXAMPLE.output)

diff --color=always <(echo -n "$OUTPUT") <(echo -n "$REFERENCE")
