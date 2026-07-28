export $(grep -v '^#' "$1" | xargs) &&
  cargo sqlx prepare --workspace &&
  cargo clippy --all-targets &&
  cargo build --release &&
  ./target/release/appleland
