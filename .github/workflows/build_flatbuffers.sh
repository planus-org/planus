#!/usr/bin/env bash

set -eux -o pipefail

sudo apt-get install -y cmake

echo Building flatbuffer v${FLATBUFFER_VERSION}

TMPDIR="$(mktemp -d /tmp/flatbuffers-XXXXXX)"

cd $TMPDIR

wget https://github.com/google/flatbuffers/archive/refs/tags/v${FLATBUFFER_VERSION}.tar.gz -O flatbuffers.tar.gz

tar xfv flatbuffers.tar.gz
cd flatbuffers-*
cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release
make -j flatc
cp flatc $HOME
