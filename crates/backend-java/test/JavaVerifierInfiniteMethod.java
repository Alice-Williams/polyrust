package org.polyrust.valid;

public final class JavaVerifierInfiniteMethod {
    private JavaVerifierInfiniteMethod() {}

    static int loopsForever() {
        while (true) {}
    }

    public static void main(String[] arguments) {
        if (arguments.length != 0) {
            throw new AssertionError("unexpected arguments");
        }
    }
}
