package org.polyrust.hostile;

public final class JavaInterfaceHostileImplementation {
  private JavaInterfaceHostileImplementation() {}

  private static final class Evil implements org.polyrust.generated.Generated.Labelled {
    @Override
    public org.polyrust.generated.Runtime.PolyResult<String> label(final String prefix) {
      return org.polyrust.generated.Runtime.ok(prefix);
    }
  }
}
