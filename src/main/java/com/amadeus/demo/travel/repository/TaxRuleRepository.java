package com.amadeus.demo.travel.repository;

import com.amadeus.demo.travel.model.TaxRule;
import org.springframework.data.jpa.repository.JpaRepository;

public interface TaxRuleRepository extends JpaRepository<TaxRule, String> {
}
