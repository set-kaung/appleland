#!/bin/bash

set -a
source "$1"
set +a

cargo build --release || exit 1

read -r -p "Build successful. Run application? [y/N] " answer

if [[ "$answer" =~ ^[Yy]$ ]]; then
  if command -v jq >/dev/null 2>&1; then
    ./target/release/appleland | jq -Rr 'fromjson? // .'
  else
    ./target/release/appleland
  fi
else
  echo "Build complete. Not starting application."
fi
