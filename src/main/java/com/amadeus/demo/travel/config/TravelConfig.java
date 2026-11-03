package com.amadeus.demo.travel.config;

import java.time.Clock;

import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.info.Info;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TravelConfig {

    @Bean
    public Clock clock() {
        return Clock.systemDefaultZone();
    }

    @Bean
    public OpenAPI travelOpenApi() {
        return new OpenAPI().info(new Info()
            .title("Travel Shopping & Reservation API")
            .version("1.0.0")
            .description("Reference data, schedules, flight shopping (priced offers) and reservations."));
    }
}
