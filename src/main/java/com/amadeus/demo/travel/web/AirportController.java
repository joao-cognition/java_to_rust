package com.amadeus.demo.travel.web;

import java.util.List;

import com.amadeus.demo.travel.service.ReferenceDataService;
import com.amadeus.demo.travel.web.dto.ApiDtos.AirportDto;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/airports")
@Tag(name = "Reference data")
public class AirportController {

    private final ReferenceDataService referenceData;

    public AirportController(ReferenceDataService referenceData) {
        this.referenceData = referenceData;
    }

    @GetMapping
    @Operation(summary = "All airports, sorted by IATA code")
    public List<AirportDto> list() {
        return referenceData.airports().stream().map(ApiMapper::airport).toList();
    }

    @GetMapping("/{iata}")
    @Operation(summary = "One airport by IATA code")
    public AirportDto get(@PathVariable String iata) {
        return ApiMapper.airport(referenceData.airport(iata));
    }
}
