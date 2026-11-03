package com.amadeus.demo.travel.web;

import java.time.LocalDate;
import java.util.List;

import com.amadeus.demo.travel.config.DemoDataSeeder;
import com.amadeus.demo.travel.model.Fare;
import com.amadeus.demo.travel.repository.FareRepository;
import com.amadeus.demo.travel.repository.FlightRepository;
import com.amadeus.demo.travel.repository.BookingRepository;
import tools.jackson.databind.JsonNode;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.boot.webmvc.test.autoconfigure.AutoConfigureMockMvc;
import org.springframework.http.MediaType;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.web.servlet.assertj.MockMvcTester;
import org.springframework.test.web.servlet.assertj.MvcTestResult;
import tools.jackson.databind.ObjectMapper;

import static org.assertj.core.api.Assertions.assertThat;

/** End-to-end tests over MockMvc against the H2 profile with a reduced (2-day, 10-booking) generated data set. */
@SpringBootTest
@AutoConfigureMockMvc
@ActiveProfiles({"h2", "test"})
class TravelApiTest {

    private static final String TOMORROW = LocalDate.now().plusDays(1).toString();

    @Autowired
    MockMvcTester mvc;
    @Autowired
    ObjectMapper json;
    @Autowired
    FlightRepository flights;
    @Autowired
    FareRepository fares;
    @Autowired
    BookingRepository bookings;

    // --- reference data -------------------------------------------------------------------------------------------

    @Test
    void airportsAreSortedByIataAndCarryCoordinates() {
        JsonNode airports = body(mvc.get().uri("/api/airports"));
        assertThat(airports.size()).isGreaterThanOrEqualTo(40);
        assertThat(airports.get(0).get("iata").asText()).isEqualTo("AMS");
        JsonNode nce = body(mvc.get().uri("/api/airports/nce"));
        assertThat(nce.get("icao").asText()).isEqualTo("LFMN");
        assertThat(nce.get("timezone").asText()).isEqualTo("Europe/Paris");
        assertThat(nce.get("latitude").asDouble()).isBetween(43.0, 44.0);
    }

    @Test
    void unknownAirportAndAirlineAreProblemDetails404() {
        assertProblem(mvc.get().uri("/api/airports/XXX"), 404, "Airport XXX not found");
        assertProblem(mvc.get().uri("/api/airlines/ZZ"), 404, "Airline ZZ not found");
    }

    @Test
    void airlinesAreSortedByIata() {
        JsonNode airlines = body(mvc.get().uri("/api/airlines"));
        assertThat(airlines.size()).isGreaterThanOrEqualTo(12);
        assertThat(airlines.get(0).get("iata").asText()).isEqualTo("AF");
        assertThat(body(mvc.get().uri("/api/airlines/AF")).get("alliance").asText()).isEqualTo("SKYTEAM");
    }

    // --- flights --------------------------------------------------------------------------------------------------

    @Test
    void flightsOnRouteAreSortedByDepartureWithOffsetTimestamps() {
        JsonNode list = body(mvc.get().uri("/api/flights?origin=NCE&destination=LHR&date=" + TOMORROW));
        assertThat(list.size()).isBetween(4, 8);
        String previous = "";
        for (JsonNode f : list) {
            assertThat(f.get("departure").asText()).matches("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}[+-]\\d{2}:\\d{2}")
                .startsWith(TOMORROW).isGreaterThanOrEqualTo(previous);
            previous = f.get("departure").asText();
            assertThat(f.get("origin").get("iata").asText()).isEqualTo("NCE");
            assertThat(f.get("destination").get("iata").asText()).isEqualTo("LHR");
            assertThat(f.get("durationMinutes").asInt()).isPositive();
        }
        JsonNode one = body(mvc.get().uri("/api/flights/" + list.get(0).get("id").asLong()));
        assertThat(one.get("flightNumber").asText()).isEqualTo(list.get(0).get("flightNumber").asText());
    }

