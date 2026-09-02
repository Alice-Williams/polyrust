@SuppressWarnings({"unchecked", "rawtypes"})
public final class Runtime {
  public record PolyError(String code, String message) {}

  public record PolyResult<T>(boolean ok, T value, PolyError error) {
    static <T> PolyResult<T> ok(T value) {
      return new PolyResult<>(true, value, null);
    }

    static <T> PolyResult<T> fail(String code, String message) {
      return new PolyResult<>(false, null, new PolyError(code, message));
    }
  }

  public record PolyOption<T>(String tag, T value) {
    static <T> PolyOption<T> none() { return new PolyOption<>("none", null); }
    static <T> PolyOption<T> some(T value) { return new PolyOption<>("some", value); }
  }

  public record PolyValueResult<T, E>(String tag, T value, E error) {
    static <T, E> PolyValueResult<T, E> ok(T value) {
      return new PolyValueResult<>("ok", value, null);
    }
    static <T, E> PolyValueResult<T, E> err(E error) {
      return new PolyValueResult<>("err", null, error);
    }
  }

  public interface PolyRecord {
    Map<String, Object> polyValue();
  }

  record TestOutcome(PolyResult<Object> actual, Object expected, boolean expectsError) {}
  private record Flow(boolean returned, PolyResult<Object> result) {}

  private final Map<Long, Map<String, Object>> declarations = new LinkedHashMap<>();
  private final Map<Long, PolyResult<Object>> constants = new LinkedHashMap<>();

  Runtime(String source) {
    Map<String, Object> document = asMap(new JsonParser(source).parse());
    Map<String, Object> module = asMap(document.get("module"));
    for (Object raw : asList(module.get("declarations"))) {
      Map<String, Object> declaration = asMap(raw);
      declarations.put(nodeId(declaration), declaration);
    }
  }

  static List<Object> jsonArray(String source) {
    return asList(new JsonParser(source).parse());
  }

  PolyResult<Object> invoke(long functionId, List<?> arguments) {
    Map<String, Object> declaration = declarations.get(functionId);
    if (declaration == null || !"function".equals(declaration.get("kind"))) {
      return fail("invalid_call", "unknown function " + functionId);
    }
    return invokeBody(asMap(declaration.get("data")), arguments, null);
  }

  PolyResult<Object> invokeMethod(
      long implementationId, long methodId, Object receiver, List<?> arguments) {
    Map<String, Object> implementation = declarations.get(implementationId);
    if (implementation == null || !"implementation".equals(implementation.get("kind"))) {
      return fail("invalid_call", "unknown implementation " + implementationId);
    }
    for (Object raw : asList(asMap(implementation.get("data")).get("methods"))) {
      Map<String, Object> method = asMap(raw);
      long ownId = nodeIdFromHeader(asMap(method.get("header")));
      if (ownId == methodId || number(method.get("contract_method")) == methodId) {
        return invokeBody(method, arguments, receiver);
      }
    }
    return fail("invalid_call", "unknown method " + methodId);
  }

  Object decode(Object typed) {
    return value(asMap(asMap(typed).get("value")));
  }

  PolyResult<Object> readConstant(long identifier) {
    return constant(identifier);
  }

  TestOutcome invokeTest(List<Object> tests, int index) {
    if (index < 0 || index >= tests.size()) {
      return new TestOutcome(fail("invalid_test", "unknown test"), null, true);
    }
    Map<String, Object> test = asMap(tests.get(index));
    Map<String, Object> invocation = asMap(test.get("invocation"));
    Map<String, Object> data = asMap(invocation.get("data"));
    List<Object> arguments = new ArrayList<>();
    for (Object argument : asList(data.get("arguments"))) {
      arguments.add(decode(argument));
    }
    PolyResult<Object> actual =
        "function".equals(invocation.get("kind"))
            ? invoke(number(data.get("function")), arguments)
            : invokeMethod(
                number(data.get("implementation")),
                number(data.get("method")),
                decode(data.get("receiver")),
                arguments);
    Map<String, Object> expected = asMap(test.get("expected"));
    return new TestOutcome(
        actual, decode(expected.get("data")), "error".equals(expected.get("kind")));
  }

  private PolyResult<Object> invokeBody(
      Map<String, Object> callable, List<?> arguments, Object self) {
    Map<String, Object> environment = new LinkedHashMap<>();
    List<Object> parameters = asList(callable.get("parameters"));
    for (int index = 0; index < parameters.size(); index++) {
      Map<String, Object> parameter = asMap(parameters.get(index));
      environment.put(
          string(asMap(parameter.get("header")).get("name")), arguments.get(index));
    }
    return block(asMap(callable.get("body")), environment, self).result();
  }

  private Object value(Map<String, Object> value) {
    String kind = string(value.get("kind"));
    Object data = value.get("data");
    return switch (kind) {
      case "unit" -> null;
      case "bool", "string", "char" -> data;
      case "i32" -> Math.toIntExact(number(data));
      case "i64" -> Long.parseLong(String.valueOf(data));
      case "f64" -> Double.longBitsToDouble(Long.parseUnsignedLong(String.valueOf(data)));
      case "bytes" -> immutableIntegers(asList(data));
      case "list" -> {
        List<Object> items = new ArrayList<>();
        for (Object item : asList(data)) items.add(value(asMap(item)));
        yield List.copyOf(items);
      }
      case "none" -> PolyOption.none();
      case "some" -> PolyOption.some(value(asMap(data)));
      case "ok" -> PolyValueResult.ok(value(asMap(data)));
      case "err" -> PolyValueResult.err(value(asMap(data)));
      case "record" -> aggregate(asMap(data), null);
      case "enum" -> aggregate(asMap(data), number(asMap(data).get("variant")));
      default -> null;
    };
  }

