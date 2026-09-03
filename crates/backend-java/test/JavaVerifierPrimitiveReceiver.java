package org.polyrust.invalid;

final class JavaVerifierPrimitiveReceiver {
  static boolean invalid() {
    return ((int) 1).equals(2);
  }
}
