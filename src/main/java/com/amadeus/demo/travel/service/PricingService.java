package com.amadeus.demo.travel.service;

import java.math.BigDecimal;
import java.math.RoundingMode;
import java.util.List;

import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.model.Cabin;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Flight;
import com.amadeus.demo.travel.model.TaxKind;
import com.amadeus.demo.travel.model.TaxRule;
import org.springframework.stereotype.Service;

/**
 * Fare pricing in plain Java (the CPU hot path of the service, kept free of SQL so that a port can be compared
 * like for like).
 * <p>
 * Taxes per passenger = departure-country tax + arrival-country tax + departure-airport passenger service charge
 * + airline fuel surcharge for the great-circle distance band (scaled by cabin) + domestic VAT on the base fare when
 * origin and destination are in the same country. Amounts are rounded half-even to cents.
 */
@Service
public class PricingService {

    private static final BigDecimal HUNDRED = BigDecimal.valueOf(100);

    /** Fuel surcharge multiplier per cabin. */
    static BigDecimal cabinFactor(Cabin cabin) {
        return switch (cabin) {
            case ECONOMY -> BigDecimal.ONE;
            case PREMIUM_ECONOMY -> new BigDecimal("1.25");
            case BUSINESS -> new BigDecimal("1.60");
            case FIRST -> new BigDecimal("2.00");
        };
    }

    public Pricing price(Fare fare, int passengers, List<TaxRule> rules) {
        Flight flight = fare.getFlight();
        double distanceKm = Geo.distanceKm(flight.getOrigin(), flight.getDestination());
        return price(fare, passengers, rules, distanceKm);
    }

    /** Variant taking a precomputed distance so that callers pricing several fares of one flight compute it once. */
    public Pricing price(Fare fare, int passengers, List<TaxRule> rules, double distanceKm) {
        Flight flight = fare.getFlight();
        Airport origin = flight.getOrigin();
        Airport destination = flight.getDestination();
        String airline = flight.getAirline().getIata();
        BigDecimal base = fare.getBaseAmount().setScale(2, RoundingMode.HALF_EVEN);

        BigDecimal taxes = BigDecimal.ZERO;
        taxes = taxes.add(flat(rules, TaxKind.DEPARTURE_COUNTRY, origin.getCountry()));
        taxes = taxes.add(flat(rules, TaxKind.ARRIVAL_COUNTRY, destination.getCountry()));
        taxes = taxes.add(passengerServiceCharge(rules, origin.getIata()));
        taxes = taxes.add(fuelSurcharge(rules, airline, distanceKm).multiply(cabinFactor(fare.getCabin())));
        if (origin.getCountry().equals(destination.getCountry())) {
            BigDecimal vatPercent = flat(rules, TaxKind.DOMESTIC_VAT, origin.getCountry());
            taxes = taxes.add(base.multiply(vatPercent).divide(HUNDRED, 6, RoundingMode.HALF_EVEN));
        }
        taxes = taxes.setScale(2, RoundingMode.HALF_EVEN);

        BigDecimal perPassenger = base.add(taxes);
        BigDecimal pax = BigDecimal.valueOf(passengers);
        return new Pricing(passengers, base, taxes, perPassenger,
            base.multiply(pax).setScale(2, RoundingMode.HALF_EVEN),
            taxes.multiply(pax).setScale(2, RoundingMode.HALF_EVEN),
            perPassenger.multiply(pax).setScale(2, RoundingMode.HALF_EVEN),
            fare.getCurrency());
    }

    private static BigDecimal flat(List<TaxRule> rules, TaxKind kind, String country) {
        BigDecimal sum = BigDecimal.ZERO;
        for (TaxRule rule : rules) {
            if (rule.getKind() == kind && country.equals(rule.getCountry())) {
                sum = sum.add(rule.getAmount());
            }
        }
        return sum;
    }

    private static BigDecimal passengerServiceCharge(List<TaxRule> rules, String airportIata) {
        BigDecimal sum = BigDecimal.ZERO;
        for (TaxRule rule : rules) {
            if (rule.getKind() == TaxKind.AIRPORT_PSC && airportIata.equals(rule.getAirportIata())) {
                sum = sum.add(rule.getAmount());
            }
        }
        return sum;
    }

    /** Airline-specific band if the airline publishes one, otherwise the default band (airline_iata NULL). */
    private static BigDecimal fuelSurcharge(List<TaxRule> rules, String airlineIata, double distanceKm) {
        BigDecimal airlineBand = null;
        BigDecimal defaultBand = null;
        for (TaxRule rule : rules) {
            if (rule.getKind() != TaxKind.FUEL_SURCHARGE || !inBand(rule, distanceKm)) {
                continue;
            }
            if (airlineIata.equals(rule.getAirlineIata())) {
                airlineBand = rule.getAmount();
            } else if (rule.getAirlineIata() == null) {
                defaultBand = rule.getAmount();
            }
        }
        if (airlineBand != null) {
            return airlineBand;
        }
        return defaultBand != null ? defaultBand : BigDecimal.ZERO;
    }

    private static boolean inBand(TaxRule rule, double distanceKm) {
        int min = rule.getMinDistanceKm() == null ? 0 : rule.getMinDistanceKm();
        if (distanceKm < min) {
            return false;
        }
        return rule.getMaxDistanceKm() == null || distanceKm < rule.getMaxDistanceKm();
    }
}
