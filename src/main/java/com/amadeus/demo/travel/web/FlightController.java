package com.amadeus.demo.travel.web;

import java.time.LocalDate;
import java.util.List;

import com.amadeus.demo.travel.service.FlightService;
import com.amadeus.demo.travel.web.dto.ApiDtos.FlightDto;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import org.springframework.format.annotation.DateTimeFormat;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/flights")
@Tag(name = "Schedules")
public class FlightController {

    private final FlightService flights;

    public FlightController(FlightService flights) {
        this.flights = flights;
    }

    @GetMapping
    @Operation(summary = "Flights on a route for one departure date (local date at the origin), sorted by departure")
    public List<FlightDto> search(@RequestParam String origin, @RequestParam String destination,
                                  @RequestParam @DateTimeFormat(iso = DateTimeFormat.ISO.DATE) LocalDate date) {
        return flights.search(origin, destination, date).stream().map(ApiMapper::flight).toList();
    }

    @GetMapping("/{id}")
    @Operation(summary = "One flight by id")
    public FlightDto get(@PathVariable Long id) {
        return ApiMapper.flight(flights.get(id));
    }
}
