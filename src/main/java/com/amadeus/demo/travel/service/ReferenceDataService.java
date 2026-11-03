package com.amadeus.demo.travel.service;

import java.util.List;
import java.util.Locale;

import com.amadeus.demo.travel.model.Airline;
import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.repository.AirlineRepository;
import com.amadeus.demo.travel.repository.AirportRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
@Transactional(readOnly = true)
public class ReferenceDataService {

    private final AirportRepository airports;
    private final AirlineRepository airlines;

    public ReferenceDataService(AirportRepository airports, AirlineRepository airlines) {
        this.airports = airports;
        this.airlines = airlines;
    }

    public List<Airport> airports() {
        return airports.findAllByOrderByIataAsc();
    }

    public Airport airport(String iata) {
        return airports.findById(normalize(iata))
            .orElseThrow(() -> new NotFoundException("Airport " + iata + " not found"));
    }

    public List<Airline> airlines() {
        return airlines.findAllByOrderByIataAsc();
    }

    public Airline airline(String iata) {
        return airlines.findById(normalize(iata))
            .orElseThrow(() -> new NotFoundException("Airline " + iata + " not found"));
    }

    static String normalize(String code) {
        return code == null ? "" : code.trim().toUpperCase(Locale.ROOT);
    }
}
