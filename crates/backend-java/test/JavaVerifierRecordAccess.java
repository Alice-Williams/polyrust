package org.polyrust.invalid;

public record JavaVerifierRecordAccess(int value) {
  private JavaVerifierRecordAccess(int value) {
    this.value = value;
  }
}
