//! JSON contract tests: the serialized shape must match what the Spring Boot service emits
//! for the same rows (see `openapi.yml`).

use chrono::NaiveDate;
use rust_petclinic::dto::{Health, Owner, OwnerFields, Pet, PetType, Specialty, Vet, Visit};
use serde_json::json;

fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

#[test]
fn specialty_json() {
    let s = Specialty {
        id: 1,
        name: Some("radiology".into()),
    };
    assert_eq!(
        serde_json::to_string(&s).unwrap(),
        r#"{"id":1,"name":"radiology"}"#
    );
}

#[test]
fn pet_type_json() {
    let t = PetType {
        name: Some("cat".into()),
        id: 1,
    };
    assert_eq!(
        serde_json::to_string(&t).unwrap(),
        r#"{"name":"cat","id":1}"#
    );
}

#[test]
fn vet_json_with_and_without_specialties() {
    let v = Vet {
        first_name: Some("Helen".into()),
        last_name: Some("Leary".into()),
        specialties: vec![Specialty {
            id: 1,
            name: Some("radiology".into()),
        }],
        id: 2,
    };
    assert_eq!(
        serde_json::to_value(&v).unwrap(),
        json!({"firstName":"Helen","lastName":"Leary","specialties":[{"id":1,"name":"radiology"}],"id":2})
    );

    let v = Vet {
        first_name: Some("James".into()),
        last_name: Some("Carter".into()),
        specialties: vec![],
        id: 1,
    };
    assert_eq!(
        serde_json::to_string(&v).unwrap(),
        r#"{"firstName":"James","lastName":"Carter","specialties":[],"id":1}"#
    );
}

#[test]
fn visit_json_uses_iso_date() {
    let v = Visit {
        date: date("2010-03-04"),
        description: "rabies shot".into(),
        id: 1,
        pet_id: 7,
    };
    assert_eq!(
        serde_json::to_string(&v).unwrap(),
        r#"{"date":"2010-03-04","description":"rabies shot","id":1,"petId":7}"#
    );
}

#[test]
fn pet_json_nests_type_and_visits() {
    let p = Pet {
        name: "Samantha".into(),
        birth_date: date("1995-09-04"),
        pet_type: PetType {
            name: Some("cat".into()),
            id: 1,
        },
        id: 7,
        owner_id: 6,
        visits: vec![Visit {
            date: date("2010-03-04"),
            description: "rabies shot".into(),
            id: 1,
            pet_id: 7,
        }],
    };
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        json!({
            "name": "Samantha",
            "birthDate": "1995-09-04",
            "type": {"name": "cat", "id": 1},
            "id": 7,
            "ownerId": 6,
            "visits": [{"date": "2010-03-04", "description": "rabies shot", "id": 1, "petId": 7}]
        })
    );
}

#[test]
fn owner_json_nests_pets_and_keeps_empty_arrays() {
    let o = Owner {
        first_name: Some("George".into()),
        last_name: Some("Franklin".into()),
        address: Some("110 W. Liberty St.".into()),
        city: Some("Madison".into()),
        telephone: Some("6085551023".into()),
        id: 1,
        pets: vec![Pet {
            name: "Leo".into(),
            birth_date: date("2000-09-07"),
            pet_type: PetType {
                name: Some("cat".into()),
                id: 1,
            },
            id: 1,
            owner_id: 1,
            visits: vec![],
        }],
    };
    let expected = json!({
        "firstName": "George",
        "lastName": "Franklin",
        "address": "110 W. Liberty St.",
        "city": "Madison",
        "telephone": "6085551023",
        "id": 1,
        "pets": [{
            "name": "Leo",
            "birthDate": "2000-09-07",
            "type": {"name": "cat", "id": 1},
            "id": 1,
            "ownerId": 1,
            "visits": []
        }]
    });
    assert_eq!(serde_json::to_value(&o).unwrap(), expected);
}

#[test]
fn owner_sort_children_orders_pets_by_name_and_visits_newest_first() {
    let visit = |id, d| Visit {
        date: date(d),
        description: "x".into(),
        id,
        pet_id: 1,
    };
    let mut o = Owner {
        first_name: None,
        last_name: None,
        address: None,
        city: None,
        telephone: None,
        id: 1,
        pets: vec![
            Pet {
                name: "zed".into(),
                birth_date: date("2000-01-01"),
                pet_type: PetType { name: None, id: 1 },
                id: 2,
                owner_id: 1,
                visits: vec![visit(1, "2008-09-04"), visit(2, "2010-03-04")],
            },
            Pet {
                name: "Alpha".into(),
                birth_date: date("2000-01-01"),
                pet_type: PetType { name: None, id: 1 },
                id: 1,
                owner_id: 1,
                visits: vec![],
            },
        ],
    };
    o.sort_children();
    assert_eq!(o.pets[0].name, "Alpha");
    assert_eq!(o.pets[1].visits[0].id, 2);
}

#[test]
fn owner_fields_deserializes_missing_fields_as_none() {
    let f: OwnerFields = serde_json::from_str(r#"{"firstName":"George"}"#).unwrap();
    assert_eq!(f.first_name.as_deref(), Some("George"));
    assert!(f.last_name.is_none());
}

#[test]
fn health_json() {
    assert_eq!(
        serde_json::to_string(&Health::UP).unwrap(),
        r#"{"groups":["liveness","readiness"],"status":"UP"}"#
    );
}

#[test]
fn integer_ids_are_coerced_like_jackson() {
    use rust_petclinic::dto::VisitInput;
    let pet_id = |raw: &str| {
        serde_json::from_str::<VisitInput>(&format!(r#"{{"petId":{raw}}}"#)).map(|v| v.pet_id)
    };
    assert_eq!(pet_id("1").unwrap(), Some(1));
    assert_eq!(pet_id(r#""1""#).unwrap(), Some(1));
    assert_eq!(pet_id(r#"" 2""#).unwrap(), Some(2));
    assert_eq!(pet_id(r#""+1""#).unwrap(), Some(1));
    assert_eq!(pet_id("1.9").unwrap(), Some(1));
    assert_eq!(pet_id("-0.5").unwrap(), Some(0));
    assert_eq!(pet_id(r#""""#).unwrap(), None);
    assert_eq!(pet_id("null").unwrap(), None);
    for unreadable in [r#""1.5""#, r#""0x1""#, "true", "3000000000", "[1]"] {
        assert!(pet_id(unreadable).is_err(), "{unreadable} should not parse");
    }
}
