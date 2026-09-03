package org.polyrust.invalid;

final class JavaVerifierBoxedCast {
  static Long invalid() {
    return (Long) 1;
  }
}
