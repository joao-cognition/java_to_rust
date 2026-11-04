//! Fare pricing (`PricingService.java`) and great-circle distance (`Geo.java`): the CPU hot path.

use crate::model::{Cabin, Fare, Pricing};
use rust_decimal::{Decimal, RoundingStrategy};

const EARTH_RADIUS_KM: f64 = 6371.0088;

/// Haversine great-circle distance in kilometres, same operation order as `Geo.distanceKm`.
pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let d_phi = (lat2 - lat1).to_radians();
    let d_lambda = (lon2 - lon1).to_radians();
    let sin_d_phi = (d_phi / 2.0).sin();
    let sin_d_lambda = (d_lambda / 2.0).sin();
    let h = sin_d_phi * sin_d_phi + phi1.cos() * phi2.cos() * sin_d_lambda * sin_d_lambda;
    2.0 * EARTH_RADIUS_KM * h.sqrt().min(1.0).asin()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaxKind {
    DepartureCountry,
    ArrivalCountry,
    AirportPsc,
    FuelSurcharge,
    DomesticVat,
}

impl TaxKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "DEPARTURE_COUNTRY" => Some(Self::DepartureCountry),
            "ARRIVAL_COUNTRY" => Some(Self::ArrivalCountry),
            "AIRPORT_PSC" => Some(Self::AirportPsc),
            "FUEL_SURCHARGE" => Some(Self::FuelSurcharge),
            "DOMESTIC_VAT" => Some(Self::DomesticVat),
            _ => None,
        }
    }
}

pub struct TaxRule {
    pub kind: TaxKind,
    pub country: Option<String>,
    pub airport_iata: Option<String>,
    pub airline_iata: Option<String>,
    pub min_distance_km: Option<i32>,
    pub max_distance_km: Option<i32>,
    pub amount: Decimal,
}

fn cabin_factor(cabin: Cabin) -> Decimal {
    match cabin {
        Cabin::Economy => Decimal::ONE,
        Cabin::PremiumEconomy => Decimal::new(125, 2),
        Cabin::Business => Decimal::new(160, 2),
        Cabin::First => Decimal::new(200, 2),
    }
}

/// `setScale(dp, HALF_EVEN)`: rounds and pads to exactly `dp` decimals.
fn scale(d: Decimal, dp: u32) -> Decimal {
    let mut r = d.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven);
    r.rescale(dp);
    r
}

/// The parts of the tax computation that only depend on the route, resolved once per search.
///
/// Decimal additions are exact, so pre-summing the route-level terms gives the same result as `PricingService`
/// adding them one by one for every fare.
pub struct RouteTaxes<'r> {
    rules: &'r [TaxRule],
    /// departure-country + arrival-country + departure-airport PSC
    flat: Decimal,
    /// `Some(percent)` on domestic routes
    vat_percent: Option<Decimal>,
}

