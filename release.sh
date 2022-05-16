#!/bin/bash

cargo bump $1 && git add ./Cargo.toml && git commit -m "Bumping package version" && cargo publish && git push origin main -f
