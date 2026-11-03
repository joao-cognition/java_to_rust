# bench

## Parity — `parity.sh`

Checks that the Rust service (`rust-petclinic/`, port 9977) answers byte-for-byte like the Java
service (port 9966) for every ported endpoint when both run against the **same PostgreSQL**.

```bash
docker compose --profile postgres up -d
./rust-petclinic/scripts/init-db.sh

# Java on :9966 — either this repo's jar ...
./mvnw -q -DskipTests package
java -jar target/spring-petclinic-rest-*.jar --spring.profiles.active=postgres,spring-data-jpa
# ... or the upstream image on the same database
docker run --rm --network host -e SPRING_PROFILES_ACTIVE=postgres,spring-data-jpa \
  -e SPRING_DATASOURCE_URL=jdbc:postgresql://localhost:5432/petclinic \
  springcommunity/spring-petclinic-rest

# Rust on :9977
(cd rust-petclinic && cargo run --release)

bench/parity.sh                 # JAVA_URL / RUST_URL override the defaults, VERBOSE=1 prints diffs
```

The script fetches each endpoint from both services — all vets / specialties / pet types /
owners / pets by id, `?lastName=` prefix searches, the `404` paths, the `500` problem bodies
Java produces for a non-numeric id, an unknown route and unreadable JSON, and `POST /owners`
(valid, invalid, empty, numeric field) — normalises key order with `jq -S`, prints an
`endpoint → identical / different` table and exits non-zero on any difference. Error bodies are
compared after dropping `timestamp` and the host part of `type`; the created owner after
dropping its generated `id`.

Last run (Java jar from this repo, PostgreSQL 16.3): **60 / 60 identical**.

| endpoint(s)                                | cases | HTTP status Java / Rust | result    |
|--------------------------------------------|------:|-------------------------|-----------|
| `GET /actuator/health`                     |     1 | 200 / 200               | identical |
| `GET /api/vets`, `/vets/{id}`              |     8 | 200, 404                | identical |
| `GET /api/specialties`, `/{id}`            |     5 | 200, 404                | identical |
| `GET /api/pettypes`, `/{id}`               |     8 | 200, 404                | identical |
| `GET /api/owners`, `?lastName=`, `/{id}`   |    17 | 200, 404                | identical |
| `GET /api/pets/{id}`                       |    14 | 200, 404                | identical |
| `GET /api/owners/abc`, `GET /api/nope`     |     2 | 500 / 500               | identical |
| `POST /api/owners` (valid, invalid ×3, malformed) | 5 | 201, 400, 500      | identical |

## Benchmark

The Java-vs-Rust load test lives in the `cli/` crate (`j2r`):

```bash
cargo run --manifest-path cli/Cargo.toml -- bench
```

It drives the same traffic mix against both services, samples CPU and RSS of each service
process and writes `bench/results/{timeseries.json,summary.json,run.log}`.
