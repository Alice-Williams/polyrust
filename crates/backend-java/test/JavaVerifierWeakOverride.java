package org.polyrust.invalid;

public final class JavaVerifierWeakOverride {
  private interface SemanticValue {
    boolean semanticEquals(Object other);
  }

  private static final class ForgedValue implements SemanticValue {
    @Override
    private boolean semanticEquals(Object other) {
      return true;
    }
  }
}
