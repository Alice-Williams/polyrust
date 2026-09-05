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

    org.polyrust.generated.Runtime.PolyResult<Boolean> extended =
        org.polyrust.generated.Generated.extended_features();
    if (!extended.ok() || !extended.value()) {
      throw new AssertionError("typed extended feature families did not preserve semantics");
    }

    org.polyrust.generated.Runtime.PolyResult<
            org.polyrust.generated.Generated.TrafficLight>
        light = org.polyrust.generated.Generated.stop_light();
    if (!light.ok()
        || light.value() != org.polyrust.generated.Generated.TrafficLight.RED) {
      throw new AssertionError("typed payload-free enum did not preserve its variant");
    }

    org.polyrust.generated.Runtime.PolyResult<Boolean> enumEquality =
        org.polyrust.generated.Generated.stop_light_is_red(
            org.polyrust.generated.Generated.TrafficLight.RED);
    if (!enumEquality.ok() || !enumEquality.value()) {
      throw new AssertionError("typed payload-free enum equality did not preserve semantics");
    }

    org.polyrust.generated.Runtime.PolyResult<Integer> enumBranch =
        org.polyrust.generated.Generated.traffic_light_priority(
            org.polyrust.generated.Generated.TrafficLight.AMBER);
    if (!enumBranch.ok() || enumBranch.value() != 2) {
      throw new AssertionError("typed exhaustive enum branch did not preserve semantics");
    }

    org.polyrust.generated.Generated.Counter counter =
        new org.polyrust.generated.Generated.Counter(9);
    org.polyrust.generated.Runtime.PolyResult<Integer> concrete =
        org.polyrust.generated.Generated.read_counter_concrete(counter);
    org.polyrust.generated.Runtime.PolyResult<Integer> dynamic =
        org.polyrust.generated.Generated.read_counter_dynamic(counter);
    if (!concrete.ok()
        || concrete.value() != 9
        || !dynamic.ok()
        || dynamic.value() != 9) {
      throw new AssertionError("typed interface dispatch did not preserve semantics");
    }
  }
}
