package com.amadeus.demo.travel.service;

import java.math.BigDecimal;

/** Priced amounts for one fare and a number of passengers; all amounts have two decimals. */
public record Pricing(
    int passengers,
    BigDecimal basePerPassenger,
    BigDecimal taxesPerPassenger,
    BigDecimal perPassenger,
    BigDecimal base,
    BigDecimal taxes,
    BigDecimal total,
    String currency) {
}