  private Object aggregate(Map<String, Object> data, Long variantId) {
    long declarationId = number(data.get("declaration"));
    Map<String, Object> declaration = asMap(declarations.get(declarationId).get("data"));
    Map<String, Object> variant = findVariant(declaration, variantId);
    Map<String, Object> result = new LinkedHashMap<>();
    result.put("__polyDecl", declarationId);
    List<Object> members;
    if (variant == null) {
      members = asList(declaration.get("fields"));
    } else {
      result.put("tag", asMap(variant.get("header")).get("name"));
      members = asList(variant.get("fields"));
    }
    for (Object raw : asList(data.get("fields"))) {
      Map<String, Object> field = asMap(raw);
      result.put(memberName(members, number(field.get("field"))), value(asMap(field.get("value"))));
    }
    return materialize(declarationId, variant, result);
  }

  private PolyResult<Object> expression(
      Map<String, Object> expression, Map<String, Object> environment, Object self) {
    String kind = string(expression.get("kind"));
    Map<String, Object> data = asMap(expression.get("data"));
    switch (kind) {
      case "literal": return ok(value(asMap(data.get("value"))));
      case "local": return ok(environment.get(string(data.get("name"))));
      case "self_value": return ok(self);
      case "constant": return constant(number(data.get("declaration")));
      case "construct_none": return ok(PolyOption.none());
      case "construct_some":
        return map(expression(asMap(data.get("value")), environment, self), PolyOption::some);
      case "construct_ok":
        return map(expression(asMap(data.get("value")), environment, self), PolyValueResult::ok);
      case "construct_err":
        return map(expression(asMap(data.get("value")), environment, self), PolyValueResult::err);
      case "construct_list":
        return sequence(asList(data.get("elements")), environment, self);
      case "construct_record":
        return construct(number(data.get("declaration")), null, asList(data.get("fields")), environment, self);
      case "construct_enum":
        return construct(
            number(data.get("declaration")),
            number(data.get("variant")),
            asList(data.get("fields")),
            environment,
            self);
      case "field": {
        PolyResult<Object> base = expression(asMap(data.get("base")), environment, self);
        return base.ok() ? ok(field(base.value(), fieldName(number(data.get("field"))))) : base;
      }
      case "call": {
        PolyResult<Object> arguments = sequence(asList(data.get("arguments")), environment, self);
        return arguments.ok() ? invoke(number(data.get("function")), asList(arguments.value())) : arguments;
      }
      case "intrinsic": {
        PolyResult<Object> arguments = sequence(asList(data.get("arguments")), environment, self);
        return arguments.ok()
            ? intrinsic(string(data.get("operation")), asList(arguments.value()))
            : arguments;
      }
      case "method_call": {
        PolyResult<Object> arguments = sequence(asList(data.get("arguments")), environment, self);
        if (!arguments.ok()) return arguments;
        PolyResult<Object> receiver = expression(asMap(data.get("receiver")), environment, self);
        if (!receiver.ok()) return receiver;
        Map<String, Object> dispatch = asMap(data.get("dispatch"));
        Map<String, Object> target = asMap(dispatch.get("data"));
        long implementation =
            "contract".equals(dispatch.get("kind"))
                ? findImplementation(
                    number(target.get("contract")),
                    number(field(receiver.value(), "__polyDecl")))
                : number(target.get("implementation"));
        return invokeMethod(
            implementation,
            number(target.get("method")),
            receiver.value(),
            asList(arguments.value()));
      }
      case "if": {
        PolyResult<Object> condition = expression(asMap(data.get("condition")), environment, self);
        if (!condition.ok()) return condition;
        String branch = Boolean.TRUE.equals(condition.value()) ? "then_block" : "else_block";
        return block(asMap(data.get(branch)), copy(environment), self).result();
      }
      case "match": return match(data, environment, self);
      case "block": return block(data, copy(environment), self).result();
      default: return fail("invalid_expression", "unknown expression " + kind);
    }
  }

  private Flow block(
      Map<String, Object> block, Map<String, Object> environment, Object self) {
    for (Object raw : asList(block.get("statements"))) {
      Map<String, Object> statement = asMap(raw);
      String kind = string(statement.get("kind"));
      Map<String, Object> data = asMap(statement.get("data"));
      if ("let".equals(kind) || "expression".equals(kind)) {
        PolyResult<Object> item = expression(asMap(data.get("value")), environment, self);
        if (!item.ok()) return new Flow(true, item);
        if ("let".equals(kind)) environment.put(string(data.get("name")), item.value());
      } else if ("return".equals(kind)) {
        return new Flow(
            true,
            data.get("value") == null
                ? ok(null)
                : expression(asMap(data.get("value")), environment, self));
      } else if ("for_each".equals(kind)) {
        PolyResult<Object> items = expression(asMap(data.get("iterable")), environment, self);
        if (!items.ok()) return new Flow(true, items);
        for (Object item : asList(items.value())) {
          Map<String, Object> inner = copy(environment);
          inner.put(string(data.get("binding")), item);
          Flow flow = block(asMap(data.get("body")), inner, self);
          if (flow.returned() || !flow.result().ok()) return flow;
        }
      }
    }
    return new Flow(
        false,
        block.get("result") == null
            ? ok(null)
            : expression(asMap(block.get("result")), environment, self));
  }

