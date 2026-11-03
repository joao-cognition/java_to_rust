# rust-petclinic

Rust (Axum + sqlx) port of the Java service, same URL layout: `http://localhost:9977/petclinic/api/...`
(Java: `:9966`). Endpoints are added as they are ported; today it serves only
`GET /petclinic/actuator/health`, with the same body, status and content type as Spring Actuator.

```sh
docker compose --profile postgres up -d     # shared PostgreSQL; the Java service owns schema.sql/data.sql
cd rust-petclinic && cargo run --release    # listens on 0.0.0.0:9977
```

| variable | default | |
|---|---|---|
| `POSTGRES_URL` | `jdbc:postgresql://localhost/petclinic` | same as the Java `postgres` profile; `postgres://…` or `host[:port]/db` also accepted |
| `POSTGRES_USER` / `POSTGRES_PASS` | `petclinic` / `petclinic` | |
| `DB_MAX_CONNECTIONS` | `72` | above the 64 requests `j2r bench --peak` keeps in flight |
| `RUST_LOG` | `info` | |

Tests (`cargo test --release`) need the compose PostgreSQL for the `UP` case.
