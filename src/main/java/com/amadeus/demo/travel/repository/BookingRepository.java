package com.amadeus.demo.travel.repository;

import java.util.List;
import java.util.Optional;

import com.amadeus.demo.travel.model.Booking;
import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

public interface BookingRepository extends JpaRepository<Booking, Long> {

    Optional<Booking> findByRecordLocator(String recordLocator);

    boolean existsByRecordLocator(String recordLocator);

    @Query("""
        select distinct b from Booking b join b.travelers t
        where lower(t.lastName) like lower(concat(:prefix, '%'))
        order by b.createdAt desc, b.id desc
        """)
    List<Booking> findByTravelerLastNamePrefix(@Param("prefix") String prefix);
}
