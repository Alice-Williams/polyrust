// Generated packages receive a copy of this dependency-free strict ESM runtime.
export type PolyError = Readonly<{ code: string; message: string }>;
export type PolyResult<T> =
  | Readonly<{ ok: true; value: T }>
  | Readonly<{ ok: false; error: PolyError }>;
export type PolyOption<T> =
  | Readonly<{ tag: "none" }>
  | Readonly<{ tag: "some"; value: T }>;
export type PolyValueResult<T, E> =
  | Readonly<{ tag: "ok"; value: T }>
  | Readonly<{ tag: "err"; error: E }>;

export const ok = <T>(value: T): PolyResult<T> => ({ ok: true, value });
export const fail = <T>(code: string, message: string): PolyResult<T> => ({ ok: false, error: { code, message } });
export const some = <T>(value: T): PolyOption<T> => ({ tag: "some", value });
export const none = <T>(): PolyOption<T> => ({ tag: "none" });
export const valueOk = <T, E>(value: T): PolyValueResult<T, E> => ({ tag: "ok", value });
export const valueErr = <T, E>(error: E): PolyValueResult<T, E> => ({ tag: "err", error });

export const checkedI32 = (value: number): PolyResult<number> =>
  Number.isInteger(value) && value >= -2147483648 && value <= 2147483647
    ? ok(Object.is(value, -0) ? 0 : value)
    : fail("integer_overflow", "i32 result is out of range");
export const checkedI64 = (value: bigint): PolyResult<bigint> =>
  value >= -9223372036854775808n && value <= 9223372036854775807n
    ? ok(value)
    : fail("integer_overflow", "i64 result is out of range");
export const wrappingI32 = (value: number): number => Number(BigInt.asIntN(32, BigInt(value)));
export const wrappingI64 = (value: bigint): bigint => BigInt.asIntN(64, value);
export const scalarLength = (value: string): PolyResult<number> => {
  for (let index = 0; index < value.length; index += 1) {
    const unit = value.charCodeAt(index);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return fail("invalid_unicode", "lone high surrogate");
      index += 1;
    } else if (unit >= 0xdc00 && unit <= 0xdfff) {
      return fail("invalid_unicode", "lone low surrogate");
    }
  }
  return ok([...value].length);
};
type Json = any;
type Env = Map<string, unknown>;
type Flow = { returned: boolean; result: PolyResult<unknown> };

// POLYRUST-BEGIN top.string-index-of-literal
export const indexOfLiteralScalars = (
  source: string,
  needle: string,
): PolyResult<PolyOption<bigint>> => {
  const utf16Index = source.indexOf(needle);
  if (utf16Index < 0) return ok(none<bigint>());
  const prefixLength = scalarLength(source.slice(0, utf16Index));
  if (!prefixLength.ok) return prefixLength;
  return ok(some(BigInt(prefixLength.value)));
};
// POLYRUST-END top.string-index-of-literal
// POLYRUST-BEGIN top.string-slice-scalars
export const sliceScalars = (source: string, rawStart: bigint, rawEnd: bigint): string => {
  const scalars = Array.from(source);
  const length = BigInt(scalars.length);
  const clamp = (value: bigint): bigint => value < 0n ? 0n : value > length ? length : value;
  const start = clamp(rawStart);
  const end = clamp(rawEnd);
  return start >= end ? "" : scalars.slice(Number(start), Number(end)).join("");
};
// POLYRUST-END top.string-slice-scalars