    @Test
    void flightsRequireAllThreeParameters() {
        assertProblem(mvc.get().uri("/api/flights?origin=NCE&destination=LHR"), 400, "date");
        assertProblem(mvc.get().uri("/api/flights?origin=NCE&date=" + TOMORROW), 400, "destination");
        assertProblem(mvc.get().uri("/api/flights?origin=NCE&destination=LHR&date=yesterday"), 400, "date");
        assertProblem(mvc.get().uri("/api/flights/999999999"), 404, "Flight 999999999 not found");
        assertProblem(mvc.get().uri("/api/flights/abc"), 400, "id");
    }

    // --- offers ---------------------------------------------------------------------------------------------------

    @Test
    void offersAreSortedByTotalAndCarryTwoDecimalMoney() {
        MvcTestResult result = mvc.get().uri("/api/offers?origin=NCE&destination=LHR&date=" + TOMORROW).exchange();
        assertThat(result).hasStatusOk();
        String raw = text(result);
        assertThat(raw).contains("\"baseAmount\":").containsPattern("\"total\":\\d+\\.\\d{2}[,}]");
        JsonNode offers = body(result);
        assertThat(offers.size()).isGreaterThanOrEqualTo(4);
        double previous = 0;
        for (JsonNode o : offers) {
            assertThat(o.get("offerId").asText())
                .isEqualTo(o.get("flight").get("id").asText() + "-" + o.get("fare").get("bookingClass").asText());
            double total = o.get("pricing").get("total").asDouble();
            assertThat(total).isGreaterThanOrEqualTo(previous);
            previous = total;
            JsonNode p = o.get("pricing");
            assertThat(p.get("passengers").asInt()).isEqualTo(1);
            assertThat(p.get("perPassenger").decimalValue())
                .isEqualByComparingTo(p.get("basePerPassenger").decimalValue().add(p.get("taxesPerPassenger").decimalValue()));
            assertThat(p.get("total").decimalValue()).isEqualByComparingTo(p.get("perPassenger").decimalValue());
            assertThat(p.get("currency").asText()).isEqualTo("EUR");
        }
    }

    @Test
    void offersFilterByCabinAndMultiplyByAdults() {
        JsonNode business = body(mvc.get().uri(
            "/api/offers?origin=CDG&destination=JFK&date=" + TOMORROW + "&adults=2&cabin=BUSINESS"));
        assertThat(business.size()).isPositive();
        for (JsonNode o : business) {
            assertThat(o.get("fare").get("cabin").asText()).isEqualTo("BUSINESS");
            assertThat(o.get("fare").get("seatsAvailable").asInt()).isGreaterThanOrEqualTo(2);
            JsonNode p = o.get("pricing");
            assertThat(p.get("passengers").asInt()).isEqualTo(2);
            assertThat(p.get("total").decimalValue())
                .isEqualByComparingTo(p.get("perPassenger").decimalValue().multiply(java.math.BigDecimal.valueOf(2)));
        }
    }

    @Test
    void offersValidateParameters() {
        String route = "origin=NCE&destination=LHR&date=" + TOMORROW;
        assertProblem(mvc.get().uri("/api/offers?" + route + "&adults=0"), 400, null);
        assertProblem(mvc.get().uri("/api/offers?" + route + "&adults=10"), 400, null);
        assertProblem(mvc.get().uri("/api/offers?" + route + "&cabin=COACH"), 400, "cabin");
        assertProblem(mvc.get().uri("/api/offers?origin=NCE&destination=LHR"), 400, "date");
        assertProblem(mvc.get().uri("/api/offers?origin=NCE&destination=NCE&date=" + TOMORROW), 400, "differ");
        assertProblem(mvc.get().uri("/api/offers?origin=NCE&destination=XXX&date=" + TOMORROW), 400, "XXX");
        assertThat(body(mvc.get().uri("/api/offers?origin=NCE&destination=LHR&date=2000-01-01"))).isEmpty();
    }

    // --- travelers ------------------------------------------------------------------------------------------------

