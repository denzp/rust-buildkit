export DOCKER_BUILDKIT="1"

export WORKSPACE_DIR=$(readlink -f "$(dirname $0)/..")
export EXAMPLES_DIR="$WORKSPACE_DIR/buildkit-frontend/examples"