// POLYRUST-BEGIN top.string-replace-all
export const replaceAllLiteral = (source: string, needle: string, replacement: string): string => {
  if (needle !== "") return source.split(needle).join(replacement);
  const scalars = Array.from(source);
  return scalars.length === 0
    ? replacement
    : replacement + scalars.join(replacement) + replacement;
};
// POLYRUST-END top.string-replace-all
// POLYRUST-BEGIN top.bytes-replace-all
export const replaceBytesAll = (
  source: readonly number[],
  needle: readonly number[],
  replacement: readonly number[],
): readonly number[] => {
  const output: number[] = [];
  if (needle.length === 0) {
    output.push(...replacement);
    for (const byte of source) output.push(byte, ...replacement);
    return output;
  }
  for (let offset = 0; offset < source.length;) {
    const matches = offset + needle.length <= source.length
      && needle.every((byte, index) => source[offset + index] === byte);
    if (matches) {
      output.push(...replacement);
      offset += needle.length;
    } else {
      output.push(source[offset]!);
      offset += 1;
    }
  }
  return output;
};
// POLYRUST-END top.bytes-replace-all
// POLYRUST-BEGIN top.string-replace-many
export const replaceManyLiteral = (
  source: string,
  mappings: readonly (readonly [string, string])[],
): string => {
  let output = "";
  let offset = 0;
  while (true) {
    const remaining = source.slice(offset);
    const mapping = mappings.find(([needle]) => remaining.startsWith(needle));
    if (mapping !== undefined) {
      const [needle, replacement] = mapping;
      output += replacement;
      if (needle.length > 0) {
        offset += needle.length;
        continue;
      }
      const scalar = Array.from(remaining)[0];
      if (scalar === undefined) break;
      output += scalar;
      offset += scalar.length;
      continue;
    }
    const scalar = Array.from(remaining)[0];
    if (scalar === undefined) break;
    output += scalar;
    offset += scalar.length;
  }
  return output;
};
// POLYRUST-END top.string-replace-many
// POLYRUST-BEGIN top.string-truncate-utf8
export const truncateUtf8Bytes = (source: string, budget: number): string => {
  let bytes = 0;
  let end = 0;
  for (const scalar of source) {
    const codePoint = scalar.codePointAt(0)!;
    const width = codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
    bytes += width;
    end += scalar.length;
    if (bytes === budget) return source.slice(0, end);
    if (bytes > budget) return source.slice(0, end - scalar.length);
  }
  return source;
};
// POLYRUST-END top.string-truncate-utf8
// POLYRUST-BEGIN top.string-trim-start
export const trimStartScalars = (source: string, characters: string): string => {
  const scalars = Array.from(source);
  const trim = new Set(Array.from(characters));
  let start = 0;
  while (start < scalars.length) {
    const scalar = scalars[start];
    if (scalar === undefined || !trim.has(scalar)) break;
    start += 1;
  }
  return scalars.slice(start).join("");
};
// POLYRUST-END top.string-trim-start
// POLYRUST-BEGIN top.string-trim-end
export const trimEndScalars = (source: string, characters: string): string => {
  const scalars = Array.from(source);
  const trim = new Set(Array.from(characters));
  let end = scalars.length;
  while (end > 0) {
    const scalar = scalars[end - 1];
    if (scalar === undefined || !trim.has(scalar)) break;
    end -= 1;
  }
  return scalars.slice(0, end).join("");
};
// POLYRUST-END top.string-trim-end
export const listAppend = <T>(items: readonly T[], item: T): readonly T[] => [...items, item];
// POLYRUST-BEGIN top.list-concat
export const listConcat = <T>(left: readonly T[], right: readonly T[]): readonly T[] => [...left, ...right];
// POLYRUST-END top.list-concat

export class Runtime {
  private readonly declarations = new Map<number, Json>();
  private readonly constants = new Map<number, PolyResult<unknown>>();

  public constructor(private readonly document: Json) {
    for (const declaration of document.module.declarations as Json[]) {
      this.declarations.set(declaration.data.header.node.id as number, declaration);
    }
  }

  public invoke(functionId: number, arguments_: readonly unknown[]): PolyResult<unknown> {
    const declaration = this.declarations.get(functionId);
    if (declaration?.kind !== "function") return fail("invalid_call", "unknown function " + functionId);
    return this.invokeBody(declaration.data, arguments_, undefined);
  }

  public invokeMethod(implementationId: number, methodId: number, receiver: unknown, arguments_: readonly unknown[]): PolyResult<unknown> {
    const implementation = this.declarations.get(implementationId);
    if (implementation?.kind !== "implementation") return fail("invalid_call", "unknown implementation " + implementationId);
    const method = (implementation.data.methods as Json[]).find((candidate) => candidate.header.node.id === methodId || candidate.contract_method === methodId);
    if (method === undefined) return fail("invalid_call", "unknown method " + methodId);
    return this.invokeBody(method, arguments_, receiver);
  }

  public decode(typed: Json): unknown { return this.value(typed.value); }

  public readConstant(id: number): PolyResult<unknown> { return this.constant(id); }

  private invokeBody(callable: Json, arguments_: readonly unknown[], self: unknown): PolyResult<unknown> {
    const env: Env = new Map();
    (callable.parameters as Json[]).forEach((parameter, index) => env.set(parameter.header.name as string, arguments_[index]));
    return this.block(callable.body, env, self).result;
  }

