package com.amadeus.demo.travel.config;

import java.math.BigDecimal;
import java.math.RoundingMode;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.time.Clock;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.LocalTime;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;

import com.amadeus.demo.travel.model.Airport;
import com.amadeus.demo.travel.model.Cabin;
import com.amadeus.demo.travel.repository.AirportRepository;
import com.amadeus.demo.travel.repository.FlightRepository;
import com.amadeus.demo.travel.service.BookingService;
import com.amadeus.demo.travel.service.Geo;
import com.amadeus.demo.travel.service.RecordLocatorGenerator;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.jdbc.core.BatchPreparedStatementSetter;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Component;

/**
 * Generates the schedule (flights, fares) and ~200 confirmed bookings when the {@code flight} table is empty.
 * <p>
 * The generator is seeded ({@code travel.demo.seed}), so two services started on the same day against an empty
 * database produce the same data. The schedule window covers {@code travel.demo.days} days starting the day after
 * the server starts; a database that already has flights is left untouched, so drop it (or restart the compose
 * stack with a fresh volume) to move the window.
 */
@Component
public class DemoDataSeeder implements ApplicationRunner {

    /** The first booking is always this PNR for traveler 1 (+ traveler 2) on the first NCE-LHR flight of day 1. */
    public static final String FIRST_RECORD_LOCATOR = "QX7K2M";

    private static final Logger log = LoggerFactory.getLogger(DemoDataSeeder.class);
    private static final int TRAVELER_COUNT = 50;
    private static final int LONG_HAUL_KM = 3500;

    /** City pairs served in both directions, with the carriers flying them and the first flight-number block. */
    private static final List<Route> ROUTES = List.of(
        new Route("NCE", "LHR", List.of("BA", "AF"), 340),
        new Route("NCE", "CDG", List.of("AF"), 7700),
        new Route("CDG", "JFK", List.of("AF", "DL"), 10),
        new Route("LHR", "SIN", List.of("BA", "SQ"), 300),
        new Route("MAD", "FRA", List.of("IB", "LH"), 1100),
        new Route("AMS", "LIS", List.of("KL", "TP"), 1690),
        new Route("FCO", "MUC", List.of("AZ", "LH"), 430),
        new Route("BCN", "DXB", List.of("EK"), 180),
        new Route("DXB", "BOM", List.of("EK", "AI"), 500));

    private static final String[] SHORT_HAUL_AIRCRAFT = {"A320", "A321", "A220", "B738", "E190"};
    private static final String[] LONG_HAUL_AIRCRAFT = {"A350", "B777", "B787", "A380", "A330"};

    private final JdbcTemplate jdbc;
    private final FlightRepository flights;
    private final AirportRepository airports;
    private final BookingService bookings;
    private final DemoDataProperties properties;
    private final Clock clock;

    public DemoDataSeeder(JdbcTemplate jdbc, FlightRepository flights, AirportRepository airports,
                          BookingService bookings, DemoDataProperties properties, Clock clock) {
        this.jdbc = jdbc;
        this.flights = flights;
        this.airports = airports;
        this.bookings = bookings;
        this.properties = properties;
        this.clock = clock;
    }

    @Override
    public void run(ApplicationArguments args) {
        if (flights.count() > 0) {
            log.info("Flight table already populated ({} flights); demo data generation skipped", flights.count());
            return;
        }
        LocalDate firstDay = LocalDate.now(clock).plusDays(1);
        LocalDate lastDay = firstDay.plusDays(properties.days() - 1L);
        Random random = new Random(properties.seed());
        Map<String, Airport> airportsByIata = new LinkedHashMap<>();
        airports.findAll().forEach(a -> airportsByIata.put(a.getIata(), a));

        List<FlightRow> flightRows = generateFlights(random, firstDay, airportsByIata);
        insertFlights(flightRows);
        List<Long> flightIds = jdbc.queryForList("select id from flight order by id", Long.class);
        for (int i = 0; i < flightIds.size(); i++) {
            flightRows.get(i).id = flightIds.get(i);
        }

        List<FareRow> fareRows = generateFares(random, flightRows);
        insertFares(fareRows);

        int bookingCount = generateBookings(random, flightRows, fareRows, firstDay);
        log.info("Demo data generated with seed {}: {} flights, {} fares, {} bookings; schedule window {} to {}",
            properties.seed(), flightRows.size(), fareRows.size(), bookingCount, firstDay, lastDay);
    }

