#!/bin/bash

exec > >(tee build.log) 2>&1

export $(grep -v '^#' "$1" | xargs) &&
cargo sqlx prepare --workspace &&
cargo clippy --all-targets &&
cargo build --release &&
./target/release/appleland