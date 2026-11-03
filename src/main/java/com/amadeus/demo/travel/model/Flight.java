package com.amadeus.demo.travel.model;

import java.time.LocalDateTime;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;

/**
 * A scheduled flight. {@code departure} and {@code arrival} are stored as UTC wall-clock timestamps; the API renders
 * them in the local time of the origin / destination airport.
 */
@Entity
@Table(name = "flight")
public class Flight {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "flight_number", length = 7)
    private String flightNumber;

    @ManyToOne(optional = false)
    @JoinColumn(name = "airline_iata")
    private Airline airline;

    @ManyToOne(optional = false)
    @JoinColumn(name = "origin_iata")
    private Airport origin;

    @ManyToOne(optional = false)
    @JoinColumn(name = "destination_iata")
    private Airport destination;

    @Column(name = "departure")
    private LocalDateTime departureUtc;

    @Column(name = "arrival")
    private LocalDateTime arrivalUtc;

    private String aircraft;

    @Column(name = "duration_minutes")
    private int durationMinutes;


    protected Flight() {
    }

    public Flight(String flightNumber, Airline airline, Airport origin, Airport destination,
                  LocalDateTime departureUtc, LocalDateTime arrivalUtc, String aircraft, int durationMinutes) {
        this.flightNumber = flightNumber;
        this.airline = airline;
        this.origin = origin;
        this.destination = destination;
        this.departureUtc = departureUtc;
        this.arrivalUtc = arrivalUtc;
        this.aircraft = aircraft;
        this.durationMinutes = durationMinutes;
    }

    public Long getId() {
        return id;
    }

    public String getFlightNumber() {
        return flightNumber;
    }

    public Airline getAirline() {
        return airline;
    }

    public Airport getOrigin() {
        return origin;
    }

    public Airport getDestination() {
        return destination;
    }

    public LocalDateTime getDepartureUtc() {
        return departureUtc;
    }

    public LocalDateTime getArrivalUtc() {
        return arrivalUtc;
    }

    public String getAircraft() {
        return aircraft;
    }

    public int getDurationMinutes() {
        return durationMinutes;
    }
}