  private value(value: Json): unknown {
    switch (value.kind as string) {
      case "unit": return undefined;
      case "bool": case "i32": case "string": case "char": return value.data;
      case "i64": return BigInt(value.data as string | number);
      case "f64": {
        const bytes = new ArrayBuffer(8);
        new DataView(bytes).setBigUint64(0, BigInt(value.data as string | number), false);
        return new DataView(bytes).getFloat64(0, false);
      }
      case "bytes": return Object.freeze([...(value.data as number[])]);
      case "list": return Object.freeze((value.data as Json[]).map((item) => this.value(item)));
      case "none": return none();
      case "some": return some(this.value(value.data));
      case "ok": return valueOk(this.value(value.data));
      case "err": return valueErr(this.value(value.data));
      case "record": return this.aggregateValue(value.data, undefined);
      case "enum": return this.aggregateValue(value.data, value.data.variant as number);
      default: return undefined;
    }
  }

  private aggregateValue(data: Json, variantId: number | undefined): unknown {
    const declaration = this.declarations.get(data.declaration as number)?.data;
    const variant = variantId === undefined ? undefined : (declaration.variants as Json[]).find((item) => item.header.node.id === variantId);
    const result: Json = { __polyDecl: data.declaration };
    if (variant !== undefined) result.tag = variant.header.name;
    const members = variant?.fields ?? declaration.fields;
    for (const entry of data.fields as Json[]) result[this.memberName(members, entry.field)] = this.value(entry.value);
    return Object.freeze(result);
  }

  private expression(expression: Json, env: Env, self: unknown): PolyResult<unknown> {
    const data = expression.data;
    switch (expression.kind as string) {
      case "literal": return ok(this.value(data.value));
      case "local": return ok(env.get(data.name as string));
      case "self_value": return ok(self);
      case "constant": return this.constant(data.declaration as number);
      case "construct_none": return ok(none());
      case "construct_some": return this.map(this.expression(data.value, env, self), some);
      case "construct_ok": return this.map(this.expression(data.value, env, self), valueOk);
      case "construct_err": return this.map(this.expression(data.value, env, self), valueErr);
      case "construct_list": return this.sequence(data.elements as Json[], env, self);
      case "construct_record": return this.construct(data.declaration, undefined, data.fields, env, self);
      case "construct_enum": return this.construct(data.declaration, data.variant, data.fields, env, self);
      case "field": return this.map(this.expression(data.base, env, self), (base) => (base as Json)[this.fieldName(data.field)]);
      case "call": return this.arguments(data.arguments, env, self, (values) => this.invoke(data.function, values));
      case "method_call": return this.arguments(data.arguments, env, self, (values) => {
        const receiver = this.expression(data.receiver, env, self);
        if (!receiver.ok) return receiver;
        let implementation = data.dispatch.data.implementation as number | undefined;
        if (data.dispatch.kind === "contract") {
          implementation = this.findImplementation(data.dispatch.data.contract, (receiver.value as Json).__polyDecl as number);
        }
        return this.invokeMethod(implementation as number, data.dispatch.data.method, receiver.value, values);
      });
      case "intrinsic": return this.arguments(data.arguments, env, self, (values) => this.intrinsic(data.operation, values));
      case "if": {
        const condition = this.expression(data.condition, env, self);
        return condition.ok ? this.block(condition.value ? data.then_block : data.else_block, new Map(env), self).result : condition;
      }
      case "match": return this.match(data, env, self);
      case "block": return this.block(data, new Map(env), self).result;
      default: return fail("invalid_expression", "unknown expression " + String(expression.kind));
    }
  }

  private block(block: Json, env: Env, self: unknown): Flow {
    for (const statement of block.statements as Json[]) {
      const data = statement.data;
      if (statement.kind === "let") {
        const value = this.expression(data.value, env, self);
        if (!value.ok) return { returned: true, result: value };
        env.set(data.name as string, value.value);
      } else if (statement.kind === "expression") {
        const value = this.expression(data.value, env, self);
        if (!value.ok) return { returned: true, result: value };
      } else if (statement.kind === "return") {
        return { returned: true, result: data.value === null ? ok(undefined) : this.expression(data.value, env, self) };
      } else if (statement.kind === "for_each") {
        const values = this.expression(data.iterable, env, self);
        if (!values.ok) return { returned: true, result: values };
        for (const item of values.value as readonly unknown[]) {
          const inner = new Map(env);
          inner.set(data.binding as string, item);
          const flow = this.block(data.body, inner, self);
          if (flow.returned || !flow.result.ok) return flow;
        }
      }
    }
    return { returned: false, result: block.result === null ? ok(undefined) : this.expression(block.result, env, self) };
  }

