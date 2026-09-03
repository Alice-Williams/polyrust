package org.polyrust.consumer;

public final class JavaStaticV1ConsumerTest {
  private JavaStaticV1ConsumerTest() {}

  public static void main(String[] arguments) {
    org.polyrust.generated.Runtime.PolyResult<Integer> computed =
        org.polyrust.generated.Generated.computed();
    if (!computed.ok() || computed.value() != 45) {
      throw new AssertionError("nested expression did not produce 45");
    }

    org.polyrust.generated.Runtime.PolyResult<org.polyrust.generated.Generated.Point> point =
        org.polyrust.generated.Generated.make_point(3, 4);
    if (!point.ok() || point.value().x() != 3 || point.value().y() != 4) {
      throw new AssertionError("typed record construction did not preserve fields");
    }
  }
}
