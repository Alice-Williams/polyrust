package org.polyrust.consumer;

public final class JavaTypedConsumerTest {
  private JavaTypedConsumerTest() {}

  public static void main(String[] arguments) {
    org.polyrust.generated.Runtime.PolyResult<Integer> computed =
        org.polyrust.generated.Generated.computed();
    if (!computed.ok() || computed.value() != 50) {
      throw new AssertionError("three-argument expression did not produce 50");
    }

    org.polyrust.generated.Runtime.PolyResult<org.polyrust.generated.Generated.Point3> point =
        org.polyrust.generated.Generated.make_point(3, 4, 5);
    if (!point.ok()
        || point.value().x() != 3
        || point.value().y() != 4
        || point.value().z() != 5) {
      throw new AssertionError("typed three-field construction did not preserve fields");
    }
  }
}
