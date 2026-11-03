# Running the reference stack

The Java service on the compose PostgreSQL is the reference every parity check and Rust test compares against.
All commands run from the repo root.

## 1. PostgreSQL (compose)

```sh
docker compose --profile postgres up -d     # postgres:16.3 on :5432, user/password/db: petclinic
```

If Docker Hub answers `429 Too Many Requests`, pull the same image from Google's Docker Hub mirror first:

```sh
docker pull mirror.gcr.io/library/postgres:16.3 && docker tag mirror.gcr.io/library/postgres:16.3 postgres:16.3
```

## 2. Java service (:9966, `/petclinic/api`)

```sh
./mvnw spring-boot:run -Dspring-boot.run.profiles=postgres,spring-data-jpa
curl -s http://localhost:9966/petclinic/api/owners/1    # George Franklin, from Postgres
```

`spring.sql.init.mode=always` applies `src/main/resources/db/postgres/schema.sql` + `data.sql` on every boot. Both
files are idempotent (`CREATE TABLE IF NOT EXISTS`, `INSERT ... WHERE NOT EXISTS`), so a restart does **not** undo
writes; use the reset below.

If Maven Central answers HTTP 429 (or the system Maven is older than the enforcer's 3.9.9), go through the GCS mirror:

```sh
cat > ~/.m2/settings.xml <<'XML'
<settings><mirrors><mirror>
  <id>gcs-central</id><mirrorOf>central</mirrorOf>
  <url>https://maven-central-eu.storage-download.googleapis.com/maven2</url>
</mirror></mirrors></settings>
XML
MVNW_REPOURL=https://maven-central-eu.storage-download.googleapis.com/maven2 \
  ./mvnw spring-boot:run -Dspring-boot.run.profiles=postgres,spring-data-jpa
```

## 3. `j2r` console

```sh
cargo install --path cli
j2r api owners/1     # java 200 with the JSON; rust "down" until rust-petclinic/ exists
j2r watch            # one line per second: java cpu/rss/probe up, rust down
```

### Parity harness (`j2r parity`, run from the repo root)

Replays a fixed, ordered catalogue (125 cases: every `openapi.yml` operation with its happy path and the 404 / 400
Bean Validation / 500 responses Java really produces) and diffs status, `Content-Type`, `Location` and the JSON body
(key order ignored; problem-detail `timestamp`, the host in `type` and the order of `schemaValidationErrors` are
normalized). `./scripts/reset-db.sh` runs before each service's pass so generated ids match. Prints the case count per
operation (gaps show as `0`) and exits non-zero on any difference.

```sh
j2r parity                                        # live Java (:9966) vs Rust (:9977)
j2r parity --golden parity/golden                 # Rust vs the committed Java responses, no JVM needed
j2r parity --record parity/golden                 # re-record the golden fixtures from Java
j2r parity --rust http://localhost:9966/petclinic/api   # Java vs Java: must be 125/125 identical
```

`parity/golden/NNN-<operationId>-<case>.json` holds one request + Java response per case, in run order.

## Reset the database to seed data

```sh
./scripts/reset-db.sh
```

Drops and recreates the `public` schema, then re-runs `schema.sql` + `data.sql` through `psql` inside the compose
container (no local `psql` needed) in one transaction, so ids restart at 1 and concurrent writes cannot interleave. Safe while the services are running (the pool reconnects to the new tables); use it
before every parity run or test that writes. The same thing as a one-liner:

```sh
{ echo 'SET client_min_messages=warning; DROP SCHEMA public CASCADE; CREATE SCHEMA public;'; cat src/main/resources/db/postgres/{schema,data}.sql; } | docker compose --profile postgres exec -T postgres psql -q -1 -v ON_ERROR_STOP=1 -U petclinic -d petclinic
```