  private PolyResult<Object> match(
      Map<String, Object> data, Map<String, Object> environment, Object self) {
    PolyResult<Object> matched = expression(asMap(data.get("value")), environment, self);
    if (!matched.ok()) return matched;
    for (Object raw : asList(data.get("arms"))) {
      Map<String, Object> arm = asMap(raw);
      Map<String, Object> bindings = pattern(asMap(arm.get("pattern")), matched.value());
      if (bindings != null) {
        Map<String, Object> inner = copy(environment);
        inner.putAll(bindings);
        return block(asMap(arm.get("body")), inner, self).result();
      }
    }
    return fail("non_exhaustive_match", "checked match had no matching arm");
  }

  private Map<String, Object> pattern(Map<String, Object> pattern, Object value) {
    String kind = string(pattern.get("kind"));
    Map<String, Object> data = asMap(pattern.get("data"));
    Map<String, Object> result = new LinkedHashMap<>();
    if ("wildcard".equals(kind)) return result;
    if ("bool".equals(kind)) return Objects.equals(value, data.get("value")) ? result : null;
    if ("none".equals(kind) || "some".equals(kind)) {
      if (!(value instanceof PolyOption<?> option) || !kind.equals(option.tag())) return null;
      if ("some".equals(kind)) result.put(string(data.get("binding")), option.value());
      return result;
    }
    if ("ok".equals(kind) || "err".equals(kind)) {
      if (!(value instanceof PolyValueResult<?, ?> item) || !kind.equals(item.tag())) return null;
      result.put(string(data.get("binding")), "err".equals(kind) ? item.error() : item.value());
      return result;
    }
    if ("enum_variant".equals(kind)) {
      Map<String, Object> object = recordMap(value);
      Map<String, Object> declaration =
          asMap(declarations.get(number(data.get("declaration"))).get("data"));
      Map<String, Object> variant = findVariant(declaration, number(data.get("variant")));
      if (variant == null
          || !Objects.equals(object.get("tag"), asMap(variant.get("header")).get("name"))) {
        return null;
      }
      List<Object> members = asList(variant.get("fields"));
      for (Object raw : asList(data.get("bindings"))) {
        Map<String, Object> binding = asMap(raw);
        result.put(
            string(binding.get("binding")),
            object.get(memberName(members, number(binding.get("field")))));
      }
      return result;
    }
    return null;
  }

