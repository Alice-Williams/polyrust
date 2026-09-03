package org.polyrust.invalid;

public final class JavaVerifierPatternCollision {
    private JavaVerifierPatternCollision() {}

    static boolean invalid(Object input, String text) {
        return input instanceof String text;
    }
}
