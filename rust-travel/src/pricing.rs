//! Port of `service/PricingService.java` and `service/Geo.java`.
//!
//! Taxes per passenger are the departure-country tax, the arrival-country tax, the departure-airport passenger
//! service charge, the airline fuel surcharge for the great-circle distance band (scaled by cabin), and domestic VAT
//! on the base fare when origin and destination are in the same country. Amounts are rounded half-even to cents.

use crate::model::{Airport, Fare, Pricing, TaxRule};
use rust_decimal::{Decimal, RoundingStrategy};

const EARTH_RADIUS_KM: f64 = 6371.0088;

/// Haversine great-circle distance in kilometres (`Geo.distanceKm`).
pub fn distance_km(a: &Airport, b: &Airport) -> f64 {
    let phi1 = a.latitude.to_radians();
    let phi2 = b.latitude.to_radians();
    let d_phi = (b.latitude - a.latitude).to_radians();
    let d_lambda = (b.longitude - a.longitude).to_radians();
    let s_phi = (d_phi / 2.0).sin();
    let s_lambda = (d_lambda / 2.0).sin();
    let h = s_phi * s_phi + phi1.cos() * phi2.cos() * s_lambda * s_lambda;
    2.0 * EARTH_RADIUS_KM * h.sqrt().min(1.0).asin()
}

/// Fuel surcharge multiplier per cabin.
fn cabin_factor(cabin: &str) -> Decimal {
    match cabin {
        "PREMIUM_ECONOMY" => Decimal::new(125, 2),
        "BUSINESS" => Decimal::new(160, 2),
        "FIRST" => Decimal::new(200, 2),
        _ => Decimal::ONE,
    }
}

/// `setScale(2, RoundingMode.HALF_EVEN)`.
fn cents(d: Decimal) -> Decimal {
    let mut r = d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
    r.rescale(2);
    r
}

pub struct Route<'a> {
    pub origin: &'a Airport,
    pub destination: &'a Airport,
    pub airline_iata: &'a str,
    pub distance_km: f64,
}

pub fn price(fare: &Fare, passengers: i32, rules: &[TaxRule], route: &Route) -> Pricing {
    let base = cents(fare.base_amount);

    let mut taxes = Decimal::ZERO;
    taxes += flat(rules, "DEPARTURE_COUNTRY", &route.origin.country);
    taxes += flat(rules, "ARRIVAL_COUNTRY", &route.destination.country);
    taxes += passenger_service_charge(rules, &route.origin.iata);
    taxes +=
        fuel_surcharge(rules, route.airline_iata, route.distance_km) * cabin_factor(&fare.cabin);
    if route.origin.country == route.destination.country {
        let vat_percent = flat(rules, "DOMESTIC_VAT", &route.origin.country);
        taxes += (base * vat_percent / Decimal::ONE_HUNDRED)
            .round_dp_with_strategy(6, RoundingStrategy::MidpointNearestEven);
    }
    let taxes = cents(taxes);

    let per_passenger = base + taxes;
    let pax = Decimal::from(passengers);
    Pricing {
        passengers,
        base_per_passenger: base,
        taxes_per_passenger: taxes,
        per_passenger,
        base: cents(base * pax),
        taxes: cents(taxes * pax),
        total: cents(per_passenger * pax),
        currency: fare.currency.clone(),
    }
}

fn flat(rules: &[TaxRule], kind: &str, country: &str) -> Decimal {
    rules
        .iter()
        .filter(|r| r.kind == kind && r.country.as_deref() == Some(country))
        .map(|r| r.amount)
        .sum()
}

fn passenger_service_charge(rules: &[TaxRule], airport_iata: &str) -> Decimal {
    rules
        .iter()
        .filter(|r| r.kind == "AIRPORT_PSC" && r.airport_iata.as_deref() == Some(airport_iata))
        .map(|r| r.amount)
        .sum()
}

/// Airline-specific band if the airline publishes one, otherwise the default band (`airline_iata` NULL).
fn fuel_surcharge(rules: &[TaxRule], airline_iata: &str, distance_km: f64) -> Decimal {
    let mut airline_band = None;
    let mut default_band = None;
    for r in rules {
        if r.kind != "FUEL_SURCHARGE" || !in_band(r, distance_km) {
            continue;
        }
        match r.airline_iata.as_deref() {
            Some(a) if a == airline_iata => airline_band = Some(r.amount),
            None => default_band = Some(r.amount),
            _ => {}
        }
    }
    airline_band.or(default_band).unwrap_or(Decimal::ZERO)
}