  private PolyResult<Object> intrinsic(String name, List<Object> values) {
    Object a = values.isEmpty() ? null : values.get(0);
    Object b = values.size() < 2 ? null : values.get(1);
    Object c = values.size() < 3 ? null : values.get(2);
    switch (name) {
      case "bool_not": return ok(!((Boolean) a));
      case "bool_and": return ok((Boolean) a && (Boolean) b);
      case "bool_or": return ok((Boolean) a || (Boolean) b);
      case "equal": return ok(semanticEqual(a, b));
      case "not_equal": return ok(!semanticEqual(a, b));
      case "less": return ok(compare(a, b) < 0);
      case "less_equal": return ok(compare(a, b) <= 0);
      case "greater": return ok(compare(a, b) > 0);
      case "greater_equal": return ok(compare(a, b) >= 0);
// POLYRUST-BEGIN numeric-cases-primary
      case "int_neg_checked":
      case "int_add_checked":
      case "int_sub_checked":
      case "int_mul_checked":
      case "int_div_checked":
      case "int_rem_checked":
        return checkedInteger(name, a, b);
      case "int_neg_wrapping":
      case "int_add_wrapping":
      case "int_sub_wrapping":
      case "int_mul_wrapping":
        return wrappingInteger(name, a, b);
      case "int_bit_not":
        return a instanceof Integer ? ok(~(Integer) a) : ok(~(Long) a);
      case "int_bit_and":
        return a instanceof Integer ? ok((Integer) a & (Integer) b) : ok((Long) a & (Long) b);
      case "int_bit_or":
        return a instanceof Integer ? ok((Integer) a | (Integer) b) : ok((Long) a | (Long) b);
      case "int_bit_xor":
        return a instanceof Integer ? ok((Integer) a ^ (Integer) b) : ok((Long) a ^ (Long) b);
      case "int_shift_left_checked":
        if (a instanceof Integer integer) {
          return cast(checkedI32(BigInteger.valueOf(integer).shiftLeft((Integer) b)));
        }
        return cast(checkedI64(BigInteger.valueOf((Long) a).shiftLeft((Integer) b)));
      case "int_shift_right_checked":
        return a instanceof Integer
            ? ok((Integer) a >> (Integer) b)
            : ok((Long) a >> (Integer) b);
// POLYRUST-END numeric-cases-primary
      case "float_neg": return ok(-((Double) a));
      case "float_trunc": {
        double value = (Double) a;
        return ok(value > 0.0 ? Math.floor(value) : Math.ceil(value));
      }
      case "float_is_nan": return ok(Double.isNaN((Double) a));
      case "float_is_negative_zero":
        return ok(Double.doubleToRawLongBits((Double) a) == Long.MIN_VALUE);
      case "float_add": return ok((Double) a + (Double) b);
      case "float_sub": return ok((Double) a - (Double) b);
      case "float_mul": return ok((Double) a * (Double) b);
      case "float_div": return ok((Double) a / (Double) b);
      case "float_rem_trunc": return ok((Double) a % (Double) b);
      case "string_concat": return ok((String) a + (String) b);
      case "string_scalar_length": return cast(scalarLength((String) a));
// POLYRUST-BEGIN string-utf16-length-case
      case "string_utf16_length": return ok((long) ((String) a).length());
// POLYRUST-END string-utf16-length-case
// POLYRUST-BEGIN string-index-of-literal-case
      case "string_index_of_literal": {
        String source = (String) a;
        int utf16Index = source.indexOf((String) b);
        return ok(
            utf16Index < 0
                ? PolyOption.none()
                : PolyOption.some((long) source.codePointCount(0, utf16Index)));
      }
// POLYRUST-END string-index-of-literal-case
// POLYRUST-BEGIN string-slice-scalars-case
      case "string_slice_scalars": {
        String source = (String) a;
        long length = source.codePointCount(0, source.length());
        long start = Math.max(0L, Math.min((Long) b, length));
        long end = Math.max(0L, Math.min((Long) c, length));
        if (start >= end) return ok("");
        int utf16Start = source.offsetByCodePoints(0, (int) start);
        int utf16End = source.offsetByCodePoints(0, (int) end);
        return ok(source.substring(utf16Start, utf16End));
      }
// POLYRUST-END string-slice-scalars-case
      case "string_is_empty": return ok(((String) a).isEmpty());
      case "string_contains": return ok(((String) a).contains((String) b));
      case "string_starts_with": return ok(((String) a).startsWith((String) b));
      case "string_strip_prefix":
        return ok(
            !((String) b).isEmpty() && ((String) a).startsWith((String) b)
                ? ((String) a).substring(((String) b).length())
                : a);
      case "string_ends_with": return ok(((String) a).endsWith((String) b));
      case "string_replace_all": return ok(((String) a).replace((String) b, (String) c));
      case "string_replace_many": return ok(replaceManyLiteral((String) a, values));
      case "string_truncate_utf8_bytes":
        return ok(truncateUtf8Bytes((String) a, (Double) b));
      case "string_trim_start": return ok(trimStartScalars((String) a, (String) b));
      case "string_trim_end": return ok(trimEndScalars((String) a, (String) b));
      case "bytes_concat":
      case "list_concat":
        return ok(concat(asList(a), asList(b)));
      case "bytes_replace_all":
        return ok(replaceBytesAll(asList(a), asList(b), asList(c)));
      case "bytes_length":
      case "list_length":
        return ok(asList(a).size());
      case "bytes_is_empty":
      case "list_is_empty":
        return ok(asList(a).isEmpty());
      case "list_get_checked": {
        int index = (Integer) b;
        return index >= 0 && index < asList(a).size()
            ? ok(asList(a).get(index))
            : fail("index_out_of_bounds", "list index out of bounds");
      }
      case "list_append": return ok(listAppend(asList(a), b));
      case "list_contains": return ok(asList(a).stream().anyMatch(item -> semanticEqual(item, b)));
// POLYRUST-BEGIN list-index-of-case
      case "list_index_of": {
        List<Object> list = asList(a);
        for (int index = 0; index < list.size(); index++) {
          if (semanticEqual(list.get(index), b)) return ok(PolyOption.some((long) index));
        }
        return ok(PolyOption.none());
      }
// POLYRUST-END list-index-of-case
      case "option_is_some": return ok("some".equals(((PolyOption<?>) a).tag()));
      case "option_is_none": return ok("none".equals(((PolyOption<?>) a).tag()));
      case "option_unwrap_or":
        return ok("some".equals(((PolyOption<?>) a).tag()) ? ((PolyOption<?>) a).value() : b);
      case "result_is_ok": return ok("ok".equals(((PolyValueResult<?, ?>) a).tag()));
      case "result_is_err": return ok("err".equals(((PolyValueResult<?, ?>) a).tag()));
      case "widen_i32_to_i64": return ok(((Integer) a).longValue());
// POLYRUST-BEGIN numeric-case-narrow
      case "narrow_i64_to_i32_checked": return cast(checkedI32(BigInteger.valueOf((Long) a)));
// POLYRUST-END numeric-case-narrow
// POLYRUST-BEGIN utf8-cases
      case "string_to_utf8": {
        byte[] bytes = ((String) a).getBytes(StandardCharsets.UTF_8);
        List<Integer> result = new ArrayList<>();
        for (byte item : bytes) result.add(Byte.toUnsignedInt(item));
        return ok(List.copyOf(result));
      }
      case "string_from_utf8_checked": return stringFromUtf8(asList(a));
// POLYRUST-END utf8-cases
      default: return fail("invalid_intrinsic", "unknown intrinsic " + name);
    }
  }

// POLYRUST-BEGIN numeric-private-methods
  private PolyResult<Object> checkedInteger(String name, Object a, Object b) {
    boolean wide = a instanceof Long;
    BigInteger left = BigInteger.valueOf(((Number) a).longValue());
    BigInteger right = b == null ? BigInteger.ZERO : BigInteger.valueOf(((Number) b).longValue());
    if (("int_div_checked".equals(name) || "int_rem_checked".equals(name))
        && right.signum() == 0) {
      return fail("division_by_zero", "integer division by zero");
    }
    BigInteger result =
        switch (name) {
          case "int_neg_checked" -> left.negate();
          case "int_add_checked" -> left.add(right);
          case "int_sub_checked" -> left.subtract(right);
          case "int_mul_checked" -> left.multiply(right);
          case "int_div_checked" -> left.divide(right);
          case "int_rem_checked" -> left.remainder(right);
          default -> throw new IllegalStateException(name);
        };
    return wide ? cast(checkedI64(result)) : cast(checkedI32(result));
  }

