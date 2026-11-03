package com.amadeus.demo.travel.service;

import java.util.List;

import com.amadeus.demo.travel.model.Traveler;
import com.amadeus.demo.travel.repository.TravelerRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
@Transactional(readOnly = true)
public class TravelerService {

    private final TravelerRepository travelers;

    public TravelerService(TravelerRepository travelers) {
        this.travelers = travelers;
    }

    public Traveler get(Long id) {
        return travelers.findById(id).orElseThrow(() -> new NotFoundException("Traveler " + id + " not found"));
    }

    public List<Traveler> searchByLastName(String lastNamePrefix) {
        return travelers.findByLastNameStartingWithIgnoreCaseOrderByLastNameAscFirstNameAsc(lastNamePrefix.trim());
    }
}
