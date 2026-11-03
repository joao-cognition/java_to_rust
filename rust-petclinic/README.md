# rust-petclinic

Rust port (Axum + sqlx, Tokio) of **every endpoint** of `spring-petclinic-rest`
(`src/main/resources/openapi.yml`): owners, pets, visits, vets, specialties, pet types, the
paginated `/v2/owners` and `/v2/pets`, and `POST /users`. The hot read path measured by
`j2r bench` (owner with nested pets/visits, owner search, vets with specialties, reference data,
pet lookup) was ported first and is one SQL statement per request.

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
| GET    | `/api/owners`                 | optional `?lastName=<prefix>` (prefix match like Java), owners with nested `pets[].visits[]`, 1 query |
| GET    | `/api/owners/{ownerId}`       | owner with nested `pets[].visits[]`, 1 query                 |
| GET    | `/api/vets`, `/api/vets/{id}` | vets with sorted `specialties`, 1 query                      |
| GET    | `/api/specialties[/{id}]`, `/api/pettypes[/{id}]` | reference data                           |
| GET    | `/api/pets`, `/api/pets/{petId}` | pets with `type` and sorted `visits`, 1 query             |
| GET    | `/api/visits[/{id}]`          |                                                              |
| GET    | `/api/v2/owners`, `/api/v2/pets` | `page` (default 0, ≥ 0), `size` (default 20, 1–100), ordered by id; `/v2/owners` takes `lastName` |
| POST   | `/api/owners`, `/api/pettypes`, `/api/specialties`, `/api/vets`, `/api/visits` | `201` + `Location: /api/<resource>/{id}` + body |
| POST   | `/api/owners/{ownerId}/pets`, `/api/owners/{ownerId}/pets/{petId}/visits` | `201` + `Location` + body |
| GET/PUT | `/api/owners/{ownerId}/pets/{petId}` |                                                       |
| PUT    | `/api/owners/{id}`, `/api/pets/{id}`, `/api/pettypes/{id}`, `/api/specialties/{id}`, `/api/vets/{id}`, `/api/visits/{id}` | `204`, empty body |
| DELETE | `/api/owners/{id}`, `/api/pets/{id}`, `/api/pettypes/{id}`, `/api/specialties/{id}`, `/api/vets/{id}`, `/api/visits/{id}` | `204`; pet type and pet deletes cascade to pets/visits like JPA |
| POST   | `/api/users`                  | `201` (no `Location`); role names get the `ROLE_` prefix, at least one role required |

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
* foreign-key / unique violations (unknown pet type or pet id, deleting an owner that still has
  pets or a specialty still assigned to a vet, duplicate username) → `404` problem titled
  `DataIntegrityViolationException`; `POST` with an id that no longer exists → `500`
  `ObjectOptimisticLockingFailureException` (JPA `merge`), with an existing id it updates;
* entity-level constraints that only Hibernate checks (owner telephone must be exactly 10 digits)
  → `500` `ConstraintViolationException`; unsupported HTTP method → `500`
  `HttpRequestMethodNotSupportedException`;
* collections are ordered as the Java model orders them (pets by name, visits newest first,
  specialties by name).

### Query count

JPA with the eager `Owner → pets → visits` mapping issues one query per association on this
workload. Here every read is one SQL statement with `LEFT JOIN`s that is folded into DTOs
in memory (`src/db.rs`); multi-statement writes (vet specialties, cascading deletes) run in one
transaction.

## Tests

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                      # DTO JSON contract, validation, folding, router 404/400 paths (no DB needed)
DATABASE_URL=postgres://petclinic:petclinic@localhost:5432/petclinic \
  cargo test -- --ignored       # every endpoint end-to-end against the seeded database (tests/crud_integration.rs, tests/db_integration.rs)
```

## Parity with the Java service

Start both services on the same database and run [`bench/parity.sh`](../bench/parity.sh).
It sends 176 requests covering every OpenAPI endpoint (reads, `v2` paging, every write, the
404 / 400 / 500 problem paths) to both ports, resetting the database to the seed data before
each write so both sides start from the same state. Bodies are compared with `jq -S` (key
order ignored) after masking `timestamp` and the host in problem `type`; status, `Location`
and `Content-Type` are compared too. It exits non-zero on any difference.

```bash
java -jar target/spring-petclinic-rest-*.jar --spring.profiles.active=postgres,spring-data-jpa &
(cd rust-petclinic && cargo run --release) &
bench/parity.sh                 # Java vs Rust, live
RECORD=1 bench/parity.sh        # re-record the Java characterization in bench/characterization/
GOLDEN=1 bench/parity.sh        # Rust vs the recorded Java responses (Java not needed)
```

Not ported: Spring Security (disabled by default in the Java service, `petclinic.security.enable=false`).
