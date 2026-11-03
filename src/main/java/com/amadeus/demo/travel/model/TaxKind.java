package com.amadeus.demo.travel.model;

/**
 * Kinds of rules in the {@code tax_rule} table.
 * <ul>
 * <li>{@link #DEPARTURE_COUNTRY}: flat per-passenger tax levied by the origin country ({@code country}).</li>
 * <li>{@link #ARRIVAL_COUNTRY}: flat per-passenger tax levied by the destination country ({@code country}).</li>
 * <li>{@link #AIRPORT_PSC}: passenger service charge of the departure airport ({@code airport_iata}).</li>
 * <li>{@link #FUEL_SURCHARGE}: airline surcharge by great-circle distance band ({@code airline_iata} or NULL for the
 * default table, {@code min_distance_km} inclusive, {@code max_distance_km} exclusive or NULL).</li>
 * <li>{@link #DOMESTIC_VAT}: percentage of the base fare on domestic flights ({@code country}, {@code amount} is a
 * percentage).</li>
 * </ul>
 */
public enum TaxKind {
    DEPARTURE_COUNTRY, ARRIVAL_COUNTRY, AIRPORT_PSC, FUEL_SURCHARGE, DOMESTIC_VAT
}
