#!/usr/bin/env bash
# Characterization / parity harness for the Java → Rust port.
#
# Sends the same requests to the Java service and to the Rust service (both on the same
# PostgreSQL) and diffs status, Location / Content-Type headers and the JSON body
# (key order normalised with `jq -S`). Every OpenAPI endpoint is covered, including 404s,
# validation errors and the 500 problems Java returns for bad input.
#
# Write cases reset the database to the seed (`src/main/resources/db/postgres/data.sql`)
# before each service runs them, then read back the affected resource, so generated ids and
# side effects (cascades, merges) are compared too.
#
#   bench/parity.sh                 # live diff Java vs Rust
#   RECORD=1 bench/parity.sh        # capture Java's responses into bench/characterization/
#   GOLDEN=1 bench/parity.sh        # diff Rust against the captured Java responses (no Java needed)
#   VERBOSE=1 bench/parity.sh       # print diffs
#   ONLY=owners bench/parity.sh     # only cases whose label contains "owners"
#
# Env: JAVA_URL, RUST_URL, RESET_DB (command that reseeds the database).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JAVA_URL="${JAVA_URL:-http://localhost:9966/petclinic}"
RUST_URL="${RUST_URL:-http://localhost:9977/petclinic}"
GOLDEN_DIR="${GOLDEN_DIR:-$ROOT/bench/characterization}"
RECORD="${RECORD:-0}"
GOLDEN="${GOLDEN:-0}"
ONLY="${ONLY:-}"
SEED_SQL="$ROOT/src/main/resources/db/postgres/data.sql"
TRUNCATE_SQL="TRUNCATE visits, pets, owners, types, vet_specialties, vets, specialties, roles, users RESTART IDENTITY CASCADE;"
RESET_DB="${RESET_DB:-docker compose -f $ROOT/docker-compose.yml --profile postgres exec -T postgres psql -q -v ON_ERROR_STOP=1 -U petclinic -d petclinic}"

for tool in curl jq; do
  command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 2; }
done

services=()
[[ "$GOLDEN" == "1" ]] || services+=("$JAVA_URL")
[[ "$RECORD" == "1" ]] || services+=("$RUST_URL")
for url in "${services[@]}"; do
  if ! curl -fs "$url/actuator/health" >/dev/null; then
    echo "service at $url is not healthy" >&2
    exit 2
  fi
done

reset_db() {
  { echo "$TRUNCATE_SQL"; cat "$SEED_SQL"; } | $RESET_DB >/dev/null \
    || { echo "database reset failed ($RESET_DB)" >&2; exit 2; }
}

failures=0
total=0
rows=()

# Volatile parts of a response: the problem timestamp, the host in `type`, and the order of
# Bean Validation errors (Hibernate reports them in no particular order).
NORMALISE='
  if type == "object" and has("schemaValidationErrors") then
    del(.timestamp)
    | .type |= (if type == "string" then sub("^https?://[^/]+"; "") else . end)
    | .schemaValidationErrors |= sort_by(.field, .defaultMessage)
  else . end'
# Java returns `findAll` / `findByLastName("")` in heap order: compare as sets.
SORT_BY_ID='sort_by(.id)'

# request <base> <method> <path> [body] [filter] -> "<status> <location> <content-type>\n<body>"
request() {
  local base="$1" method="$2" path="$3" body="${4:-}" filter="${5:-.}"
  local headers payload status location ctype
  headers="$(mktemp)"
  if [[ -n "$body" ]]; then
    payload="$(curl -s -D "$headers" -X "$method" -H 'content-type: application/json' \
      --data-binary "$body" "$base$path")"
  else
    payload="$(curl -s -D "$headers" -X "$method" "$base$path")"
  fi
  status="$(awk 'toupper($1) ~ /^HTTP/ {s=$2} END {print s+0}' "$headers")"
  location="$(awk 'tolower($1) == "location:" {print $2}' "$headers" | tr -d '\r')"
  ctype="$(awk 'tolower($1) == "content-type:" {print $2}' "$headers" | tr -d '\r;')"
  rm -f "$headers"
  printf '%s location=%s type=%s\n' "$status" "${location:--}" "${ctype:--}"
  if [[ -z "$payload" ]]; then
    echo "<empty>"
  elif printf '%s' "$payload" | jq -e . >/dev/null 2>&1; then
    printf '%s' "$payload" | jq -S "$NORMALISE | $filter"
  else
    printf '%s\n' "$payload"
  fi
}

