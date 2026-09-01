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
  }
}
