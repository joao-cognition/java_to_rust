package com.amadeus.demo.travel.model;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

@Entity
@Table(name = "airport")
public class Airport {

    @Id
    @Column(length = 3)
    private String iata;

    @Column(length = 4)
    private String icao;

    private String name;

    private String city;

    @Column(length = 2)
    private String country;

    private String timezone;

    private double latitude;

    private double longitude;


    protected Airport() {
    }

    public Airport(String iata, String icao, String name, String city, String country, String timezone,
                   double latitude, double longitude) {
        this.iata = iata;
        this.icao = icao;
        this.name = name;
        this.city = city;
        this.country = country;
        this.timezone = timezone;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public String getIata() {
        return iata;
    }

    public String getIcao() {
        return icao;
    }

    public String getName() {
        return name;
    }

    public String getCity() {
        return city;
    }

    public String getCountry() {
        return country;
    }

    public String getTimezone() {
        return timezone;
    }

    public double getLatitude() {
        return latitude;
    }

    public double getLongitude() {
        return longitude;
    }
}
