mod common;

#[tokio::test]
async fn smoke_route_reads_seeded_reference_data() {
    let app = require_app!();
    let r = common::get(&app, "/travel/api/_smoke").await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(r.body["airports"].as_i64().unwrap() > 0, "{}", r.body);
    assert!(r.body["airlines"].as_i64().unwrap() > 0, "{}", r.body);
}

#[tokio::test]
async fn unknown_path_is_a_problem_detail() {
    let app = require_app!();
    let r = common::get(&app, "/travel/api/nope").await;
    assert_eq!(r.status, 404);
    assert_eq!(r.content_type.as_deref(), Some("application/problem+json"));
    assert_eq!(r.body["status"], 404);
}