  private match(data: Json, env: Env, self: unknown): PolyResult<unknown> {
    const value = this.expression(data.value, env, self);
    if (!value.ok) return value;
    for (const arm of data.arms as Json[]) {
      const bindings = this.pattern(arm.pattern, value.value);
      if (bindings !== undefined) {
        const inner = new Map(env);
        for (const [name, item] of bindings) inner.set(name, item);
        return this.block(arm.body, inner, self).result;
      }
    }
    return fail("non_exhaustive_match", "checked match had no matching arm");
  }

  private pattern(pattern: Json, value: unknown): Map<string, unknown> | undefined {
    const data = pattern.data;
    const result = new Map<string, unknown>();
    const object = value as Json;
    switch (pattern.kind as string) {
      case "wildcard": return result;
      case "bool": return value === data.value ? result : undefined;
      case "none": return object.tag === "none" ? result : undefined;
      case "some": if (object.tag !== "some") return undefined; result.set(data.binding, object.value); return result;
      case "ok": if (object.tag !== "ok") return undefined; result.set(data.binding, object.value); return result;
      case "err": if (object.tag !== "err") return undefined; result.set(data.binding, object.error); return result;
      case "enum_variant": {
        const declaration = this.declarations.get(data.declaration as number)?.data;
        const variant = (declaration.variants as Json[]).find((item) => item.header.node.id === data.variant);
        if (object.tag !== variant.header.name) return undefined;
        for (const binding of data.bindings as Json[]) result.set(binding.binding, object[this.memberName(variant.fields, binding.field)]);
        return result;
      }
      default: return undefined;
    }
  }

