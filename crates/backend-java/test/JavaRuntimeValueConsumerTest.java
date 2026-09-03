package org.polyrust.consumer;

public final class JavaRuntimeValueConsumerTest {
  private JavaRuntimeValueConsumerTest() {}

  public static void main(String[] arguments) {
    expectIllegalArgument(
        () -> new org.polyrust.generated.Runtime.PolyOption<String>(false, "payload"),
        "contradictory option accepted");
    expectIllegalArgument(
        () -> new org.polyrust.generated.Runtime.PolyValueResult<String, String>(false, "value", "error"),
        "contradictory value result accepted");
    expectIllegalArgument(() -> org.polyrust.generated.Runtime.bytesOf(java.util.List.of(-1)), "negative byte accepted");
    expectIllegalArgument(() -> org.polyrust.generated.Runtime.bytesOf(java.util.List.of(256)), "large byte accepted");
    expectIllegalArgument(
        () -> new org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>>(true, java.util.List.of(java.util.List.of("\uD800"))),
        "nested unpaired surrogate accepted");

    org.polyrust.generated.Runtime.PolyOption<String> none = org.polyrust.generated.Runtime.optionNone();
    try {
      none.value();
      throw new AssertionError("None payload was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }

    byte[] mutable = new byte[] {1, 2};
    org.polyrust.generated.Runtime.Bytes bytes = new org.polyrust.generated.Runtime.Bytes(mutable);
    mutable[0] = 99;
    if (!java.util.Arrays.equals(bytes.values(), new byte[] {1, 2})) {
      throw new AssertionError("Bytes retained a mutable input alias");
    }
    byte[] exposed = bytes.values();
    exposed[0] = 99;
    if (!java.util.Arrays.equals(bytes.values(), new byte[] {1, 2})) {
      throw new AssertionError("Bytes exposed mutable backing storage");
    }

    double nan = Double.longBitsToDouble(0x7ff8000000000001L);
    assertSemanticUnequal(org.polyrust.generated.Runtime.optionSome(nan), org.polyrust.generated.Runtime.optionSome(nan), "option NaN");
    assertSemanticEqual(org.polyrust.generated.Runtime.optionSome(0.0), org.polyrust.generated.Runtime.optionSome(-0.0), "option signed zero");
    assertSemanticUnequal(
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(nan))),
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(nan))),
        "nested tagged-list NaN");
    assertSemanticEqual(
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(0.0))),
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(-0.0))),
        "nested tagged-list signed zero");
    assertDeepEqual(
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(nan))),
        org.polyrust.generated.Runtime.valueResultOk(java.util.List.of(org.polyrust.generated.Runtime.optionSome(nan))),
        "nested tagged-list NaN bits");
    assertDeepUnequal(
        org.polyrust.generated.Runtime.optionSome(0.0),
        org.polyrust.generated.Runtime.optionSome(-0.0),
        "option signed-zero bits");

    java.util.ArrayList<String> mutableInner = new java.util.ArrayList<>(java.util.List.of("first"));
    java.util.ArrayList<java.util.List<String>> mutableOuter = new java.util.ArrayList<>();
    mutableOuter.add(mutableInner);
    org.polyrust.generated.Runtime.PolyResult<java.util.List<java.util.List<String>>> copied = org.polyrust.generated.Generated.echo_nested_list(mutableOuter);
    mutableInner.add("aliased");
    mutableOuter.add(java.util.List.of("aliased"));
    if (!copied.ok() || !copied.value().equals(java.util.List.of(java.util.List.of("first")))) {
      throw new AssertionError("nested list boundary retained a mutable alias");
    }
    expectIllegalArgument(
        () -> org.polyrust.generated.Generated.echo_nested_list(java.util.List.of(java.util.List.of("\uD800"))),
        "nested malformed string crossed a generated API boundary");

    java.util.ArrayList<String> optionInner = new java.util.ArrayList<>(java.util.List.of("option"));
    org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>> option =
        org.polyrust.generated.Runtime.optionSome(java.util.List.of(optionInner));
    org.polyrust.generated.Runtime.PolyResult<org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>>> copiedOption =
        org.polyrust.generated.Generated.echo_nested_option(option);
    optionInner.add("aliased");
    if (!copiedOption.ok()
        || !copiedOption.value().some()
        || !copiedOption.value().value().equals(java.util.List.of(java.util.List.of("option")))) {
      throw new AssertionError("nested option/list boundary retained a mutable alias");
    }
  }

  private static void assertSemanticEqual(Object left, Object right, String message) {
    if (!org.polyrust.generated.Runtime.semanticEqual(left, right)) {
      throw new AssertionError(message + " should be equal");
    }
  }

  private static void assertSemanticUnequal(Object left, Object right, String message) {
    if (org.polyrust.generated.Runtime.semanticEqual(left, right)) {
      throw new AssertionError(message + " should be unequal");
    }
  }

  private static void assertDeepEqual(Object left, Object right, String message) {
    if (!org.polyrust.generated.Runtime.deepEqual(left, right)) {
      throw new AssertionError(message + " should be bit-exactly equal");
    }
  }

  private static void assertDeepUnequal(Object left, Object right, String message) {
    if (org.polyrust.generated.Runtime.deepEqual(left, right)) {
      throw new AssertionError(message + " should be bit-exactly unequal");
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