impl<'r> RouteTaxes<'r> {
    pub fn new(
        rules: &'r [TaxRule],
        origin_country: &str,
        destination_country: &str,
        origin_iata: &str,
    ) -> Self {
        let sum = |pred: &dyn Fn(&TaxRule) -> bool| {
            rules
                .iter()
                .filter(|r| pred(r))
                .fold(Decimal::ZERO, |acc, r| acc + r.amount)
        };
        fn is(kind: TaxKind, country: &str) -> impl Fn(&TaxRule) -> bool + '_ {
            move |r: &TaxRule| r.kind == kind && r.country.as_deref() == Some(country)
        }
        let flat = sum(&is(TaxKind::DepartureCountry, origin_country))
            + sum(&is(TaxKind::ArrivalCountry, destination_country))
            + sum(&|r: &TaxRule| {
                r.kind == TaxKind::AirportPsc && r.airport_iata.as_deref() == Some(origin_iata)
            });
        let vat_percent = (origin_country == destination_country)
            .then(|| sum(&is(TaxKind::DomesticVat, origin_country)));
        Self {
            rules,
            flat,
            vat_percent,
        }
    }

    /// Airline-specific band if the airline publishes one, otherwise the default band (`airline_iata` NULL).
    pub fn fuel_surcharge(&self, airline_iata: &str, distance_km: f64) -> Decimal {
        let mut airline_band = None;
        let mut default_band = None;
        for rule in self.rules {
            if rule.kind != TaxKind::FuelSurcharge || !in_band(rule, distance_km) {
                continue;
            }
            match rule.airline_iata.as_deref() {
                Some(a) if a == airline_iata => airline_band = Some(rule.amount),
                None => default_band = Some(rule.amount),
                _ => {}
            }
        }
        airline_band.or(default_band).unwrap_or(Decimal::ZERO)
    }

    /// `PricingService.price(fare, passengers, rules, distanceKm)` with the flight's fuel surcharge precomputed.
    pub fn price<'f>(
        &self,
        fare: &'f Fare,
        passengers: i32,
        fuel_surcharge: Decimal,
    ) -> Pricing<'f> {
        let base = scale(fare.base_amount, 2);
        let mut taxes = self.flat + fuel_surcharge * cabin_factor(fare.cabin);
        if let Some(vat) = self.vat_percent {
            let vat_amount = (base * vat / Decimal::ONE_HUNDRED)
                .round_dp_with_strategy(6, RoundingStrategy::MidpointNearestEven);
            taxes += vat_amount;
        }
        let taxes = scale(taxes, 2);
        let per_passenger = base + taxes;
        let pax = Decimal::from(passengers);
        Pricing {
            passengers,
            base_per_passenger: base,
            taxes_per_passenger: taxes,
            per_passenger,
            base: scale(base * pax, 2),
            taxes: scale(taxes * pax, 2),
            total: scale(per_passenger * pax, 2),
            currency: &fare.currency,
        }
    }
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

    fn rule(
        kind: TaxKind,
        country: Option<&str>,
        airport: Option<&str>,
        airline: Option<&str>,
        band: (Option<i32>, Option<i32>),
        amount: &str,
    ) -> TaxRule {
        TaxRule {
            kind,
            country: country.map(Into::into),
            airport_iata: airport.map(Into::into),
            airline_iata: airline.map(Into::into),
            min_distance_km: band.0,
            max_distance_km: band.1,
            amount: amount.parse().unwrap(),
        }
    }

    fn fare(cabin: Cabin, base: &str) -> Fare {
        Fare {
            booking_class: "Y".into(),
            cabin,
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
        let d = distance_km(43.6584, 7.2159, 51.47, -0.4543);
        assert!((d - 1040.7006).abs() < 1e-3, "{d}");
    }

    #[test]
    fn prices_like_pricing_service() {
        let rules = vec![
            rule(
                TaxKind::DepartureCountry,
                Some("FR"),
                None,
                None,
                (None, None),
                "7.40",
            ),
            rule(
                TaxKind::ArrivalCountry,
                Some("FR"),
                None,
                None,
                (None, None),
                "2.00",
            ),
            rule(
                TaxKind::AirportPsc,
                None,
                Some("NCE"),
                None,
                (None, None),
                "12.15",
            ),
            rule(
                TaxKind::FuelSurcharge,
                None,
                None,
                None,
                (Some(0), Some(1500)),
                "20.00",
            ),
            rule(
                TaxKind::FuelSurcharge,
                None,
                None,
                Some("AF"),
                (Some(0), Some(1500)),
                "25.00",
            ),
            rule(
                TaxKind::DomesticVat,
                Some("FR"),
                None,
                None,
                (None, None),
                "10.00",
            ),
        ];
        let route = RouteTaxes::new(&rules, "FR", "FR", "NCE");
        assert_eq!(route.fuel_surcharge("AF", 700.0), "25.00".parse().unwrap());
        assert_eq!(route.fuel_surcharge("BA", 700.0), "20.00".parse().unwrap());
        assert_eq!(route.fuel_surcharge("BA", 1500.0), Decimal::ZERO);
        let f = fare(Cabin::Business, "99.95");
        let p = route.price(&f, 3, route.fuel_surcharge("AF", 700.0));
        // 7.40 + 2.00 + 12.15 + 25.00 * 1.60 + 99.95 * 10 / 100 = 71.545 -> 71.54 (half-even)
        assert_eq!(p.taxes_per_passenger.to_string(), "71.54");
        assert_eq!(p.per_passenger.to_string(), "171.49");
        assert_eq!(p.total.to_string(), "514.47");
        assert_eq!(p.base.to_string(), "299.85");
        let whole = fare(Cabin::Economy, "384");
        assert_eq!(
            route
                .price(&whole, 1, Decimal::ZERO)
                .base_per_passenger
                .to_string(),
            "384.00"
        );
    }
}
