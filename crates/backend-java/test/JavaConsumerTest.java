package org.polyrust.consumer;

import java.util.List;
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
    expectIllegalArgument(
        () ->
            new Runtime.PolyResult<String>(
                true, "value", new Runtime.PolyError("unexpected", "unexpected")),
        "contradictory result accepted");
    expectIllegalArgument(
        () -> Runtime.ok(List.of(List.of("\uD800"))),
        "nested unpaired surrogate accepted");
    Runtime.PolyResult<String> failed = Runtime.fail("expected", "failure");
    try {
      failed.value();
      throw new AssertionError("failed result value was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }

    double nan = Double.longBitsToDouble(0x7ff8000000000001L);
    assertSemanticUnequal(Runtime.ok(nan), Runtime.ok(nan), "result NaN");
    assertSemanticEqual(Runtime.ok(0.0), Runtime.ok(-0.0), "result signed zero");
    assertSemanticUnequal(
        Runtime.ok(List.of(List.of(nan))),
        Runtime.ok(List.of(List.of(nan))),
        "nested result-list NaN");
    assertSemanticEqual(
        Runtime.ok(List.of(List.of(0.0))),
        Runtime.ok(List.of(List.of(-0.0))),
        "nested result-list signed zero");
  }

  private static void assertSemanticEqual(Object left, Object right, String message) {
    if (!Runtime.semanticEqual(left, right)) {
      throw new AssertionError(message + " should be equal");
    }
  }

  private static void assertSemanticUnequal(Object left, Object right, String message) {
    if (Runtime.semanticEqual(left, right)) {
      throw new AssertionError(message + " should be unequal");
    }
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
