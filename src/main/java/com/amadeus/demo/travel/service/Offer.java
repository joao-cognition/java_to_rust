package com.amadeus.demo.travel.service;

import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Flight;

/** A priced, bookable fare. The offer id is {@code <flightId>-<bookingClass>}. */
public record Offer(String offerId, Flight flight, Fare fare, Pricing pricing) {

    public static String offerId(Fare fare) {
        return fare.getFlight().getId() + "-" + fare.getBookingClass();
    }
}
