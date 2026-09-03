package org.polyrust.invalid;

public final class JavaVerifierFieldInitializer {
    private final int x;
    private final int y = this.x;

    private JavaVerifierFieldInitializer() {
        this.x = 1;
    }
}
