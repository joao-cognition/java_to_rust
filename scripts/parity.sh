#!/usr/bin/env bash
# Java ↔ Rust parity acceptance check: one command, every GET endpoint.
#
#   scripts/parity.sh            live: pristine postgres → java :9966 (seeds) → rust :9977 if built → j2r parity
#   scripts/parity.sh --record   same, and also store java's responses + a dump of the seeded database in parity/golden/
#   scripts/parity.sh --golden   no java: restore the recorded database, start rust, compare it against the goldens
#
# Extra arguments go to `j2r parity` (e.g. `--only offers`). Writes parity/results.json and parity/report.md and exits
# non-zero on any difference (including "rust down"). J2R_RUST=http://localhost:9966/travel/api runs Java vs Java.
# KEEP=1 leaves the services running.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="${OUT:-parity}"
GOLDEN="${GOLDEN:-parity/golden}"
LOGS="${LOGS:-target/parity-logs}"
JAR=target/travel-reservation-api-1.0.0.jar
RUST_BIN="${RUST_BIN:-rust-travel/target/release/rust-travel}"
JAVA_URL="${J2R_JAVA:-http://localhost:9966/travel/api}"
RUST_URL="${J2R_RUST:-http://localhost:9977/travel/api}"
export MVNW_REPOURL="${MVNW_REPOURL:-https://maven-central-eu.storage-download.googleapis.com/maven2}"

mode=live
for a in "$@"; do
  case "$a" in
    --record) mode=record ;;
    --golden) mode=golden ;;
  esac
done

mkdir -p "$LOGS"
pids=()
cleanup() { [ "${KEEP:-0}" = 1 ] || for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT

up() { curl -s -o /dev/null --max-time 2 "$1"; }
wait_for() { # wait_for <url> <seconds>
  for _ in $(seq "$2"); do curl -sf -o /dev/null --max-time 2 "$1" && return 0; sleep 1; done
  return 1
}
psql_travel() { docker compose exec -T postgres psql -q -v ON_ERROR_STOP=1 -U travel -d travel "$@"; }

for port in 9966 9977; do
  if up "http://localhost:$port/"; then echo "port $port is busy: stop the running service first" >&2; exit 1; fi
done

echo "== build"
(cd cli && cargo build --release -q)
if [ "$mode" != golden ] && [ ! -f "$JAR" ]; then ./mvnw -q -B package -DskipTests -Djacoco.skip=true; fi
if [ -f rust-travel/Cargo.toml ]; then (cd rust-travel && cargo build --release -q); fi

echo "== postgres (pristine schema)"
docker image inspect postgres:16.3 >/dev/null 2>&1 \
  || { docker pull mirror.gcr.io/library/postgres:16.3 && docker tag mirror.gcr.io/library/postgres:16.3 postgres:16.3; } \
  || docker pull postgres:16.3
docker compose --profile postgres up -d postgres
for _ in $(seq 60); do docker compose exec -T postgres pg_isready -U travel -d travel >/dev/null 2>&1 && break; sleep 1; done
psql_travel -c 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;'

if [ "$mode" = golden ]; then
  [ -f "$GOLDEN/db.sql.gz" ] || { echo "no $GOLDEN/db.sql.gz: record first with scripts/parity.sh --record" >&2; exit 1; }
  echo "== java: not started (restoring the recorded database from $GOLDEN/db.sql.gz)"
  gunzip -c "$GOLDEN/db.sql.gz" | psql_travel >/dev/null
else
  echo "== java :9966 (schema.sql + data.sql + DemoDataSeeder)"
  java -jar "$JAR" --spring.profiles.active=postgres >"$LOGS/java.log" 2>&1 &
  pids+=($!)
  wait_for http://localhost:9966/travel/actuator/health 180 || { echo "java did not come up (see $LOGS/java.log)" >&2; exit 1; }
  grep -o 'Demo data generated.*' "$LOGS/java.log" || true
fi

if [ "$RUST_URL" != "http://localhost:9977/travel/api" ]; then
  echo "== rust: not started, J2R_RUST=$RUST_URL"
elif [ -x "$RUST_BIN" ]; then
  echo "== rust :9977"
  "$RUST_BIN" >"$LOGS/rust.log" 2>&1 &
  pids+=($!)
  wait_for "$RUST_URL/airports" 30 || echo "rust: down (did not come up, see $LOGS/rust.log)"
else
  echo "rust: down ($RUST_BIN not built)"
fi

echo "== j2r parity ($mode)"
status=0
J2R_JAVA="$JAVA_URL" J2R_RUST="$RUST_URL" cli/target/release/j2r parity --out "$OUT" --golden-dir "$GOLDEN" "$@" || status=$?

if [ "$mode" = record ] && [ -f "$GOLDEN/manifest.json" ]; then
  docker compose exec -T postgres pg_dump --no-owner --no-privileges -U travel -d travel | gzip -n -9 >"$GOLDEN/db.sql.gz"
  echo "recorded database → $GOLDEN/db.sql.gz ($(du -h "$GOLDEN/db.sql.gz" | cut -f1))"
fi
exit "$status"
