#!/usr/bin/env bash
# Loads the Java repo's PostgreSQL schema and seed data into the petclinic database, so the
# Rust service can be started without ever running the Spring Boot app.
#
# Usage:
#   ./scripts/init-db.sh                 # uses docker compose (profile "postgres") from the repo root
#   DATABASE_URL=postgres://... ./scripts/init-db.sh   # uses a local psql against that URL
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SQL_DIR="${REPO_ROOT}/src/main/resources/db/postgres"

if [[ ! -f "${SQL_DIR}/schema.sql" || ! -f "${SQL_DIR}/data.sql" ]]; then
  echo "schema.sql / data.sql not found under ${SQL_DIR}" >&2
  exit 1
fi

run_sql() {
  local file="$1"
  if [[ -n "${DATABASE_URL:-}" ]] && command -v psql >/dev/null 2>&1; then
    psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -q -f "${file}"
  else
    docker compose -f "${REPO_ROOT}/docker-compose.yml" --profile postgres exec -T postgres \
      psql -U petclinic -d petclinic -v ON_ERROR_STOP=1 -q < "${file}"
  fi
}

wait_for_db() {
  for _ in $(seq 1 30); do
    if [[ -n "${DATABASE_URL:-}" ]] && command -v psql >/dev/null 2>&1; then
      psql "${DATABASE_URL}" -c 'select 1' >/dev/null 2>&1 && return 0
    else
      docker compose -f "${REPO_ROOT}/docker-compose.yml" --profile postgres exec -T postgres \
        pg_isready -U petclinic -d petclinic >/dev/null 2>&1 && return 0
    fi
    sleep 1
  done
  echo "PostgreSQL did not become ready" >&2
  exit 1
}

if [[ -z "${DATABASE_URL:-}" ]]; then
  docker compose -f "${REPO_ROOT}/docker-compose.yml" --profile postgres up -d postgres
fi
wait_for_db

# The SQL files are idempotent (CREATE TABLE IF NOT EXISTS / INSERT ... WHERE NOT EXISTS).
# schema.sql recreates indexes unconditionally, so index errors on a re-run are tolerated.
run_sql "${SQL_DIR}/schema.sql" 2>/dev/null || true
run_sql "${SQL_DIR}/data.sql"
echo "petclinic schema and seed data loaded"
