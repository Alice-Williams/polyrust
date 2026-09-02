# mypy: ignore-errors
"""Dependency-free runtime copied into generated Python packages."""

T = TypeVar("T")
E = TypeVar("E")


@dataclass(frozen=True, slots=True)
class PolyError:
    code: str
    message: str


@dataclass(frozen=True, slots=True)
class PolyResult(Generic[T]):
    ok: bool
    value: T | None = None
    error: PolyError | None = None


@dataclass(frozen=True, slots=True)
class PolyOption(Generic[T]):
    tag: str
    value: T | None = None


@dataclass(frozen=True, slots=True)
class PolyValueResult(Generic[T, E]):
    tag: str
    value: T | None = None
    error: E | None = None


def ok(value: T) -> PolyResult[T]:
    return PolyResult(ok=True, value=value)


def fail(code: str, message: str) -> PolyResult[Any]:
    return PolyResult(ok=False, error=PolyError(code, message))


def checked_i32(value: int) -> PolyResult[int]:
    return ok(value) if -(2**31) <= value < 2**31 else fail("integer_overflow", "i32 out of range")


def checked_i64(value: int) -> PolyResult[int]:
    return ok(value) if -(2**63) <= value < 2**63 else fail("integer_overflow", "i64 out of range")


def wrapping(value: int, bits: int) -> int:
    return ((value + 2 ** (bits - 1)) % 2**bits) - 2 ** (bits - 1)


def semantic_equal(left: Any, right: Any) -> bool:
    if isinstance(left, float) and isinstance(right, float):
        return left == right
    if isinstance(left, (tuple, list)) and isinstance(right, (tuple, list)):
        return len(left) == len(right) and all(semantic_equal(a, b) for a, b in zip(left, right))
    if isinstance(left, (dict, MappingProxyType)) and isinstance(right, (dict, MappingProxyType)):
        return left.keys() == right.keys() and all(semantic_equal(left[key], right[key]) for key in left)
    return left == right


def portable_test_equal(left: Any, right: Any) -> bool:
    # POLYRUST-BEGIN f64-portable-equality
    if isinstance(left, float) and isinstance(right, float):
        return (math.isnan(left) and math.isnan(right)) or struct.pack(">d", left) == struct.pack(">d", right)
    # POLYRUST-END f64-portable-equality
    if isinstance(left, (tuple, list)) and isinstance(right, (tuple, list)):
        return len(left) == len(right) and all(portable_test_equal(a, b) for a, b in zip(left, right))
    if isinstance(left, (dict, MappingProxyType)) and isinstance(right, (dict, MappingProxyType)):
        return left.keys() == right.keys() and all(portable_test_equal(left[key], right[key]) for key in left)
    return left == right


def replace_bytes_all(source: bytes, needle: bytes, replacement: bytes) -> bytes:
    output: list[int] = []
    if not needle:
        output.extend(replacement)
        for byte in source:
            output.append(byte)
            output.extend(replacement)
        return bytes(output)
    offset = 0
    while offset < len(source):
        if source[offset : offset + len(needle)] == needle:
            output.extend(replacement)
            offset += len(needle)
        else:
            output.append(source[offset])
            offset += 1
    return bytes(output)


# POLYRUST-BEGIN f64-functions
def float_div(left: float, right: float) -> float:
    if right != 0.0:
        return left / right
    if math.isnan(left) or left == 0.0:
        return math.nan
    sign = math.copysign(1.0, left) * math.copysign(1.0, right)
    return math.copysign(math.inf, sign)


def float_rem_trunc(left: float, right: float) -> float:
    try:
        return math.fmod(left, right)
    except ValueError:
        return math.nan


def float_abs(value: float) -> float:
    bits = int.from_bytes(struct.pack(">d", value), "big")
    return struct.unpack(">d", (bits & 0x7FFF_FFFF_FFFF_FFFF).to_bytes(8, "big"))[0]
