#!/usr/bin/env bash
# Compares the JSON of every ported endpoint between the Java service and the Rust service
# (both must run against the same PostgreSQL). Key order is normalised with `jq -S`.
# Exits non-zero if any endpoint differs.
#
#   JAVA_URL=http://localhost:9966/petclinic RUST_URL=http://localhost:9977/petclinic bench/parity.sh
set -uo pipefail

JAVA_URL="${JAVA_URL:-http://localhost:9966/petclinic}"
RUST_URL="${RUST_URL:-http://localhost:9977/petclinic}"

for tool in curl jq; do
  command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 2; }
done

for url in "$JAVA_URL" "$RUST_URL"; do
  if ! curl -fs "$url/actuator/health" >/dev/null; then
    echo "service at $url is not healthy" >&2
    exit 2
  fi
done

failures=0
rows=()

# fetch <base> <method> <path> [body] -> prints "<status>\n<normalised body>"
fetch() {
  local base="$1" method="$2" path="$3" body="${4:-}" filter="${5:-.}"
  local out status payload
  if [[ "$method" == "POST" ]]; then
    out="$(curl -s -o /dev/stdout -w '\n%{http_code}' -X POST -H 'content-type: application/json' \
      -d "$body" "$base$path")"
  else
    out="$(curl -s -o /dev/stdout -w '\n%{http_code}' "$base$path")"
  fi
  status="${out##*$'\n'}"
  payload="${out%$'\n'*}"
  if [[ -z "$payload" ]]; then
    printf '%s\n<empty>\n' "$status"
  elif printf '%s' "$payload" | jq -e . >/dev/null 2>&1; then
    printf '%s\n%s\n' "$status" "$(printf '%s' "$payload" | jq -S "$filter")"
  else
    printf '%s\n%s\n' "$status" "$payload"
  fi
}

# compare <label> <method> <path> [body] [jq filter applied to both responses]
compare() {
  local label="$1" method="$2" path="$3" body="${4:-}" filter="${5:-.}"
  local java rust java_status rust_status verdict
  java="$(fetch "$JAVA_URL" "$method" "$path" "$body" "$filter")"
  rust="$(fetch "$RUST_URL" "$method" "$path" "$body" "$filter")"
  java_status="${java%%$'\n'*}"
  rust_status="${rust%%$'\n'*}"
  if [[ "$java" == "$rust" ]]; then
    verdict="identical"
  else
    verdict="DIFFERENT"
    failures=$((failures + 1))
    if [[ "${VERBOSE:-0}" == "1" ]]; then
      echo "--- $label"
      diff <(printf '%s\n' "$java") <(printf '%s\n' "$rust") || true
    fi
  fi
  rows+=("$(printf '%-44s | %-4s | %-4s | %s' "$label" "$java_status" "$rust_status" "$verdict")")
}

# Error responses: timestamp and the host part of `type` legitimately differ.
PROBLEM_FILTER='del(.timestamp) | if .type then .type |= sub("^http://[^/]+"; "") else . end'
# Created owner: the generated id differs between two inserts.
CREATED_FILTER='del(.id)'

compare "GET /actuator/health"                 GET "/actuator/health"
compare "GET /api/vets"                        GET "/api/vets"
for id in 1 2 3 4 5 6 999; do
  compare "GET /api/vets/$id"                  GET "/api/vets/$id"
done
compare "GET /api/specialties"                 GET "/api/specialties"
for id in 1 2 3 999; do
  compare "GET /api/specialties/$id"           GET "/api/specialties/$id"
done
compare "GET /api/pettypes"                    GET "/api/pettypes"
for id in 1 2 3 4 5 6 999; do
  compare "GET /api/pettypes/$id"              GET "/api/pettypes/$id"
done
compare "GET /api/owners"                      GET "/api/owners"
for name in Davis D Franklin Mc Zzz; do
  compare "GET /api/owners?lastName=$name"     GET "/api/owners?lastName=$name"
done
for id in 1 2 3 4 5 6 7 8 9 10 999; do
  compare "GET /api/owners/$id"                GET "/api/owners/$id"
done
for id in 1 2 3 4 5 6 7 8 9 10 11 12 13 999; do
  compare "GET /api/pets/$id"                  GET "/api/pets/$id"
done
compare "GET /api/owners/abc (500)"            GET "/api/owners/abc" "" "$PROBLEM_FILTER"
compare "GET /api/nope (500)"                  GET "/api/nope" "" "$PROBLEM_FILTER"
compare "POST /api/owners (201)"               POST "/api/owners" \
  '{"firstName":"Parity","lastName":"Check","address":"1 Main St.","city":"Madison","telephone":"6085550000"}' \
  "$CREATED_FILTER"
compare "POST /api/owners invalid telephone (400)" POST "/api/owners" \
  '{"firstName":"Parity","lastName":"Check","address":"1 Main St.","city":"Madison","telephone":"not-a-number"}' \
  "$PROBLEM_FILTER"
compare "POST /api/owners empty body {} (400)" POST "/api/owners" '{}' \
  "$PROBLEM_FILTER | .schemaValidationErrors |= sort_by(.field)"
compare "POST /api/owners numeric field (400)"  POST "/api/owners" '{"firstName":1}' \
  "$PROBLEM_FILTER | .schemaValidationErrors |= sort_by(.field)"
compare "POST /api/owners malformed JSON (500)" POST "/api/owners" '{not json' "$PROBLEM_FILTER"

printf '%-44s | %-4s | %-4s | %s\n' "endpoint" "java" "rust" "result"
printf '%s\n' "---------------------------------------------|------|------|----------"
printf '%s\n' "${rows[@]}"
echo
if [[ "$failures" -eq 0 ]]; then
  echo "PARITY OK: ${#rows[@]} endpoints identical"
else
  echo "PARITY FAILED: $failures of ${#rows[@]} endpoints differ (VERBOSE=1 shows diffs)"
  exit 1
fi
