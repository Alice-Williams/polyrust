package org.polyrust.consumer;

import org.polyrust.generated.Generated;
import org.polyrust.generated.Runtime;

public final class JavaConsumerTest {
  private JavaConsumerTest() {}

  public static void main(String[] arguments) {
    Generated.Label label = new Generated.Label("hello");
    Runtime.PolyResult<String> direct = label.render();
    Runtime.PolyResult<String> dispatched = Generated.call_render(label);
    if (!direct.ok() || !"hello".equals(direct.value())) {
      throw new AssertionError("record method API");
    }
    if (!dispatched.ok() || !"hello".equals(dispatched.value())) {
      throw new AssertionError("contract dispatch API");
    }
    try {
      new Generated.Label(null);
      throw new AssertionError("null record field accepted");
    } catch (NullPointerException expected) {
      // Required boundary rejection.
    }
    expectIllegalArgument(() -> new Generated.Label("\uD800"), "unpaired surrogate accepted");
    expectIllegalArgument(() -> new Runtime.Scalar(0xD800), "surrogate scalar accepted");
    expectIllegalArgument(() -> new Runtime.Scalar(0x110000), "out-of-range scalar accepted");
  }

  private static void expectIllegalArgument(Runnable action, String message) {
    try {
      action.run();
      throw new AssertionError(message);
    } catch (IllegalArgumentException expected) {
      // Required invariant rejection.
    }
  }
}
