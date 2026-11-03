#!/usr/bin/env sh
# Reset the compose PostgreSQL to the seed data in src/main/resources/db/postgres (ids restart at 1).
set -eu
cd "$(dirname "$0")/.."
db=src/main/resources/db/postgres
{
  echo 'SET client_min_messages = warning; DROP SCHEMA public CASCADE; CREATE SCHEMA public;'
  cat "$db/schema.sql" "$db/data.sql"
} | docker compose --profile postgres exec -T postgres psql -q -1 -v ON_ERROR_STOP=1 -U petclinic -d petclinic
