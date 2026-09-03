package org.polyrust.invalid;

public final class JavaVerifierConstantLoops {
    private JavaVerifierConstantLoops() {}

    static int statementAfterInfiniteLoop() {
        while (true) {}
        return 1;
    }

    static void statementInsideFalseLoop() {
        while (false) {
            int unreachable = 1;
        }
    }
}
