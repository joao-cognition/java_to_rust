package com.amadeus.demo.travel.repository;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Optional;

import com.amadeus.demo.travel.model.Fare;
import jakarta.persistence.LockModeType;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Lock;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface FareRepository extends JpaRepository<Fare, Long> {

    /** All fares of all flights on a route inside a UTC departure window, with flight, airline and airports loaded. */
    @Query("""
        select fa from Fare fa
        join fetch fa.flight f join fetch f.airline join fetch f.origin join fetch f.destination
        where f.origin.iata = :origin and f.destination.iata = :destination
          and f.departureUtc >= :fromUtc and f.departureUtc < :toUtc
        order by f.departureUtc, f.id, fa.bookingClass
        """)
    List<Fare> findForRoute(@Param("origin") String origin, @Param("destination") String destination,
                            @Param("fromUtc") LocalDateTime fromUtc, @Param("toUtc") LocalDateTime toUtc);

    Optional<Fare> findByFlightIdAndBookingClass(Long flightId, String bookingClass);

    @Lock(LockModeType.PESSIMISTIC_WRITE)
    @Query("select fa from Fare fa where fa.flight.id = :flightId and fa.bookingClass = :bookingClass")
    Optional<Fare> lockByFlightIdAndBookingClass(@Param("flightId") Long flightId,
                                                 @Param("bookingClass") String bookingClass);

    @Lock(LockModeType.PESSIMISTIC_WRITE)
    @Query("select fa from Fare fa where fa.id = :id")
    Optional<Fare> lockById(@Param("id") Long id);
}
