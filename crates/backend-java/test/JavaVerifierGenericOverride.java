package org.polyrust.invalid;

public final class JavaVerifierGenericOverride {
  private interface ReviewInterface {
    boolean render(java.util.List<String> values);
  }

  private static final class ForgedImplementation implements ReviewInterface {
    @Override
    public boolean render(java.util.List<Integer> values) {
      return true;
    }
  }
}
