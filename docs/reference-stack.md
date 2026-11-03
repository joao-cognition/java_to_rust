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

## 4. Newman non-regression suite

```sh
./scripts/reset-db.sh                                        # the collection writes data
BASE_URL=http://localhost:9966/petclinic ./postman-tests.sh  # Java (default when BASE_URL is unset)
BASE_URL=http://localhost:9977/petclinic ./postman-tests.sh  # Rust port
```

The base URL can also be passed as the first argument. Needs `node`/`npx` and `jq`; the HTML report goes to
`src/test/postman/reports/report.html`.

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
