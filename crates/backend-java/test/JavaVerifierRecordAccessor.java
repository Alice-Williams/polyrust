package org.polyrust.invalid;

record JavaVerifierRecordAccessor(int value) {
  private int value() {
    return value;
  }
}
