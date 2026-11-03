package com.amadeus.demo.travel.repository;

import java.util.List;

import com.amadeus.demo.travel.model.Traveler;
import org.springframework.data.jpa.repository.JpaRepository;

public interface TravelerRepository extends JpaRepository<Traveler, Long> {

    List<Traveler> findByLastNameStartingWithIgnoreCaseOrderByLastNameAscFirstNameAsc(String lastNamePrefix);
}