# run_case <base> <write?> <method> <path> <body> <filter> <verify path>
run_case() {
  local base="$1" write="$2" method="$3" path="$4" body="$5" filter="$6" verify="$7"
  [[ "$write" == "1" ]] && reset_db
  request "$base" "$method" "$path" "$body" "$filter"
  if [[ -n "$verify" ]]; then
    echo "--- then GET $verify"
    request "$base" GET "$verify"
  fi
}

slug() { printf '%s' "$1" | tr -cs 'A-Za-z0-9' '_' | sed 's/^_*//; s/_*$//'; }

# check <write?> <label> <method> <path> [body] [filter] [verify path]
check() {
  local write="$1" label="$2" method="$3" path="$4" body="${5:-}" filter="${6:-.}" verify="${7:-}"
  [[ -z "$ONLY" || "$label" == *"$ONLY"* ]] || return 0
  local java rust java_status rust_status verdict file
  file="$GOLDEN_DIR/$(slug "$label").txt"
  total=$((total + 1))
  if [[ "$GOLDEN" == "1" ]]; then
    java="$(cat "$file" 2>/dev/null || echo "000 missing golden file")"
  else
    java="$(run_case "$JAVA_URL" "$write" "$method" "$path" "$body" "$filter" "$verify")"
  fi
  if [[ "$RECORD" == "1" ]]; then
    mkdir -p "$GOLDEN_DIR"
    printf '%s\n' "$java" >"$file"
    rows+=("$(printf '%-58s | %-4s | %s' "$label" "${java%% *}" "recorded")")
    return 0
  fi
  rust="$(run_case "$RUST_URL" "$write" "$method" "$path" "$body" "$filter" "$verify")"
  java_status="${java%% *}"
  rust_status="${rust%% *}"
  if [[ "$java_status" == "000" || "$rust_status" == "000" ]]; then
    verdict="UNREACHABLE"
    failures=$((failures + 1))
  elif [[ "$java" == "$rust" ]]; then
    verdict="identical"
  else
    verdict="DIFFERENT"
    failures=$((failures + 1))
    if [[ "${VERBOSE:-0}" == "1" ]]; then
      echo "--- $label"
      diff <(printf '%s\n' "$java") <(printf '%s\n' "$rust") || true
    fi
  fi
  rows+=("$(printf '%-58s | %-4s | %-4s | %s' "$label" "$java_status" "$rust_status" "$verdict")")
}

read_case() { check 0 "$@"; }
write_case() { check 1 "$@"; }

OWNER='{"firstName":"Parity","lastName":"Check","address":"1 Main St.","city":"Madison","telephone":"6085550000"}'
PET='{"name":"Parity","birthDate":"2020-01-01","type":{"id":2,"name":"dog"}}'
VISIT='{"date":"2024-01-01","description":"parity visit"}'

[[ "$GOLDEN" == "1" ]] || reset_db

# ---- reads (benchmark mix first) ---------------------------------------------------------
for id in 1 2 3 4 5 6 7 8 9 10 999; do
  read_case "GET /api/owners/$id"                 GET "/api/owners/$id"
done
read_case "GET /api/owners"                       GET "/api/owners"
for name in Davis D Franklin Mc Zzz; do
  read_case "GET /api/owners?lastName=$name"      GET "/api/owners?lastName=$name"
done
read_case "GET /api/owners?lastName= (all)"       GET "/api/owners?lastName=" "" "$SORT_BY_ID"
read_case "GET /api/vets"                         GET "/api/vets"
for id in 1 2 3 4 5 6 999; do
  read_case "GET /api/vets/$id"                   GET "/api/vets/$id"
done
read_case "GET /api/specialties"                  GET "/api/specialties"
for id in 1 2 3 999; do
  read_case "GET /api/specialties/$id"            GET "/api/specialties/$id"
done
read_case "GET /api/pettypes"                     GET "/api/pettypes"
for id in 1 2 3 4 5 6 999; do
  read_case "GET /api/pettypes/$id"               GET "/api/pettypes/$id"
done
read_case "GET /api/pets"                         GET "/api/pets" "" "$SORT_BY_ID"
for id in 1 2 3 4 5 6 7 8 9 10 11 12 13 999; do
  read_case "GET /api/pets/$id"                   GET "/api/pets/$id"
done
read_case "GET /api/visits"                       GET "/api/visits" "" "$SORT_BY_ID"
for id in 1 2 3 4 999; do
  read_case "GET /api/visits/$id"                 GET "/api/visits/$id"
