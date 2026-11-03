package com.amadeus.demo.travel.model;

import java.math.BigDecimal;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.Table;

/**
 * A bookable fare (booking class) on one flight, with its inventory.
 */
@Entity
@Table(name = "fare")
public class Fare {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @ManyToOne(optional = false)
    @JoinColumn(name = "flight_id")
    private Flight flight;

    @Column(name = "booking_class", length = 1)
    private String bookingClass;

    @Enumerated(EnumType.STRING)
    private Cabin cabin;

    @Column(name = "base_amount", precision = 10, scale = 2)
    private BigDecimal baseAmount;

    @Column(length = 3)
    private String currency;

    private boolean refundable;

    private boolean changeable;

    @Column(name = "baggage_included_kg")
    private int baggageIncludedKg;

    @Column(name = "seats_available")
    private int seatsAvailable;


    protected Fare() {
    }

    public Fare(Flight flight, String bookingClass, Cabin cabin, BigDecimal baseAmount, String currency,
                boolean refundable, boolean changeable, int baggageIncludedKg, int seatsAvailable) {
        this.flight = flight;
        this.bookingClass = bookingClass;
        this.cabin = cabin;
        this.baseAmount = baseAmount;
        this.currency = currency;
        this.refundable = refundable;
        this.changeable = changeable;
        this.baggageIncludedKg = baggageIncludedKg;
        this.seatsAvailable = seatsAvailable;
    }

    public Long getId() {
        return id;
    }

    public Flight getFlight() {
        return flight;
    }

    public String getBookingClass() {
        return bookingClass;
    }

    public Cabin getCabin() {
        return cabin;
    }

    public BigDecimal getBaseAmount() {
        return baseAmount;
    }

    public String getCurrency() {
        return currency;
    }

    public boolean isRefundable() {
        return refundable;
    }

    public boolean isChangeable() {
        return changeable;
    }

    public int getBaggageIncludedKg() {
        return baggageIncludedKg;
    }

    public int getSeatsAvailable() {
        return seatsAvailable;
    }

    public void setSeatsAvailable(int seatsAvailable) {
        this.seatsAvailable = seatsAvailable;
    }
}