  private PolyResult<Object> wrappingInteger(String name, Object a, Object b) {
    BigInteger left = BigInteger.valueOf(((Number) a).longValue());
    BigInteger right = b == null ? BigInteger.ZERO : BigInteger.valueOf(((Number) b).longValue());
    BigInteger result =
        switch (name) {
          case "int_neg_wrapping" -> left.negate();
          case "int_add_wrapping" -> left.add(right);
          case "int_sub_wrapping" -> left.subtract(right);
          case "int_mul_wrapping" -> left.multiply(right);
          default -> throw new IllegalStateException(name);
        };
    return a instanceof Long ? ok(wrappingI64(result)) : ok(wrappingI32(result.longValue()));
  }
// POLYRUST-END numeric-private-methods

  private PolyResult<Object> sequence(
      List<Object> expressions, Map<String, Object> environment, Object self) {
    List<Object> values = new ArrayList<>();
    for (Object raw : expressions) {
      PolyResult<Object> item = expression(asMap(raw), environment, self);
      if (!item.ok()) return item;
      values.add(item.value());
    }
    return ok(List.copyOf(values));
  }

  private PolyResult<Object> construct(
      long declarationId,
      Long variantId,
      List<Object> fields,
      Map<String, Object> environment,
      Object self) {
    Map<String, Object> declaration = asMap(declarations.get(declarationId).get("data"));
    Map<String, Object> variant = findVariant(declaration, variantId);
    List<Object> members =
        variant == null ? asList(declaration.get("fields")) : asList(variant.get("fields"));
    Map<String, Object> result = new LinkedHashMap<>();
    result.put("__polyDecl", declarationId);
    if (variant != null) result.put("tag", asMap(variant.get("header")).get("name"));
    for (Object raw : fields) {
      Map<String, Object> field = asMap(raw);
      PolyResult<Object> item = expression(asMap(field.get("value")), environment, self);
      if (!item.ok()) return item;
      result.put(memberName(members, number(field.get("field"))), item.value());
    }
    return ok(materialize(declarationId, variant, result));
  }

  private PolyResult<Object> constant(long identifier) {
    PolyResult<Object> cached = constants.get(identifier);
    if (cached != null) return cached;
    Map<String, Object> declaration = declarations.get(identifier);
    if (declaration == null || !"constant".equals(declaration.get("kind"))) {
      return fail("invalid_constant", "unknown constant " + identifier);
    }
    PolyResult<Object> result =
        constantExpression(asMap(asMap(declaration.get("data")).get("value")));
    constants.put(identifier, result);
    return result;
  }

  private PolyResult<Object> constantExpression(Map<String, Object> expression) {
    String kind = string(expression.get("kind"));
    Map<String, Object> data = asMap(expression.get("data"));
    switch (kind) {
      case "literal": return ok(value(asMap(data.get("value"))));
      case "reference": return constant(number(data.get("declaration")));
      case "none": return ok(PolyOption.none());
      case "some": return map(constantExpression(asMap(data.get("value"))), PolyOption::some);
      case "ok": return map(constantExpression(asMap(data.get("value"))), PolyValueResult::ok);
      case "err": return map(constantExpression(asMap(data.get("value"))), PolyValueResult::err);
      case "list": {
        List<Object> values = new ArrayList<>();
        for (Object raw : asList(data.get("elements"))) {
          PolyResult<Object> item = constantExpression(asMap(raw));
          if (!item.ok()) return item;
          values.add(item.value());
        }
        return ok(List.copyOf(values));
      }
      case "record":
      case "enum": {
        long declarationId = number(data.get("declaration"));
        Long variantId = "enum".equals(kind) ? number(data.get("variant")) : null;
        Map<String, Object> declaration = asMap(declarations.get(declarationId).get("data"));
        Map<String, Object> variant = findVariant(declaration, variantId);
        List<Object> members =
            variant == null ? asList(declaration.get("fields")) : asList(variant.get("fields"));
        Map<String, Object> result = new LinkedHashMap<>();
        result.put("__polyDecl", declarationId);
        if (variant != null) result.put("tag", asMap(variant.get("header")).get("name"));
        for (Object raw : asList(data.get("fields"))) {
          Map<String, Object> field = asMap(raw);
          PolyResult<Object> item = constantExpression(asMap(field.get("value")));
          if (!item.ok()) return item;
          result.put(memberName(members, number(field.get("field"))), item.value());
        }
        return ok(materialize(declarationId, variant, result));
      }
      case "intrinsic": {
        List<Object> values = new ArrayList<>();
        for (Object raw : asList(data.get("arguments"))) {
          PolyResult<Object> item = constantExpression(asMap(raw));
          if (!item.ok()) return item;
          values.add(item.value());
        }
        return intrinsic(string(data.get("operation")), values);
      }
      default: return fail("invalid_constant", "unknown constant expression " + kind);
    }
  }

  private Object field(Object value, String name) {
    return recordMap(value).get(name);
  }

  private Map<String, Object> recordMap(Object value) {
    return value instanceof PolyRecord record ? record.polyValue() : asMap(value);
  }

  private String memberName(List<Object> members, long identifier) {
    for (Object raw : members) {
      Map<String, Object> header = asMap(asMap(raw).get("header"));
      if (nodeIdFromHeader(header) == identifier) return string(header.get("name"));
    }
    return "field_" + identifier;
  }

  private String fieldName(long identifier) {
    for (Map<String, Object> declaration : declarations.values()) {
      Map<String, Object> data = asMap(declaration.get("data"));
      for (Object raw : nullableList(data.get("fields"))) {
        Map<String, Object> header = asMap(asMap(raw).get("header"));
        if (nodeIdFromHeader(header) == identifier) return string(header.get("name"));
      }
      for (Object variantRaw : nullableList(data.get("variants"))) {
        for (Object fieldRaw : nullableList(asMap(variantRaw).get("fields"))) {
          Map<String, Object> header = asMap(asMap(fieldRaw).get("header"));
          if (nodeIdFromHeader(header) == identifier) return string(header.get("name"));
        }
      }
    }
    return "field_" + identifier;
  }

