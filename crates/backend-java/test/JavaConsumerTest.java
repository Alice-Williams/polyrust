package org.polyrust.consumer;

import java.util.ArrayList;
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
        () -> new Runtime.PolyOption<String>(false, "payload"),
        "contradictory option accepted");
    expectIllegalArgument(
        () ->
            new Runtime.PolyResult<String>(
                true, "value", new Runtime.PolyError("unexpected", "unexpected")),
        "contradictory result accepted");
    expectIllegalArgument(
        () -> new Runtime.PolyValueResult<String, String>(false, "value", "error"),
        "contradictory value result accepted");
    expectIllegalArgument(() -> new Runtime.Bytes(List.of(-1)), "negative byte accepted");
    expectIllegalArgument(() -> new Runtime.Bytes(List.of(256)), "large byte accepted");

    Runtime.PolyOption<String> none = Runtime.optionNone();
    try {
      none.value();
      throw new AssertionError("None payload was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }
    Runtime.PolyResult<String> failed = Runtime.fail("expected", "failure");
    try {
      failed.value();
      throw new AssertionError("failed result value was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }

    ArrayList<Integer> mutable = new ArrayList<>(List.of(1, 2));
    Runtime.Bytes bytes = new Runtime.Bytes(mutable);
    mutable.set(0, 99);
    if (!bytes.values().equals(List.of(1, 2))) {
      throw new AssertionError("Bytes retained a mutable input alias");
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
