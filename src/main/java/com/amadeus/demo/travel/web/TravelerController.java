package com.amadeus.demo.travel.web;

import java.util.List;

import com.amadeus.demo.travel.service.TravelerService;
import com.amadeus.demo.travel.web.dto.ApiDtos.TravelerDto;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.constraints.NotBlank;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/travelers")
@Tag(name = "Travelers")
public class TravelerController {

    private final TravelerService travelers;

    public TravelerController(TravelerService travelers) {
        this.travelers = travelers;
    }

    @GetMapping
    @Operation(summary = "Travelers whose last name starts with the given prefix (case-insensitive)")
    public List<TravelerDto> search(@RequestParam @NotBlank String lastName) {
        return travelers.searchByLastName(lastName).stream().map(ApiMapper::traveler).toList();
    }

    @GetMapping("/{id}")
    @Operation(summary = "One traveler by id")
    public TravelerDto get(@PathVariable Long id) {
        return ApiMapper.traveler(travelers.get(id));
    }
}
