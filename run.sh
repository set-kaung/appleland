#!/bin/bash

exec > >(tee build.log) 2>&1

set -a
source "$1"
set +a

cargo sqlx prepare --workspace -- --quiet &&
  cargo clippy --all-targets --release --quiet &&
  cargo build --release || exit 1

read -p "Build successful. Run application? [y/N] " answer

if [[ "$answer" =~ ^[Yy]$ ]]; then
  ./target/release/appleland
else
  echo "Build complete. Not starting application."
fi
