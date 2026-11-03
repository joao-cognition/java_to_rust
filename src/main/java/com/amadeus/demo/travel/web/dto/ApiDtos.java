package com.amadeus.demo.travel.web.dto;

import java.math.BigDecimal;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.util.List;

import com.amadeus.demo.travel.model.BookingStatus;
import com.amadeus.demo.travel.model.Cabin;
import com.fasterxml.jackson.annotation.JsonFormat;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;
import jakarta.validation.constraints.Size;

/** JSON shapes of the REST API (camelCase, ISO-8601 with offset, money with two decimals). */
public final class ApiDtos {

    /** ISO-8601 local time with numeric offset, e.g. {@code 2026-11-05T07:35:00+01:00}. */
    public static final String TIMESTAMP_PATTERN = "yyyy-MM-dd'T'HH:mm:ssxxx";

    private ApiDtos() {
    }

    public record AirportDto(String iata, String icao, String name, String city, String country, String timezone,
                             double latitude, double longitude) {
    }

    public record AirlineDto(String iata, String icao, String name, String country, String alliance) {
    }

    public record FlightDto(Long id, String flightNumber, AirlineDto airline, AirportDto origin,
                            AirportDto destination,
                            @JsonFormat(pattern = TIMESTAMP_PATTERN) OffsetDateTime departure,
                            @JsonFormat(pattern = TIMESTAMP_PATTERN) OffsetDateTime arrival,
                            String aircraft, int durationMinutes) {
    }

    public record FareDto(String bookingClass, Cabin cabin, BigDecimal baseAmount, String currency,
                          boolean refundable, boolean changeable, int baggageIncludedKg, int seatsAvailable) {
    }

    public record PricingDto(int passengers, BigDecimal basePerPassenger, BigDecimal taxesPerPassenger,
                             BigDecimal perPassenger, BigDecimal base, BigDecimal taxes, BigDecimal total,
                             String currency) {
    }

    public record OfferDto(String offerId, FlightDto flight, FareDto fare, PricingDto pricing) {
    }

    public record TravelerDto(Long id, String firstName, String lastName, LocalDate dateOfBirth, String email,
                              String nationality, String frequentFlyerNumber) {
    }

    public record SegmentDto(FlightDto flight, String bookingClass, Cabin cabin) {
    }

    public record BookingDto(String recordLocator, BookingStatus status,
                             @JsonFormat(pattern = TIMESTAMP_PATTERN) OffsetDateTime createdAt,
                             List<TravelerDto> travelers, List<SegmentDto> segments, BigDecimal totalAmount,
                             String currency) {
    }

    public record CreateBookingRequest(
        @NotNull @Pattern(regexp = "\\d+-[A-Z]", message = "must look like <flightId>-<bookingClass>, e.g. 123-Y")
        String offerId,
        @NotEmpty @Size(max = 9) List<@NotNull Long> travelerIds) {
    }
}
