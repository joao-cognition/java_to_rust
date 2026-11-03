#!/usr/bin/env bash
# Java ↔ Rust parity harness. Starts PostgreSQL (docker compose `postgres` profile), drops the schema so the Java
# service re-creates and re-seeds it (data.sql + deterministic generator, schedule from tomorrow), starts the Java
# reference service (:9966) and rust-travel (:9977) on that database, then runs `j2r parity` over the full read
# catalogue. Extra arguments go to `j2r parity` (e.g. `--only offers -v`). KEEP=1 leaves both services running.
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="${OUT:-bench/results}"
LOGS="${LOGS:-target/parity-logs}"
mkdir -p "$LOGS"
pids=()
cleanup() { [ "${KEEP:-0}" = 1 ] || for p in "${pids[@]}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT

wait_for() { # wait_for <url> <name> <seconds>
  for _ in $(seq "$3"); do curl -sf -o /dev/null "$1" && return 0; sleep 1; done
  echo "$2 did not come up at $1 (logs in $LOGS)" >&2; exit 1
}
for port in 9966 9977; do
  if curl -s -o /dev/null "http://localhost:$port/"; then echo "port $port is busy — stop the running service first" >&2; exit 1; fi
done

echo "== postgres"
docker image inspect postgres:16.3 >/dev/null 2>&1 || docker pull postgres:16.3 \
  || { docker pull mirror.gcr.io/library/postgres:16.3 && docker tag mirror.gcr.io/library/postgres:16.3 postgres:16.3; }
docker compose --profile postgres up -d postgres
for _ in $(seq 60); do docker compose exec -T postgres pg_isready -U travel -d travel >/dev/null 2>&1 && break; sleep 1; done
docker compose exec -T postgres psql -q -U travel -d travel -c 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;'

echo "== build"
[ -f target/travel-reservation-api-1.0.0.jar ] || ./mvnw -q -B package -DskipTests -Djacoco.skip=true
(cd rust-travel && cargo build --release -q)
(cd cli && cargo build --release -q)

echo "== java :9966 (seeds the database)"
java -jar target/travel-reservation-api-1.0.0.jar --spring.profiles.active=postgres >"$LOGS/java.log" 2>&1 &
pids+=($!)
wait_for http://localhost:9966/travel/actuator/health java 120
grep -o 'Demo data generated.*' "$LOGS/java.log" || true

echo "== rust :9977"
rust-travel/target/release/rust-travel >"$LOGS/rust.log" 2>&1 &
pids+=($!)
wait_for http://localhost:9977/travel/api/airports rust 30

echo "== j2r parity"
cli/target/release/j2r parity --out "$OUT" "$@"
