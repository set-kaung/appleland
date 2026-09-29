# Task runner for appleland.
#
#   just              run the full ordered dev loop (sqlx cache, lint, build)
#   just --list       list every recipe
#   just build        run one recipe
#   just build .env.dev
#
# Recipes in a chain run in order and the first failure aborts the rest, so the
# default recipe behaves like a sequential script. Every recipe stays runnable
# on its own. Recipes that touch the database take an env file and default to
# .env.local; pass a path as the single argument to target another environment.

set positional-arguments

# Full ordered dev loop. `docs` comes last because it runs the release binary
# built by `build`.
default: sqlx-prepare check build docs

# Local stand-in for the GitLab test stage, which is commented out in
# .gitlab-ci.yml. Verifies the committed sqlx cache is not stale, then lints,
# builds and tests with warnings denied, matching the CI variables
# SQLX_OFFLINE / CARGO_BUILD_WARNINGS. Unlike the default recipe this rewrites
# no committed artifact, so a clean tree stays clean.
[script('bash')]
ci env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{env_file}}"
    set +a
    cargo sqlx prepare --workspace --check
    CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets
    CARGO_BUILD_WARNINGS=deny cargo build --release
    cargo test

# List every recipe.
list:
    @just --list --unsorted

# Refresh the offline sqlx query cache in .sqlx/. Requires a reachable database.
[script('bash')]
sqlx-prepare env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo sqlx prepare --workspace

# Lint every target, including tests.
[script('bash')]
check env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo clippy --all-targets

# Compile the release binary.
[script('bash')]
build env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo build --release

# Compile and run the unit tests. Needs no database.
[script('bash')]
test env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo test

# Migrate, lint, build, then ask before starting the server.
[script('bash')]
run env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo sqlx prepare --workspace
    cargo clippy --all-targets
    cargo build --release
    read -r -p "Build successful. Run application? [y/N] " answer
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        ./target/release/appleland
    else
        echo "Build complete. Not starting application."
    fi

# Start the release binary. Build it first with `just build`.
[script('bash')]
serve env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    ./target/release/appleland

# Format the workspace.
fmt:
    cargo fmt

# Apply pending migrations.
[script('bash')]
migrate env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    cargo sqlx migrate run --source=./db/migrations

# Create a new timestamped migration: just new-migration add_user_email
new-migration name:
    cargo sqlx migrate add "{{ name }}" --source=db/migrations

# Regenerate db/schema.sql from the database. Documentation only; the file
# is not meant to recreate a database.
[script('bash')]
schema env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    [[ -n "${DATABASE_URL:-}" ]] || { echo "DATABASE_URL is not set in {{ env_file }}" >&2; exit 1; }
    pg_dump "$DATABASE_URL" --schema-only --no-owner --no-privileges |
        sed -E -e '/^\\(un)?restrict /d' -e '/^SET /d' -e '/^SELECT pg_catalog\.set_config/d' |
        cat -s > ./db/schema.sql

# Migrate, then refresh db/schema.sql.
[script('bash')]
amig env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    [[ -n "${DATABASE_URL:-}" ]] || { echo "DATABASE_URL is not set in {{ env_file }}" >&2; exit 1; }
    cargo sqlx migrate run --source=./db/migrations
    pg_dump "$DATABASE_URL" --schema-only --no-owner --no-privileges |
        sed -E -e '/^\\(un)?restrict /d' -e '/^SET /d' -e '/^SELECT pg_catalog\.set_config/d' |
        cat -s > ./db/schema.sql

# Dump table data for backup: just dump-data .env.local backup.sql
[script('bash')]
dump-data env_file=".env.local" out="data_dump.sql":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    [[ -n "${DATABASE_URL:-}" ]] || { echo "DATABASE_URL is not set in {{ env_file }}" >&2; exit 1; }
    pg_dump "$DATABASE_URL" --data-only --file="{{ out }}" -n public \
        --exclude-table=_sqlx_migrations --column-inserts --disable-triggers

# Regenerate the OpenAPI YAML from the utoipa annotations. Requires `just build`
# to have produced the release binary first.
[script('bash')]
docs env_file=".env.local":
    set -euo pipefail
    set -a
    source "{{ env_file }}"
    set +a
    ./target/release/appleland --gen-docs-only
