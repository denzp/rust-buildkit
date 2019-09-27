#!/bin/bash
set -e

for example in reverse; do
    source .ci/examples/$example.sh
done