    private List<FlightRow> generateFlights(Random random, LocalDate firstDay, Map<String, Airport> airports) {
        List<FlightRow> rows = new ArrayList<>();
        for (Route route : ROUTES) {
            for (int direction = 0; direction < 2; direction++) {
                Airport origin = airports.get(direction == 0 ? route.from() : route.to());
                Airport destination = airports.get(direction == 0 ? route.to() : route.from());
                double distanceKm = Geo.distanceKm(origin, destination);
                boolean longHaul = distanceKm >= LONG_HAUL_KM;
                String[] fleet = longHaul ? LONG_HAUL_AIRCRAFT : SHORT_HAUL_AIRCRAFT;
                int baseMinutes = (int) Math.round(35 + distanceKm / 800 * 60);

                int flightsPerDay = 4 + random.nextInt(5);
                List<LocalTime> slots = new ArrayList<>();
                for (int s = 0; s < flightsPerDay; s++) {
                    slots.add(LocalTime.of(6 + (s * 16) / flightsPerDay, 5 * random.nextInt(12)));
                }
                for (int day = 0; day < properties.days(); day++) {
                    LocalDate date = firstDay.plusDays(day);
                    for (int s = 0; s < flightsPerDay; s++) {
                        String carrier = route.carriers().get(s % route.carriers().size());
                        int number = route.numberBlock() + direction * 50 + s;
                        int duration = roundToFive(baseMinutes + random.nextInt(21) - 10);
                        LocalDateTime departureUtc = toUtc(date, slots.get(s), origin);
                        rows.add(new FlightRow(carrier + number, carrier, origin.getIata(), destination.getIata(),
                            departureUtc, departureUtc.plusMinutes(duration), fleet[random.nextInt(fleet.length)],
                            duration, distanceKm, longHaul, date));
                    }
                }
            }
        }
        return rows;
    }

    private List<FareRow> generateFares(Random random, List<FlightRow> flights) {
        List<FareRow> rows = new ArrayList<>();
        for (FlightRow flight : flights) {
            double economy = (45 + flight.distanceKm * 0.085) * (0.85 + random.nextDouble() * 0.4);
            if (flight.date.getDayOfWeek().getValue() >= 5) {
                economy *= 1.12;
            }
            rows.add(fare(flight, "Y", Cabin.ECONOMY, economy, true, true, 23, 20 + random.nextInt(61)));
            rows.add(fare(flight, "M", Cabin.ECONOMY, economy * 0.8, false, true, 23, 10 + random.nextInt(31)));
            rows.add(fare(flight, "K", Cabin.ECONOMY, economy * 0.6, false, false, 0, 5 + random.nextInt(26)));
            if (flight.longHaul) {
                rows.add(fare(flight, "W", Cabin.PREMIUM_ECONOMY, economy * 1.7, false, true, 23,
                    4 + random.nextInt(17)));
                rows.add(fare(flight, "J", Cabin.BUSINESS, economy * 3.2, true, true, 32, 2 + random.nextInt(15)));
                if (random.nextInt(4) > 0) {
                    rows.add(fare(flight, "F", Cabin.FIRST, economy * 5.0, true, true, 40, 1 + random.nextInt(8)));
                }
            } else {
                if (random.nextInt(10) < 3) {
                    rows.add(fare(flight, "W", Cabin.PREMIUM_ECONOMY, economy * 1.7, false, true, 23,
                        4 + random.nextInt(17)));
                }
                if (random.nextInt(10) < 6) {
                    rows.add(fare(flight, random.nextBoolean() ? "J" : "C", Cabin.BUSINESS, economy * 2.9, true,
                        true, 32, 2 + random.nextInt(15)));
                }
            }
        }
        return rows;
    }

    private int generateBookings(Random random, List<FlightRow> flights, List<FareRow> fares, LocalDate firstDay) {
        Map<Long, List<FareRow>> faresByFlight = new LinkedHashMap<>();
        for (FareRow fare : fares) {
            faresByFlight.computeIfAbsent(fare.flight.id, k -> new ArrayList<>()).add(fare);
        }
        LocalDateTime nowUtc = LocalDateTime.now(clock.withZone(ZoneOffset.UTC)).withNano(0);
        Set<String> locators = new HashSet<>();
        int created = 0;

        FlightRow firstFlight = flights.stream()
            .filter(f -> f.origin.equals("NCE") && f.destination.equals("LHR") && f.date.equals(firstDay))
            .findFirst().orElseThrow();
        bookings.create(firstFlight.id + "-Y", List.of(1L, 2L), FIRST_RECORD_LOCATOR, nowUtc.minusDays(12));
        locators.add(FIRST_RECORD_LOCATOR);
        created++;

        while (created < properties.bookings()) {
            FlightRow flight = flights.get(random.nextInt(flights.size()));
            List<FareRow> candidates = faresByFlight.get(flight.id);
            FareRow fare = candidates.get(random.nextInt(candidates.size()));
            int pax = 1 + random.nextInt(3);
            if (fare.seats < pax) {
                continue;
            }
            List<Long> travelerIds = new ArrayList<>();
            while (travelerIds.size() < pax) {
                long id = 1 + random.nextInt(TRAVELER_COUNT);
                if (!travelerIds.contains(id)) {
                    travelerIds.add(id);
                }
            }
            String locator;
            do {
                locator = RecordLocatorGenerator.next(random);
            } while (!locators.add(locator));
            LocalDateTime createdAt = nowUtc.minusMinutes(60L + random.nextInt(60 * 24 * 45));
            bookings.create(flight.id + "-" + fare.bookingClass, travelerIds, locator, createdAt);
            fare.seats -= pax;
            created++;
        }
        return created;
    }