done
read_case "GET /api/owners/6/pets/7"              GET "/api/owners/6/pets/7"
read_case "GET /api/owners/1/pets/2 (other owner)" GET "/api/owners/1/pets/2"
read_case "GET /api/owners/999/pets/1"            GET "/api/owners/999/pets/1"
for q in "" "?page=0&size=3" "?page=1&size=5" "?page=3&size=4" "?page=99" "?lastName=Da" \
         "?lastName=Zzz" "?size=100" "?page=-1" "?size=0" "?size=101" "?size=abc"; do
  read_case "GET /api/v2/owners$q"                GET "/api/v2/owners$q"
  read_case "GET /api/v2/pets$q"                  GET "/api/v2/pets$q"
done
read_case "GET /actuator/health"                  GET "/actuator/health"

# ---- errors common to every endpoint -----------------------------------------------------
read_case "GET /api/owners/abc"                   GET "/api/owners/abc"
read_case "GET /api/owners/1/pets/abc"            GET "/api/owners/1/pets/abc"
read_case "GET /api/pets/abc"                     GET "/api/pets/abc"
read_case "GET /api/nope"                         GET "/api/nope"
read_case "GET /api/oops"                         GET "/api/oops"
read_case "PATCH /api/owners/1"                   PATCH "/api/owners/1"
read_case "GET /api/users"                        GET "/api/users"
read_case "DELETE /api/owners"                    DELETE "/api/owners"

# ---- owners ------------------------------------------------------------------------------
write_case "POST /api/owners"                     POST "/api/owners" "$OWNER" . "/api/owners/11"
write_case "POST /api/owners invalid telephone"   POST "/api/owners" "${OWNER/6085550000/not-a-number}"
write_case "POST /api/owners short telephone"     POST "/api/owners" "${OWNER/6085550000/123}"
write_case "POST /api/owners empty body"          POST "/api/owners" '{}'
write_case "POST /api/owners numeric field"       POST "/api/owners" '{"firstName":1}'
write_case "POST /api/owners malformed JSON"      POST "/api/owners" '{not json'
write_case "POST /api/owners bad names"           POST "/api/owners" \
  '{"firstName":"R2D2","lastName":"","address":"x","city":"y","telephone":"6085550000"}'
write_case "PUT /api/owners/1"                    PUT "/api/owners/1" "${OWNER/Parity/Georgina}" . "/api/owners/1"
write_case "PUT /api/owners/999"                  PUT "/api/owners/999" "$OWNER"
write_case "PUT /api/owners/1 invalid"            PUT "/api/owners/1" '{"firstName":"G"}'
write_case "DELETE /api/owners/3 (cascade)"       DELETE "/api/owners/3" "" . "/api/pets"
write_case "DELETE /api/owners/999"               DELETE "/api/owners/999"
write_case "DELETE /api/owners/abc"               DELETE "/api/owners/abc"

# ---- owner pets / visits -----------------------------------------------------------------
write_case "POST /api/owners/1/pets"              POST "/api/owners/1/pets" "$PET" . "/api/owners/1"
write_case "POST /api/owners/999/pets"            POST "/api/owners/999/pets" "$PET"
write_case "POST /api/owners/1/pets unknown type" POST "/api/owners/1/pets" \
  '{"name":"Parity","birthDate":"2020-01-01","type":{"id":99,"name":"x"}}'
write_case "POST /api/owners/1/pets future birth" POST "/api/owners/1/pets" "${PET/2020-01-01/2099-01-01}"
write_case "POST /api/owners/1/pets too old"      POST "/api/owners/1/pets" "${PET/2020-01-01/1960-01-01}"
write_case "POST /api/owners/1/pets invalid"      POST "/api/owners/1/pets" '{"name":"","type":{"id":-1}}'
write_case "POST /api/owners/1/pets bad date"     POST "/api/owners/1/pets" "${PET/2020-01-01/01-01-2020}"
write_case "PUT /api/owners/1/pets/1"             PUT "/api/owners/1/pets/1" "${PET/Parity/Leonardo}" . "/api/pets/1"
write_case "PUT /api/owners/1/pets/2 (other owner)" PUT "/api/owners/1/pets/2" "$PET" . "/api/pets/2"
write_case "PUT /api/owners/999/pets/1"           PUT "/api/owners/999/pets/1" "$PET"
write_case "PUT /api/owners/1/pets/999"           PUT "/api/owners/1/pets/999" "$PET"
write_case "PUT /api/owners/1/pets/1 invalid"     PUT "/api/owners/1/pets/1" '{"name":"Leo"}'
write_case "POST /api/owners/6/pets/7/visits"     POST "/api/owners/6/pets/7/visits" "$VISIT" . "/api/pets/7"
write_case "POST /api/owners/1/pets/999/visits"   POST "/api/owners/1/pets/999/visits" "$VISIT"
write_case "POST /api/owners/1/pets/1/visits invalid" POST "/api/owners/1/pets/1/visits" '{"description":""}'
write_case "POST /api/owners/1/pets/1/visits bad date" POST "/api/owners/1/pets/1/visits" \
  '{"date":"bad","description":"x"}'

