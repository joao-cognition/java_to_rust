# Amadeus · Java → Rust modernization demo

A Spring Boot **Travel Shopping & Reservation API** (airports, airlines, flight schedules, fare search, travelers,
bookings) and `j2r`, a terminal console that compares it with a Rust port of the same contract and measures what the
same traffic costs in CPU before and after.

The modernization workflow this repo is built for:

1. understand the service (DeepWiki / Ask Devin),
2. have Devin port the CPU-hot read path — fare search and the lookups around it — to Rust (Axum) against the **same
   PostgreSQL**, same JSON contract, on port 9977,
3. prove it in the terminal: `j2r api` shows the two responses are identical, `j2r bench` shows cores freed.

The Rust port lives in `rust-travel/` and is filled in endpoint by endpoint during the migration.

| Part | Where | Port |
|---|---|---|
| Java service — Spring Boot 4, Spring Data JPA / Hibernate, PostgreSQL 16 or in-memory H2 | `src/` | 9966 (`/travel/api`) |
| Rust port of the read hot path — produced by Devin during the migration | `rust-travel/` | 9977 (`/travel/api`) |
| **`j2r` CLI** — raw API comparison, benchmark with real CPU numbers, live watch | `cli/` | — |

## The Java service

```sh
./mvnw spring-boot:run                                        # H2 in-memory, http://localhost:9966/travel/api
# or, to share a database with the Rust port:
docker compose --profile postgres up -d
./mvnw spring-boot:run -Dspring-boot.run.profiles=postgres
```

Swagger UI: http://localhost:9966/travel/swagger-ui.html · health: http://localhost:9966/travel/actuator/health

Reference data (airports, airlines, tax rules, travelers) comes from `data.sql`; flights, fares and bookings are
generated once at startup with a fixed seed — 30 days of schedules on ~16 city pairs starting tomorrow, several fares
per flight, ~200 confirmed bookings (the first one is `QX7K2M`). The generator only runs when the `flight` table is
empty, so a Rust service pointed at the same PostgreSQL sees exactly the same data.

| Method | Endpoint | |
|---|---|---|
| GET | `/api/airports`, `/api/airports/{iata}` | reference data |
| GET | `/api/airlines`, `/api/airlines/{iata}` | reference data |
| GET | `/api/flights?origin=NCE&destination=LHR&date=YYYY-MM-DD`, `/api/flights/{id}` | schedule |
| GET | `/api/offers?origin=NCE&destination=LHR&date=YYYY-MM-DD&adults=1&cabin=ECONOMY` | **fare search — the CPU hot path**: joins flights and fares, prices every fare (base + departure/arrival taxes, airport charges, distance-banded fuel surcharge) per passenger, filters by seats, sorts by total |
| GET | `/api/travelers/{id}`, `/api/travelers?lastName=Mar` | travelers |
| GET | `/api/bookings/{recordLocator}`, `/api/bookings?lastName=Mar` | reservations (PNR) |
| POST | `/api/bookings` `{"offerId":"123-Y","travelerIds":[1]}` | book an offer (201, decrements seats, 409 when sold out) |
| DELETE | `/api/bookings/{recordLocator}` | cancel (204) |

Errors are RFC 9457 `ProblemDetail` bodies. The read endpoints above (everything but `POST`/`DELETE`) are the
migration slice.

## The Rust port (`rust-travel/`)

Axum + Tokio + sqlx, read-only against the PostgreSQL the Java `postgres` profile uses (same `POSTGRES_URL`,
`POSTGRES_USER`, `POSTGRES_PASS` variables and defaults; `PORT` defaults to 9977). Java owns the schema and seed data,
so start Java first.

```sh
cargo run --release --manifest-path rust-travel/Cargo.toml   # http://localhost:9977/travel/api
curl -s localhost:9977/travel/api/_smoke                      # row counts read from the shared database
cd rust-travel && cargo test                                  # integration tests use DATABASE_URL, skip without a DB
```

## `j2r` — the terminal console (`cli/`)

One static Rust binary, no UI framework. Everything it prints is measured on the spot.

```sh
cargo install --path cli            # or: cargo run --manifest-path cli/Cargo.toml -- <cmd>
j2r --help
```

| Command | What you see |
|---|---|
| `j2r api bookings/QX7K2M -n 50` | the same `GET` sent to Java (`:9966`) and Rust (`:9977`): status, latency, bytes and the raw JSON of both side by side, a `diff:` line (`identical`, key order ignored, or the differing JSON paths), then p50/p99/min of 50 sequential requests per side |
| `j2r api "offers?origin=NCE&destination=LHR&date=$(date -d tomorrow +%F)"` | same, on the hot path |
| `j2r bench --rate 300 --duration 30 --peak 10` | fixed-rate load against each service in turn (2 s warm-up, then one line per second: `cpu=… cores rps=… p50=… p99=… rss=…`), a closed-loop peak phase (64 in flight), then sparklines per metric and a `summary` table with deltas ending in `cpu freed at 300 req/s: X cores (Y% of the java footprint)`; writes `bench/results/{timeseries.json,summary.json,run.log}` |
| `j2r watch` | one line per second with CPU cores, RSS and probe latency of both services — leave it running while you flip traffic |
| `j2r show bench/results` | re-render a saved run |

The bench mix is 50 % fare search (`offers`, tomorrow, NCE→LHR / CDG→JFK), the rest booking and traveler lookups,
flight schedule and reference lists — see `MIX` in `cli/src/bench.rs`.

CPU and RSS are read from the **service process**, not from the load generator: `/proc/<pid>/stat` for a native
process (found from the listening port), or the container's cgroup (`cpu.stat`, `memory.current`) when the port is
published by Docker. For a remote host pass `--java http://host:9966/travel/api --java-pid <pid>` (or
`--java-container <name>`), same for `--rust`. Everything in `bench/results/` is produced by this tool; nothing is
typed in.

Until a Rust service answers on `:9977`, `j2r api` shows the rust column as `down` and `j2r bench` prints the Java
baseline with rust `n/a` — that is the "before".

## Licence

Apache 2.0, see `LICENSE.txt`. Airport, airline and tax figures are illustrative demo data, not a tariff source.
