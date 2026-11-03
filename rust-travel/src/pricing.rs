//! Port of `PricingService`: per-passenger taxes = departure-country tax + arrival-country tax + departure-airport
//! passenger service charge + airline fuel surcharge for the distance band (scaled by cabin) + domestic VAT on the
//! base fare. Decimal arithmetic mirrors `BigDecimal`: additions and products are exact, VAT is divided at scale 6
//! and every amount is rounded half-even to cents.

use crate::dto::PricingDto;
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashMap;

/// The `Cabin` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cabin {
    Economy,
    PremiumEconomy,
    Business,
    First,
}

impl Cabin {
    /// Java constant names, in declaration order.
    pub const NAMES: [&'static str; 4] = ["ECONOMY", "PREMIUM_ECONOMY", "BUSINESS", "FIRST"];

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ECONOMY" => Some(Self::Economy),
            "PREMIUM_ECONOMY" => Some(Self::PremiumEconomy),
            "BUSINESS" => Some(Self::Business),
            "FIRST" => Some(Self::First),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    /// `PricingService.cabinFactor`: fuel surcharge multiplier (`1`, `1.25`, `1.60`, `2.00`).
    pub fn factor(self) -> Decimal {
        match self {
            Self::Economy => Decimal::ONE,
            Self::PremiumEconomy => Decimal::new(125, 2),
            Self::Business => Decimal::new(160, 2),
            Self::First => Decimal::new(200, 2),
        }
    }
}

/// One `tax_rule` row.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct TaxRule {
    pub code: String,
    pub kind: String,
    pub country: Option<String>,
    pub airport_iata: Option<String>,
    pub airline_iata: Option<String>,
    pub min_distance_km: Option<i32>,
    pub max_distance_km: Option<i32>,
    pub amount: Decimal,
}

pub const TAX_RULE_COLUMNS: &str = "code, kind, country, airport_iata, airline_iata, min_distance_km, max_distance_km, amount";

#[derive(Debug, Clone)]
struct FuelBand {
    airline: Option<String>,
    min_km: i32,
    max_km: Option<i32>,
    amount: Decimal,
}

impl FuelBand {
    /// `PricingService.inBand`: min inclusive (NULL = 0), max exclusive (NULL = open).
    fn contains(&self, distance_km: f64) -> bool {
        if distance_km < f64::from(self.min_km) {
            return false;
        }
        self.max_km.is_none_or(|max| distance_km < f64::from(max))
    }
}

/// The tax rule table indexed for pricing. Flat rules of one key are summed (as `PricingService.flat` does over the
/// whole list); fuel bands keep table order because Java keeps the last matching band.
#[derive(Debug, Clone, Default)]
pub struct TaxTable {
    departure: HashMap<String, Decimal>,
    arrival: HashMap<String, Decimal>,
    psc: HashMap<String, Decimal>,
    vat: HashMap<String, Decimal>,
    fuel: Vec<FuelBand>,
}

fn add_to(map: &mut HashMap<String, Decimal>, key: &Option<String>, amount: Decimal) {
    if let Some(k) = key {
        *map.entry(k.clone()).or_insert(Decimal::ZERO) += amount;
    }
}

impl TaxTable {
    /// `rules` in `findAll()` order.
    pub fn new(rules: &[TaxRule]) -> Self {
        let mut t = Self::default();
        for r in rules {
            match r.kind.as_str() {
                "DEPARTURE_COUNTRY" => add_to(&mut t.departure, &r.country, r.amount),
                "ARRIVAL_COUNTRY" => add_to(&mut t.arrival, &r.country, r.amount),
                "AIRPORT_PSC" => add_to(&mut t.psc, &r.airport_iata, r.amount),
                "DOMESTIC_VAT" => add_to(&mut t.vat, &r.country, r.amount),
                "FUEL_SURCHARGE" => t.fuel.push(FuelBand {
                    airline: r.airline_iata.clone(),
                    min_km: r.min_distance_km.unwrap_or(0),
                    max_km: r.max_distance_km,
                    amount: r.amount,
                }),
                _ => {}
            }
        }
        t
    }

