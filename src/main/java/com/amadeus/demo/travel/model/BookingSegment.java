package com.amadeus.demo.travel.model;

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

@Entity
@Table(name = "booking_segment")
public class BookingSegment {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @ManyToOne(optional = false)
    @JoinColumn(name = "booking_id")
    private Booking booking;

    @ManyToOne(optional = false)
    @JoinColumn(name = "fare_id")
    private Fare fare;

    @Column(name = "booking_class", length = 1)
    private String bookingClass;

    @Enumerated(EnumType.STRING)
    private Cabin cabin;

    protected BookingSegment() {
    }

    BookingSegment(Booking booking, Fare fare) {
        this.booking = booking;
        this.fare = fare;
        this.bookingClass = fare.getBookingClass();
        this.cabin = fare.getCabin();
    }

    public Long getId() {
        return id;
    }

    public Booking getBooking() {
        return booking;
    }

    public Fare getFare() {
        return fare;
    }

    public Flight getFlight() {
        return fare.getFlight();
    }

    public String getBookingClass() {
        return bookingClass;
    }

    public Cabin getCabin() {
        return cabin;
    }
}
