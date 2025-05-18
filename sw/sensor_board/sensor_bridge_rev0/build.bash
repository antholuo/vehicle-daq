#!/bin/bash

# Enable exit tracing and variable protection
set -o errtrace
set -o nounset

CLEAN=false
TEST=false
FLASH=false
BUILD_TYPE="Debug"
GENERATOR="Unix Makefiles"

while getopts "c,t,h,f,r" opt; do
    case $opt in
        c)
            CLEAN=true
        ;;
        t)
            TEST=true
        ;;
        f)
            FLASH=true
        ;;
        r)
            BUILD_TYPE="Release"
        ;;
        h|\?)
            printf "%s\n" "Usage: $0 [OPTIONS]"\
                "Script to build the sensor_board_rev0 project"\
                "    -f                 - flashes the Safety after building"\
                "    -c                 - removes previous build files before building"\
                "    -h                 - outputs this message"\
                "    -t                 - runs tests after building if build is successful"\
                "    -r                 - Sets the build type to release"
            exit 1
        ;;
    esac
done

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

# Set up exit condition
trap 'die $? $LINENO' ERR

BUILD_DIR="build"

if [[ $CLEAN == true ]]; then
    echo "Cleaning old build environment"
    cmake -E remove_directory $BUILD_DIR
fi

# Prebuild info display
echo "Building ..."
# if [[ $# > 0 ]]; then
#     echo "with cmake parameters: $@"
# fi

# Build commands
cmake -E make_directory $BUILD_DIR
cmake -E chdir $BUILD_DIR \
  cmake \
    -G "${GENERATOR}" \
    -DCMAKE_BUILD_TYPE="${BUILD_TYPE}" \
    -DCMAKE_TOOLCHAIN_FILE="sensor_bridge_rev0.cmake" \
    -Wdev\
    -Wdeprecated\
    ../

cmake --build $BUILD_DIR

if [[ $TEST == true ]] ; then
    cmake --build $BUILD_DIR --target test
fi

if [[ $FLASH == true ]] ; then
    cmake --build $BUILD_DIR --target install
fi

# Final status display
echo ""
echo "Build SUCCESS!"
exit 0
