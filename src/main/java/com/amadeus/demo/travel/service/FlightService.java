package com.amadeus.demo.travel.service;

import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.List;

import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.model.Flight;
import com.amadeus.demo.travel.repository.FlightRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
@Transactional(readOnly = true)
public class FlightService {

    private final FlightRepository flights;
    private final ReferenceDataService referenceData;

    public FlightService(FlightRepository flights, ReferenceDataService referenceData) {
        this.flights = flights;
        this.referenceData = referenceData;
    }

    /** Flights leaving {@code origin} for {@code destination} on the given local date (local time of the origin). */
    public List<Flight> search(String origin, String destination, LocalDate date) {
        RouteDay routeDay = RouteDay.of(origin, destination, date, referenceData);
        return flights.findRoute(routeDay.origin().getIata(), routeDay.destination().getIata(),
            routeDay.fromUtc(), routeDay.toUtc());
    }

    public Flight get(Long id) {
        return flights.findById(id).orElseThrow(() -> new NotFoundException("Flight " + id + " not found"));
    }

    /** The UTC departure window of one local day at the origin airport. */
    record RouteDay(Airport origin, Airport destination, LocalDateTime fromUtc, LocalDateTime toUtc) {

        static RouteDay of(String origin, String destination, LocalDate date, ReferenceDataService referenceData) {
            Airport from = lookup(origin, "origin", referenceData);
            Airport to = lookup(destination, "destination", referenceData);
            if (from.getIata().equals(to.getIata())) {
                throw new BadRequestException("origin and destination must differ");
            }
            ZoneId zone = ZoneId.of(from.getTimezone());
            LocalDateTime fromUtc = date.atStartOfDay(zone).withZoneSameInstant(ZoneOffset.UTC).toLocalDateTime();
            LocalDateTime toUtc = date.plusDays(1).atStartOfDay(zone).withZoneSameInstant(ZoneOffset.UTC)
                .toLocalDateTime();
            return new RouteDay(from, to, fromUtc, toUtc);
        }

        private static Airport lookup(String iata, String role, ReferenceDataService referenceData) {
            try {
                return referenceData.airport(iata);
            } catch (NotFoundException e) {
                throw new BadRequestException("Unknown " + role + " airport: " + iata);
            }
        }
    }
}
