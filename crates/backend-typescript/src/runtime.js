export const ok = (value) => ({ ok: true, value });
export const fail = (code, message) => ({ ok: false, error: { code, message } });
export const some = (value) => ({ tag: "some", value });
export const none = () => ({ tag: "none" });
export const valueOk = (value) => ({ tag: "ok", value });
export const valueErr = (error) => ({ tag: "err", error });
export const checkedI32 = (value) => Number.isInteger(value) && value >= -2147483648 && value <= 2147483647
    ? ok(Object.is(value, -0) ? 0 : value)
    : fail("integer_overflow", "i32 result is out of range");
export const checkedI64 = (value) => value >= -9223372036854775808n && value <= 9223372036854775807n
    ? ok(value)
    : fail("integer_overflow", "i64 result is out of range");
export const wrappingI32 = (value) => Number(BigInt.asIntN(32, BigInt(value)));
export const wrappingI64 = (value) => BigInt.asIntN(64, value);
export const scalarLength = (value) => {
    for (let index = 0; index < value.length; index += 1) {
        const unit = value.charCodeAt(index);
        if (unit >= 0xd800 && unit <= 0xdbff) {
            const next = value.charCodeAt(index + 1);
            if (!(next >= 0xdc00 && next <= 0xdfff))
                return fail("invalid_unicode", "lone high surrogate");
            index += 1;
        }
        else if (unit >= 0xdc00 && unit <= 0xdfff) {
            return fail("invalid_unicode", "lone low surrogate");
        }
    }
    return ok([...value].length);
};
export const replaceAllLiteral = (source, needle, replacement) => {
    if (needle !== "")
        return source.split(needle).join(replacement);
    const scalars = Array.from(source);
    return scalars.length === 0
        ? replacement
        : replacement + scalars.join(replacement) + replacement;
};
export const replaceManyLiteral = (source, mappings) => {
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
            if (scalar === undefined)
                break;
            output += scalar;
            offset += scalar.length;
            continue;
        }
        const scalar = Array.from(remaining)[0];
        if (scalar === undefined)
            break;
        output += scalar;
        offset += scalar.length;
    }
    return output;
};
export const trimStartScalars = (source, characters) => {
    const scalars = Array.from(source);
    const trim = new Set(Array.from(characters));
    let start = 0;
    while (start < scalars.length) {
        const scalar = scalars[start];
        if (scalar === undefined || !trim.has(scalar))
            break;
        start += 1;
    }
    return scalars.slice(start).join("");
};
export const trimEndScalars = (source, characters) => {
    const scalars = Array.from(source);
    const trim = new Set(Array.from(characters));
    let end = scalars.length;
    while (end > 0) {
        const scalar = scalars[end - 1];
        if (scalar === undefined || !trim.has(scalar))
            break;
        end -= 1;
    }
    return scalars.slice(0, end).join("");
};
export const listAppend = (items, item) => [...items, item];
export const listConcat = (left, right) => [...left, ...right];
export class Runtime {
    document;
    declarations = new Map();
    constants = new Map();
    constructor(document) {
        this.document = document;
        for (const declaration of document.module.declarations) {
            this.declarations.set(declaration.data.header.node.id, declaration);
        }
    }
    invoke(functionId, arguments_) {
        const declaration = this.declarations.get(functionId);
        if (declaration?.kind !== "function")
            return fail("invalid_call", "unknown function " + functionId);
        return this.invokeBody(declaration.data, arguments_, undefined);
    }
    invokeMethod(implementationId, methodId, receiver, arguments_) {
        const implementation = this.declarations.get(implementationId);
        if (implementation?.kind !== "implementation")
            return fail("invalid_call", "unknown implementation " + implementationId);
        const method = implementation.data.methods.find((candidate) => candidate.header.node.id === methodId || candidate.contract_method === methodId);
        if (method === undefined)
            return fail("invalid_call", "unknown method " + methodId);
        return this.invokeBody(method, arguments_, receiver);
    }
    decode(typed) { return this.value(typed.value); }
    readConstant(id) { return this.constant(id); }
    invokeBody(callable, arguments_, self) {
        const env = new Map();
        callable.parameters.forEach((parameter, index) => env.set(parameter.header.name, arguments_[index]));
        return this.block(callable.body, env, self).result;
    }
    value(value) {
        switch (value.kind) {
            case "unit": return undefined;
            case "bool":
            case "i32":
            case "string":
            case "char": return value.data;
            case "i64": return BigInt(value.data);
            case "f64": {
                const bytes = new ArrayBuffer(8);
                new DataView(bytes).setBigUint64(0, BigInt(value.data), false);
                return new DataView(bytes).getFloat64(0, false);
            }
            case "bytes": return Object.freeze([...value.data]);
            case "list": return Object.freeze(value.data.map((item) => this.value(item)));
            case "none": return none();
            case "some": return some(this.value(value.data));
            case "ok": return valueOk(this.value(value.data));
            case "err": return valueErr(this.value(value.data));
            case "record": return this.aggregateValue(value.data, undefined);
            case "enum": return this.aggregateValue(value.data, value.data.variant);
            default: return undefined;
        }
    }
    aggregateValue(data, variantId) {
        const declaration = this.declarations.get(data.declaration)?.data;
        const variant = variantId === undefined ? undefined : declaration.variants.find((item) => item.header.node.id === variantId);
        const result = { __polyDecl: data.declaration };
        if (variant !== undefined)
            result.tag = variant.header.name;
        const members = variant?.fields ?? declaration.fields;
        for (const entry of data.fields)
            result[this.memberName(members, entry.field)] = this.value(entry.value);
        return Object.freeze(result);
    }
    expression(expression, env, self) {
        const data = expression.data;
        switch (expression.kind) {
            case "literal": return ok(this.value(data.value));
            case "local": return ok(env.get(data.name));
            case "self_value": return ok(self);
            case "constant": return this.constant(data.declaration);
            case "construct_none": return ok(none());
            case "construct_some": return this.map(this.expression(data.value, env, self), some);
            case "construct_ok": return this.map(this.expression(data.value, env, self), valueOk);
            case "construct_err": return this.map(this.expression(data.value, env, self), valueErr);
            case "construct_list": return this.sequence(data.elements, env, self);
            case "construct_record": return this.construct(data.declaration, undefined, data.fields, env, self);
            case "construct_enum": return this.construct(data.declaration, data.variant, data.fields, env, self);
            case "field": return this.map(this.expression(data.base, env, self), (base) => base[this.fieldName(data.field)]);
            case "call": return this.arguments(data.arguments, env, self, (values) => this.invoke(data.function, values));
            case "method_call": return this.arguments(data.arguments, env, self, (values) => {
                const receiver = this.expression(data.receiver, env, self);
                if (!receiver.ok)
                    return receiver;
                let implementation = data.dispatch.data.implementation;
                if (data.dispatch.kind === "contract") {
                    implementation = this.findImplementation(data.dispatch.data.contract, receiver.value.__polyDecl);
                }
                return this.invokeMethod(implementation, data.dispatch.data.method, receiver.value, values);
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
    block(block, env, self) {
        for (const statement of block.statements) {
            const data = statement.data;
            if (statement.kind === "let") {
                const value = this.expression(data.value, env, self);
                if (!value.ok)
                    return { returned: true, result: value };
                env.set(data.name, value.value);
            }
            else if (statement.kind === "expression") {
                const value = this.expression(data.value, env, self);
                if (!value.ok)
                    return { returned: true, result: value };
            }
            else if (statement.kind === "return") {
                return { returned: true, result: data.value === null ? ok(undefined) : this.expression(data.value, env, self) };
            }
            else if (statement.kind === "for_each") {
                const values = this.expression(data.iterable, env, self);
                if (!values.ok)
                    return { returned: true, result: values };
                for (const item of values.value) {
                    const inner = new Map(env);
                    inner.set(data.binding, item);
                    const flow = this.block(data.body, inner, self);
                    if (flow.returned || !flow.result.ok)
                        return flow;
                }
            }
        }
        return { returned: false, result: block.result === null ? ok(undefined) : this.expression(block.result, env, self) };
    }
    match(data, env, self) {
        const value = this.expression(data.value, env, self);
        if (!value.ok)
            return value;
        for (const arm of data.arms) {
            const bindings = this.pattern(arm.pattern, value.value);
            if (bindings !== undefined) {
                const inner = new Map(env);
                for (const [name, item] of bindings)
                    inner.set(name, item);
                return this.block(arm.body, inner, self).result;
            }
        }
        return fail("non_exhaustive_match", "checked match had no matching arm");
    }
    pattern(pattern, value) {
        const data = pattern.data;
        const result = new Map();
        const object = value;
        switch (pattern.kind) {
            case "wildcard": return result;
            case "bool": return value === data.value ? result : undefined;
            case "none": return object.tag === "none" ? result : undefined;
            case "some":
                if (object.tag !== "some")
                    return undefined;
                result.set(data.binding, object.value);
                return result;
            case "ok":
                if (object.tag !== "ok")
                    return undefined;
                result.set(data.binding, object.value);
                return result;
            case "err":
                if (object.tag !== "err")
                    return undefined;
                result.set(data.binding, object.error);
                return result;
            case "enum_variant": {
                const declaration = this.declarations.get(data.declaration)?.data;
                const variant = declaration.variants.find((item) => item.header.node.id === data.variant);
                if (object.tag !== variant.header.name)
                    return undefined;
                for (const binding of data.bindings)
                    result.set(binding.binding, object[this.memberName(variant.fields, binding.field)]);
                return result;
            }
            default: return undefined;
        }
    }
    intrinsic(name, values) {
        const a = values[0];
        const b = values[1];
        const c = values[2];
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
                if (b === 0 || b === 0n)
                    return fail("division_by_zero", "integer division by zero");
                return this.checked(a / b, a);
            case "int_rem_checked":
                if (b === 0 || b === 0n)
                    return fail("division_by_zero", "integer remainder by zero");
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
            case "float_add": return ok(a + b);
            case "float_sub": return ok(a - b);
            case "float_mul": return ok(a * b);
            case "float_div": return ok(a / b);
            case "float_rem_trunc": return ok(a % b);
            case "string_concat": return ok(a + b);
            case "string_scalar_length": return scalarLength(a);
            case "string_is_empty": return ok(a.length === 0);
            case "string_contains": return ok(a.includes(b));
            case "string_starts_with": return ok(a.startsWith(b));
            case "string_strip_prefix": return ok(b.length > 0 && a.startsWith(b) ? a.slice(b.length) : a);
            case "string_ends_with": return ok(a.endsWith(b));
            case "string_replace_all": return ok(replaceAllLiteral(a, b, c));
            case "string_replace_many": {
                const mappings = [];
                for (let index = 1; index < values.length; index += 2) {
                    mappings.push([values[index], values[index + 1]]);
                }
                return ok(replaceManyLiteral(a, mappings));
            }
            case "string_trim_start": return ok(trimStartScalars(a, b));
            case "string_trim_end": return ok(trimEndScalars(a, b));
            case "bytes_concat":
            case "list_concat": return ok(listConcat(a, b));
            case "bytes_length":
            case "list_length": return checkedI32(a.length);
            case "bytes_is_empty":
            case "list_is_empty": return ok(a.length === 0);
            case "list_get_checked": return Number(b) >= 0 && Number(b) < a.length ? ok(a[Number(b)]) : fail("index_out_of_bounds", "list index out of bounds");
            case "list_append": return ok(listAppend(a, b));
            case "list_contains": return ok(a.some((item) => this.equal(item, b)));
            case "option_is_some": return ok(a.tag === "some");
            case "option_is_none": return ok(a.tag === "none");
            case "option_unwrap_or": return ok(a.tag === "some" ? a.value : b);
            case "result_is_ok": return ok(a.tag === "ok");
            case "result_is_err": return ok(a.tag === "err");
            case "widen_i32_to_i64": return ok(BigInt(a));
            case "narrow_i64_to_i32_checked": return a < -2147483648n || a > 2147483647n ? fail("integer_overflow", "i64 does not fit i32") : checkedI32(Number(a));
            case "string_to_utf8": return ok(Object.freeze([...new TextEncoder().encode(a)]));
            case "string_from_utf8_checked": {
                try {
                    return ok(new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(a)));
                }
                catch {
                    return fail("invalid_utf8", "invalid UTF-8");
                }
            }
            default: return fail("invalid_intrinsic", "unknown intrinsic " + name);
        }
    }
    checked(value, exemplar) {
        return typeof exemplar === "bigint" ? checkedI64(value) : checkedI32(value);
    }
    equal(left, right) {
        if (Object.is(left, right))
            return true;
        if (typeof left !== "object" || typeof right !== "object" || left === null || right === null)
            return false;
        const keys = Object.keys(left);
        return keys.length === Object.keys(right).length && keys.every((key) => this.equal(left[key], right[key]));
    }
    map(result, mapper) {
        return result.ok ? ok(mapper(result.value)) : result;
    }
    sequence(expressions, env, self) {
        const values = [];
        for (const expression of expressions) {
            const value = this.expression(expression, env, self);
            if (!value.ok)
                return value;
            values.push(value.value);
        }
        return ok(Object.freeze(values));
    }
    arguments(expressions, env, self, invoke) {
        const values = this.sequence(expressions, env, self);
        return values.ok ? invoke(values.value) : values;
    }
    construct(declarationId, variantId, fields, env, self) {
        const declaration = this.declarations.get(declarationId)?.data;
        const variant = variantId === undefined ? undefined : declaration.variants.find((item) => item.header.node.id === variantId);
        const result = { __polyDecl: declarationId };
        if (variant !== undefined)
            result.tag = variant.header.name;
        const members = variant?.fields ?? declaration.fields;
        for (const field of fields) {
            const value = this.expression(field.value, env, self);
            if (!value.ok)
                return value;
            result[this.memberName(members, field.field)] = value.value;
        }
        return ok(Object.freeze(result));
    }
    constant(id) {
        const cached = this.constants.get(id);
        if (cached !== undefined)
            return cached;
        const declaration = this.declarations.get(id);
        if (declaration?.kind !== "constant")
            return fail("invalid_constant", "unknown constant " + id);
        const value = this.constantExpression(declaration.data.value);
        this.constants.set(id, value);
        return value;
    }
    constantExpression(expression) {
        const data = expression.data;
        switch (expression.kind) {
            case "literal": return ok(this.value(data.value));
            case "reference": return this.constant(data.declaration);
            case "none": return ok(none());
            case "some": return this.map(this.constantExpression(data.value), some);
            case "ok": return this.map(this.constantExpression(data.value), valueOk);
            case "err": return this.map(this.constantExpression(data.value), valueErr);
            case "list": {
                const values = [];
                for (const item of data.elements) {
                    const value = this.constantExpression(item);
                    if (!value.ok)
                        return value;
                    values.push(value.value);
                }
                return ok(Object.freeze(values));
            }
            case "record":
            case "enum": {
                const result = { __polyDecl: data.declaration };
                const declaration = this.declarations.get(data.declaration)?.data;
                const variant = expression.kind === "enum" ? declaration.variants.find((item) => item.header.node.id === data.variant) : undefined;
                if (variant !== undefined)
                    result.tag = variant.header.name;
                const members = variant?.fields ?? declaration.fields;
                for (const field of data.fields) {
                    const value = this.constantExpression(field.value);
                    if (!value.ok)
                        return value;
                    result[this.memberName(members, field.field)] = value.value;
                }
                return ok(Object.freeze(result));
            }
            case "intrinsic": {
                const values = [];
                for (const item of data.arguments) {
                    const value = this.constantExpression(item);
                    if (!value.ok)
                        return value;
                    values.push(value.value);
                }
                return this.intrinsic(data.operation, values);
            }
            default: return fail("invalid_constant", "unknown constant expression " + String(expression.kind));
        }
    }
    memberName(members, id) {
        return (members.find((item) => item.header.node.id === id)?.header.name ?? "field_" + id);
    }
    fieldName(id) {
        for (const declaration of this.declarations.values()) {
            const data = declaration.data;
            for (const field of (data.fields ?? []))
                if (field.header.node.id === id)
                    return field.header.name;
            for (const variant of (data.variants ?? []))
                for (const field of (variant.fields ?? []))
                    if (field.header.node.id === id)
                        return field.header.name;
        }
        return "field_" + id;
    }
    findImplementation(contract, record) {
        for (const [id, declaration] of this.declarations) {
            if (declaration.kind === "implementation" && declaration.data.contract === contract && declaration.data.record === record)
                return id;
        }
        return -1;
    }
}