    fn flat(map: &HashMap<String, Decimal>, key: &str) -> Decimal {
        map.get(key).copied().unwrap_or(Decimal::ZERO)
    }

    /// Airline-specific band if the airline publishes one, otherwise the default band (airline NULL); when several
    /// bands match, the last one in table order wins.
    pub fn fuel_surcharge(&self, airline: &str, distance_km: f64) -> Decimal {
        let mut airline_band = None;
        let mut default_band = None;
        for band in self.fuel.iter().filter(|b| b.contains(distance_km)) {
            match &band.airline {
                Some(a) if a == airline => airline_band = Some(band.amount),
                None => default_band = Some(band.amount),
                Some(_) => {}
            }
        }
        airline_band.or(default_band).unwrap_or(Decimal::ZERO)
    }

    /// `PricingService.price` for `passengers` passengers on a flight of `distance_km`.
    pub fn price(&self, fare: &FareToPrice<'_>, passengers: i32, distance_km: f64) -> PricingDto {
        let base = cents(fare.base_amount);
        let mut taxes = Decimal::ZERO;
        taxes += Self::flat(&self.departure, fare.origin_country);
        taxes += Self::flat(&self.arrival, fare.destination_country);
        taxes += Self::flat(&self.psc, fare.origin_iata);
        taxes += self.fuel_surcharge(fare.airline, distance_km) * fare.cabin.factor();
        if fare.origin_country == fare.destination_country {
            let vat_percent = Self::flat(&self.vat, fare.origin_country);
            taxes += (base * vat_percent / Decimal::ONE_HUNDRED).round_dp_with_strategy(6, RoundingStrategy::MidpointNearestEven);
        }
        let taxes = cents(taxes);
        let per_passenger = base + taxes;
        let pax = Decimal::from(passengers);
        PricingDto {
            passengers,
            base_per_passenger: base,
            taxes_per_passenger: taxes,
            per_passenger,
            base: cents(base * pax),
            taxes: cents(taxes * pax),
            total: cents(per_passenger * pax),
            currency: fare.currency.to_owned(),
        }
    }
}

/// What `PricingService.price` reads from a fare and its flight.
#[derive(Debug, Clone, Copy)]
pub struct FareToPrice<'a> {
    pub base_amount: Decimal,
    pub cabin: Cabin,
    pub currency: &'a str,
    pub airline: &'a str,
    pub origin_iata: &'a str,
    pub origin_country: &'a str,
    pub destination_country: &'a str,
}

