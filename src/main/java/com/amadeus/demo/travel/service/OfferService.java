package com.amadeus.demo.travel.service;

import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import com.amadeus.demo.travel.model.Cabin;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Flight;
import com.amadeus.demo.travel.model.TaxRule;
import com.amadeus.demo.travel.repository.FareRepository;
import com.amadeus.demo.travel.repository.TaxRuleRepository;
import com.amadeus.demo.travel.service.FlightService.RouteDay;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * Flight shopping: every fare on the route and day is priced for the requested passengers and returned as an offer,
 * cheapest first.
 */
@Service
@Transactional(readOnly = true)
public class OfferService {

    public static final int MAX_ADULTS = 9;

    private static final Comparator<Offer> CHEAPEST_FIRST = Comparator
        .comparing((Offer o) -> o.pricing().total())
        .thenComparing(o -> o.flight().getDepartureUtc())
        .thenComparing(o -> o.flight().getId())
        .thenComparing(o -> o.fare().getBookingClass());

    private final FareRepository fares;
    private final TaxRuleRepository taxRules;
    private final ReferenceDataService referenceData;
    private final PricingService pricing;

    public OfferService(FareRepository fares, TaxRuleRepository taxRules, ReferenceDataService referenceData,
                        PricingService pricing) {
        this.fares = fares;
        this.taxRules = taxRules;
        this.referenceData = referenceData;
        this.pricing = pricing;
    }

    public List<Offer> search(String origin, String destination, LocalDate date, int adults, Cabin cabin) {
        if (adults < 1 || adults > MAX_ADULTS) {
            throw new BadRequestException("adults must be between 1 and " + MAX_ADULTS);
        }
        RouteDay routeDay = RouteDay.of(origin, destination, date, referenceData);
        List<Fare> candidates = fares.findForRoute(routeDay.origin().getIata(), routeDay.destination().getIata(),
            routeDay.fromUtc(), routeDay.toUtc());
        List<TaxRule> rules = taxRules.findAll();

        List<Offer> offers = new ArrayList<>(candidates.size());
        Flight currentFlight = null;
        double distanceKm = 0;
        for (Fare fare : candidates) {
            if (cabin != null && fare.getCabin() != cabin) {
                continue;
            }
            if (fare.getSeatsAvailable() < adults) {
                continue;
            }
            Flight flight = fare.getFlight();
            if (flight != currentFlight) {
                currentFlight = flight;
                distanceKm = Geo.distanceKm(flight.getOrigin(), flight.getDestination());
            }
            offers.add(new Offer(Offer.offerId(fare), flight, fare, pricing.price(fare, adults, rules, distanceKm)));
        }
        offers.sort(CHEAPEST_FIRST);
        return offers;
    }

    /** Prices one fare for a number of passengers (used when a booking is created). */
    public Pricing price(Fare fare, int passengers) {
        return pricing.price(fare, passengers, taxRules.findAll());
    }
}
