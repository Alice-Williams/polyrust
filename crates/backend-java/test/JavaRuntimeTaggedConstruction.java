package org.polyrust.consumer;

final class JavaRuntimeTaggedConstruction {
  private static final java.util.List<java.util.List<String>> VALUES =
      java.util.List.of(java.util.List.of("value"));
  private static final org.polyrust.generated.Runtime.PolyError ERROR =
      new org.polyrust.generated.Runtime.PolyError("error", "error");

  private static final org.polyrust.generated.Runtime.PolyOption<
          java.util.List<java.util.List<String>>>
      OPTION_CONSTRUCTOR =
          new org.polyrust.generated.Runtime.PolyOption<
              java.util.List<java.util.List<String>>>(true, VALUES);
  private static final org.polyrust.generated.Runtime.PolyOption<
          java.util.List<java.util.List<String>>>
      OPTION_FACTORY = org.polyrust.generated.Runtime.optionSome(VALUES);

  private static final org.polyrust.generated.Runtime.PolyResult<
          java.util.List<java.util.List<String>>>
      RESULT_CONSTRUCTOR =
          new org.polyrust.generated.Runtime.PolyResult<
              java.util.List<java.util.List<String>>>(true, VALUES, null);
  private static final org.polyrust.generated.Runtime.PolyResult<
          java.util.List<java.util.List<String>>>
      RESULT_OK_FACTORY = org.polyrust.generated.Runtime.ok(VALUES);
  private static final org.polyrust.generated.Runtime.PolyResult<
          java.util.List<java.util.List<String>>>
      RESULT_FAIL_FACTORY = org.polyrust.generated.Runtime.fail("error", "error");

  private static final org.polyrust.generated.Runtime.PolyValueResult<
          java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>
      VALUE_RESULT_CONSTRUCTOR =
          new org.polyrust.generated.Runtime.PolyValueResult<
              java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>(
              true, VALUES, null);
  private static final org.polyrust.generated.Runtime.PolyValueResult<
          java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>
      VALUE_RESULT_OK_FACTORY = org.polyrust.generated.Runtime.valueResultOk(VALUES);
  private static final org.polyrust.generated.Runtime.PolyValueResult<
          java.util.List<java.util.List<String>>, java.util.List<java.util.List<String>>>
      VALUE_RESULT_ERR_FACTORY = org.polyrust.generated.Runtime.valueResultErr(VALUES);

  private static final String STRING_HELPER =
      org.polyrust.generated.Runtime.stringReplaceAll("\uD800", "", "-");
  private static final org.polyrust.generated.Runtime.PolyResult<Integer> INTEGER_HELPER =
      org.polyrust.generated.Runtime.checkedRemI32(Integer.MIN_VALUE, -1);
  private static final boolean EQUALITY_HELPER =
      org.polyrust.generated.Runtime.semanticEqual(VALUES, VALUES);
  private static final org.polyrust.generated.Runtime.Bytes BYTES_HELPER =
      org.polyrust.generated.Runtime.bytesOf(java.util.List.of(1, 2));

  private JavaRuntimeTaggedConstruction() {}
}
