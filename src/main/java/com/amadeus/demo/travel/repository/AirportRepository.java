package com.amadeus.demo.travel.repository;

import java.util.List;

import com.amadeus.demo.travel.model.Airport;
import org.springframework.data.jpa.repository.JpaRepository;

public interface AirportRepository extends JpaRepository<Airport, String> {

    List<Airport> findAllByOrderByIataAsc();
}
