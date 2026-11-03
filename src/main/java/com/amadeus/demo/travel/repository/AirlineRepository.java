package com.amadeus.demo.travel.repository;

import java.util.List;

import com.amadeus.demo.travel.model.Airline;
import org.springframework.data.jpa.repository.JpaRepository;

public interface AirlineRepository extends JpaRepository<Airline, String> {

    List<Airline> findAllByOrderByIataAsc();
}
