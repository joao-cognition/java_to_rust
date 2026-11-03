mod common;

use common::Api;
use serde_json::json;

#[tokio::test]
async fn airports_are_listed_by_iata() {
    let Some(api) = Api::connect().await else { return };
    let r = api.get("airports").await;
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json");
    let codes: Vec<&str> = r.json.as_array().unwrap().iter().map(|a| a["iata"].as_str().unwrap()).collect();
    let mut sorted = codes.clone();
    sorted.sort();
    assert_eq!(codes, sorted);
    assert!(codes.contains(&"NCE") && codes.contains(&"LHR"));
}

#[tokio::test]
async fn airport_lookup_normalizes_the_code() {
    let Some(api) = Api::connect().await else { return };
    let r = api.get("airports/%20nce%20").await;
    assert_eq!(r.status, 200);
    assert_eq!(
        r.json,
        json!({"iata":"NCE","icao":"LFMN","name":"Nice Côte d'Azur","city":"Nice","country":"FR",
               "timezone":"Europe/Paris","latitude":43.6584,"longitude":7.2159})
    );
}

#[tokio::test]
async fn unknown_airport_is_a_problem() {
    let Some(api) = Api::connect().await else { return };
    api.problem("airports/ZZZ", 404, "Airport ZZZ not found").await;
    api.problem("airports/zz%20z", 404, "Airport zz z not found").await;
}

#[tokio::test]
async fn airlines_are_listed_and_looked_up() {
    let Some(api) = Api::connect().await else { return };
    let r = api.get("airlines").await;
    assert_eq!(r.status, 200);
    let codes: Vec<&str> = r.json.as_array().unwrap().iter().map(|a| a["iata"].as_str().unwrap()).collect();
    assert!(codes.windows(2).all(|w| w[0] <= w[1]));
    let af = api.get("airlines/af").await;
    assert_eq!(
        af.json,
        json!({"iata":"AF","icao":"AFR","name":"Air France","country":"FR","alliance":"SKYTEAM"})
    );
    api.problem("airlines/XX", 404, "Airline XX not found").await;
}

#[tokio::test]
async fn unmapped_paths_are_spring_404s() {
    let Some(api) = Api::connect().await else { return };
    api.problem("nope", 404, "No static resource api/nope.").await;
}
