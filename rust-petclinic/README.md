# rust-petclinic

Rust port (Axum + sqlx, Tokio) of the **read hot path** of `spring-petclinic-rest`: the
reference-data and record-lookup endpoints that make up ~95% of the traffic in
[`bench/`](../bench), plus the `POST /owners` write path used by the same benchmark.

It serves the **same JSON contract** (`src/main/resources/openapi.yml`) under the **same
context path** (`/petclinic/api/...`) against the **same PostgreSQL schema and seed data**
(`src/main/resources/db/postgres/{schema,data}.sql`). You can point any client at either
service. The Java service listens on `9966`, the Rust one on `9977`.

## Run

```bash
# 1. PostgreSQL (user / password / db = petclinic) with the Java repo's schema and seed data
docker compose --profile postgres up -d
./rust-petclinic/scripts/init-db.sh

# 2. the Rust service
cd rust-petclinic
cargo run --release
# -> http://localhost:9977/petclinic/actuator/health  => {"status":"UP"}
```

`init-db.sh` applies `schema.sql` + `data.sql` through `docker compose exec postgres psql`
(or through a local `psql` if `DATABASE_URL` is set), so the Java app never has to run.

### Environment variables

| Variable             | Default                                                   | Purpose                                 |
|----------------------|-----------------------------------------------------------|-----------------------------------------|
| `DATABASE_URL`       | `postgres://petclinic:petclinic@localhost:5432/petclinic` | PostgreSQL connection string            |
| `PORT`               | `9977`                                                    | HTTP listen port                        |
| `DB_MAX_CONNECTIONS` | `16`                                                      | sqlx pool size                          |
| `RUST_LOG`           | `info`                                                    | `tracing` filter (`tower_http=debug` logs every request) |

## Endpoints

All paths are prefixed with `/petclinic`.

| Method | Path                          | Notes                                                        |
|--------|-------------------------------|--------------------------------------------------------------|
| GET    | `/actuator/health`            | `{"groups":["liveness","readiness"],"status":"UP"}`, same as Actuator; `503` + `"DOWN"` when PostgreSQL does not answer |
| GET    | `/api/vets`                   | vets with sorted `specialties`, 1 query                      |
| GET    | `/api/vets/{vetId}`           |                                                              |
| GET    | `/api/specialties`            |                                                              |
| GET    | `/api/specialties/{id}`       |                                                              |
| GET    | `/api/pettypes`               |                                                              |
| GET    | `/api/pettypes/{id}`          |                                                              |
| GET    | `/api/owners`                 | optional `?lastName=<prefix>` (prefix match like Java), owners with nested `pets[].visits[]`, 1 query |
| GET    | `/api/owners/{ownerId}`       |                                                              |
| GET    | `/api/pets/{petId}`           | pet with `type` and sorted `visits`, 1 query                 |
| POST   | `/api/owners`                 | validates `OwnerFields` like Bean Validation, `201` + `Location: /api/owners/{id}` + body |

Behaviour mirrored from the Java controllers:

* unknown id (and an empty list, e.g. `?lastName=Zzz`) → `404` with an empty body;
* invalid body → `400` RFC 9457 problem (`MethodArgumentNotValidException`) with
  `schemaValidationErrors[]` carrying `field`, `rejectedValue` and `defaultMessage`, using the
  Hibernate Validator default messages; scalars are coerced to strings like Jackson does;
* unknown route, non-numeric id and unreadable JSON → `500` problem titled
  `NoResourceFoundException` / `MethodArgumentTypeMismatchException` /
  `HttpMessageNotReadableException`. That is what the Java `ExceptionControllerAdvice` really
  returns (only Bean Validation errors are mapped to `400`), and parity means reproducing it;
  the problem carries `type` (request URL), `instance` (request path) and `timestamp`;
* collections are ordered as the Java model orders them (pets by name, visits newest first,
  specialties by name).

### Query count

JPA with the eager `Owner → pets → visits` mapping issues one query per association on this
workload. Here every endpoint is one SQL statement with `LEFT JOIN`s that is folded into DTOs
in memory (`src/db.rs`); `POST /owners` is one `INSERT ... RETURNING id`.

## Tests

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                      # DTO JSON contract, validation, folding, router 404/400 paths (no DB needed)
DATABASE_URL=postgres://petclinic:petclinic@localhost:5432/petclinic \
  cargo test -- --ignored       # end-to-end against the seeded database
```

## Parity with the Java service

Start both services on the same database and run [`bench/parity.sh`](../bench/parity.sh).
It fetches every ported endpoint (several ids, the 404 / 400 paths and `POST /owners`) from
both ports, normalises key order with `jq -S`, prints an `endpoint → identical / different`
table and exits non-zero on any difference.

```bash
java -jar target/spring-petclinic-rest-*.jar --spring.profiles.active=postgres,spring-data-jpa &
(cd rust-petclinic && cargo run --release) &
bench/parity.sh
```

## Not ported (yet)

Everything outside the read hot path: `PUT`/`DELETE` on owners, all pet / visit / vet /
specialty / pet-type write endpoints, the `/users` endpoint, the paginated `/owners/page`
and `/pets/page` variants, the `/api/v2` controllers and Spring Security.
