package com.amadeus.demo.travel.web;

import java.time.LocalDate;
import java.util.List;

import com.amadeus.demo.travel.model.Cabin;
import com.amadeus.demo.travel.service.OfferService;
import com.amadeus.demo.travel.web.dto.ApiDtos.OfferDto;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import org.springframework.format.annotation.DateTimeFormat;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/offers")
@Tag(name = "Shopping")
public class OfferController {

    private final OfferService offers;

    public OfferController(OfferService offers) {
        this.offers = offers;
    }

    @GetMapping
    @Operation(summary = "Priced offers on a route for one date, cheapest first",
        description = "Every bookable fare of every flight on the route is priced for the given number of adults; "
            + "fares with fewer seats than adults are excluded. Omit cabin to get all cabins.")
    public List<OfferDto> search(@RequestParam String origin, @RequestParam String destination,
                                 @RequestParam @DateTimeFormat(iso = DateTimeFormat.ISO.DATE) LocalDate date,
                                 @RequestParam(defaultValue = "1") @Min(1) @Max(OfferService.MAX_ADULTS) int adults,
                                 @RequestParam(required = false) Cabin cabin) {
        return offers.search(origin, destination, date, adults, cabin).stream().map(ApiMapper::offer).toList();
    }
}
