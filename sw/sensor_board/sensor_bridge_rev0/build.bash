#!/bin/bash

# Enable exit tracing and variable protection
set -o errtrace
set -o nounset

CLEAN=false
TEST=false
FLASH=false
BUILD_TYPE="Debug"
GENERATOR="Unix Makefiles"
NODE_ID="20"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -c)
            CLEAN=true
            shift
            ;;
        -t)
            TEST=true
            shift
            ;;
        -f)
            FLASH=true
            shift
            ;;
        -r)
            BUILD_TYPE="Release"
            shift
            ;;
        -id)
            if [[ -n "$2" && "$2" =~ ^[0-9]+$ ]]; then
                NODE_ID="$2"
                shift 2
            else
                echo "Error: -id must be followed by a numeric value"
                exit 1
            fi
            ;;
        -h|--help|*)
            printf "%s\n" "Usage: $0 [OPTIONS]"\
                "Script to build the sensor_bridge_rev0 project"\
                "    -f                 - flashes the Safety after building"\
                "    -c                 - removes previous build files before building"\
                "    -h                 - outputs this message"\
                "    -t                 - runs tests after building if build is successful"\
                "    -r                 - sets the build type to Release"\
                "    -id <value>        - sets NODE_ID (e.g., -id 20)"\
            exit 1
            ;;
    esac
done

# Determine generator
if command -v ninja >/dev/null 2>&1; then
    GENERATOR="Ninja"
elif command -v make >/dev/null 2>&1; then
    GENERATOR="Unix Makefiles"
elif command -v mingw32-make >/dev/null 2>&1; then
    GENERATOR="MinGW Makefiles"
fi

die() {
    echo ""
    echo "this build FAILED!"
    echo "Error $1 was encountered on line $2."
    exit $1
}

trap 'die $? $LINENO' ERR

BUILD_DIR="build"

if [[ $CLEAN == true ]]; then
    echo "Cleaning old build environment"
    cmake -E remove_directory $BUILD_DIR
fi

echo "Building..."

# Construct cmake command
CMAKE_CMD=(
    cmake -E chdir "$BUILD_DIR"
    cmake
    -G "$GENERATOR"
    -DCMAKE_BUILD_TYPE="$BUILD_TYPE"
    -DCMAKE_TOOLCHAIN_FILE="sensor_bridge_rev0.cmake"
    -Wdev
    -Wdeprecated
)

# Include NODE_ID if specified
if [[ -n "$NODE_ID" ]]; then
    echo "Using NODE_ID=$NODE_ID"
    CMAKE_CMD+=("-DNODE_ID=$NODE_ID")
fi

CMAKE_CMD+=("../")

# Build steps
cmake -E make_directory "$BUILD_DIR"
"${CMAKE_CMD[@]}"
cmake --build "$BUILD_DIR"

if [[ $TEST == true ]]; then
    cmake --build "$BUILD_DIR" --target test
fi

if [[ $FLASH == true ]]; then
    cmake --build "$BUILD_DIR" --target install
fi

echo ""
echo "Build SUCCESS!"
exit 0
