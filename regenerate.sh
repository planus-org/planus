#!/usr/bin/env bash

for f in testfiles/valid/simple.fbs; do
    cargo run --bin codegen -- $f -o planus_util/src/generated/$(basename -s .fbs $f)_generated.rs
    flatc --rust -o planus_util/src/upstream/ $f
done
