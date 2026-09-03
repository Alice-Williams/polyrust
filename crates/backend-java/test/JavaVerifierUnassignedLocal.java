package org.polyrust.invalid;

public final class JavaVerifierUnassignedLocal {
    private JavaVerifierUnassignedLocal() {}

    static int invalid() {
        int result;
        return result;
    }
}