  private long findImplementation(long contract, long record) {
    for (Map.Entry<Long, Map<String, Object>> entry : declarations.entrySet()) {
      if ("implementation".equals(entry.getValue().get("kind"))) {
        Map<String, Object> data = asMap(entry.getValue().get("data"));
        if (number(data.get("contract")) == contract && number(data.get("record")) == record) {
          return entry.getKey();
        }
      }
    }
    return -1L;
  }

  private static Map<String, Object> findVariant(
      Map<String, Object> declaration, Long variantId) {
    if (variantId == null) return null;
    for (Object raw : nullableList(declaration.get("variants"))) {
      Map<String, Object> variant = asMap(raw);
      if (nodeIdFromHeader(asMap(variant.get("header"))) == variantId) return variant;
    }
    return null;
  }

  private Object materialize(
      long declarationId, Map<String, Object> variant, Map<String, Object> fields) {
    Map<String, Object> declaration = asMap(declarations.get(declarationId).get("data"));
    String declarationName = identifier(string(asMap(declaration.get("header")).get("name")));
    String variantName =
        variant == null
            ? ""
            : identifier(string(asMap(variant.get("header")).get("name")));
    List<Object> members =
        variant == null ? asList(declaration.get("fields")) : asList(variant.get("fields"));
    Object[] arguments = new Object[members.size()];
    for (int index = 0; index < members.size(); index++) {
      String name = string(asMap(asMap(members.get(index)).get("header")).get("name"));
      arguments[index] = fields.get(name);
    }
    try {
      Class<?> type =
          Class.forName("org.polyrust.generated.Generated$" + declarationName + variantName);
      var constructors = type.getDeclaredConstructors();
      if (constructors.length != 1) {
        throw new IllegalStateException("generated aggregate has unexpected constructors");
      }
      return constructors[0].newInstance(arguments);
    } catch (ReflectiveOperationException error) {
      throw new IllegalStateException("cannot materialize generated aggregate", error);
    }
  }

  private static String identifier(String name) {
    return switch (name) {
      case "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
          "class", "const", "continue", "default", "do", "double", "else", "enum",
          "extends", "final", "finally", "float", "for", "goto", "if", "implements",
          "import", "instanceof", "int", "interface", "long", "native", "new", "package",
          "private", "protected", "public", "return", "short", "static", "strictfp",
          "super", "switch", "synchronized", "this", "throw", "throws", "transient",
          "try", "void", "volatile", "while", "true", "false", "null", "record", "sealed",
          "permits", "non-sealed", "var", "yield" -> name + "_";
      default -> name;
    };
  }

// POLYRUST-BEGIN numeric-static-methods
  static PolyResult<Integer> checkedI32(long value) {
    return checkedI32(BigInteger.valueOf(value));
  }

  static PolyResult<Integer> checkedI32(BigInteger value) {
    if (value.compareTo(BigInteger.valueOf(Integer.MIN_VALUE)) < 0
        || value.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
      return PolyResult.fail("integer_overflow", "i32 result is out of range");
    }
    return PolyResult.ok(value.intValue());
  }

  static PolyResult<Long> checkedI64(BigInteger value) {
    if (value.compareTo(BigInteger.valueOf(Long.MIN_VALUE)) < 0
        || value.compareTo(BigInteger.valueOf(Long.MAX_VALUE)) > 0) {
      return PolyResult.fail("integer_overflow", "i64 result is out of range");
    }
    return PolyResult.ok(value.longValue());
  }

  static int wrappingI32(long value) {
    return (int) value;
  }

  static long wrappingI64(BigInteger value) {
    BigInteger modulus = BigInteger.ONE.shiftLeft(64);
    BigInteger wrapped = value.mod(modulus);
    if (wrapped.testBit(63)) wrapped = wrapped.subtract(modulus);
    return wrapped.longValue();
  }
// POLYRUST-END numeric-static-methods

  static PolyResult<Integer> scalarLength(String value) {
    for (int index = 0; index < value.length(); index++) {
      char character = value.charAt(index);
      if (Character.isHighSurrogate(character)) {
        if (index + 1 >= value.length() || !Character.isLowSurrogate(value.charAt(index + 1))) {
          return PolyResult.fail("invalid_unicode", "surrogate is not a Unicode scalar");
        }
        index++;
      } else if (Character.isLowSurrogate(character)) {
        return PolyResult.fail("invalid_unicode", "surrogate is not a Unicode scalar");
      }
    }
    return PolyResult.ok(value.codePointCount(0, value.length()));
  }

  static <T> List<T> listAppend(List<T> items, T item) {
    List<T> result = new ArrayList<>(items);
    result.add(item);
    return List.copyOf(result);
  }

  static boolean deepEqual(Object left, Object right) {
    if (left instanceof PolyRecord record) left = record.polyValue();
    if (right instanceof PolyRecord record) right = record.polyValue();
    if (left instanceof Double leftDouble && right instanceof Double rightDouble) {
      return Double.doubleToRawLongBits(leftDouble) == Double.doubleToRawLongBits(rightDouble)
          || (Double.isNaN(leftDouble) && Double.isNaN(rightDouble));
    }
    if (left instanceof List<?> leftList && right instanceof List<?> rightList) {
      if (leftList.size() != rightList.size()) return false;
      for (int index = 0; index < leftList.size(); index++) {
        if (!deepEqual(leftList.get(index), rightList.get(index))) return false;
      }
      return true;
    }
    if (left instanceof Map<?, ?> leftMap && right instanceof Map<?, ?> rightMap) {
      if (!leftMap.keySet().equals(rightMap.keySet())) return false;
      for (Object key : leftMap.keySet()) {
        if (!deepEqual(leftMap.get(key), rightMap.get(key))) return false;
      }
      return true;
    }
    return Objects.equals(left, right);
  }

