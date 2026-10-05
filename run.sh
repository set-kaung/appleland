#!/bin/bash

set -a
source "$1"
set +a

cargo build --release || exit 1

read -r -p "Build successful. Run application? [y/N] " answer

if [[ "$answer" =~ ^[Yy]$ ]]; then
  ./target/release/appleland
else
  echo "Build complete. Not starting application."
fi
