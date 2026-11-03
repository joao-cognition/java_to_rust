package com.amadeus.demo.travel.repository;

import java.time.LocalDateTime;
import java.util.List;

import com.amadeus.demo.travel.model.Flight;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface FlightRepository extends JpaRepository<Flight, Long> {

    @Query("""
        select f from Flight f
        join fetch f.airline join fetch f.origin join fetch f.destination
        where f.origin.iata = :origin and f.destination.iata = :destination
          and f.departureUtc >= :fromUtc and f.departureUtc < :toUtc
        order by f.departureUtc, f.id
        """)
    List<Flight> findRoute(@Param("origin") String origin, @Param("destination") String destination,
                           @Param("fromUtc") LocalDateTime fromUtc, @Param("toUtc") LocalDateTime toUtc);
}
