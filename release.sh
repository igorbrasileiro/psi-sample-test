#!/bin/bash

echo "Install cargo-bump. Run `cargo install cargo-bump`"

cargo bump $1 && git add ./Cargo.toml && git commit -m "Bumping package version" && cargo publish && git push origin main -f
