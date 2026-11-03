package com.amadeus.demo.travel.model;

import java.math.BigDecimal;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

/**
 * One row of the tax rule table; see {@link TaxKind} for which columns each kind uses.
 */
@Entity
@Table(name = "tax_rule")
public class TaxRule {

    @Id
    @Column(length = 16)
    private String code;

    @Enumerated(EnumType.STRING)
    private TaxKind kind;

    @Column(length = 2)
    private String country;

    @Column(name = "airport_iata", length = 3)
    private String airportIata;

    @Column(name = "airline_iata", length = 2)
    private String airlineIata;

    @Column(name = "min_distance_km")
    private Integer minDistanceKm;

    @Column(name = "max_distance_km")
    private Integer maxDistanceKm;

    @Column(precision = 10, scale = 2)
    private BigDecimal amount;

    @Column(length = 3)
    private String currency;

    protected TaxRule() {
    }

    public TaxRule(String code, TaxKind kind, String country, String airportIata, String airlineIata,
                   Integer minDistanceKm, Integer maxDistanceKm, BigDecimal amount, String currency) {
        this.code = code;
        this.kind = kind;
        this.country = country;
        this.airportIata = airportIata;
        this.airlineIata = airlineIata;
        this.minDistanceKm = minDistanceKm;
        this.maxDistanceKm = maxDistanceKm;
        this.amount = amount;
        this.currency = currency;
    }

    public String getCode() {
        return code;
    }

    public TaxKind getKind() {
        return kind;
    }

    public String getCountry() {
        return country;
    }

    public String getAirportIata() {
        return airportIata;
    }

    public String getAirlineIata() {
        return airlineIata;
    }

    public Integer getMinDistanceKm() {
        return minDistanceKm;
    }

    public Integer getMaxDistanceKm() {
        return maxDistanceKm;
    }

    public BigDecimal getAmount() {
        return amount;
    }

    public String getCurrency() {
        return currency;
    }
}
