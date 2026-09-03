package org.polyrust.valid;

public final class JavaVerifierArrayCastAliasing {
  private JavaVerifierArrayCastAliasing() {}

  public static void main(String[] args) {
    byte[] boundary = {0};
    Object opaque = boundary;
    byte[] internal = (byte[]) opaque;
    internal[0] = 1;
    if (boundary[0] != 1) {
      throw new AssertionError("a Java array cast unexpectedly copied its operand");
    }
  }
}
