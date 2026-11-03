package com.amadeus.demo.travel.service;

import com.amadeus.demo.travel.model.Airport;

/** Great-circle geometry on the WGS84 mean earth radius. */
public final class Geo {

    static final double EARTH_RADIUS_KM = 6371.0088;

    private Geo() {
    }

    /** Haversine great-circle distance between two airports, in kilometres. */
    public static double distanceKm(Airport a, Airport b) {
        return distanceKm(a.getLatitude(), a.getLongitude(), b.getLatitude(), b.getLongitude());
    }

    public static double distanceKm(double lat1, double lon1, double lat2, double lon2) {
        double phi1 = Math.toRadians(lat1);
        double phi2 = Math.toRadians(lat2);
        double dPhi = Math.toRadians(lat2 - lat1);
        double dLambda = Math.toRadians(lon2 - lon1);
        double sinDPhi = Math.sin(dPhi / 2);
        double sinDLambda = Math.sin(dLambda / 2);
        double h = sinDPhi * sinDPhi + Math.cos(phi1) * Math.cos(phi2) * sinDLambda * sinDLambda;
        return 2 * EARTH_RADIUS_KM * Math.asin(Math.min(1.0, Math.sqrt(h)));
    }
}