    private void insertFlights(List<FlightRow> rows) {
        jdbc.batchUpdate("insert into flight (flight_number, airline_iata, origin_iata, destination_iata, departure,"
            + " arrival, aircraft, duration_minutes) values (?, ?, ?, ?, ?, ?, ?, ?)", new BatchPreparedStatementSetter() {
            @Override
            public void setValues(PreparedStatement ps, int i) throws SQLException {
                FlightRow r = rows.get(i);
                ps.setString(1, r.flightNumber);
                ps.setString(2, r.airline);
                ps.setString(3, r.origin);
                ps.setString(4, r.destination);
                ps.setObject(5, r.departureUtc);
                ps.setObject(6, r.arrivalUtc);
                ps.setString(7, r.aircraft);
                ps.setInt(8, r.durationMinutes);
            }

            @Override
            public int getBatchSize() {
                return rows.size();
            }
        });
    }

    private void insertFares(List<FareRow> rows) {
        jdbc.batchUpdate("insert into fare (flight_id, booking_class, cabin, base_amount, currency, refundable,"
            + " changeable, baggage_included_kg, seats_available) values (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            new BatchPreparedStatementSetter() {
                @Override
                public void setValues(PreparedStatement ps, int i) throws SQLException {
                    FareRow r = rows.get(i);
                    ps.setLong(1, r.flight.id);
                    ps.setString(2, r.bookingClass);
                    ps.setString(3, r.cabin.name());
                    ps.setBigDecimal(4, r.baseAmount);
                    ps.setString(5, "EUR");
                    ps.setBoolean(6, r.refundable);
                    ps.setBoolean(7, r.changeable);
                    ps.setInt(8, r.baggageKg);
                    ps.setInt(9, r.seats);
                }

                @Override
                public int getBatchSize() {
                    return rows.size();
                }
            });
    }

    private static FareRow fare(FlightRow flight, String bookingClass, Cabin cabin, double amount,
                                boolean refundable, boolean changeable, int baggageKg, int seats) {
        BigDecimal base = BigDecimal.valueOf(Math.max(29, amount)).setScale(0, RoundingMode.HALF_EVEN)
            .setScale(2, RoundingMode.UNNECESSARY);
        return new FareRow(flight, bookingClass, cabin, base, refundable, changeable, baggageKg, seats);
    }

    private static LocalDateTime toUtc(LocalDate date, LocalTime localTime, Airport airport) {
        return date.atTime(localTime).atZone(ZoneId.of(airport.getTimezone()))
            .withZoneSameInstant(ZoneOffset.UTC).toLocalDateTime();
    }

    private static int roundToFive(int minutes) {
        return Math.max(5, (int) Math.round(minutes / 5.0) * 5);
    }

    private record Route(String from, String to, List<String> carriers, int numberBlock) {
    }

    private static final class FlightRow {
        Long id;
        final String flightNumber;
        final String airline;
        final String origin;
        final String destination;
        final LocalDateTime departureUtc;
        final LocalDateTime arrivalUtc;
        final String aircraft;
        final int durationMinutes;
        final double distanceKm;
        final boolean longHaul;
        final LocalDate date;

        FlightRow(String flightNumber, String airline, String origin, String destination, LocalDateTime departureUtc,
                  LocalDateTime arrivalUtc, String aircraft, int durationMinutes, double distanceKm,
                  boolean longHaul, LocalDate date) {
            this.flightNumber = flightNumber;
            this.airline = airline;
            this.origin = origin;
            this.destination = destination;
            this.departureUtc = departureUtc;
            this.arrivalUtc = arrivalUtc;
            this.aircraft = aircraft;
            this.durationMinutes = durationMinutes;
            this.distanceKm = distanceKm;
            this.longHaul = longHaul;
            this.date = date;
        }
    }

    private static final class FareRow {
        final FlightRow flight;
        final String bookingClass;
        final Cabin cabin;
        final BigDecimal baseAmount;
        final boolean refundable;
        final boolean changeable;
        final int baggageKg;
        int seats;

        FareRow(FlightRow flight, String bookingClass, Cabin cabin, BigDecimal baseAmount, boolean refundable,
                boolean changeable, int baggageKg, int seats) {
            this.flight = flight;
            this.bookingClass = bookingClass;
            this.cabin = cabin;
            this.baseAmount = baseAmount;
            this.refundable = refundable;
            this.changeable = changeable;
            this.baggageKg = baggageKg;
            this.seats = seats;
        }
    }
}
