package com.amadeus.demo.travel.service;

import java.security.SecureRandom;
import java.time.Clock;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.random.RandomGenerator;

import com.amadeus.demo.travel.model.Booking;
import com.amadeus.demo.travel.model.BookingSegment;
import com.amadeus.demo.travel.model.BookingStatus;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.model.Traveler;
import com.amadeus.demo.travel.repository.BookingRepository;
import com.amadeus.demo.travel.repository.FareRepository;
import com.amadeus.demo.travel.repository.TravelerRepository;
import org.hibernate.Hibernate;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class BookingService {

    private final BookingRepository bookings;
    private final FareRepository fares;
    private final TravelerRepository travelers;
    private final OfferService offers;
    private final Clock clock;
    private final RandomGenerator random = new SecureRandom();

    public BookingService(BookingRepository bookings, FareRepository fares, TravelerRepository travelers,
                          OfferService offers, Clock clock) {
        this.bookings = bookings;
        this.fares = fares;
        this.travelers = travelers;
        this.offers = offers;
        this.clock = clock;
    }

    @Transactional(readOnly = true)
    public Booking get(String recordLocator) {
        return loaded(bookings.findByRecordLocator(recordLocator.trim().toUpperCase())
            .orElseThrow(() -> new NotFoundException("Booking " + recordLocator + " not found")));
    }

    @Transactional(readOnly = true)
    public List<Booking> searchByTravelerLastName(String lastNamePrefix) {
        List<Booking> found = bookings.findByTravelerLastNamePrefix(lastNamePrefix.trim());
        found.forEach(BookingService::loaded);
        return found;
    }

    /** Loads the lazy traveler and segment collections while the transaction is still open. */
    private static Booking loaded(Booking booking) {
        Hibernate.initialize(booking.getTravelers());
        Hibernate.initialize(booking.getSegments());
        return booking;
    }

    /** Books an offer for the given travelers, taking the seats from the fare's inventory. */
    @Transactional
    public Booking create(String offerId, List<Long> travelerIds) {
        return create(offerId, travelerIds, null, LocalDateTime.now(clock.withZone(ZoneOffset.UTC)));
    }

    /**
     * Same as {@link #create(String, List)} with an explicit record locator (unique, or {@code null} to generate
     * one) and creation time; used by the demo data seeder.
     */
    @Transactional
    public Booking create(String offerId, List<Long> travelerIds, String recordLocator, LocalDateTime createdAtUtc) {
        OfferId parsed = OfferId.parse(offerId);
        Set<Long> distinctIds = new LinkedHashSet<>(travelerIds);
        List<Traveler> pax = new ArrayList<>(distinctIds.size());
        for (Long id : distinctIds) {
            pax.add(travelers.findById(id).orElseThrow(() -> new NotFoundException("Traveler " + id + " not found")));
        }
        Fare fare = fares.lockByFlightIdAndBookingClass(parsed.flightId(), parsed.bookingClass())
            .orElseThrow(() -> new NotFoundException("Offer " + offerId + " not found"));
        if (fare.getSeatsAvailable() < pax.size()) {
            throw new ConflictException("Only " + fare.getSeatsAvailable() + " seat(s) left in booking class "
                + fare.getBookingClass() + " on flight " + fare.getFlight().getFlightNumber());
        }
        fare.setSeatsAvailable(fare.getSeatsAvailable() - pax.size());

        Pricing pricing = offers.price(fare, pax.size());
        String locator = recordLocator != null ? recordLocator : newRecordLocator();
        Booking booking = new Booking(locator, createdAtUtc, pax, fare, pricing.total(), pricing.currency());
        return bookings.save(booking);
    }

    /** Cancels a booking and returns its seats to inventory; cancelling twice is a no-op. */
    @Transactional
    public void cancel(String recordLocator) {
        Booking booking = get(recordLocator);
        if (booking.getStatus() == BookingStatus.CANCELLED) {
            return;
        }
        int seats = booking.getTravelers().size();
        for (BookingSegment segment : booking.getSegments()) {
            Fare fare = fares.lockById(segment.getFare().getId()).orElseThrow();
            fare.setSeatsAvailable(fare.getSeatsAvailable() + seats);
        }
        booking.cancel();
    }

    private String newRecordLocator() {
        String locator;
        do {
            locator = RecordLocatorGenerator.next(random);
        } while (bookings.existsByRecordLocator(locator));
        return locator;
    }

    record OfferId(Long flightId, String bookingClass) {

        static OfferId parse(String offerId) {
            if (offerId == null) {
                throw new BadRequestException("offerId is required");
            }
            int dash = offerId.indexOf('-');
            if (dash <= 0 || dash != offerId.length() - 2) {
                throw new BadRequestException("offerId must look like <flightId>-<bookingClass>, e.g. 123-Y");
            }
            try {
                return new OfferId(Long.parseLong(offerId.substring(0, dash)), offerId.substring(dash + 1));
            } catch (NumberFormatException e) {
                throw new BadRequestException("offerId must look like <flightId>-<bookingClass>, e.g. 123-Y");
            }
        }
    }
}
