package org.polyrust.consumer;

public final class JavaRuntimeValueConsumerTest {
  private JavaRuntimeValueConsumerTest() {}

  public static void main(String[] arguments) {
    expectIllegalArgument(
        () ->
            org.polyrust.generated.Generated
                .__polyrust_someOfOption19_List12_List6_String(
                    java.util.List.of(java.util.List.of("\uD800"))),
        "nested unpaired surrogate accepted");

    org.polyrust.generated.Runtime.PolyResult<Integer> i32RemainderOverflow =
        org.polyrust.generated.Generated.checked_rem_i32(Integer.MIN_VALUE, -1);
    assertCheckedOverflow(i32RemainderOverflow, "i32 remainder");
    try {
      i32RemainderOverflow.value();
      throw new AssertionError("failed result value was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }
    assertSemanticEqual(
        i32RemainderOverflow,
        org.polyrust.generated.Generated.checked_rem_i32(Integer.MIN_VALUE, -1),
        "failed result payload");
    assertDeepEqual(
        org.polyrust.generated.Generated.checked_rem_i32(5, 2),
        org.polyrust.generated.Generated.checked_rem_i32(5, 2),
        "successful result payload");
    assertCheckedOverflow(
        org.polyrust.generated.Generated.checked_rem_i64(Long.MIN_VALUE, -1L), "i64 remainder");

    org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>> none =
        org.polyrust.generated.Generated.__polyrust_noneOfOption19_List12_List6_String();
    assertSemanticEqual(
        none,
        org.polyrust.generated.Generated.__polyrust_noneOfOption19_List12_List6_String(),
        "None option");
    try {
      none.value();
      throw new AssertionError("None payload was readable");
    } catch (IllegalStateException expected) {
      // Required partial-accessor rejection.
    }

    byte[] mutable = new byte[] {1, 2};
    org.polyrust.generated.Runtime.Bytes bytes =
        new org.polyrust.generated.Runtime.Bytes(mutable);
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
    assertSemanticUnequal(
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan),
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan),
        "option NaN");
    assertSemanticEqual(
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(0.0),
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(-0.0),
        "option signed zero");
    assertSemanticUnequal(
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan))),
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan))),
        "nested tagged-list NaN");
    assertSemanticEqual(
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(0.0))),
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(-0.0))),
        "nested tagged-list signed zero");
    assertDeepEqual(
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan))),
        org.polyrust.generated.Generated
            .__polyrust_okOfResult18_List11_Option3_F646_String(
                java.util.List.of(
                    org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(nan))),
        "nested tagged-list NaN bits");
    assertDeepUnequal(
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(0.0),
        org.polyrust.generated.Generated.__polyrust_someOfOption3_F64(-0.0),
        "option signed-zero bits");

    java.util.ArrayList<String> mutableInner =
        new java.util.ArrayList<>(java.util.List.of("first"));
    java.util.ArrayList<java.util.List<String>> mutableOuter = new java.util.ArrayList<>();
    mutableOuter.add(mutableInner);
    org.polyrust.generated.Runtime.PolyResult<java.util.List<java.util.List<String>>> copied =
        org.polyrust.generated.Generated.echo_nested_list(mutableOuter);
    mutableInner.add("aliased");
    mutableOuter.add(java.util.List.of("aliased"));
    if (!copied.ok()
        || !copied.value().equals(java.util.List.of(java.util.List.of("first")))) {
      throw new AssertionError("nested list boundary retained a mutable alias");
    }
    expectIllegalArgument(
        () ->
            org.polyrust.generated.Generated.echo_nested_list(
                java.util.List.of(java.util.List.of("\uD800"))),
        "nested malformed string crossed a generated API boundary");

    java.util.ArrayList<String> optionInner =
        new java.util.ArrayList<>(java.util.List.of("option"));
    java.util.ArrayList<java.util.List<String>> optionOuter = new java.util.ArrayList<>();
    optionOuter.add(optionInner);
    org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>> option =
        org.polyrust.generated.Generated.__polyrust_someOfOption19_List12_List6_String(optionOuter);
    optionInner.add("aliased");
    optionOuter.add(java.util.List.of("aliased"));
    if (!option.value().equals(java.util.List.of(java.util.List.of("option")))) {
      throw new AssertionError("nested option factory retained a mutable input alias");
    }
    org.polyrust.generated.Runtime.PolyResult<
            org.polyrust.generated.Runtime.PolyOption<java.util.List<java.util.List<String>>>>
        copiedOption = org.polyrust.generated.Generated.echo_nested_option(option);
    if (!copiedOption.ok()
        || !copiedOption.value().some()
        || !copiedOption
            .value()
            .value()
            .equals(java.util.List.of(java.util.List.of("option")))) {
      throw new AssertionError("nested option/list boundary retained a mutable alias");
    }

    java.util.ArrayList<String> okInner =
        new java.util.ArrayList<>(java.util.List.of("ok"));
    java.util.ArrayList<java.util.List<String>> okOuter = new java.util.ArrayList<>();
    okOuter.add(okInner);
    org.polyrust.generated.Runtime.PolyValueResult<
            java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>
        ok =
            org.polyrust.generated.Generated
                .__polyrust_okOfResult19_List12_List6_String19_List12_List6_String(okOuter);
    okInner.add("aliased");
    okOuter.add(java.util.List.of("aliased"));
    if (!ok.value().equals(java.util.List.of(java.util.List.of("ok")))) {
      throw new AssertionError("nested value-result Ok factory retained a mutable input alias");
    }
    org.polyrust.generated.Runtime.PolyResult<
            org.polyrust.generated.Runtime.PolyValueResult<
                java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>>
        copiedOk = org.polyrust.generated.Generated.echo_nested_result(ok);
    if (!copiedOk.ok()
        || !copiedOk.value().ok()
        || !copiedOk.value().value().equals(java.util.List.of(java.util.List.of("ok")))) {
      throw new AssertionError("nested value-result Ok boundary retained a mutable alias");
    }

    java.util.ArrayList<String> errorInner =
        new java.util.ArrayList<>(java.util.List.of("error"));
    java.util.ArrayList<java.util.List<String>> errorOuter = new java.util.ArrayList<>();
    errorOuter.add(errorInner);
    org.polyrust.generated.Runtime.PolyValueResult<
            java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>
        error =
            org.polyrust.generated.Generated
                .__polyrust_errorOfResult19_List12_List6_String19_List12_List6_String(errorOuter);
    errorInner.add("aliased");
    errorOuter.add(java.util.List.of("aliased"));
    if (!error.error().equals(java.util.List.of(java.util.List.of("error")))) {
      throw new AssertionError("nested value-result Err factory retained a mutable input alias");
    }
    assertDeepEqual(
        error,
        org.polyrust.generated.Generated
            .__polyrust_errorOfResult19_List12_List6_String19_List12_List6_String(
                java.util.List.of(java.util.List.of("error"))),
        "nested value-result Err payload");
    org.polyrust.generated.Runtime.PolyResult<
            org.polyrust.generated.Runtime.PolyValueResult<
                java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>>
        copiedError = org.polyrust.generated.Generated.echo_nested_result(error);
    if (!copiedError.ok()
        || copiedError.value().ok()
        || !copiedError
            .value()
            .error()
            .equals(java.util.List.of(java.util.List.of("error")))) {
      throw new AssertionError("nested value-result Err boundary retained a mutable alias");
    }
  }

  private static void assertCheckedOverflow(
      org.polyrust.generated.Runtime.PolyResult<?> result, String message) {
    if (result.ok()) {
      throw new AssertionError(message + " unexpectedly succeeded");
    }
    if (!"checked_overflow".equals(result.error().code())
        || !"checked_overflow".equals(result.error().message())) {
      throw new AssertionError(
          message
              + " returned "
              + result.error().code()
              + "/"
              + result.error().message());
    }
  }

  private static void assertSemanticEqual(
      org.polyrust.generated.Runtime.SemanticValue left, Object right, String message) {
    if (!left.semanticEquals(right)) {
      throw new AssertionError(message + " should be equal");
    }
  }

  private static void assertSemanticUnequal(
      org.polyrust.generated.Runtime.SemanticValue left, Object right, String message) {
    if (left.semanticEquals(right)) {
      throw new AssertionError(message + " should be unequal");
    }
  }

  private static void assertDeepEqual(
      org.polyrust.generated.Runtime.SemanticValue left, Object right, String message) {
    if (!left.deepEquals(right)) {
      throw new AssertionError(message + " should be bit-exactly equal");
    }
  }

  private static void assertDeepUnequal(
      org.polyrust.generated.Runtime.SemanticValue left, Object right, String message) {
    if (left.deepEquals(right)) {
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
