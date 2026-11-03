package com.amadeus.demo.travel.service;

import java.util.random.RandomGenerator;

/** Six-character PNR record locators from an alphabet without the ambiguous I, O, 0 and 1. */
public final class RecordLocatorGenerator {

    public static final String ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    public static final int LENGTH = 6;

    private RecordLocatorGenerator() {
    }

    public static String next(RandomGenerator random) {
        char[] chars = new char[LENGTH];
        for (int i = 0; i < LENGTH; i++) {
            chars[i] = ALPHABET.charAt(random.nextInt(ALPHABET.length()));
        }
        return new String(chars);
    }
}