fn in_band(rule: &TaxRule, distance_km: f64) -> bool {
    let min = rule.min_distance_km.unwrap_or(0);
    if distance_km < f64::from(min) {
        return false;
    }
    rule.max_distance_km
        .is_none_or(|max| distance_km < f64::from(max))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn airport(iata: &str, country: &str, lat: f64, lon: f64) -> Airport {
        Airport {
            iata: iata.into(),
            icao: String::new(),
            name: String::new(),
            city: String::new(),
            country: country.into(),
            timezone: "UTC".into(),
            latitude: lat,
            longitude: lon,
            tz: None,
        }
    }

    fn rule(
        kind: &str,
        country: Option<&str>,
        airport: Option<&str>,
        airline: Option<&str>,
        band: (Option<i32>, Option<i32>),
        amount: &str,
    ) -> TaxRule {
        TaxRule {
            kind: kind.into(),
            country: country.map(Into::into),
            airport_iata: airport.map(Into::into),
            airline_iata: airline.map(Into::into),
            min_distance_km: band.0,
            max_distance_km: band.1,
            amount: amount.parse().unwrap(),
        }
    }

    fn fare(cabin: &str, base: &str) -> Fare {
        Fare {
            booking_class: "Y".into(),
            cabin: cabin.into(),
            base_amount: base.parse().unwrap(),
            currency: "EUR".into(),
            refundable: false,
            changeable: false,
            baggage_included_kg: 0,
            seats_available: 9,
        }
    }

    #[test]
    fn nice_heathrow_distance() {
        let d = distance_km(
            &airport("NCE", "FR", 43.6584, 7.2159),
            &airport("LHR", "GB", 51.47, -0.4543),
        );
        assert!((d - 1043.0).abs() < 5.0, "{d}");
    }

    #[test]
    fn prices_international_fare_with_airline_band_and_cabin_factor() {
        let (o, d) = (
            airport("NCE", "FR", 43.6584, 7.2159),
            airport("LHR", "GB", 51.47, -0.4543),
        );
        let rules = vec![
            rule(
                "DEPARTURE_COUNTRY",
                Some("FR"),
                None,
                None,
                (None, None),
                "7.50",
            ),
            rule(
                "ARRIVAL_COUNTRY",
                Some("GB"),
                None,
                None,
                (None, None),
                "13.00",
            ),
            rule(
                "AIRPORT_PSC",
                None,
                Some("NCE"),
                None,
                (None, None),
                "10.25",
            ),
            rule(
                "FUEL_SURCHARGE",
                None,
                None,
                None,
                (Some(0), Some(1500)),
                "20.00",
            ),
            rule(
                "FUEL_SURCHARGE",
                None,
                None,
                Some("AF"),
                (Some(0), Some(1500)),
                "25.00",
            ),
        ];
        let route = Route {
            origin: &o,
            destination: &d,
            airline_iata: "AF",
            distance_km: distance_km(&o, &d),
        };
        let p = price(&fare("BUSINESS", "300.00"), 2, &rules, &route);
        assert_eq!(p.taxes_per_passenger.to_string(), "70.75");
        assert_eq!(p.per_passenger.to_string(), "370.75");
        assert_eq!(p.total.to_string(), "741.50");
        let p = price(
            &fare("ECONOMY", "100"),
            1,
            &rules,
            &Route {
                airline_iata: "BA",
                ..route
            },
        );
        assert_eq!(p.base_per_passenger.to_string(), "100.00");
        assert_eq!(p.taxes_per_passenger.to_string(), "50.75");
    }

    #[test]
    fn domestic_vat_rounds_half_even() {
        let (o, d) = (
            airport("NCE", "FR", 43.6584, 7.2159),
            airport("CDG", "FR", 49.0097, 2.5479),
        );
        let rules = vec![rule(
            "DOMESTIC_VAT",
            Some("FR"),
            None,
            None,
            (None, None),
            "10.00",
        )];
        let route = Route {
            origin: &o,
            destination: &d,
            airline_iata: "AF",
            distance_km: distance_km(&o, &d),
        };
        assert_eq!(
            price(&fare("ECONOMY", "100.25"), 1, &rules, &route)
                .taxes_per_passenger
                .to_string(),
            "10.02"
        );
        assert_eq!(
            price(&fare("ECONOMY", "100.35"), 1, &rules, &route)
                .taxes_per_passenger
                .to_string(),
            "10.04"
        );
    }
}
