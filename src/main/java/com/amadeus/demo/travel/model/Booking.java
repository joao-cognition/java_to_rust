package com.amadeus.demo.travel.model;

import java.math.BigDecimal;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.JoinTable;
import jakarta.persistence.ManyToMany;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderBy;
import jakarta.persistence.Table;
import org.hibernate.annotations.Fetch;
import org.hibernate.annotations.FetchMode;

/**
 * A reservation (PNR). {@code createdAt} is a UTC wall-clock timestamp.
 */
@Entity
@Table(name = "booking")
public class Booking {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "record_locator", length = 6, unique = true)
    private String recordLocator;

    @Enumerated(EnumType.STRING)
    private BookingStatus status;

    @Column(name = "created_at")
    private LocalDateTime createdAt;

    @Column(name = "total_amount", precision = 12, scale = 2)
    private BigDecimal totalAmount;

    @Column(length = 3)
    private String currency;

    @ManyToMany
    @Fetch(FetchMode.SUBSELECT)
    @JoinTable(name = "booking_traveler",
        joinColumns = @JoinColumn(name = "booking_id"),
        inverseJoinColumns = @JoinColumn(name = "traveler_id"))
    @OrderBy("id")
    private Set<Traveler> travelers = new LinkedHashSet<>();

    @OneToMany(mappedBy = "booking", cascade = CascadeType.ALL, orphanRemoval = true)
    @Fetch(FetchMode.SUBSELECT)
    @OrderBy("id")
    private List<BookingSegment> segments = new ArrayList<>();

    protected Booking() {
    }

    public Booking(String recordLocator, LocalDateTime createdAt, List<Traveler> travelers, Fare fare,
                   BigDecimal totalAmount, String currency) {
        this.recordLocator = recordLocator;
        this.status = BookingStatus.CONFIRMED;
        this.createdAt = createdAt;
        this.travelers.addAll(travelers);
        this.segments.add(new BookingSegment(this, fare));
        this.totalAmount = totalAmount;
        this.currency = currency;
    }

    public Long getId() {
        return id;
    }

    public String getRecordLocator() {
        return recordLocator;
    }

    public BookingStatus getStatus() {
        return status;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public BigDecimal getTotalAmount() {
        return totalAmount;
    }

    public String getCurrency() {
        return currency;
    }

    public Set<Traveler> getTravelers() {
        return travelers;
    }

    public List<BookingSegment> getSegments() {
        return segments;
    }

    public void cancel() {
        this.status = BookingStatus.CANCELLED;
    }
}
