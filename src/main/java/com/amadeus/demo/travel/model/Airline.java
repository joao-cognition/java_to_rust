package com.amadeus.demo.travel.model;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

@Entity
@Table(name = "airline")
public class Airline {

    @Id
    @Column(length = 2)
    private String iata;

    @Column(length = 3)
    private String icao;

    private String name;

    @Column(length = 2)
    private String country;

    private String alliance;


    protected Airline() {
    }

    public Airline(String iata, String icao, String name, String country, String alliance) {
        this.iata = iata;
        this.icao = icao;
        this.name = name;
        this.country = country;
        this.alliance = alliance;
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

    public String getCountry() {
        return country;
    }

    public String getAlliance() {
        return alliance;
    }
}