    @Test
    void travelersByIdAndByLastNamePrefix() {
        JsonNode marin = body(mvc.get().uri("/api/travelers/1"));
        assertThat(marin.get("lastName").asText()).startsWith("Mar");
        assertThat(marin.get("dateOfBirth").asText()).matches("\\d{4}-\\d{2}-\\d{2}");
        JsonNode list = body(mvc.get().uri("/api/travelers?lastName=mAr"));
        assertThat(list.size()).isGreaterThanOrEqualTo(3);
        assertThat(list.findValues("id")).extracting(JsonNode::asLong).contains(1L);
        List<String> names = list.findValues("lastName").stream().map(JsonNode::asText).toList();
        assertThat(names).isSorted().allMatch(n -> n.toLowerCase().startsWith("mar"));
        assertProblem(mvc.get().uri("/api/travelers/999999"), 404, "Traveler 999999 not found");
        assertProblem(mvc.get().uri("/api/travelers"), 400, "lastName");
    }

    // --- bookings -------------------------------------------------------------------------------------------------

    @Test
    void seededBookingQx7k2mBelongsToTravelerOne() {
        JsonNode b = body(mvc.get().uri("/api/bookings/" + DemoDataSeeder.FIRST_RECORD_LOCATOR));
        assertThat(b.get("status").asText()).isEqualTo("CONFIRMED");
        assertThat(b.get("createdAt").asText()).matches("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}[+-]\\d{2}:\\d{2}");
        assertThat(b.get("travelers").findValues("id")).extracting(JsonNode::asLong).contains(1L);
        assertThat(b.get("segments")).hasSize(1);
        assertThat(b.get("segments").get(0).get("flight").get("origin").get("iata").asText()).isEqualTo("NCE");
        assertThat(b.get("totalAmount").decimalValue()).isPositive();
        assertThat(b.get("currency").asText()).isEqualTo("EUR");

        JsonNode list = body(mvc.get().uri("/api/bookings?lastName=Mar"));
        assertThat(list.findValues("recordLocator")).extracting(JsonNode::asText)
            .contains(DemoDataSeeder.FIRST_RECORD_LOCATOR);
        List<String> created = list.findValues("createdAt").stream().map(JsonNode::asText).toList();
        for (int i = 1; i < created.size(); i++) {
            assertThat(created.get(i)).isLessThanOrEqualTo(created.get(i - 1));
        }
        assertProblem(mvc.get().uri("/api/bookings/ZZZZZZ"), 404, "Booking ZZZZZZ not found");
        assertProblem(mvc.get().uri("/api/bookings"), 400, "lastName");
    }

    @Test
    void bookingDecrementsSeatsAndCancelReturnsThem() {
        Fare fare = anyFareWithSeats(5);
        String offerId = fare.getFlight().getId() + "-" + fare.getBookingClass();
        int before = fare.getSeatsAvailable();

        MvcTestResult created = mvc.post().uri("/api/bookings").contentType(MediaType.APPLICATION_JSON)
            .content("{\"offerId\":\"" + offerId + "\",\"travelerIds\":[3,4,3]}").exchange();
        assertThat(created).hasStatus(201);
        JsonNode booking = body(created);
        String locator = booking.get("recordLocator").asText();
        assertThat(locator).matches("[A-HJ-NP-Z2-9]{6}");
        assertThat(booking.get("status").asText()).isEqualTo("CONFIRMED");
        assertThat(booking.get("travelers")).hasSize(2);
        assertThat(booking.get("segments").get(0).get("bookingClass").asText()).isEqualTo(fare.getBookingClass());
        assertThat(fares.findById(fare.getId()).orElseThrow().getSeatsAvailable()).isEqualTo(before - 2);

        MvcTestResult cancelled = mvc.delete().uri("/api/bookings/" + locator).exchange();
        assertThat(cancelled).hasStatus(204);
        assertThat(body(mvc.get().uri("/api/bookings/" + locator)).get("status").asText()).isEqualTo("CANCELLED");
        assertThat(fares.findById(fare.getId()).orElseThrow().getSeatsAvailable()).isEqualTo(before);
        assertThat(mvc.delete().uri("/api/bookings/" + locator).exchange()).hasStatus(204);
    }