# ---- pets --------------------------------------------------------------------------------
write_case "PUT /api/pets/1"                      PUT "/api/pets/1" "${PET/Parity/Leonardo}" . "/api/pets/1"
write_case "PUT /api/pets/1 full dto"             PUT "/api/pets/1" \
  '{"id":1,"name":"Leo","birthDate":"2010-09-07","type":{"id":1,"name":"cat"},"ownerId":1,"visits":[]}' . "/api/pets/1"
write_case "PUT /api/pets/1 unknown type"         PUT "/api/pets/1" \
  '{"name":"Leo","birthDate":"2010-09-07","type":{"id":99,"name":"x"}}'
write_case "PUT /api/pets/1 invalid"              PUT "/api/pets/1" \
  '{"id":-1,"name":"","ownerId":-2,"visits":[{"description":""}]}'
write_case "PUT /api/pets/999"                    PUT "/api/pets/999" "$PET"
write_case "DELETE /api/pets/7 (with visits)"     DELETE "/api/pets/7" "" . "/api/visits"
write_case "DELETE /api/pets/999"                 DELETE "/api/pets/999"

# ---- visits ------------------------------------------------------------------------------
write_case "POST /api/visits"                     POST "/api/visits" \
  '{"date":"2024-02-02","description":"parity","petId":1}' . "/api/pets/1"
write_case "POST /api/visits unknown pet"         POST "/api/visits" \
  '{"date":"2024-02-02","description":"parity","petId":999}'
write_case "POST /api/visits existing id (merge)" POST "/api/visits" \
  '{"id":1,"date":"2011-01-01","description":"merged","petId":7}' . "/api/visits/1"
write_case "POST /api/visits stale id"            POST "/api/visits" \
  '{"id":99,"date":"2011-01-01","description":"stale","petId":7}'
write_case "POST /api/visits invalid"             POST "/api/visits" '{"id":-1,"description":""}'
while read -r name pet_id; do
  write_case "POST /api/visits petId $name (Jackson coercion)" POST "/api/visits" \
    "{\"date\":\"2024-02-02\",\"description\":\"coerce\",\"petId\":$pet_id}"
done <<'CASES'
string "1"
padded-string " 2"
plus-string "+1"
float 1.9
exponent 1e0
negative-fraction -0.5
empty-string ""
negative-string "-1"
fraction-string "1.5"
hex-string "0x1"
boolean true
overflow 3000000000
array [1]
CASES
write_case "POST /api/owners/1/pets type.id as string" POST "/api/owners/1/pets" \
  '{"name":"Drax","birthDate":"2020-01-01","type":{"id":"1","name":"cat"}}' . "/api/owners/1"
write_case "PUT /api/visits/1"                    PUT "/api/visits/1" '{"date":"2013-01-01","description":"booster"}' . "/api/visits/1"
write_case "PUT /api/visits/999"                  PUT "/api/visits/999" "$VISIT"
write_case "PUT /api/visits/1 invalid"            PUT "/api/visits/1" '{}'
write_case "DELETE /api/visits/1"                 DELETE "/api/visits/1" "" . "/api/visits"
write_case "DELETE /api/visits/999"               DELETE "/api/visits/999"

# ---- vets --------------------------------------------------------------------------------
write_case "POST /api/vets"                       POST "/api/vets" \
  '{"firstName":"Parity","lastName":"Vet","specialties":[{"name":"radiology"},{"name":"surgery"},{"name":"unknown"}]}' . "/api/vets"
write_case "POST /api/vets no specialties"        POST "/api/vets" '{"firstName":"Parity","lastName":"Vet","specialties":[]}'
write_case "POST /api/vets existing id (merge)"   POST "/api/vets" \
  '{"id":2,"firstName":"Helena","lastName":"Leary","specialties":[{"name":"dentistry"}]}' . "/api/vets/2"
write_case "POST /api/vets stale id"              POST "/api/vets" '{"id":99,"firstName":"A","lastName":"B","specialties":[]}'
write_case "POST /api/vets invalid"               POST "/api/vets" \
  '{"firstName":"Vet2","lastName":"","specialties":[{"id":-1,"name":""}]}'
