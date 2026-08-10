#!/bin/bash

exec > >(tee build.log) 2>&1

export $(grep -v '^#' "$1" | xargs) &&
cargo sqlx prepare --workspace &&
cargo clippy --all-targets &&
cargo build --release || exit 1

read -p "Build successful. Run application? [y/N] " answer

if [[ "$answer" =~ ^[Yy]$ ]]; then
    ./target/release/appleland
else
    echo "Build complete. Not starting application."
fi