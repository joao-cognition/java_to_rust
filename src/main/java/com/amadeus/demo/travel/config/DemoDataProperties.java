package com.amadeus.demo.travel.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

/**
 * Parameters of the deterministic demo data generated at startup.
 *
 * @param seed     seed of the random generator; same seed, same data
 * @param days     number of schedule days, starting the day after the server starts
 * @param bookings number of confirmed bookings to create
 */
@ConfigurationProperties(prefix = "travel.demo")
public record DemoDataProperties(long seed, int days, int bookings) {
}