write_case "POST /api/vets missing specialties"   POST "/api/vets" '{"firstName":"Parity","lastName":"Vet"}'
write_case "PUT /api/vets/1"                      PUT "/api/vets/1" \
  '{"firstName":"Jim","lastName":"Carter","specialties":[{"name":"surgery"}]}' . "/api/vets/1"
write_case "PUT /api/vets/3 clear specialties"    PUT "/api/vets/3" \
  '{"firstName":"Linda","lastName":"Douglas","specialties":[]}' . "/api/vets/3"
write_case "PUT /api/vets/999"                    PUT "/api/vets/999" '{"firstName":"A","lastName":"B","specialties":[]}'
write_case "PUT /api/vets/1 invalid"              PUT "/api/vets/1" '{}'
write_case "DELETE /api/vets/3"                   DELETE "/api/vets/3" "" . "/api/vets"
write_case "DELETE /api/vets/999"                 DELETE "/api/vets/999"

# ---- pet types ---------------------------------------------------------------------------
write_case "POST /api/pettypes"                   POST "/api/pettypes" '{"name":"parrot"}' . "/api/pettypes"
write_case "POST /api/pettypes ignores id"        POST "/api/pettypes" '{"id":1,"name":"parrot"}' . "/api/pettypes"
write_case "POST /api/pettypes invalid"           POST "/api/pettypes" '{"name":""}'
write_case "PUT /api/pettypes/1"                  PUT "/api/pettypes/1" '{"id":1,"name":"kitten"}' . "/api/pettypes/1"
write_case "PUT /api/pettypes/1 missing id"       PUT "/api/pettypes/1" '{"name":"kitten"}'
write_case "PUT /api/pettypes/999"                PUT "/api/pettypes/999" '{"id":999,"name":"x"}'
write_case "DELETE /api/pettypes/1 (cascade)"     DELETE "/api/pettypes/1" "" . "/api/pets"
write_case "DELETE /api/pettypes/999"             DELETE "/api/pettypes/999"

# ---- specialties -------------------------------------------------------------------------
write_case "POST /api/specialties"                POST "/api/specialties" '{"name":"oncology"}' . "/api/specialties"
write_case "POST /api/specialties existing id (merge)" POST "/api/specialties" '{"id":1,"name":"imaging"}' . "/api/vets"
write_case "POST /api/specialties stale id"       POST "/api/specialties" '{"id":50,"name":"x"}'
write_case "POST /api/specialties invalid"        POST "/api/specialties" '{"id":-1}'
write_case "PUT /api/specialties/2"               PUT "/api/specialties/2" '{"name":"general surgery"}' . "/api/vets"
write_case "PUT /api/specialties/999"             PUT "/api/specialties/999" '{"name":"x"}'
write_case "PUT /api/specialties/2 invalid"       PUT "/api/specialties/2" '{"name":""}'
write_case "DELETE /api/specialties/1"            DELETE "/api/specialties/1" "" . "/api/vets"
write_case "DELETE /api/specialties/999"          DELETE "/api/specialties/999"

# ---- users -------------------------------------------------------------------------------
write_case "POST /api/users"                      POST "/api/users" \
  '{"username":"parity","password":"secret","enabled":true,"roles":[{"name":"OWNER_ADMIN"}]}'
write_case "POST /api/users prefixed role"        POST "/api/users" \
  '{"username":"parity","password":"secret","enabled":true,"roles":[{"name":"ROLE_VET_ADMIN"}]}'
write_case "POST /api/users no roles"             POST "/api/users" '{"username":"parity","password":"secret"}'
write_case "POST /api/users invalid"              POST "/api/users" '{"password":"","roles":[{"name":""}]}'
write_case "POST /api/users missing enabled"      POST "/api/users" \
  '{"username":"parity","password":"secret","roles":[{"name":"ADMIN"}]}'

[[ "$GOLDEN" == "1" || "$RECORD" == "1" ]] || reset_db

if [[ "$RECORD" == "1" ]]; then
  printf '%s\n' "${rows[@]}"
  echo
  echo "RECORDED: $total Java responses in ${GOLDEN_DIR#"$ROOT"/}"
  exit 0
fi
printf '%-58s | %-4s | %-4s | %s\n' "endpoint" "java" "rust" "result"
printf '%s\n' "-----------------------------------------------------------|------|------|----------"
printf '%s\n' "${rows[@]}"
echo
if [[ "$failures" -eq 0 ]]; then
  echo "PARITY OK: $total cases identical"
else
  echo "PARITY FAILED: $failures of $total cases differ (VERBOSE=1 shows diffs)"
  exit 1
fi
