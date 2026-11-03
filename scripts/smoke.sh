#!/usr/bin/env bash
# HTTP smoke test against a running Travel API. Usage: scripts/smoke.sh [http://localhost:9966/travel]
set -euo pipefail
BASE="${1:-http://localhost:9966/travel}"
TOMORROW="$(date -u -d tomorrow +%F 2>/dev/null || date -u -v+1d +%F)"
fail=0

check() { # check <expected status> <method> <path> [json body]
  local expected="$1" method="$2" path="$3" body="${4:-}" status
  if [ -n "$body" ]; then
    status=$(curl -s -o /tmp/smoke-body.json -w '%{http_code}' -X "$method" -H 'Content-Type: application/json' -d "$body" "$BASE$path")
  else
    status=$(curl -s -o /tmp/smoke-body.json -w '%{http_code}' -X "$method" "$BASE$path")
  fi
  if [ "$status" = "$expected" ]; then
    printf 'ok   %s %s -> %s\n' "$method" "$path" "$status"
  else
    printf 'FAIL %s %s -> %s (expected %s)\n' "$method" "$path" "$status" "$expected"; head -c 300 /tmp/smoke-body.json; echo; fail=1
  fi
}
count() { python3 -c 'import json,sys; print(len(json.load(sys.stdin)))' < /tmp/smoke-body.json 2>/dev/null || grep -o '"offerId"' /tmp/smoke-body.json | wc -l; }

check 200 GET /actuator/health
check 200 GET /api/airports
check 200 GET /api/airports/NCE
check 404 GET /api/airports/XXX
check 200 GET /api/airlines
check 404 GET /api/airlines/ZZ
check 200 GET "/api/flights?origin=NCE&destination=LHR&date=$TOMORROW"
check 400 GET "/api/flights?origin=NCE&destination=LHR"
check 404 GET /api/flights/999999999
check 200 GET "/api/offers?origin=NCE&destination=LHR&date=$TOMORROW"
offers=$(count); [ "$offers" -ge 4 ] && echo "ok   $offers offers NCE-LHR on $TOMORROW" || { echo "FAIL only $offers offers"; fail=1; }
check 400 GET "/api/offers?origin=NCE&destination=LHR&date=$TOMORROW&adults=0"
check 400 GET "/api/offers?origin=NCE&destination=LHR&date=$TOMORROW&cabin=COACH"
check 200 GET /api/travelers/1
check 404 GET /api/travelers/999999
check 200 GET "/api/travelers?lastName=Mar"
grep -q '"id":1,' /tmp/smoke-body.json && echo "ok   traveler 1 matches lastName=Mar" || { echo "FAIL traveler 1 missing"; fail=1; }
check 200 GET /api/bookings/QX7K2M
check 404 GET /api/bookings/ZZZZZZ
check 200 GET "/api/bookings?lastName=Mar"
offer=$(python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["offerId"])' < <(curl -s "$BASE/api/offers?origin=NCE&destination=LHR&date=$TOMORROW"))
check 201 POST /api/bookings "{\"offerId\":\"$offer\",\"travelerIds\":[3,4]}"
locator=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["recordLocator"])' < /tmp/smoke-body.json)
check 404 POST /api/bookings '{"offerId":"999999999-Y","travelerIds":[3]}'
check 400 POST /api/bookings '{"offerId":"nope","travelerIds":[]}'
check 204 DELETE "/api/bookings/$locator"
check 404 DELETE /api/bookings/ZZZZZZ
check 200 GET "/api/bookings/$locator"
grep -q '"status":"CANCELLED"' /tmp/smoke-body.json && echo "ok   $locator cancelled" || { echo "FAIL $locator not cancelled"; fail=1; }

[ $fail -eq 0 ] && echo "SMOKE OK" || { echo "SMOKE FAILED"; exit 1; }
