#!/bin/bash
set -e

export DOCKER_BUILDKIT="1"
export EXAMPLES_DIR="$(dirname $0)/../buildkit-frontend/examples"

source $(dirname $0)/test-reverse-example.sh
