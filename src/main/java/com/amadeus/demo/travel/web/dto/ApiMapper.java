package com.amadeus.demo.travel.web.dto;

import java.time.LocalDateTime;
import java.time.OffsetDateTime;
import java.time.ZoneId;
import java.time.ZoneOffset;

import com.amadeus.demo.travel.model.Airline;
import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.model.Booking;
import com.amadeus.demo.travel.model.BookingSegment;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Flight;
import com.amadeus.demo.travel.model.Traveler;
import com.amadeus.demo.travel.service.Offer;
import com.amadeus.demo.travel.service.Pricing;
import com.amadeus.demo.travel.web.dto.ApiDtos.AirlineDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.AirportDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.BookingDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.FareDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.FlightDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.OfferDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.PricingDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.SegmentDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.TravelerDto;

/** Entity to JSON mapping. Flight times are rendered in the local time of the airport they refer to. */
public final class ApiMapper {

    private ApiMapper() {
    }

    public static AirportDto airport(Airport a) {
        return new AirportDto(a.getIata(), a.getIcao(), a.getName(), a.getCity(), a.getCountry(), a.getTimezone(),
            a.getLatitude(), a.getLongitude());
    }

    public static AirlineDto airline(Airline a) {
        return new AirlineDto(a.getIata(), a.getIcao(), a.getName(), a.getCountry(), a.getAlliance());
    }

    public static FlightDto flight(Flight f) {
        return new FlightDto(f.getId(), f.getFlightNumber(), airline(f.getAirline()), airport(f.getOrigin()),
            airport(f.getDestination()),
            atAirport(f.getDepartureUtc(), f.getOrigin()),
            atAirport(f.getArrivalUtc(), f.getDestination()),
            f.getAircraft(), f.getDurationMinutes());
    }

    public static FareDto fare(Fare f) {
        return new FareDto(f.getBookingClass(), f.getCabin(), f.getBaseAmount(), f.getCurrency(), f.isRefundable(),
            f.isChangeable(), f.getBaggageIncludedKg(), f.getSeatsAvailable());
    }

    public static PricingDto pricing(Pricing p) {
        return new PricingDto(p.passengers(), p.basePerPassenger(), p.taxesPerPassenger(), p.perPassenger(),
            p.base(), p.taxes(), p.total(), p.currency());
    }

    public static OfferDto offer(Offer o) {
        return new OfferDto(o.offerId(), flight(o.flight()), fare(o.fare()), pricing(o.pricing()));
    }

    public static TravelerDto traveler(Traveler t) {
        return new TravelerDto(t.getId(), t.getFirstName(), t.getLastName(), t.getDateOfBirth(), t.getEmail(),
            t.getNationality(), t.getFrequentFlyerNumber());
    }

    public static SegmentDto segment(BookingSegment s) {
        return new SegmentDto(flight(s.getFlight()), s.getBookingClass(), s.getCabin());
    }

    public static BookingDto booking(Booking b) {
        return new BookingDto(b.getRecordLocator(), b.getStatus(), utc(b.getCreatedAt()),
            b.getTravelers().stream().map(ApiMapper::traveler).toList(),
            b.getSegments().stream().map(ApiMapper::segment).toList(),
            b.getTotalAmount(), b.getCurrency());
    }

    static OffsetDateTime atAirport(LocalDateTime utc, Airport airport) {
        return utc.atOffset(ZoneOffset.UTC).atZoneSameInstant(ZoneId.of(airport.getTimezone())).toOffsetDateTime();
    }

    static OffsetDateTime utc(LocalDateTime utc) {
        return utc.atOffset(ZoneOffset.UTC);
    }
}
