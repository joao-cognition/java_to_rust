package com.amadeus.demo.travel.web;

import java.util.List;

import com.amadeus.demo.travel.service.BookingService;
import com.amadeus.demo.travel.web.dto.ApiDtos.BookingDto;
import com.amadeus.demo.travel.web.dto.ApiDtos.CreateBookingRequest;
import com.amadeus.demo.travel.web.dto.ApiMapper;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotBlank;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

@RestController
@RequestMapping("/api/bookings")
@Tag(name = "Reservation")
public class BookingController {

    private final BookingService bookings;

    public BookingController(BookingService bookings) {
        this.bookings = bookings;
    }

    @GetMapping("/{recordLocator}")
    @Operation(summary = "One booking by record locator")
    public BookingDto get(@PathVariable String recordLocator) {
        return ApiMapper.booking(bookings.get(recordLocator));
    }

    @GetMapping
    @Operation(summary = "Bookings where any traveler's last name starts with the prefix, newest first")
    public List<BookingDto> search(@RequestParam @NotBlank String lastName) {
        return bookings.searchByTravelerLastName(lastName).stream().map(ApiMapper::booking).toList();
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    @Operation(summary = "Book an offer for one or more travelers",
        description = "Decrements the fare's seat inventory. 404 for an unknown offer or traveler, "
            + "409 when not enough seats are left.")
    public BookingDto create(@RequestBody @Valid CreateBookingRequest request) {
        return ApiMapper.booking(bookings.create(request.offerId(), request.travelerIds()));
    }

    @DeleteMapping("/{recordLocator}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    @Operation(summary = "Cancel a booking and return its seats to inventory")
    public void cancel(@PathVariable String recordLocator) {
        bookings.cancel(recordLocator);
    }
}