  private intrinsic(name: string, values: readonly unknown[]): PolyResult<unknown> {
    const a = values[0] as Json;
    const b = values[1] as Json;
    const c = values[2] as Json;
    switch (name) {
      case "bool_not": return ok(!a);
      case "bool_and": return ok(a && b);
      case "bool_or": return ok(a || b);
      case "equal": return ok(this.equal(a, b));
      case "not_equal": return ok(!this.equal(a, b));
      case "less": return ok(a < b);
      case "less_equal": return ok(a <= b);
      case "greater": return ok(a > b);
      case "greater_equal": return ok(a >= b);
      case "int_neg_checked": return this.checked(-a, a);
      case "int_add_checked": return this.checked(a + b, a);
      case "int_sub_checked": return this.checked(a - b, a);
      case "int_mul_checked": return this.checked(a * b, a);
      case "int_div_checked":
        if (b === 0 || b === 0n) return fail("division_by_zero", "integer division by zero");
        return this.checked(a / b, a);
      case "int_rem_checked":
        if (b === 0 || b === 0n) return fail("division_by_zero", "integer remainder by zero");
        return this.checked(a % b, a);
      case "int_neg_wrapping": return ok(typeof a === "bigint" ? wrappingI64(-a) : wrappingI32(-a));
      case "int_add_wrapping": return ok(typeof a === "bigint" ? wrappingI64(a + b) : wrappingI32(a + b));
      case "int_sub_wrapping": return ok(typeof a === "bigint" ? wrappingI64(a - b) : wrappingI32(a - b));
      case "int_mul_wrapping": return ok(typeof a === "bigint" ? wrappingI64(a * b) : wrappingI32(a * b));
      case "int_bit_not": return ok(~a);
      case "int_bit_and": return ok(a & b);
      case "int_bit_or": return ok(a | b);
      case "int_bit_xor": return ok(a ^ b);
      case "int_shift_left_checked": return this.checked(a << b, a);
      case "int_shift_right_checked": return ok(a >> b);
      case "float_neg": return ok(-a);
      case "float_trunc": return ok(Math.trunc(a));
      case "float_is_nan": return ok(Number.isNaN(a));
      case "float_is_negative_zero": return ok(Object.is(a, -0));
      case "float_abs": {
        const view = new DataView(new ArrayBuffer(8));
        view.setFloat64(0, a);
        view.setBigUint64(0, view.getBigUint64(0) & 0x7fff_ffff_ffff_ffffn);
        return ok(view.getFloat64(0));
      }
      case "float_add": return ok(a + b);
      case "float_sub": return ok(a - b);
      case "float_mul": return ok(a * b);
      case "float_div": return ok(a / b);
      case "float_rem_trunc": return ok(a % b);
      case "string_concat": return ok(a + b);
      case "string_scalar_length": return scalarLength(a);
      // POLYRUST-BEGIN case.string-utf16-length
      case "string_utf16_length": return ok(BigInt(a.length));
      // POLYRUST-END case.string-utf16-length
      // POLYRUST-BEGIN case.string-index-of-literal
      case "string_index_of_literal": return indexOfLiteralScalars(a, b);
      // POLYRUST-END case.string-index-of-literal
      // POLYRUST-BEGIN case.string-slice-scalars
      case "string_slice_scalars": return ok(sliceScalars(a, b, c));
      // POLYRUST-END case.string-slice-scalars
      case "string_is_empty": return ok(a.length === 0);
      case "string_contains": return ok(a.includes(b));
      case "string_starts_with": return ok(a.startsWith(b));
      case "string_strip_prefix": return ok(b.length > 0 && a.startsWith(b) ? a.slice(b.length) : a);
      case "string_ends_with": return ok(a.endsWith(b));
      // POLYRUST-BEGIN case.string-replace-all
      case "string_replace_all": return ok(replaceAllLiteral(a, b, c));
      // POLYRUST-END case.string-replace-all
      // POLYRUST-BEGIN case.string-replace-many
      case "string_replace_many": {
        const mappings: [string, string][] = [];
        for (let index = 1; index < values.length; index += 2) {
          mappings.push([values[index] as string, values[index + 1] as string]);
        }
        return ok(replaceManyLiteral(a, mappings));
      }
      // POLYRUST-END case.string-replace-many
      // POLYRUST-BEGIN case.string-truncate-utf8
      case "string_truncate_utf8_bytes": return ok(truncateUtf8Bytes(a, b));
      // POLYRUST-END case.string-truncate-utf8
      // POLYRUST-BEGIN case.string-trim-start
      case "string_trim_start": return ok(trimStartScalars(a, b));
      // POLYRUST-END case.string-trim-start
      // POLYRUST-BEGIN case.string-trim-end
      case "string_trim_end": return ok(trimEndScalars(a, b));
      // POLYRUST-END case.string-trim-end
      // POLYRUST-BEGIN case.list-concat
      case "bytes_concat": case "list_concat": return ok(listConcat(a, b));
      // POLYRUST-END case.list-concat
      // POLYRUST-BEGIN case.bytes-replace-all
      case "bytes_replace_all": return ok(replaceBytesAll(a, b, c));
      // POLYRUST-END case.bytes-replace-all
      case "bytes_length": case "list_length": return checkedI32(a.length);
      case "bytes_is_empty": case "list_is_empty": return ok(a.length === 0);
      case "list_get_checked": return Number(b) >= 0 && Number(b) < a.length ? ok(a[Number(b)]) : fail("index_out_of_bounds", "list index out of bounds");
      case "list_append": return ok(listAppend(a, b));
      case "list_contains": return ok(a.some((item: unknown) => this.equal(item, b)));
      // POLYRUST-BEGIN case.list-index-of
      case "list_index_of": {
        const index = a.findIndex((item: unknown) => this.equal(item, b));
        return ok(index < 0 ? none<bigint>() : some(BigInt(index)));
      }
      // POLYRUST-END case.list-index-of
      case "option_is_some": return ok(a.tag === "some");
      case "option_is_none": return ok(a.tag === "none");
      case "option_unwrap_or": return ok(a.tag === "some" ? a.value : b);
      case "result_is_ok": return ok(a.tag === "ok");
      case "result_is_err": return ok(a.tag === "err");
      case "widen_i32_to_i64": return ok(BigInt(a));
      case "narrow_i64_to_i32_checked": return a < -2147483648n || a > 2147483647n ? fail("integer_overflow", "i64 does not fit i32") : checkedI32(Number(a));
      // POLYRUST-BEGIN case.string-to-utf8
      case "string_to_utf8": return ok(Object.freeze([...new TextEncoder().encode(a)]));
      // POLYRUST-END case.string-to-utf8
      // POLYRUST-BEGIN case.string-from-utf8
      case "string_from_utf8_checked": {
        try { return ok(new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(a))); }
        catch { return fail("invalid_utf8", "invalid UTF-8"); }
      }
      // POLYRUST-END case.string-from-utf8
      default: return fail("invalid_intrinsic", "unknown intrinsic " + name);
    }
  }

  private checked(value: Json, exemplar: Json): PolyResult<unknown> {
    return typeof exemplar === "bigint" ? checkedI64(value) : checkedI32(value);
  }

  private equal(left: Json, right: Json): boolean {
    if (typeof left !== "object" || typeof right !== "object" || left === null || right === null) return left === right;
    const keys = Object.keys(left);
    return keys.length === Object.keys(right).length && keys.every((key) => this.equal(left[key], right[key]));
  }

  private map<T>(result: PolyResult<unknown>, mapper: (value: any) => T): PolyResult<T> {
    return result.ok ? ok(mapper(result.value)) : result;
  }

  private sequence(expressions: Json[], env: Env, self: unknown): PolyResult<readonly unknown[]> {
    const values: unknown[] = [];
    for (const expression of expressions) {
      const value = this.expression(expression, env, self);
      if (!value.ok) return value;
      values.push(value.value);
    }
    return ok(Object.freeze(values));
  }

  private arguments(expressions: Json[], env: Env, self: unknown, invoke: (values: readonly unknown[]) => PolyResult<unknown>): PolyResult<unknown> {
    const values = this.sequence(expressions, env, self);
    return values.ok ? invoke(values.value) : values;
  }

  private construct(declarationId: number, variantId: number | undefined, fields: Json[], env: Env, self: unknown): PolyResult<unknown> {
    const declaration = this.declarations.get(declarationId)?.data;
    const variant = variantId === undefined ? undefined : (declaration.variants as Json[]).find((item) => item.header.node.id === variantId);
    const result: Json = { __polyDecl: declarationId };
    if (variant !== undefined) result.tag = variant.header.name;
    const members = variant?.fields ?? declaration.fields;
    for (const field of fields) {
      const value = this.expression(field.value, env, self);
      if (!value.ok) return value;
      result[this.memberName(members, field.field)] = value.value;
    }
    return ok(Object.freeze(result));
  }

  private constant(id: number): PolyResult<unknown> {
    const cached = this.constants.get(id);
    if (cached !== undefined) return cached;
    const declaration = this.declarations.get(id);
    if (declaration?.kind !== "constant") return fail("invalid_constant", "unknown constant " + id);
    const value = this.constantExpression(declaration.data.value);
    this.constants.set(id, value);
    return value;
  }

  private constantExpression(expression: Json): PolyResult<unknown> {
    const data = expression.data;
    switch (expression.kind as string) {
      case "literal": return ok(this.value(data.value));
      case "reference": return this.constant(data.declaration);
      case "none": return ok(none());
      case "some": return this.map(this.constantExpression(data.value), some);
      case "ok": return this.map(this.constantExpression(data.value), valueOk);
      case "err": return this.map(this.constantExpression(data.value), valueErr);
      case "list": {
        const values: unknown[] = [];
        for (const item of data.elements as Json[]) { const value = this.constantExpression(item); if (!value.ok) return value; values.push(value.value); }
        return ok(Object.freeze(values));
      }
      case "record": case "enum": {
        const result: Json = { __polyDecl: data.declaration };
        const declaration = this.declarations.get(data.declaration)?.data;
        const variant = expression.kind === "enum" ? (declaration.variants as Json[]).find((item) => item.header.node.id === data.variant) : undefined;
        if (variant !== undefined) result.tag = variant.header.name;
        const members = variant?.fields ?? declaration.fields;
        for (const field of data.fields as Json[]) { const value = this.constantExpression(field.value); if (!value.ok) return value; result[this.memberName(members, field.field)] = value.value; }
        return ok(Object.freeze(result));
      }
      case "intrinsic": {
        const values: unknown[] = [];
        for (const item of data.arguments as Json[]) { const value = this.constantExpression(item); if (!value.ok) return value; values.push(value.value); }
        return this.intrinsic(data.operation, values);
      }
      default: return fail("invalid_constant", "unknown constant expression " + String(expression.kind));
    }
  }

  private memberName(members: Json[], id: number): string {
    return (members.find((item) => item.header.node.id === id)?.header.name ?? "field_" + id) as string;
  }

  private fieldName(id: number): string {
    for (const declaration of this.declarations.values()) {
      const data = declaration.data;
      for (const field of (data.fields ?? []) as Json[]) if (field.header.node.id === id) return field.header.name as string;
      for (const variant of (data.variants ?? []) as Json[]) for (const field of (variant.fields ?? []) as Json[]) if (field.header.node.id === id) return field.header.name as string;
    }
    return "field_" + id;
  }

  private findImplementation(contract: number, record: number): number {
    for (const [id, declaration] of this.declarations) {
      if (declaration.kind === "implementation" && declaration.data.contract === contract && declaration.data.record === record) return id;
    }
    return -1;
  }
}
