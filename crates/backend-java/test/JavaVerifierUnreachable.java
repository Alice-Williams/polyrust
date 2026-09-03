package org.polyrust.invalid;

public final class JavaVerifierUnreachable {
    private JavaVerifierUnreachable() {}

    static int invalid() {
        return 1;
        return 2;
    }
}
