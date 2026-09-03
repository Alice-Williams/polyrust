package org.polyrust.consumer;

import org.polyrust.generated.Generated;
import org.polyrust.generated.Runtime;

public final class JavaInterfaceConsumerTest {
  private JavaInterfaceConsumerTest() {}

  public static void main(String[] arguments) {
    Generated.Label label = new Generated.Label("value");
    Runtime.PolyResult<Generated.Labelled> returned = Generated.return_interface(label);
    if (!returned.ok()) {
      throw new AssertionError("return_interface failed");
    }
    Runtime.PolyResult<String> dispatched = returned.value().label("prefix:");
    if (!dispatched.ok() || !"prefix:value".equals(dispatched.value())) {
      throw new AssertionError("returned interface did not preserve dynamic dispatch");
    }

    expectIllegalArgument(
        () -> Generated.dynamic_dispatch(label, "\uD800"),
        "public String parameter accepted an unpaired surrogate");
  }

  private static void expectIllegalArgument(Runnable action, String message) {
    try {
      action.run();
      throw new AssertionError(message);
    } catch (IllegalArgumentException expected) {
      // Required generated boundary rejection.
    }
  }
}