  static boolean semanticEqual(Object left, Object right) {
    if (left instanceof PolyRecord record) left = record.polyValue();
    if (right instanceof PolyRecord record) right = record.polyValue();
    if (left instanceof Double leftDouble && right instanceof Double rightDouble) {
      return leftDouble.doubleValue() == rightDouble.doubleValue();
    }
    if (left instanceof List<?> leftList && right instanceof List<?> rightList) {
      if (leftList.size() != rightList.size()) return false;
      for (int index = 0; index < leftList.size(); index++) {
        if (!semanticEqual(leftList.get(index), rightList.get(index))) return false;
      }
      return true;
    }
    if (left instanceof Map<?, ?> leftMap && right instanceof Map<?, ?> rightMap) {
      if (!leftMap.keySet().equals(rightMap.keySet())) return false;
      for (Object key : leftMap.keySet()) {
        if (!semanticEqual(leftMap.get(key), rightMap.get(key))) return false;
      }
      return true;
    }
    return Objects.equals(left, right);
  }

  static <T> PolyResult<T> cast(PolyResult<?> result) {
    return (PolyResult<T>) result;
  }

  private static PolyResult<Object> ok(Object value) {
    return PolyResult.ok(value);
  }

  private static PolyResult<Object> fail(String code, String message) {
    return PolyResult.fail(code, message);
  }

  private static PolyResult<Object> map(
      PolyResult<Object> result, java.util.function.Function<Object, Object> mapper) {
    return result.ok() ? ok(mapper.apply(result.value())) : result;
  }

  private static int compare(Object left, Object right) {
    if (left instanceof Integer) return Integer.compare((Integer) left, (Integer) right);
    if (left instanceof Long) return Long.compare((Long) left, (Long) right);
    if (left instanceof Double) return Double.compare((Double) left, (Double) right);
    return ((String) left).compareTo((String) right);
  }

  private static List<Object> concat(List<Object> left, List<Object> right) {
    List<Object> result = new ArrayList<>(left);
    result.addAll(right);
    return List.copyOf(result);
  }

  private static List<Object> replaceBytesAll(
      List<Object> source, List<Object> needle, List<Object> replacement) {
    List<Object> result = new ArrayList<>();
    if (needle.isEmpty()) {
      result.addAll(replacement);
      for (Object value : source) {
        result.add(value);
        result.addAll(replacement);
      }
      return List.copyOf(result);
    }
    for (int offset = 0; offset < source.size();) {
      boolean matches = offset + needle.size() <= source.size();
      for (int index = 0; matches && index < needle.size(); index++) {
        matches = Objects.equals(source.get(offset + index), needle.get(index));
      }
      if (matches) {
        result.addAll(replacement);
        offset += needle.size();
      } else {
        result.add(source.get(offset));
        offset++;
      }
    }
    return List.copyOf(result);
  }

  private static String replaceManyLiteral(String source, List<Object> values) {
    StringBuilder output = new StringBuilder();
    int offset = 0;
    while (true) {
      String remaining = source.substring(offset);
      boolean matched = false;
      for (int index = 1; index < values.size(); index += 2) {
        String needle = (String) values.get(index);
        if (!remaining.startsWith(needle)) continue;
        output.append((String) values.get(index + 1));
        if (!needle.isEmpty()) {
          offset += needle.length();
        } else if (remaining.isEmpty()) {
          return output.toString();
        } else {
          int width = Character.charCount(remaining.codePointAt(0));
          output.append(remaining, 0, width);
          offset += width;
        }
        matched = true;
        break;
      }
      if (matched) continue;
      if (remaining.isEmpty()) return output.toString();
      int width = Character.charCount(remaining.codePointAt(0));
      output.append(remaining, 0, width);
      offset += width;
    }
  }

  private static String truncateUtf8Bytes(String source, double budget) {
    int consumed = 0;
    int offset = 0;
    while (offset < source.length()) {
      int codePoint = source.codePointAt(offset);
      int end = offset + Character.charCount(codePoint);
      consumed +=
          codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
      if (consumed == budget) return source.substring(0, end);
      if (consumed > budget) return source.substring(0, offset);
      offset = end;
    }
    return source;
  }

  private static String trimStartScalars(String source, String characters) {
    int start = 0;
    while (start < source.length()) {
      int codePoint = source.codePointAt(start);
      if (!containsCodePoint(characters, codePoint)) break;
      start += Character.charCount(codePoint);
    }
    return source.substring(start);
  }

  private static String trimEndScalars(String source, String characters) {
    int end = source.length();
    while (end > 0) {
      int codePoint = source.codePointBefore(end);
      if (!containsCodePoint(characters, codePoint)) break;
      end -= Character.charCount(codePoint);
    }
    return source.substring(0, end);
  }

