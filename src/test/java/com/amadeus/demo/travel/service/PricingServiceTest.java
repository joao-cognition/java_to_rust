package com.amadeus.demo.travel.service;

import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.util.List;

import com.amadeus.demo.travel.model.Airline;
import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.model.Cabin;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Flight;
import com.amadeus.demo.travel.model.TaxKind;
import com.amadeus.demo.travel.model.TaxRule;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;

class PricingServiceTest {

    private static final Airport NCE = new Airport("NCE", "LFMN", "Nice Côte d'Azur", "Nice", "FR", "Europe/Paris",
        43.6584, 7.2159);
    private static final Airport LHR = new Airport("LHR", "EGLL", "London Heathrow", "London", "GB", "Europe/London",
        51.4700, -0.4543);
    private static final Airport JFK = new Airport("JFK", "KJFK", "New York JFK", "New York", "US",
        "America/New_York", 40.6413, -73.7781);
    private static final Airline BA = new Airline("BA", "BAW", "British Airways", "GB", "ONEWORLD");

    private final PricingService pricing = new PricingService();

    private static final List<TaxRule> RULES = List.of(
        flat("FR-DEP", TaxKind.DEPARTURE_COUNTRY, "FR", null, null, "7.40"),
        flat("GB-DEP", TaxKind.DEPARTURE_COUNTRY, "GB", null, null, "15.00"),
        flat("GB-ARR", TaxKind.ARRIVAL_COUNTRY, "GB", null, null, "0.80"),
        flat("FR-ARR", TaxKind.ARRIVAL_COUNTRY, "FR", null, null, "1.20"),
        flat("NCE-PSC", TaxKind.AIRPORT_PSC, null, "NCE", null, "12.80"),
        flat("LHR-PSC", TaxKind.AIRPORT_PSC, null, "LHR", null, "29.90"),
        band("YQ-DEF-1", null, 0, 1500, "18.00"),
        band("YQ-DEF-2", null, 1500, 3500, "32.00"),
        band("YQ-DEF-3", null, 3500, 7000, "95.00"),
        band("YQ-BA-1", "BA", 0, 1500, "24.00"),
        flat("FR-VAT", TaxKind.DOMESTIC_VAT, "FR", null, null, "10.00"));

    @Test
    void greatCircleDistanceMatchesKnownValues() {
        assertThat(Geo.distanceKm(NCE, LHR)).isCloseTo(1039, within(5.0));
        assertThat(Geo.distanceKm(NCE, JFK)).isCloseTo(6407, within(30.0));
    }

    @Test
    void economyNiceToLondonUsesDepartureArrivalPscAndAirlineFuelBand() {
        Fare fare = fare(flight(NCE, LHR), "Y", Cabin.ECONOMY, "89.00");

        Pricing p = pricing.price(fare, 1, RULES);

        // 7.40 (FR departure) + 0.80 (GB arrival) + 12.80 (NCE PSC) + 24.00 (BA fuel surcharge, 0-1500 km)
        assertThat(p.taxesPerPassenger()).isEqualByComparingTo("45.00");
        assertThat(p.basePerPassenger()).isEqualByComparingTo("89.00");
        assertThat(p.perPassenger()).isEqualByComparingTo("134.00");
        assertThat(p.total()).isEqualByComparingTo("134.00");
        assertThat(p.currency()).isEqualTo("EUR");
        assertThat(p.total().scale()).isEqualTo(2);
    }

    @Test
    void totalsScaleWithPassengersAndPremiumCabinsPayHigherTaxes() {
        Fare fare = fare(flight(LHR, NCE), "J", Cabin.BUSINESS, "420.00");

        Pricing p = pricing.price(fare, 3, RULES);

        // 15.00 (GB departure) + 1.20 (FR arrival) + 29.90 (LHR PSC) + 24.00 × 1.60 (BA fuel, business factor)
        assertThat(p.passengers()).isEqualTo(3);
        assertThat(p.taxesPerPassenger()).isEqualByComparingTo("84.50");
        assertThat(p.base()).isEqualByComparingTo("1260.00");
        assertThat(p.taxes()).isEqualByComparingTo("253.50");
        assertThat(p.total()).isEqualByComparingTo("1513.50");
    }

    @Test
    void airlineWithoutOwnFuelSurchargeFallsBackToDefaultBand() {
        Airline af = new Airline("AF", "AFR", "Air France", "FR", "SKYTEAM");
        Fare fare = fare(new Flight("AF1234", af, NCE, JFK, LocalDateTime.of(2026, 11, 5, 10, 0),
            LocalDateTime.of(2026, 11, 5, 19, 0), "A350", 540), "K", Cabin.ECONOMY, "300.00");

        Pricing p = pricing.price(fare, 1, RULES);

        // 7.40 (FR departure) + 12.80 (NCE PSC) + 95.00 (default fuel band 3500-7000 km); no US rules in this table
        assertThat(p.taxesPerPassenger()).isEqualByComparingTo("115.20");
    }

    @Test
    void domesticFlightsAddVatOnTheBaseFareRoundedHalfEven() {
        Airport cdg = new Airport("CDG", "LFPG", "Paris CDG", "Paris", "FR", "Europe/Paris", 49.0097, 2.5479);
        Airline af = new Airline("AF", "AFR", "Air France", "FR", "SKYTEAM");
        Fare fare = fare(new Flight("AF7700", af, NCE, cdg, LocalDateTime.of(2026, 11, 5, 6, 0),
            LocalDateTime.of(2026, 11, 5, 7, 30), "A320", 90), "Y", Cabin.ECONOMY, "100.25");

        Pricing p = pricing.price(fare, 1, RULES);

        // 7.40 + 1.20 + 12.80 + 18.00 (default fuel band) + 10% VAT of 100.25 = 10.025 -> 10.02 (half-even)
        assertThat(p.taxesPerPassenger()).isEqualByComparingTo("49.42");
    }

    private static Flight flight(Airport from, Airport to) {
        return new Flight("BA340", BA, from, to, LocalDateTime.of(2026, 11, 5, 7, 35),
            LocalDateTime.of(2026, 11, 5, 9, 45), "A320", 130);
    }

    private static Fare fare(Flight flight, String bookingClass, Cabin cabin, String base) {
        return new Fare(flight, bookingClass, cabin, new BigDecimal(base), "EUR", false, true, 23, 40);
    }

    private static TaxRule flat(String code, TaxKind kind, String country, String airport, String airline,
                                String amount) {
        return new TaxRule(code, kind, country, airport, airline, null, null, new BigDecimal(amount), "EUR");
    }

    private static TaxRule band(String code, String airline, int min, Integer max, String amount) {
        return new TaxRule(code, TaxKind.FUEL_SURCHARGE, null, null, airline, min, max, new BigDecimal(amount), "EUR");
    }
}