/// `setScale(2, HALF_EVEN)`: rounded and always carrying exactly two decimals (`234.00`).
pub fn cents(amount: Decimal) -> Decimal {
    let mut r = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
    r.rescale(2);
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::distance_km;

    fn d(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn rule(code: &str, kind: &str, country: Option<&str>, airport: Option<&str>, amount: &str) -> TaxRule {
        TaxRule {
            code: code.into(),
            kind: kind.into(),
            country: country.map(Into::into),
            airport_iata: airport.map(Into::into),
            airline_iata: None,
            min_distance_km: None,
            max_distance_km: None,
            amount: d(amount),
        }
    }

    fn band(code: &str, airline: Option<&str>, min: i32, max: Option<i32>, amount: &str) -> TaxRule {
        TaxRule {
            code: code.into(),
            kind: "FUEL_SURCHARGE".into(),
            country: None,
            airport_iata: None,
            airline_iata: airline.map(Into::into),
            min_distance_km: Some(min),
            max_distance_km: max,
            amount: d(amount),
        }
    }

    /// The rule list of `PricingServiceTest`.
    fn rules() -> Vec<TaxRule> {
        vec![
            rule("FR-DEP", "DEPARTURE_COUNTRY", Some("FR"), None, "7.40"),
            rule("GB-DEP", "DEPARTURE_COUNTRY", Some("GB"), None, "15.00"),
            rule("GB-ARR", "ARRIVAL_COUNTRY", Some("GB"), None, "0.80"),
            rule("FR-ARR", "ARRIVAL_COUNTRY", Some("FR"), None, "1.20"),
            rule("NCE-PSC", "AIRPORT_PSC", None, Some("NCE"), "12.80"),
            rule("LHR-PSC", "AIRPORT_PSC", None, Some("LHR"), "29.90"),
            band("YQ-DEF-1", None, 0, Some(1500), "18.00"),
            band("YQ-DEF-2", None, 1500, Some(3500), "32.00"),
            band("YQ-DEF-3", None, 3500, Some(7000), "95.00"),
            band("YQ-BA-1", Some("BA"), 0, Some(1500), "24.00"),
            rule("FR-VAT", "DOMESTIC_VAT", Some("FR"), None, "10.00"),
        ]
    }

    const NCE: (f64, f64) = (43.6584, 7.2159);
    const LHR: (f64, f64) = (51.4700, -0.4543);
    const JFK: (f64, f64) = (40.6413, -73.7781);
    const CDG: (f64, f64) = (49.0097, 2.5479);

    fn km(a: (f64, f64), b: (f64, f64)) -> f64 {
        distance_km(a.0, a.1, b.0, b.1)
    }

    fn fare<'a>(base: &str, cabin: Cabin, airline: &'a str, from: (&'a str, &'a str), to: &'a str) -> FareToPrice<'a> {
        FareToPrice {
            base_amount: d(base),
            cabin,
            currency: "EUR",
            airline,
            origin_iata: from.0,
            origin_country: from.1,
            destination_country: to,
        }
    }

    fn s(v: Decimal) -> String {
        v.to_string()
    }

    #[test]
    fn economy_nice_to_london_uses_departure_arrival_psc_and_airline_fuel_band() {
        let t = TaxTable::new(&rules());
        let p = t.price(&fare("89.00", Cabin::Economy, "BA", ("NCE", "FR"), "GB"), 1, km(NCE, LHR));
        assert_eq!(s(p.taxes_per_passenger), "45.00");
        assert_eq!(s(p.base_per_passenger), "89.00");
        assert_eq!(s(p.per_passenger), "134.00");
        assert_eq!(s(p.total), "134.00");
        assert_eq!(p.currency, "EUR");
    }

    #[test]
    fn totals_scale_with_passengers_and_premium_cabins_pay_higher_taxes() {
        let t = TaxTable::new(&rules());
        let p = t.price(&fare("420.00", Cabin::Business, "BA", ("LHR", "GB"), "FR"), 3, km(LHR, NCE));
        assert_eq!(p.passengers, 3);
        assert_eq!(s(p.taxes_per_passenger), "84.50");
        assert_eq!(s(p.base), "1260.00");
        assert_eq!(s(p.taxes), "253.50");
        assert_eq!(s(p.total), "1513.50");
    }

    #[test]
    fn airline_without_own_fuel_surcharge_falls_back_to_default_band() {
        let t = TaxTable::new(&rules());
        let p = t.price(&fare("300.00", Cabin::Economy, "AF", ("NCE", "FR"), "US"), 1, km(NCE, JFK));
        assert_eq!(s(p.taxes_per_passenger), "115.20");
    }

    #[test]
    fn domestic_flights_add_vat_on_the_base_fare_rounded_half_even() {
        let t = TaxTable::new(&rules());
        let p = t.price(&fare("100.25", Cabin::Economy, "AF", ("NCE", "FR"), "FR"), 1, km(NCE, CDG));
        // 7.40 + 1.20 + 12.80 + 18.00 + 10.025 = 49.425 -> 49.42 (half-even)
        assert_eq!(s(p.taxes_per_passenger), "49.42");
        let p = t.price(&fare("100.35", Cabin::Economy, "AF", ("NCE", "FR"), "FR"), 1, km(NCE, CDG));
        // 39.40 + 10.035 = 49.435 -> 49.44 (half-even rounds to the even cent)
        assert_eq!(s(p.taxes_per_passenger), "49.44");
    }

    #[test]
    fn non_domestic_routes_pay_no_vat() {
        let t = TaxTable::new(&rules());
        let p = t.price(&fare("100.25", Cabin::Economy, "AF", ("NCE", "FR"), "GB"), 1, km(NCE, LHR));
        // 7.40 (FR departure) + 0.80 (GB arrival) + 12.80 (NCE PSC) + 18.00 (default band): no VAT
        assert_eq!(s(p.taxes_per_passenger), "39.00");
    }

    #[test]
    fn cabin_factor_products_round_half_even() {
        let rules = vec![band("YQ-X", None, 0, None, "18.05"), band("YQ-Y", Some("ZZ"), 0, None, "18.15")];
        let t = TaxTable::new(&rules);
        // 18.05 × 1.25 = 22.5625 -> 22.56; 18.15 × 1.25 = 22.6875 -> 22.69
        let p = t.price(&fare("10", Cabin::PremiumEconomy, "AF", ("NCE", "FR"), "GB"), 1, 100.0);
        assert_eq!(s(p.taxes_per_passenger), "22.56");
        let p = t.price(&fare("10", Cabin::PremiumEconomy, "ZZ", ("NCE", "FR"), "GB"), 1, 100.0);
        assert_eq!(s(p.taxes_per_passenger), "22.69");
        // exact ties: 0.125 -> 0.12, 0.135 -> 0.14
        let t = TaxTable::new(&[band("YQ", None, 0, None, "0.10")]);
        let p = t.price(&fare("1", Cabin::PremiumEconomy, "AF", ("NCE", "FR"), "GB"), 1, 1.0);
        assert_eq!(s(p.taxes_per_passenger), "0.12");
        assert_eq!(s(cents(d("0.135"))), "0.14");
        assert_eq!(s(cents(d("-0.125"))), "-0.12");
    }

    #[test]
    fn base_is_rounded_half_even_and_amounts_keep_two_decimals() {
        let t = TaxTable::new(&[]);
        let p = t.price(&fare("234", Cabin::First, "AF", ("NCE", "FR"), "GB"), 2, 1.0);
        assert_eq!(s(p.base_per_passenger), "234.00");
        assert_eq!(s(p.taxes_per_passenger), "0.00");
        assert_eq!(s(p.total), "468.00");
        assert_eq!(serde_json::to_string(&p.total).unwrap(), "468.00");
        let p = t.price(&fare("10.005", Cabin::Economy, "AF", ("NCE", "FR"), "GB"), 1, 1.0);
        assert_eq!(s(p.base_per_passenger), "10.00");
        let p = t.price(&fare("10.015", Cabin::Economy, "AF", ("NCE", "FR"), "GB"), 1, 1.0);
        assert_eq!(s(p.base_per_passenger), "10.02");
    }

    #[test]
    fn fuel_bands_are_min_inclusive_max_exclusive() {
        let rules = vec![
            band("D1", None, 0, Some(1500), "18.00"),
            band("D2", None, 1500, Some(3500), "32.00"),
            band("D4", None, 7000, None, "160.00"),
        ];
        let t = TaxTable::new(&rules);
        assert_eq!(t.fuel_surcharge("AF", 0.0), d("18.00"));
        assert_eq!(t.fuel_surcharge("AF", 1499.9999999), d("18.00"));
        assert_eq!(t.fuel_surcharge("AF", 1500.0), d("32.00"));
        assert_eq!(t.fuel_surcharge("AF", 3499.999), d("32.00"));
        assert_eq!(t.fuel_surcharge("AF", 3500.0), Decimal::ZERO);
        assert_eq!(t.fuel_surcharge("AF", 7000.0), d("160.00"));
        assert_eq!(t.fuel_surcharge("AF", 1e9), d("160.00"));
        assert_eq!(t.fuel_surcharge("AF", -1.0), Decimal::ZERO);
    }

    #[test]
    fn null_min_distance_means_zero() {
        let mut r = band("D", None, 0, Some(10), "5.00");
        r.min_distance_km = None;
        let t = TaxTable::new(&[r]);
        assert_eq!(t.fuel_surcharge("AF", 0.0), d("5.00"));
        assert_eq!(t.fuel_surcharge("AF", -0.5), Decimal::ZERO);
    }

    #[test]
    fn airline_band_overrides_default_and_last_match_wins() {
        let rules = vec![
            band("A1", Some("EK"), 0, None, "1.00"),
            band("D1", None, 0, None, "2.00"),
            band("A2", Some("EK"), 0, None, "3.00"),
            band("D2", None, 0, None, "4.00"),
            band("O1", Some("SQ"), 0, None, "9.00"),
        ];
        let t = TaxTable::new(&rules);
        assert_eq!(t.fuel_surcharge("EK", 100.0), d("3.00"));
        assert_eq!(t.fuel_surcharge("AF", 100.0), d("4.00"));
        assert_eq!(t.fuel_surcharge("SQ", 100.0), d("9.00"));
    }

    #[test]
    fn flat_rules_of_one_key_are_summed() {
        let rules = vec![
            rule("FR-DEP", "DEPARTURE_COUNTRY", Some("FR"), None, "7.40"),
            rule("FR-DEP2", "DEPARTURE_COUNTRY", Some("FR"), None, "1.05"),
            rule("FR-VAT", "DOMESTIC_VAT", Some("FR"), None, "10.00"),
            rule("FR-VAT2", "DOMESTIC_VAT", Some("FR"), None, "5.50"),
        ];
        let t = TaxTable::new(&rules);
        // 8.45 + 15.5% of 33.33 (5.16615) = 13.61615 -> 13.62
        let p = t.price(&fare("33.33", Cabin::Economy, "AF", ("NCE", "FR"), "FR"), 1, 1.0);
        assert_eq!(s(p.taxes_per_passenger), "13.62");
    }

    #[test]
    fn seed_tax_table_mad_fra_is_not_domestic() {
        // data.sql rows used by MAD→FRA: ES-DEP 3.50 + DE-ARR 2.00 + MAD-PSC 14.50 + YQ-DEF-1 18.00 (1,449 km)
        let rules = vec![
            rule("ES-DEP", "DEPARTURE_COUNTRY", Some("ES"), None, "3.50"),
            rule("DE-ARR", "ARRIVAL_COUNTRY", Some("DE"), None, "2.00"),
            rule("MAD-PSC", "AIRPORT_PSC", None, Some("MAD"), "14.50"),
            band("YQ-DEF-1", None, 0, Some(1500), "18.00"),
            band("YQ-DEF-2", None, 1500, Some(3500), "32.00"),
            rule("ES-VAT", "DOMESTIC_VAT", Some("ES"), None, "10.00"),
        ];
        let t = TaxTable::new(&rules);
        let dist = distance_km(40.4983, -3.5676, 50.0379, 8.5622);
        assert!(dist < 1500.0, "{dist}");
        let p = t.price(&fare("150.00", Cabin::Business, "IB", ("MAD", "ES"), "DE"), 2, dist);
        // 3.50 + 2.00 + 14.50 + 18.00 × 1.60 = 48.80
        assert_eq!(s(p.taxes_per_passenger), "48.80");
        assert_eq!(s(p.total), "397.60");
    }

    #[test]
    fn cabin_names_round_trip() {
        for n in Cabin::NAMES {
            assert_eq!(Cabin::from_name(n).unwrap().name(), n);
        }
        assert_eq!(Cabin::from_name("economy"), None);
    }
}
