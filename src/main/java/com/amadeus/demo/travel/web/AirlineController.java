package com.amadeus.demo.travel.web;

import java.util.List;

import com.amadeus.demo.travel.service.ReferenceDataService;
import com.amadeus.demo.travel.web.dto.ApiDtos.AirlineDto;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/airlines")
@Tag(name = "Reference data")
public class AirlineController {

    private final ReferenceDataService referenceData;

    public AirlineController(ReferenceDataService referenceData) {
        this.referenceData = referenceData;
    }

    @GetMapping
    @Operation(summary = "All airlines, sorted by IATA code")
    public List<AirlineDto> list() {
        return referenceData.airlines().stream().map(ApiMapper::airline).toList();
    }

    @GetMapping("/{iata}")
    @Operation(summary = "One airline by IATA code")
    public AirlineDto get(@PathVariable String iata) {
        return ApiMapper.airline(referenceData.airline(iata));
    }
}