# POLYRUST-END f64-functions


def scalar_length(value: str) -> PolyResult[int]:
    if any(0xD800 <= ord(character) <= 0xDFFF for character in value):
        return fail("invalid_unicode", "surrogate is not a Unicode scalar")
    return ok(len(value))

def replace_many_literal(source: str, values: tuple[Any, ...]) -> str:
    output: list[str] = []
    offset = 0
    mappings = tuple((values[index], values[index + 1]) for index in range(1, len(values), 2))
    while True:
        remaining = source[offset:]
        mapping = next((item for item in mappings if remaining.startswith(item[0])), None)
        if mapping is not None:
            needle, replacement = mapping
            output.append(replacement)
            if needle:
                offset += len(needle)
                continue
            if not remaining:
                break
            output.append(remaining[0])
            offset += 1
            continue
        if not remaining:
            break
        output.append(remaining[0])
        offset += 1
    return "".join(output)


def truncate_utf8_bytes(source: str, budget: float) -> str:
    consumed = 0
    for offset, character in enumerate(source):
        consumed += len(character.encode("utf-8"))
        if consumed == budget:
            return source[: offset + 1]
        if consumed > budget:
            return source[:offset]
    return source


class Runtime:
    def __init__(self, document: dict[str, Any]) -> None:
        self.document = document
        self.declarations = {item["data"]["header"]["node"]["id"]: item for item in document["module"]["declarations"]}
        self.constants: dict[int, PolyResult[Any]] = {}

    def invoke(self, function_id: int, arguments: tuple[Any, ...]) -> PolyResult[Any]:
        declaration = self.declarations.get(function_id)
        if declaration is None or declaration["kind"] != "function":
            return fail("invalid_call", f"unknown function {function_id}")
        return self._invoke_body(declaration["data"], arguments, None)

    def invoke_method(self, implementation_id: int, method_id: int, receiver: Any, arguments: tuple[Any, ...]) -> PolyResult[Any]:
        implementation = self.declarations.get(implementation_id)
        if implementation is None or implementation["kind"] != "implementation":
            return fail("invalid_call", f"unknown implementation {implementation_id}")
        method = next((item for item in implementation["data"]["methods"] if item["header"]["node"]["id"] == method_id or item["contract_method"] == method_id), None)
        return fail("invalid_call", f"unknown method {method_id}") if method is None else self._invoke_body(method, arguments, receiver)

    def decode(self, typed: dict[str, Any]) -> Any:
        return self._value(typed["value"])

    def read_constant(self, identifier: int) -> PolyResult[Any]:
        return self._constant(identifier)

    def _invoke_body(self, callable_: dict[str, Any], arguments: tuple[Any, ...], self_value: Any) -> PolyResult[Any]:
        environment = {parameter["header"]["name"]: arguments[index] for index, parameter in enumerate(callable_["parameters"])}
        return self._block(callable_["body"], environment, self_value)[1]

    def _value(self, value: dict[str, Any]) -> Any:
        kind, data = value["kind"], value.get("data")
        if kind == "unit": return None
        if kind in {"bool", "i32", "i64", "string", "char"}: return data
        # POLYRUST-BEGIN f64-decode
        if kind == "f64":
            return struct.unpack(">d", int(data).to_bytes(8, "big"))[0]
        # POLYRUST-END f64-decode
        if kind in {"bytes", "list"}: return tuple(self._value(item) for item in data) if kind == "list" else bytes(data)
        if kind == "none": return PolyOption("none")
        if kind == "some": return PolyOption("some", self._value(data))
        if kind == "ok": return PolyValueResult("ok", value=self._value(data))
        if kind == "err": return PolyValueResult("err", error=self._value(data))
        if kind in {"record", "enum"}: return self._aggregate(data, data.get("variant"))
        return None

    def _aggregate(self, data: dict[str, Any], variant_id: int | None) -> MappingProxyType[str, Any]:
        declaration = self.declarations[data["declaration"]]["data"]
        variant = next((item for item in declaration.get("variants", []) if item["header"]["node"]["id"] == variant_id), None)
        result: dict[str, Any] = {"__poly_decl__": data["declaration"]}
        if variant is not None: result["tag"] = variant["header"]["name"]
        members = variant["fields"] if variant is not None else declaration["fields"]
        for entry in data["fields"]: result[self._member_name(members, entry["field"])] = self._value(entry["value"])
        return MappingProxyType(result)

    def _expression(self, expression: dict[str, Any], environment: dict[str, Any], self_value: Any) -> PolyResult[Any]:
        kind, data = expression["kind"], expression.get("data", expression)
        if kind == "literal": return ok(self._value(data["value"]))
        if kind == "local": return ok(environment[data["name"]])
        if kind == "self_value": return ok(self_value)
        if kind == "constant": return self._constant(data["declaration"])
        if kind == "construct_none": return ok(PolyOption("none"))
        if kind in {"construct_some", "construct_ok", "construct_err"}:
            item = self._expression(data["value"], environment, self_value)
            if not item.ok: return item
            if kind == "construct_some": return ok(PolyOption("some", item.value))
            return ok(PolyValueResult("ok", value=item.value) if kind == "construct_ok" else PolyValueResult("err", error=item.value))
        if kind == "construct_list": return self._sequence(data["elements"], environment, self_value)
        if kind in {"construct_record", "construct_enum"}: return self._construct(data, data.get("variant"), environment, self_value)
        if kind == "field":
            base = self._expression(data["base"], environment, self_value)
            return base if not base.ok else ok(self._field(base.value, self._field_name(data["field"])))
        if kind in {"call", "intrinsic", "method_call"}:
            arguments = self._sequence(data["arguments"], environment, self_value)
            if not arguments.ok: return arguments
            if kind == "call": return self.invoke(data["function"], arguments.value)
            if kind == "intrinsic": return self._intrinsic(data["operation"], arguments.value)
            receiver = self._expression(data["receiver"], environment, self_value)
            if not receiver.ok: return receiver
            dispatch = data["dispatch"]
            implementation = dispatch["data"].get("implementation")
            if dispatch["kind"] == "contract": implementation = self._find_implementation(dispatch["data"]["contract"], self._field(receiver.value, "__poly_decl__"))
            return self.invoke_method(implementation, dispatch["data"]["method"], receiver.value, arguments.value)
        if kind == "if":
            condition = self._expression(data["condition"], environment, self_value)
            return condition if not condition.ok else self._block(data["then_block"] if condition.value else data["else_block"], dict(environment), self_value)[1]
        if kind == "match": return self._match(data, environment, self_value)
        if kind == "block": return self._block(data, dict(environment), self_value)[1]
        return fail("invalid_expression", f"unknown expression {kind}")

    def _block(self, block: dict[str, Any], environment: dict[str, Any], self_value: Any) -> tuple[bool, PolyResult[Any]]:
        for statement in block["statements"]:
            kind, data = statement["kind"], statement["data"]
            if kind in {"let", "expression"}:
                item = self._expression(data["value"], environment, self_value)
                if not item.ok: return True, item
                if kind == "let": environment[data["name"]] = item.value
            elif kind == "return": return True, ok(None) if data["value"] is None else self._expression(data["value"], environment, self_value)
            elif kind == "for_each":
                items = self._expression(data["iterable"], environment, self_value)
                if not items.ok: return True, items
                for item in items.value:
                    inner = dict(environment); inner[data["binding"]] = item
                    returned, result = self._block(data["body"], inner, self_value)
                    if returned or not result.ok: return returned, result
        return False, ok(None) if block["result"] is None else self._expression(block["result"], environment, self_value)

    def _match(self, data: dict[str, Any], environment: dict[str, Any], self_value: Any) -> PolyResult[Any]:
        value = self._expression(data["value"], environment, self_value)
        if not value.ok: return value
        for arm in data["arms"]:
            bindings = self._pattern(arm["pattern"], value.value)
            if bindings is not None:
                inner = dict(environment); inner.update(bindings)
                return self._block(arm["body"], inner, self_value)[1]
        return fail("non_exhaustive_match", "checked match had no matching arm")

    def _pattern(self, pattern: dict[str, Any], value: Any) -> dict[str, Any] | None:
        kind, data = pattern["kind"], pattern["data"]
        if kind == "wildcard": return {}
        if kind == "bool": return {} if value == data["value"] else None
        if kind in {"none", "some", "ok", "err"}:
            expected = {"none": "none", "some": "some", "ok": "ok", "err": "err"}[kind]
            if value.tag != expected: return None
            if kind == "none": return {}
            return {data["binding"]: value.error if kind == "err" else value.value}
        if kind == "enum_variant":
            declaration = self.declarations[data["declaration"]]["data"]
            variant = next(item for item in declaration["variants"] if item["header"]["node"]["id"] == data["variant"])
            if self._field(value, "tag") != variant["header"]["name"]: return None
            return {binding["binding"]: self._field(value, self._member_name(variant["fields"], binding["field"])) for binding in data["bindings"]}
        return None

    def _intrinsic(self, name: str, values: tuple[Any, ...]) -> PolyResult[Any]:
        a = values[0] if values else None
        b = values[1] if len(values) > 1 else None
        c = values[2] if len(values) > 2 else None
        if name == "bool_not": return ok(not a)
        if name == "bool_and": return ok(a and b)
        if name == "bool_or": return ok(a or b)
        if name == "equal": return ok(semantic_equal(a, b))
        if name == "not_equal": return ok(not semantic_equal(a, b))
        if name in {"less", "less_equal", "greater", "greater_equal"}: return ok({"less": a < b, "less_equal": a <= b, "greater": a > b, "greater_equal": a >= b}[name])
        if name in {"int_neg_checked", "int_add_checked", "int_sub_checked", "int_mul_checked", "int_div_checked", "int_rem_checked"}:
            if name in {"int_div_checked", "int_rem_checked"} and b == 0: return fail("division_by_zero", "integer division by zero")
            result = {"int_neg_checked": lambda: -a, "int_add_checked": lambda: a + b, "int_sub_checked": lambda: a - b, "int_mul_checked": lambda: a * b, "int_div_checked": lambda: int(a / b), "int_rem_checked": lambda: a - int(a / b) * b}[name]()
            return checked_i32(result) if -(2**31) <= a < 2**31 else checked_i64(result)
        if name in {"int_neg_wrapping", "int_add_wrapping", "int_sub_wrapping", "int_mul_wrapping"}:
            result = {"int_neg_wrapping": lambda: -a, "int_add_wrapping": lambda: a + b, "int_sub_wrapping": lambda: a - b, "int_mul_wrapping": lambda: a * b}[name]()
            return ok(wrapping(result, 32 if -(2**31) <= a < 2**31 else 64))
        if name == "int_bit_not": return ok(~a)
        if name == "int_bit_and": return ok(a & b)
        if name == "int_bit_or": return ok(a | b)
        if name == "int_bit_xor": return ok(a ^ b)
        if name == "int_shift_left_checked": return checked_i32(a << b) if -(2**31) <= a < 2**31 else checked_i64(a << b)
        if name == "int_shift_right_checked": return ok(a >> b)
        # POLYRUST-BEGIN f64-intrinsics
        if name == "float_neg": return ok(-a)
        if name == "float_trunc": return ok(math.modf(a)[1])
        if name == "float_is_nan": return ok(math.isnan(a))
        if name == "float_is_negative_zero": return ok(a == 0.0 and math.copysign(1.0, a) < 0.0)
        if name == "float_abs": return ok(float_abs(a))
        if name == "float_add": return ok(a + b)
        if name == "float_sub": return ok(a - b)
        if name == "float_mul": return ok(a * b)
        if name == "float_div": return ok(float_div(a, b))
        if name == "float_rem_trunc": return ok(float_rem_trunc(a, b))
        # POLYRUST-END f64-intrinsics
        if name == "string_concat": return ok(a + b)
        if name == "string_scalar_length": return scalar_length(a)
        # POLYRUST-BEGIN case.string-utf16-length
        if name == "string_utf16_length": return ok(len(a.encode("utf-16-le")) // 2)
        # POLYRUST-END case.string-utf16-length
        # POLYRUST-BEGIN case.string-index-of-literal
        if name == "string_index_of_literal":
            index = a.find(b)
            return ok(PolyOption("none") if index < 0 else PolyOption("some", index))
        # POLYRUST-END case.string-index-of-literal
        # POLYRUST-BEGIN case.string-slice-scalars
        if name == "string_slice_scalars":
            length = len(a)
            start = max(0, min(b, length))
            end = max(0, min(c, length))
            return ok("" if start >= end else a[start:end])
        # POLYRUST-END case.string-slice-scalars
        if name == "string_is_empty": return ok(not a)
        if name == "string_contains": return ok(b in a)
        if name == "string_starts_with": return ok(a.startswith(b))
        if name == "string_strip_prefix": return ok(a.removeprefix(b) if b else a)
        if name == "string_ends_with": return ok(a.endswith(b))
        if name == "string_replace_all": return ok(a.replace(b, c))
        if name == "string_replace_many": return ok(replace_many_literal(a, values))
        if name == "string_truncate_utf8_bytes": return ok(truncate_utf8_bytes(a, b))
        if name == "string_trim_start": return ok(a.lstrip(b))
        if name == "string_trim_end": return ok(a.rstrip(b))
        if name in {"bytes_concat", "list_concat"}: return ok(a + b)
        if name == "bytes_replace_all": return ok(replace_bytes_all(a, b, c))
        if name in {"bytes_length", "list_length"}: return checked_i32(len(a))
        if name in {"bytes_is_empty", "list_is_empty"}: return ok(not a)
        if name == "list_get_checked": return ok(a[b]) if 0 <= b < len(a) else fail("index_out_of_bounds", "list index out of bounds")
        if name == "list_append": return ok(a + (b,))
        if name == "list_contains": return ok(any(semantic_equal(item, b) for item in a))
        # POLYRUST-BEGIN case.list-index-of
        if name == "list_index_of":
            for index, item in enumerate(a):
                if semantic_equal(item, b): return ok(PolyOption("some", index))
            return ok(PolyOption("none"))
        # POLYRUST-END case.list-index-of
        if name == "option_is_some": return ok(a.tag == "some")
        if name == "option_is_none": return ok(a.tag == "none")
        if name == "option_unwrap_or": return ok(a.value if a.tag == "some" else b)
        if name == "result_is_ok": return ok(a.tag == "ok")
        if name == "result_is_err": return ok(a.tag == "err")
        if name == "widen_i32_to_i64": return ok(a)
        if name == "narrow_i64_to_i32_checked": return checked_i32(a)
        if name == "string_to_utf8": return ok(a.encode("utf-8"))
        if name == "string_from_utf8_checked":
            try: return ok(bytes(a).decode("utf-8"))
            except UnicodeDecodeError: return fail("invalid_utf8", "invalid UTF-8")
        return fail("invalid_intrinsic", f"unknown intrinsic {name}")

    def _sequence(self, expressions: list[dict[str, Any]], environment: dict[str, Any], self_value: Any) -> PolyResult[tuple[Any, ...]]:
        values: list[Any] = []
        for expression in expressions:
            item = self._expression(expression, environment, self_value)
            if not item.ok: return item
            values.append(item.value)
        return ok(tuple(values))

    def _construct(self, data: dict[str, Any], variant_id: int | None, environment: dict[str, Any], self_value: Any) -> PolyResult[Any]:
        declaration = self.declarations[data["declaration"]]["data"]
        variant = next((item for item in declaration.get("variants", []) if item["header"]["node"]["id"] == variant_id), None)
        members = variant["fields"] if variant is not None else declaration["fields"]
        result: dict[str, Any] = {"__poly_decl__": data["declaration"]}
        if variant is not None: result["tag"] = variant["header"]["name"]
        for field in data["fields"]:
            item = self._expression(field["value"], environment, self_value)
            if not item.ok: return item
            result[self._member_name(members, field["field"])] = item.value
        return ok(MappingProxyType(result))

    def _constant(self, identifier: int) -> PolyResult[Any]:
        if identifier in self.constants: return self.constants[identifier]
        declaration = self.declarations.get(identifier)
        if declaration is None or declaration["kind"] != "constant": return fail("invalid_constant", f"unknown constant {identifier}")
        value = self._constant_expression(declaration["data"]["value"]); self.constants[identifier] = value
        return value

    def _constant_expression(self, expression: dict[str, Any]) -> PolyResult[Any]:
        kind, data = expression["kind"], expression["data"]
        if kind == "literal": return ok(self._value(data["value"]))
        if kind == "reference": return self._constant(data["declaration"])
        if kind == "none": return ok(PolyOption("none"))
        if kind in {"some", "ok", "err"}:
            item = self._constant_expression(data["value"])
            if not item.ok: return item
            if kind == "some": return ok(PolyOption("some", item.value))
            return ok(PolyValueResult("ok", value=item.value) if kind == "ok" else PolyValueResult("err", error=item.value))
        if kind == "list":
            values: list[Any] = []
            for expression_item in data["elements"]:
                item = self._constant_expression(expression_item)
                if not item.ok: return item
                values.append(item.value)
            return ok(tuple(values))
        if kind in {"record", "enum"}:
            declaration = self.declarations[data["declaration"]]["data"]
            variant = next((item for item in declaration.get("variants", []) if item["header"]["node"]["id"] == data.get("variant")), None)
            members = variant["fields"] if variant is not None else declaration["fields"]
            result: dict[str, Any] = {"__poly_decl__": data["declaration"]}
            if variant is not None: result["tag"] = variant["header"]["name"]
            for field in data["fields"]:
                item = self._constant_expression(field["value"])
                if not item.ok: return item
                result[self._member_name(members, field["field"])] = item.value
            return ok(MappingProxyType(result))
        if kind == "intrinsic":
            values: list[Any] = []
            for argument in data["arguments"]:
                item = self._constant_expression(argument)
                if not item.ok: return item
                values.append(item.value)
            return self._intrinsic(data["operation"], tuple(values))
        return fail("invalid_constant", f"unknown constant expression {kind}")

    @staticmethod
    def _field(value: Any, name: str) -> Any:
        return value[name] if isinstance(value, (dict, MappingProxyType)) else getattr(value, name)

    @staticmethod
    def _member_name(members: list[dict[str, Any]], identifier: int) -> str:
        return next(item["header"]["name"] for item in members if item["header"]["node"]["id"] == identifier)

    def _field_name(self, identifier: int) -> str:
        for declaration in self.declarations.values():
            for field in declaration["data"].get("fields", []):
                if field["header"]["node"]["id"] == identifier: return field["header"]["name"]
            for variant in declaration["data"].get("variants", []):
                for field in variant["fields"]:
                    if field["header"]["node"]["id"] == identifier: return field["header"]["name"]
        return f"field_{identifier}"

    def _find_implementation(self, contract: int, record: int) -> int:
        return next(identifier for identifier, declaration in self.declarations.items() if declaration["kind"] == "implementation" and declaration["data"]["contract"] == contract and declaration["data"]["record"] == record)