  private static boolean containsCodePoint(String value, int target) {
    return value.codePoints().anyMatch(codePoint -> codePoint == target);
  }

// POLYRUST-BEGIN utf8-method
  private static PolyResult<Object> stringFromUtf8(List<Object> values) {
    byte[] bytes = new byte[values.size()];
    for (int index = 0; index < values.size(); index++) {
      bytes[index] = (byte) ((Integer) values.get(index)).intValue();
    }
    try {
      return ok(
          StandardCharsets.UTF_8
              .newDecoder()
              .onMalformedInput(CodingErrorAction.REPORT)
              .onUnmappableCharacter(CodingErrorAction.REPORT)
              .decode(ByteBuffer.wrap(bytes))
              .toString());
    } catch (CharacterCodingException error) {
      return fail("invalid_utf8", "invalid UTF-8");
    }
  }
// POLYRUST-END utf8-method

  private static List<Integer> immutableIntegers(List<Object> values) {
    List<Integer> result = new ArrayList<>();
    for (Object value : values) result.add(Math.toIntExact(number(value)));
    return List.copyOf(result);
  }

  private static Map<String, Object> copy(Map<String, Object> source) {
    return new LinkedHashMap<>(source);
  }

  private static long nodeId(Map<String, Object> declaration) {
    return nodeIdFromHeader(asMap(asMap(declaration.get("data")).get("header")));
  }

  private static long nodeIdFromHeader(Map<String, Object> header) {
    return number(asMap(header.get("node")).get("id"));
  }

  private static long number(Object value) {
    return ((Number) value).longValue();
  }

  private static String string(Object value) {
    return (String) value;
  }

  private static Map<String, Object> asMap(Object value) {
    return (Map<String, Object>) value;
  }

  private static List<Object> asList(Object value) {
    return (List<Object>) value;
  }

  private static List<Object> nullableList(Object value) {
    return value == null ? List.of() : asList(value);
  }

  private static final class JsonParser {
    private final String source;
    private int offset;

    JsonParser(String source) {
      this.source = source;
    }

    Object parse() {
      Object result = parseValue();
      whitespace();
      if (offset != source.length()) throw new IllegalArgumentException("trailing JSON");
      return result;
    }

    private Object parseValue() {
      whitespace();
      if (offset >= source.length()) throw new IllegalArgumentException("unexpected end of JSON");
      return switch (source.charAt(offset)) {
        case '{' -> parseObject();
        case '[' -> parseArray();
        case '"' -> parseString();
        case 't' -> keyword("true", Boolean.TRUE);
        case 'f' -> keyword("false", Boolean.FALSE);
        case 'n' -> keyword("null", null);
        default -> parseNumber();
      };
    }

    private Map<String, Object> parseObject() {
      offset++;
      Map<String, Object> result = new LinkedHashMap<>();
      whitespace();
      if (take('}')) return result;
      while (true) {
        String key = parseString();
        whitespace();
        expect(':');
        result.put(key, parseValue());
        whitespace();
        if (take('}')) return result;
        expect(',');
        whitespace();
      }
    }

    private List<Object> parseArray() {
      offset++;
      List<Object> result = new ArrayList<>();
      whitespace();
      if (take(']')) return result;
      while (true) {
        result.add(parseValue());
        whitespace();
        if (take(']')) return result;
        expect(',');
      }
    }

    private String parseString() {
      whitespace();
      expect('"');
      StringBuilder result = new StringBuilder();
      while (offset < source.length()) {
        char character = source.charAt(offset++);
        if (character == '"') return result.toString();
        if (character != '\\') {
          result.append(character);
          continue;
        }
        char escaped = source.charAt(offset++);
        switch (escaped) {
          case '"' -> result.append('"');
          case '\\' -> result.append('\\');
          case '/' -> result.append('/');
          case 'b' -> result.append('\b');
          case 'f' -> result.append('\f');
          case 'n' -> result.append('\n');
          case 'r' -> result.append('\r');
          case 't' -> result.append('\t');
          case 'u' -> {
            int code = Integer.parseInt(source.substring(offset, offset + 4), 16);
            result.append((char) code);
            offset += 4;
          }
          default -> throw new IllegalArgumentException("invalid JSON escape");
        }
      }
      throw new IllegalArgumentException("unterminated JSON string");
    }

    private Object parseNumber() {
      int start = offset;
      if (source.charAt(offset) == '-') offset++;
      while (offset < source.length() && Character.isDigit(source.charAt(offset))) offset++;
      boolean decimal = false;
      if (offset < source.length() && source.charAt(offset) == '.') {
        decimal = true;
        offset++;
        while (offset < source.length() && Character.isDigit(source.charAt(offset))) offset++;
      }
      if (offset < source.length()
          && (source.charAt(offset) == 'e' || source.charAt(offset) == 'E')) {
        decimal = true;
        offset++;
        if (offset < source.length()
            && (source.charAt(offset) == '+' || source.charAt(offset) == '-')) offset++;
        while (offset < source.length() && Character.isDigit(source.charAt(offset))) offset++;
      }
      String token = source.substring(start, offset);
      return decimal
          ? (Number) Double.valueOf(token)
          : (Number) Long.valueOf(token);
    }

    private Object keyword(String token, Object value) {
      if (!source.startsWith(token, offset)) throw new IllegalArgumentException("invalid JSON token");
      offset += token.length();
      return value;
    }

    private void whitespace() {
      while (offset < source.length()
          && (source.charAt(offset) == ' '
              || source.charAt(offset) == '\n'
              || source.charAt(offset) == '\r'
              || source.charAt(offset) == '\t')) {
        offset++;
      }
    }

    private boolean take(char expected) {
      if (offset < source.length() && source.charAt(offset) == expected) {
        offset++;
        return true;
      }
      return false;
    }

    private void expect(char expected) {
      if (!take(expected)) throw new IllegalArgumentException("expected " + expected);
    }
  }
}
