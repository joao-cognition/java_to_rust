//! Integration tests for every write endpoint and the remaining reads, against a real
//! PostgreSQL seeded by `scripts/init-db.sh`. Each test creates the rows it mutates, so the
//! seed data stays intact. Run with
//! `DATABASE_URL=postgres://petclinic:petclinic@localhost:5432/petclinic cargo test -- --ignored`.

use axum::body::Body;
use axum::http::{header, HeaderMap, Method, Request, StatusCode};
use http_body_util::BodyExt;
use rust_petclinic::{app, db};
use serde_json::{json, Value};
use tower::ServiceExt;

const API: &str = "/petclinic/api";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
}

async fn test_app() -> axum::Router {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must point at a seeded petclinic database");
    app(db::connect(&url, 4).await.expect("connect"))
}

async fn send(app: &axum::Router, method: Method, path: &str, body: Option<Value>) -> Reply {
    let mut request = Request::builder()
        .method(method)
        .uri(format!("{API}{path}"));
    let body = match body {
        Some(json) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    Reply {
        status,
        headers,
        json,
    }
}

async fn get(app: &axum::Router, path: &str) -> Reply {
    send(app, Method::GET, path, None).await
}

fn location(reply: &Reply) -> &str {
    reply.headers[header::LOCATION].to_str().unwrap()
}

fn fields(reply: &Reply) -> Vec<String> {
    let mut fields: Vec<String> = reply.json["schemaValidationErrors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap().to_string())
        .collect();
    fields.sort();
    fields.dedup();
    fields
}

fn owner_body(first_name: &str) -> Value {
    json!({"firstName": first_name, "lastName": "Tester", "address": "1 Main St.",
           "city": "Madison", "telephone": "6085550000"})
}

async fn create_owner(app: &axum::Router) -> i64 {
    let reply = send(app, Method::POST, "/owners", Some(owner_body("Crud"))).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    reply.json["id"].as_i64().unwrap()
}

async fn create_pet(app: &axum::Router, owner_id: i64) -> i64 {
    let reply = send(
        app,
        Method::POST,
        &format!("/owners/{owner_id}/pets"),
        Some(json!({"name": "Rex", "birthDate": "2020-05-01", "type": {"id": 2, "name": "dog"}})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
    reply.json["id"].as_i64().unwrap()
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owner_update_and_delete() {
    let app = test_app().await;
    let id = create_owner(&app).await;

    let reply = send(
        &app,
        Method::PUT,
        &format!("/owners/{id}"),
        Some(owner_body("Renamed")),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(reply.headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        get(&app, &format!("/owners/{id}")).await.json["firstName"],
        "Renamed"
    );

    let invalid = send(
        &app,
        Method::PUT,
        &format!("/owners/{id}"),
        Some(json!({"firstName": "A"})),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        fields(&invalid),
        ["address", "city", "lastName", "telephone"]
    );

    let short_phone = send(
        &app,
        Method::PUT,
        &format!("/owners/{id}"),
        Some(json!({"firstName": "A", "lastName": "B", "address": "x", "city": "y", "telephone": "123"})),
    )
    .await;
    assert_eq!(short_phone.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(short_phone.json["title"], "ConstraintViolationException");

    assert_eq!(
        send(
            &app,
            Method::PUT,
            "/owners/999999",
            Some(owner_body("Nobody"))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let deleted = send(&app, Method::DELETE, &format!("/owners/{id}"), None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert!(deleted.headers.get(header::CONTENT_TYPE).is_none());
    assert_eq!(
        get(&app, &format!("/owners/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, &format!("/owners/{id}"), None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owner_with_pets_cannot_be_deleted() {
    let app = test_app().await;
    let reply = send(&app, Method::DELETE, "/owners/1", None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json["title"], "DataIntegrityViolationException");
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owner_pets_and_visits() {
    let app = test_app().await;
    let owner_id = create_owner(&app).await;
    let pet_id = create_pet(&app, owner_id).await;

    let pet = get(&app, &format!("/owners/{owner_id}/pets/{pet_id}")).await;
    assert_eq!(pet.status, StatusCode::OK);
    assert_eq!(pet.json["type"], json!({"name": "dog", "id": 2}));
    assert_eq!(pet.json["ownerId"], owner_id);
    assert_eq!(
        get(&app, &format!("/owners/1/pets/{pet_id}")).await.status,
        StatusCode::NOT_FOUND
    );

    let update = send(
        &app,
        Method::PUT,
        &format!("/owners/{owner_id}/pets/{pet_id}"),
        Some(json!({"name": "Max", "birthDate": "2019-01-01", "type": {"id": 1, "name": "cat"}})),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    assert!(update.headers.get(header::CONTENT_TYPE).is_none());
    let pet = get(&app, &format!("/pets/{pet_id}")).await;
    assert_eq!(pet.json["name"], "Max");
    assert_eq!(pet.json["type"]["name"], "cat");

    let visit = send(
        &app,
        Method::POST,
        &format!("/owners/{owner_id}/pets/{pet_id}/visits"),
        Some(json!({"date": "2024-01-01", "description": "checkup"})),
    )
    .await;
    assert_eq!(visit.status, StatusCode::CREATED);
    assert_eq!(
        location(&visit),
        format!("/api/visits/{}", visit.json["id"])
    );
    assert_eq!(visit.json["petId"], pet_id);

    let owner = get(&app, &format!("/owners/{owner_id}")).await;
    assert_eq!(owner.json["pets"][0]["visits"][0]["description"], "checkup");

    let future = send(
        &app,
        Method::POST,
        &format!("/owners/{owner_id}/pets"),
        Some(json!({"name": "Baby", "birthDate": "2999-01-01", "type": {"id": 1, "name": "cat"}})),
    )
    .await;
    assert_eq!(future.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        future.json["schemaValidationErrors"][0]["defaultMessage"],
        "Birth date cannot be in the future"
    );
    let unknown_type = send(
        &app,
        Method::POST,
        &format!("/owners/{owner_id}/pets"),
        Some(json!({"name": "Odd", "birthDate": "2020-01-01", "type": {"id": 999, "name": "x"}})),
    )
    .await;
    assert_eq!(unknown_type.status, StatusCode::NOT_FOUND);
    assert_eq!(
        unknown_type.json["title"],
        "DataIntegrityViolationException"
    );
    let no_owner = send(
        &app,
        Method::POST,
        "/owners/999999/pets",
        Some(json!({"name": "Odd", "birthDate": "2020-01-01", "type": {"id": 1, "name": "cat"}})),
    )
    .await;
    assert_eq!(no_owner.status, StatusCode::NOT_FOUND);
    assert_eq!(no_owner.json, Value::Null);
    let no_pet = send(
        &app,
        Method::POST,
        "/owners/1/pets/999999/visits",
        Some(json!({"date": "2024-01-01", "description": "x"})),
    )
    .await;
    assert_eq!(no_pet.status, StatusCode::NOT_FOUND);
    assert_eq!(no_pet.json["title"], "DataIntegrityViolationException");
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn pets_list_page_update_delete() {
    let app = test_app().await;
    let list = get(&app, "/pets").await;
    assert_eq!(list.status, StatusCode::OK);
    assert!(list.json.as_array().unwrap().len() >= 13);

    let page = get(&app, "/v2/pets?page=1&size=5").await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(page.json["page"], 1);
    assert_eq!(page.json["size"], 5);
    assert_eq!(page.json["content"].as_array().unwrap().len(), 5);
    assert_eq!(
        get(&app, "/v2/pets?size=0").await.json["title"],
        "ConstraintViolationException"
    );
    assert_eq!(
        get(&app, "/v2/pets?page=x").await.json["title"],
        "MethodArgumentTypeMismatchException"
    );

    let owner_id = create_owner(&app).await;
    let pet_id = create_pet(&app, owner_id).await;
    let update = send(
        &app,
        Method::PUT,
        &format!("/pets/{pet_id}"),
        Some(json!({"name": "Bolt", "birthDate": "2021-01-01", "type": {"id": 2, "name": "dog"}})),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&app, &format!("/pets/{pet_id}")).await.json["name"],
        "Bolt"
    );
    let invalid = send(
        &app,
        Method::PUT,
        &format!("/pets/{pet_id}"),
        Some(json!({"id": -1, "name": "", "visits": [{"description": ""}]})),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        fields(&invalid),
        [
            "birthDate",
            "id",
            "name",
            "type",
            "visits[0].date",
            "visits[0].description",
            "visits[0].petId"
        ]
    );
    assert_eq!(
        send(&app, Method::DELETE, &format!("/pets/{pet_id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/pets/{pet_id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, "/owners/0", None).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, &format!("/owners/{owner_id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn owners_page() {
    let app = test_app().await;
    let page = get(&app, "/v2/owners?lastName=Davis&size=1").await;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(page.json["totalElements"], 2);
    assert_eq!(page.json["totalPages"], 2);
    assert_eq!(page.json["content"][0]["id"], 2);
    let empty = get(&app, "/v2/owners?page=99").await;
    assert_eq!(empty.json["content"], json!([]));
    assert_eq!(
        get(&app, "/v2/owners?page=-1").await.status,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn visits_crud() {
    let app = test_app().await;
    let list = get(&app, "/visits").await;
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(get(&app, "/visits/1").await.json["petId"], 7);

    let created = send(
        &app,
        Method::POST,
        "/visits",
        Some(json!({"date": "2024-02-02", "description": "new", "petId": 1})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json["id"].as_i64().unwrap();
    assert_eq!(location(&created), format!("/api/visits/{id}"));

    let update = send(
        &app,
        Method::PUT,
        &format!("/visits/{id}"),
        Some(json!({"date": "2024-03-03", "description": "updated"})),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    let visit = get(&app, &format!("/visits/{id}")).await;
    assert_eq!(
        visit.json,
        json!({"date": "2024-03-03", "description": "updated", "id": id, "petId": 1})
    );

    let merged = send(
        &app,
        Method::POST,
        "/visits",
        Some(json!({"id": id, "date": "2024-04-04", "description": "merged", "petId": 1})),
    )
    .await;
    assert_eq!(merged.status, StatusCode::CREATED);
    assert_eq!(merged.json["id"], id);
    let stale = send(
        &app,
        Method::POST,
        "/visits",
        Some(json!({"id": 999999, "date": "2024-04-04", "description": "x", "petId": 1})),
    )
    .await;
    assert_eq!(
        stale.json["title"],
        "ObjectOptimisticLockingFailureException"
    );
    let invalid = send(&app, Method::POST, "/visits", Some(json!({"id": -1}))).await;
    assert_eq!(fields(&invalid), ["date", "description", "id", "petId"]);

    assert_eq!(
        send(&app, Method::DELETE, &format!("/visits/{id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/visits/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(
            &app,
            Method::PUT,
            "/visits/999999",
            Some(json!({"date": "2024-01-01", "description": "x"}))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn vets_crud() {
    let app = test_app().await;
    let created = send(
        &app,
        Method::POST,
        "/vets",
        Some(json!({"firstName": "Ada", "lastName": "Lovelace",
                    "specialties": [{"name": "surgery"}, {"name": "radiology"}, {"name": "nope"}]})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json["id"].as_i64().unwrap();
    assert_eq!(location(&created), format!("/api/vets/{id}"));
    assert_eq!(
        created.json["specialties"],
        json!([{"id": 1, "name": "radiology"}, {"id": 2, "name": "surgery"}])
    );

    let update = send(
        &app,
        Method::PUT,
        &format!("/vets/{id}"),
        Some(
            json!({"firstName": "Ada", "lastName": "King", "specialties": [{"name": "dentistry"}]}),
        ),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    let vet = get(&app, &format!("/vets/{id}")).await;
    assert_eq!(vet.json["lastName"], "King");
    assert_eq!(
        vet.json["specialties"],
        json!([{"id": 3, "name": "dentistry"}])
    );

    let invalid = send(
        &app,
        Method::POST,
        "/vets",
        Some(json!({"firstName": "R2D2", "lastName": ""})),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&invalid), ["firstName", "lastName", "specialties"]);
    let stale = send(
        &app,
        Method::POST,
        "/vets",
        Some(json!({"id": 999999, "firstName": "A", "lastName": "B", "specialties": []})),
    )
    .await;
    assert_eq!(
        stale.json["title"],
        "ObjectOptimisticLockingFailureException"
    );

    assert_eq!(
        send(&app, Method::DELETE, &format!("/vets/{id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/vets/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(
            &app,
            Method::PUT,
            "/vets/999999",
            Some(json!({"firstName": "A", "lastName": "B", "specialties": []}))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn pet_types_crud() {
    let app = test_app().await;
    let created = send(
        &app,
        Method::POST,
        "/pettypes",
        Some(json!({"id": 1, "name": "ferret"})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json["id"].as_i64().unwrap();
    assert_ne!(id, 1, "the body id is ignored on create");
    assert_eq!(created.json, json!({"name": "ferret", "id": id}));

    let missing_id = send(
        &app,
        Method::PUT,
        &format!("/pettypes/{id}"),
        Some(json!({"name": "x"})),
    )
    .await;
    assert_eq!(fields(&missing_id), ["id"]);
    let update = send(
        &app,
        Method::PUT,
        &format!("/pettypes/{id}"),
        Some(json!({"id": id, "name": "polecat"})),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&app, &format!("/pettypes/{id}")).await.json["name"],
        "polecat"
    );
    assert_eq!(
        send(
            &app,
            Method::PUT,
            "/pettypes/999999",
            Some(json!({"id": 1, "name": "x"}))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, Method::DELETE, &format!("/pettypes/{id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, &format!("/pettypes/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn specialties_crud() {
    let app = test_app().await;
    let created = send(
        &app,
        Method::POST,
        "/specialties",
        Some(json!({"name": "oncology"})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let id = created.json["id"].as_i64().unwrap();
    assert_eq!(location(&created), format!("/api/specialties/{id}"));

    let update = send(
        &app,
        Method::PUT,
        &format!("/specialties/{id}"),
        Some(json!({"name": "neurology"})),
    )
    .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&app, &format!("/specialties/{id}")).await.json,
        json!({"id": id, "name": "neurology"})
    );
    assert_eq!(
        send(
            &app,
            Method::PUT,
            "/specialties/999999",
            Some(json!({"name": "x"}))
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let invalid = send(&app, Method::POST, "/specialties", Some(json!({"id": -1}))).await;
    assert_eq!(fields(&invalid), ["id", "name"]);

    let in_use = send(&app, Method::DELETE, "/specialties/1", None).await;
    assert_eq!(in_use.status, StatusCode::NOT_FOUND);
    assert_eq!(in_use.json["title"], "DataIntegrityViolationException");
    assert_eq!(
        send(&app, Method::DELETE, &format!("/specialties/{id}"), None)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn users_create() {
    let app = test_app().await;
    let username = format!("u{}", std::process::id());
    let created = send(
        &app,
        Method::POST,
        "/users",
        Some(
            json!({"username": username, "password": "pw", "enabled": true,
                    "roles": [{"name": "OWNER_ADMIN"}, {"name": "ROLE_VET_ADMIN"}]}),
        ),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert!(created.headers.get(header::LOCATION).is_none());
    assert_eq!(
        created.json,
        json!({"username": username, "enabled": true, "password": "pw",
               "roles": [{"name": "ROLE_OWNER_ADMIN"}, {"name": "ROLE_VET_ADMIN"}]})
    );
    let duplicate = send(
        &app,
        Method::POST,
        "/users",
        Some(json!({"username": username, "password": "pw", "enabled": true, "roles": [{"name": "X"}]})),
    )
    .await;
    assert_eq!(duplicate.status, StatusCode::NOT_FOUND);
    assert_eq!(duplicate.json["title"], "DataIntegrityViolationException");
    let no_roles = send(
        &app,
        Method::POST,
        "/users",
        Some(json!({"username": "x", "password": "pw"})),
    )
    .await;
    assert_eq!(no_roles.json["title"], "IllegalArgumentException");
    let invalid = send(
        &app,
        Method::POST,
        "/users",
        Some(json!({"roles": [{"name": ""}]})),
    )
    .await;
    assert_eq!(fields(&invalid), ["roles[0].name", "username"]);
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn unsupported_method_is_a_problem() {
    let app = test_app().await;
    let reply = send(&app, Method::PATCH, "/owners/1", None).await;
    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        reply.json["title"],
        "HttpRequestMethodNotSupportedException"
    );
}
