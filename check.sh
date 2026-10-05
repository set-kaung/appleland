#!/bin/bash

set -a
source "$1"
set +a

cargo sqlx prepare --workspace && cargo clippy --all-targets --release 