    @Test
    void bookingWithTooFewSeatsIs409AndLeavesInventoryUntouched() {
        Fare fare = anyFareWithSeats(2);
        String offerId = fare.getFlight().getId() + "-" + fare.getBookingClass();
        int before = fare.getSeatsAvailable();
        String ids = java.util.stream.LongStream.rangeClosed(1, before + 1).mapToObj(Long::toString)
            .collect(java.util.stream.Collectors.joining(","));

        assertProblem(mvc.post().uri("/api/bookings").contentType(MediaType.APPLICATION_JSON)
            .content("{\"offerId\":\"" + offerId + "\",\"travelerIds\":[" + ids + "]}"), 409, "seat");
        assertThat(fares.findById(fare.getId()).orElseThrow().getSeatsAvailable()).isEqualTo(before);
    }

    @Test
    void bookingValidationAndNotFoundCases() {
        Fare fare = anyFareWithSeats(1);
        String offerId = fare.getFlight().getId() + "-" + fare.getBookingClass();
        assertProblem(post("{\"offerId\":\"" + offerId + "\",\"travelerIds\":[999999]}"), 404, "Traveler 999999");
        assertProblem(post("{\"offerId\":\"999999999-Y\",\"travelerIds\":[1]}"), 404, "Offer 999999999-Y");
        assertProblem(post("{\"offerId\":\"" + fare.getFlight().getId() + "-Z\",\"travelerIds\":[1]}"), 404, "not found");
        assertProblem(post("{\"offerId\":\"nope\",\"travelerIds\":[1]}"), 400, null);
        assertProblem(post("{\"offerId\":\"" + offerId + "\",\"travelerIds\":[]}"), 400, null);
        assertProblem(post("{\"travelerIds\":[1]}"), 400, null);
        assertProblem(post("{not json"), 400, null);
        assertProblem(mvc.delete().uri("/api/bookings/ZZZZZZ"), 404, "Booking ZZZZZZ not found");
    }

    @Test
    void healthIsExposed() {
        assertThat(mvc.get().uri("/actuator/health").exchange()).hasStatusOk().bodyJson()
            .extractingPath("$.status").isEqualTo("UP");
    }

    @Test
    void seederGeneratedTheReducedDataSet() {
        assertThat(flights.count()).isBetween(150L, 300L);
        assertThat(fares.count()).isGreaterThan(flights.count() * 3);
        assertThat(bookings.count()).isGreaterThanOrEqualTo(10);
    }

    // --- helpers --------------------------------------------------------------------------------------------------

    private MockMvcTester.MockMvcRequestBuilder post(String bodyJson) {
        return mvc.post().uri("/api/bookings").contentType(MediaType.APPLICATION_JSON).content(bodyJson);
    }

    private Fare anyFareWithSeats(int min) {
        return fares.findAll().stream().filter(f -> f.getSeatsAvailable() >= min && f.getSeatsAvailable() <= 9)
            .findFirst().orElseThrow();
    }

    private JsonNode body(MockMvcTester.MockMvcRequestBuilder request) {
        return body(request.exchange());
    }

    private static String text(MvcTestResult result) {
        try {
            return result.getResponse().getContentAsString();
        } catch (java.io.UnsupportedEncodingException e) {
            throw new IllegalStateException(e);
        }
    }

    private JsonNode body(MvcTestResult result) {
        assertThat(result).hasStatus2xxSuccessful();
        return json.readTree(text(result));
    }

    private void assertProblem(MockMvcTester.MockMvcRequestBuilder request, int status, String detailContains) {
        MvcTestResult result = request.exchange();
        assertThat(result).hasStatus(status);
        assertThat(result.getResponse().getContentType()).startsWith("application/problem+json");
        JsonNode problem = json.readTree(text(result));
        assertThat(problem.get("status").asInt()).isEqualTo(status);
        if (detailContains != null) {
            assertThat(problem.get("detail").asText()).containsIgnoringCase(detailContains);
        }
    }
}
