//! Port of `Geo`: great-circle geometry on the WGS84 mean earth radius.

pub const EARTH_RADIUS_KM: f64 = 6371.0088;

/// Haversine great-circle distance in kilometres, with the same evaluation order as `Geo.distanceKm` so that
/// distances falling near a fuel-surcharge band edge land in the same band as in Java.
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

#[cfg(test)]
mod tests {
    use super::*;

    const NCE: (f64, f64) = (43.6584, 7.2159);
    const LHR: (f64, f64) = (51.4700, -0.4543);
    const JFK: (f64, f64) = (40.6413, -73.7781);

    #[test]
    fn known_distances() {
        // PricingServiceTest.greatCircleDistanceMatchesKnownValues
        assert!((distance_km(NCE.0, NCE.1, LHR.0, LHR.1) - 1039.0).abs() < 5.0);
        assert!((distance_km(NCE.0, NCE.1, JFK.0, JFK.1) - 6407.0).abs() < 30.0);
    }

    #[test]
    fn same_point_and_antipodes() {
        assert_eq!(distance_km(NCE.0, NCE.1, NCE.0, NCE.1), 0.0);
        let half_circumference = std::f64::consts::PI * EARTH_RADIUS_KM;
        assert!((distance_km(0.0, 0.0, 0.0, 180.0) - half_circumference).abs() < 1e-6);
    }

    #[test]
    fn to_radians_matches_java_constant() {
        // Math.toRadians multiplies by DEGREES_TO_RADIANS = 0.017453292519943295.
        assert_eq!(1.0_f64.to_radians(), 0.017453292519943295);
        assert_eq!(43.6584_f64.to_radians(), 43.6584 * 0.017453292519943295);
    }
}
