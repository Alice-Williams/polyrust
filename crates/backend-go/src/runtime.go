type PolyError struct {
	Code    string
	Message string
}
type PolyResult[T any] struct {
	Ok    bool
	Value T
	Error *PolyError
}
type PolyOption[T any] struct {
	Tag   string
	Value T
}
type PolyValueResult[T, E any] struct {
	Tag   string
	Value T
	Error E
}
type PolyList[T any] struct{ items []T }

func NewPolyList[T any](items ...T) PolyList[T] {
	return PolyList[T]{items: append([]T(nil), items...)}
}
func (value PolyList[T]) Len() int32                { return int32(len(value.items)) }
func (value PolyList[T]) Values() []T               { return append([]T(nil), value.items...) }
func (value PolyList[T]) append(item T) PolyList[T] { return NewPolyList(append(value.items, item)...) }
func (value PolyList[T]) polyRuntimeValue() any {
	items := make([]any, len(value.items))
	for index, item := range value.items {
		items[index] = toRuntimeValue(item)
	}
	return items
}

func (value PolyOption[T]) polyRuntimeValue() any {
	result := map[string]any{"tag": value.Tag}
	if value.Tag == "some" {
		result["value"] = toRuntimeValue(value.Value)
	}
	return result
}

func (value PolyValueResult[T, E]) polyRuntimeValue() any {
	result := map[string]any{"tag": value.Tag}
	if value.Tag == "ok" {
		result["value"] = toRuntimeValue(value.Value)
	} else if value.Tag == "err" {
		result["error"] = toRuntimeValue(value.Error)
	}
	return result
}

type PolyBytes struct{ items []byte }

func NewPolyBytes(items ...byte) PolyBytes { return PolyBytes{items: append([]byte(nil), items...)} }
func (value PolyBytes) Values() []byte     { return append([]byte(nil), value.items...) }
// POLYRUST-BEGIN bytes-replace-function
func replaceBytesAll(source, needle, replacement PolyBytes) PolyBytes {
	output := make([]byte, 0, len(source.items))
	if len(needle.items) == 0 {
		output = append(output, replacement.items...)
		for _, value := range source.items {
			output = append(output, value)
			output = append(output, replacement.items...)
		}
		return NewPolyBytes(output...)
	}
	for offset := 0; offset < len(source.items); {
		end := offset + len(needle.items)
		if end <= len(source.items) && bytes.Equal(source.items[offset:end], needle.items) {
			output = append(output, replacement.items...)
			offset = end
		} else {
			output = append(output, source.items[offset])
			offset++
		}
	}
	return NewPolyBytes(output...)
}
// POLYRUST-END bytes-replace-function
func polyOk[T any](value T) PolyResult[T]  { return PolyResult[T]{Ok: true, Value: value} }
func polyFail[T any](code, message string) PolyResult[T] {
	return PolyResult[T]{Error: &PolyError{Code: code, Message: message}}
}
func castResult[T any](value PolyResult[any]) PolyResult[T] {
	if !value.Ok {
		return polyFail[T](value.Error.Code, value.Error.Message)
	}
	typed, ok := value.Value.(T)
	if !ok {
		return polyFail[T]("internal_type", "checked result type mismatch")
	}
	return polyOk(typed)
}

type polyRecord interface{ polyValue() map[string]any }
type polyRuntimeValue interface{ polyRuntimeValue() any }

func toRuntimeValue(value any) any {
	if portable, ok := value.(polyRuntimeValue); ok {
		return portable.polyRuntimeValue()
	}
	if record, ok := value.(polyRecord); ok {
		raw := record.polyValue()
		result := make(map[string]any, len(raw))
		for name, field := range raw {
			result[name] = toRuntimeValue(field)
		}
		return result
	}
	return value
}

type runtime struct {
	declarations map[int64]map[string]any
	constants    map[int64]PolyResult[any]
}

func newRuntime(source string) *runtime {
	decoder := json.NewDecoder(strings.NewReader(source))
	decoder.UseNumber()
	var document map[string]any
	if err := decoder.Decode(&document); err != nil {
		panic(err)
	}
	result := &runtime{declarations: map[int64]map[string]any{}, constants: map[int64]PolyResult[any]{}}
	module := document["module"].(map[string]any)
	for _, raw := range module["declarations"].([]any) {
		declaration := raw.(map[string]any)
		result.declarations[nodeID(declaration)] = declaration
	}
	return result
}
func nodeID(value map[string]any) int64 {
	data := value["data"].(map[string]any)
	header := data["header"].(map[string]any)
	node := header["node"].(map[string]any)
	id, _ := node["id"].(json.Number).Int64()
	return id
}
func number(value any) int64 { result, _ := value.(json.Number).Int64(); return result }
// POLYRUST-BEGIN math-unsigned-number
func unsignedNumber(value any) uint64 {
	result, err := strconv.ParseUint(value.(json.Number).String(), 10, 64)
	if err != nil {
		panic(err)
	}
	return result
}
// POLYRUST-END math-unsigned-number

func (r *runtime) invoke(id int64, arguments []any) PolyResult[any] {
	declaration := r.declarations[id]
	if declaration == nil || declaration["kind"] != "function" {
		return polyFail[any]("invalid_call", "unknown function")
	}
	return r.invokeBody(declaration["data"].(map[string]any), arguments, nil)
}
func (r *runtime) invokeMethod(implementationID, methodID int64, receiver any, arguments []any) PolyResult[any] {
	implementation := r.declarations[implementationID]
	if implementation == nil {
		return polyFail[any]("invalid_call", "unknown implementation")
	}
	data := implementation["data"].(map[string]any)
	for _, raw := range data["methods"].([]any) {
		method := raw.(map[string]any)
		header := method["header"].(map[string]any)
		if number(header["node"].(map[string]any)["id"]) == methodID || number(method["contract_method"]) == methodID {
			return r.invokeBody(method, arguments, receiver)
		}
	}
	return polyFail[any]("invalid_call", "unknown method")
}
func (r *runtime) invokeBody(callable map[string]any, arguments []any, self any) PolyResult[any] {
	environment := map[string]any{}
	for index, raw := range callable["parameters"].([]any) {
		parameter := raw.(map[string]any)
		environment[parameter["header"].(map[string]any)["name"].(string)] = toRuntimeValue(arguments[index])
	}
	_, result := r.block(callable["body"].(map[string]any), environment, toRuntimeValue(self))
	return result
}

func (r *runtime) decode(typed map[string]any) any { return r.value(typed["value"].(map[string]any)) }
func (r *runtime) value(value map[string]any) any {
	kind := value["kind"].(string)
	data := value["data"]
	switch kind {
	case "unit":
		return struct{}{}
	case "bool", "string", "char":
		return data
	case "i32":
		return int32(number(data))
	case "i64":
		return number(data)
	// POLYRUST-BEGIN math-value-case
	case "f64":
		return math.Float64frombits(unsignedNumber(data))
	// POLYRUST-END math-value-case
	case "bytes":
		raw := data.([]any)
		items := make([]byte, len(raw))
		for i, v := range raw {
			items[i] = byte(number(v))
		}
		return NewPolyBytes(items...)
	case "list":
		raw := data.([]any)
		items := make([]any, len(raw))
		for i, v := range raw {
			items[i] = r.value(v.(map[string]any))
		}
		return items
	case "none":
		return map[string]any{"tag": "none"}
	case "some":
		return map[string]any{"tag": "some", "value": r.value(data.(map[string]any))}
	case "ok":
		return map[string]any{"tag": "ok", "value": r.value(data.(map[string]any))}
	case "err":
		return map[string]any{"tag": "err", "error": r.value(data.(map[string]any))}
	case "record", "enum":
		aggregate := data.(map[string]any)
		return r.aggregate(aggregate, aggregate["variant"])
	default:
		return nil
	}
}
func (r *runtime) aggregate(data map[string]any, variantID any) map[string]any {
	declarationID := number(data["declaration"])
	declaration := r.declarations[declarationID]["data"].(map[string]any)
	result := map[string]any{"__polyDecl": declarationID}
	var members []any
	if raw, ok := declaration["fields"].([]any); ok {
		members = raw
	}
	if variantID != nil {
		for _, raw := range declaration["variants"].([]any) {
			variant := raw.(map[string]any)
			if number(variant["header"].(map[string]any)["node"].(map[string]any)["id"]) == number(variantID) {
				result["tag"] = variant["header"].(map[string]any)["name"]
				members = variant["fields"].([]any)
			}
		}
	}
	for _, raw := range data["fields"].([]any) {
		field := raw.(map[string]any)
		result[r.memberName(members, number(field["field"]))] = r.value(field["value"].(map[string]any))
	}
	return result
}
func (r *runtime) expression(expression map[string]any, environment map[string]any, self any) PolyResult[any] {
	kind := expression["kind"].(string)
	data := expression["data"].(map[string]any)
	switch kind {
	case "literal":
		return polyOk(r.value(data["value"].(map[string]any)))
	case "local":
		return polyOk(environment[data["name"].(string)])
	case "self_value":
		return polyOk(self)
	case "constant":
		return r.constant(number(data["declaration"]))
	case "construct_none":
		return polyOk(any(map[string]any{"tag": "none"}))
	case "construct_some", "construct_ok", "construct_err":
		item := r.expression(data["value"].(map[string]any), environment, self)
		if !item.Ok {
			return item
		}
		tag := map[string]string{"construct_some": "some", "construct_ok": "ok", "construct_err": "err"}[kind]
		key := "value"
		if tag == "err" {
			key = "error"
		}
		return polyOk(any(map[string]any{"tag": tag, key: item.Value}))
	case "construct_list":
		return r.sequence(data["elements"].([]any), environment, self)
	case "construct_record", "construct_enum":
		return r.construct(data, data["variant"], environment, self)
	case "field":
		base := r.expression(data["base"].(map[string]any), environment, self)
		if !base.Ok {
			return base
		}
		return polyOk(r.field(base.Value, r.fieldName(number(data["field"]))))
	case "call":
		arguments := r.sequence(data["arguments"].([]any), environment, self)
		if !arguments.Ok {
			return arguments
		}
		return r.invoke(number(data["function"]), arguments.Value.([]any))
	case "intrinsic":
		arguments := r.sequence(data["arguments"].([]any), environment, self)
		if !arguments.Ok {
			return arguments
		}
		return r.intrinsic(data["operation"].(string), arguments.Value.([]any))
	case "method_call":
		arguments := r.sequence(data["arguments"].([]any), environment, self)
		if !arguments.Ok {
			return arguments
		}
		receiver := r.expression(data["receiver"].(map[string]any), environment, self)
		if !receiver.Ok {
			return receiver
		}
		dispatch := data["dispatch"].(map[string]any)
		target := dispatch["data"].(map[string]any)
		var implementation int64
		if dispatch["kind"] == "concrete" {
			implementation = number(target["implementation"])
		}
		if dispatch["kind"] == "contract" {
			implementation = r.findImplementation(number(target["contract"]), r.field(receiver.Value, "__polyDecl").(int64))
		}
		return r.invokeMethod(implementation, number(target["method"]), receiver.Value, arguments.Value.([]any))
	case "if":
		condition := r.expression(data["condition"].(map[string]any), environment, self)
		if !condition.Ok {
			return condition
		}
		branch := "else_block"
		if condition.Value.(bool) {
			branch = "then_block"
		}
		_, result := r.block(data[branch].(map[string]any), cloneMap(environment), self)
		return result
	case "block":
		_, result := r.block(data, cloneMap(environment), self)
		return result
	}
	return polyFail[any]("unsupported", "expression not implemented")
}
func cloneMap(input map[string]any) map[string]any {
	result := map[string]any{}
	for key, value := range input {
		result[key] = value
	}
	return result
}
func (r *runtime) block(block map[string]any, environment map[string]any, self any) (bool, PolyResult[any]) {
	for _, raw := range block["statements"].([]any) {
		statement := raw.(map[string]any)
		kind := statement["kind"].(string)
		data := statement["data"].(map[string]any)
		if kind == "let" || kind == "expression" {
			value := r.expression(data["value"].(map[string]any), environment, self)
			if !value.Ok {
				return true, value
			}
			if kind == "let" {
				environment[data["name"].(string)] = value.Value
			}
		} else if kind == "return" {
			if data["value"] == nil {
				return true, polyOk(any(struct{}{}))
			}
			return true, r.expression(data["value"].(map[string]any), environment, self)
		} else if kind == "for_each" {
			values := r.expression(data["iterable"].(map[string]any), environment, self)
			if !values.Ok {
				return true, values
			}
			for _, item := range values.Value.([]any) {
				inner := cloneMap(environment)
				inner[data["binding"].(string)] = item
				returned, result := r.block(data["body"].(map[string]any), inner, self)
				if returned || !result.Ok {
					return returned, result
				}
			}
		}
	}
	if block["result"] == nil {
		return false, polyOk(any(struct{}{}))
	}
	return false, r.expression(block["result"].(map[string]any), environment, self)
}

func (r *runtime) intrinsic(name string, values []any) PolyResult[any] {
	var a, b, c any
	if len(values) > 0 {
		a = values[0]
	}
	if len(values) > 1 {
		b = values[1]
	}
	if len(values) > 2 {
		c = values[2]
	}
	switch name {
	case "bool_not":
		return polyOk(any(!a.(bool)))
	case "bool_and":
		return polyOk(any(a.(bool) && b.(bool)))
	case "bool_or":
		return polyOk(any(a.(bool) || b.(bool)))
	case "equal":
		return polyOk(any(equal(a, b)))
	case "not_equal":
		return polyOk(any(!equal(a, b)))
	case "string_concat":
		return polyOk(any(a.(string) + b.(string)))
	// POLYRUST-BEGIN utf8-scalar-case
	case "string_scalar_length":
		if !utf8.ValidString(a.(string)) {
			return polyFail[any]("invalid_unicode", "invalid scalar string")
		}
		return polyOk(any(int32(utf8.RuneCountInString(a.(string)))))
	// POLYRUST-END utf8-scalar-case
	// POLYRUST-BEGIN utf16-length-case
	case "string_utf16_length":
		if !utf8.ValidString(a.(string)) {
			return polyFail[any]("invalid_unicode", "invalid scalar string")
		}
		length := int64(0)
		for _, scalar := range a.(string) {
			length++
			if scalar > 0xffff {
				length++
			}
		}
		return polyOk(any(length))
	// POLYRUST-END utf16-length-case
	case "string_is_empty":
		return polyOk(any(a.(string) == ""))
	case "string_contains":
		return polyOk(any(strings.Contains(a.(string), b.(string))))
	case "string_starts_with":
		return polyOk(any(strings.HasPrefix(a.(string), b.(string))))
	case "string_strip_prefix":
		if b.(string) == "" {
			return polyOk(any(a.(string)))
		}
		return polyOk(any(strings.TrimPrefix(a.(string), b.(string))))
	case "string_ends_with":
		return polyOk(any(strings.HasSuffix(a.(string), b.(string))))
	case "string_replace_all":
		return polyOk(any(strings.ReplaceAll(a.(string), b.(string), c.(string))))
	// POLYRUST-BEGIN utf8-replace-many-case
	case "string_replace_many":
		return polyOk(any(replaceManyLiteral(a.(string), values)))
	// POLYRUST-END utf8-replace-many-case
	// POLYRUST-BEGIN utf8-truncate-case
	case "string_truncate_utf8_bytes":
		return polyOk(any(truncateUtf8Bytes(a.(string), b.(float64))))
	// POLYRUST-END utf8-truncate-case
	case "string_trim_start":
		return polyOk(any(strings.TrimLeft(a.(string), b.(string))))
	case "string_trim_end":
		return polyOk(any(strings.TrimRight(a.(string), b.(string))))
	case "bytes_concat":
		left, right := a.(PolyBytes), b.(PolyBytes)
		return polyOk(any(NewPolyBytes(append(left.Values(), right.items...)...)))
	// POLYRUST-BEGIN bytes-replace-case
	case "bytes_replace_all":
		return polyOk(any(replaceBytesAll(a.(PolyBytes), b.(PolyBytes), c.(PolyBytes))))
	// POLYRUST-END bytes-replace-case
	case "bytes_length":
		return polyOk(any(int64(len(a.(PolyBytes).items))))
	case "bytes_is_empty":
		return polyOk(any(len(a.(PolyBytes).items) == 0))
	case "widen_i32_to_i64":
		return polyOk(any(int64(a.(int32))))
	// POLYRUST-BEGIN math-narrow-case
	case "narrow_i64_to_i32_checked":
		value := a.(int64)
		if value < math.MinInt32 || value > math.MaxInt32 {
			return polyFail[any]("integer_overflow", "i64 does not fit i32")
		}
		return polyOk(any(int32(value)))
	// POLYRUST-END math-narrow-case
	case "string_to_utf8":
		return polyOk(any(NewPolyBytes([]byte(a.(string))...)))
	// POLYRUST-BEGIN utf8-from-bytes-case
	case "string_from_utf8_checked":
		raw := a.(PolyBytes).items
		if !utf8.Valid(raw) {
			return polyFail[any]("invalid_utf8", "invalid UTF-8")
		}
		return polyOk(any(string(raw)))
	// POLYRUST-END utf8-from-bytes-case
	case "option_is_some":
		return polyOk(any(a.(map[string]any)["tag"] == "some"))
	case "option_is_none":
		return polyOk(any(a.(map[string]any)["tag"] == "none"))
	case "option_unwrap_or":
		option := a.(map[string]any)
		if option["tag"] == "some" {
			return polyOk(option["value"])
		}
		return polyOk(b)
	case "result_is_ok":
		return polyOk(any(a.(map[string]any)["tag"] == "ok"))
	case "result_is_err":
		return polyOk(any(a.(map[string]any)["tag"] == "err"))
	}
	return r.numericOrCollection(name, a, b)
}
func (r *runtime) numericOrCollection(name string, a, b any) PolyResult[any] {
	if left, ok := a.(int32); ok {
		right, _ := b.(int32)
		switch name {
		// POLYRUST-BEGIN math-i32-checked-cases
		case "int_neg_checked":
			return checked32(-int64(left))
		case "int_add_checked":
			return checked32(int64(left) + int64(right))
		case "int_sub_checked":
			return checked32(int64(left) - int64(right))
		case "int_mul_checked":
			return checked32(int64(left) * int64(right))
		// POLYRUST-END math-i32-checked-cases
		case "int_add_wrapping":
			return polyOk(any(left + right))
		case "int_sub_wrapping":
			return polyOk(any(left - right))
		case "int_mul_wrapping":
			return polyOk(any(left * right))
		case "int_neg_wrapping":
			return polyOk(any(-left))
		case "less":
			return polyOk(any(left < right))
		case "less_equal":
			return polyOk(any(left <= right))
		case "greater":
			return polyOk(any(left > right))
		case "greater_equal":
			return polyOk(any(left >= right))
		}
	}
	if left, ok := a.(int64); ok {
		right, _ := b.(int64)
		switch name {
		// POLYRUST-BEGIN math-i64-checked-case
		case "int_add_checked":
			if (right > 0 && left > math.MaxInt64-right) || (right < 0 && left < math.MinInt64-right) {
				return polyFail[any]("integer_overflow", "i64 overflow")
			}
			return polyOk(any(left + right))
		// POLYRUST-END math-i64-checked-case
		case "int_add_wrapping":
			return polyOk(any(left + right))
		case "less":
			return polyOk(any(left < right))
		case "less_equal":
			return polyOk(any(left <= right))
		case "greater":
			return polyOk(any(left > right))
		case "greater_equal":
			return polyOk(any(left >= right))
		}
	}
	// POLYRUST-BEGIN math-float-dispatch
	if left, ok := a.(float64); ok {
		right, _ := b.(float64)
		switch name {
		case "float_neg":
			return polyOk(any(-left))
		case "float_trunc":
			return polyOk(any(math.Trunc(left)))
		case "float_is_nan":
			return polyOk(any(math.IsNaN(left)))
		case "float_add":
			return polyOk(any(left + right))
		case "float_sub":
			return polyOk(any(left - right))
		case "float_mul":
			return polyOk(any(left * right))
		case "float_div":
			return polyOk(any(left / right))
		case "float_rem_trunc":
			return polyOk(any(math.Mod(left, right)))
		case "less":
			return polyOk(any(left < right))
		case "less_equal":
			return polyOk(any(left <= right))
		case "greater":
			return polyOk(any(left > right))
		case "greater_equal":
			return polyOk(any(left >= right))
		}
	}
	// POLYRUST-END math-float-dispatch
	if list, ok := a.([]any); ok {
		switch name {
		case "list_length":
			return polyOk(any(int32(len(list))))
		case "list_is_empty":
			return polyOk(any(len(list) == 0))
		case "list_append":
			return polyOk(any(append(append([]any(nil), list...), b)))
		case "list_concat":
			return polyOk(any(append(append([]any(nil), list...), b.([]any)...)))
		case "list_get_checked":
			index := b.(int32)
			if index < 0 || int(index) >= len(list) {
				return polyFail[any]("index_out_of_bounds", "list index out of bounds")
			}
			return polyOk(list[index])
		case "list_contains":
			for _, item := range list {
				if equal(item, b) {
					return polyOk(any(true))
				}
			}
			return polyOk(any(false))
		// POLYRUST-BEGIN list-index-of-case
		case "list_index_of":
			for index, item := range list {
				if equal(item, b) {
					return polyOk(any(map[string]any{"tag": "some", "value": int64(index)}))
				}
			}
			return polyOk(any(map[string]any{"tag": "none"}))
		// POLYRUST-END list-index-of-case
		}
	}
	return polyFail[any]("unsupported", "intrinsic not implemented: "+name)
}
// POLYRUST-BEGIN utf8-replace-many-function
func replaceManyLiteral(source string, values []any) string {
	var output strings.Builder
	offset := 0
	for {
		remaining := source[offset:]
		matched := false
		for index := 1; index < len(values); index += 2 {
			needle := values[index].(string)
			if !strings.HasPrefix(remaining, needle) {
				continue
			}
			output.WriteString(values[index+1].(string))
			if needle != "" {
				offset += len(needle)
			} else if remaining == "" {
				return output.String()
			} else {
				_, width := utf8.DecodeRuneInString(remaining)
				output.WriteString(remaining[:width])
				offset += width
			}
			matched = true
			break
		}
		if matched {
			continue
		}
		if remaining == "" {
			return output.String()
		}
		_, width := utf8.DecodeRuneInString(remaining)
		output.WriteString(remaining[:width])
		offset += width
	}
}
// POLYRUST-END utf8-replace-many-function
// POLYRUST-BEGIN utf8-truncate-function
func truncateUtf8Bytes(source string, budget float64) string {
	for offset, character := range source {
		end := offset + utf8.RuneLen(character)
		consumed := float64(end)
		if consumed == budget {
			return source[:end]
		}
		if consumed > budget {
			return source[:offset]
		}
	}
	return source
}
// POLYRUST-END utf8-truncate-function
// POLYRUST-BEGIN math-checked32
func checked32(value int64) PolyResult[any] {
	if value < math.MinInt32 || value > math.MaxInt32 {
		return polyFail[any]("integer_overflow", "i32 overflow")
	}
	return polyOk(any(int32(value)))
}
// POLYRUST-END math-checked32
func equalImpl(a, b any, exactFloat bool) bool {
	if left, ok := a.(polyRecord); ok {
		a = left.polyValue()
	}
	if right, ok := b.(polyRecord); ok {
		b = right.polyValue()
	}
	switch left := a.(type) {
	case nil:
		return b == nil
	case int32:
		right, ok := b.(int32)
		return ok && left == right
	case int64:
		right, ok := b.(int64)
		return ok && left == right
	case string:
		right, ok := b.(string)
		return ok && left == right
	case bool:
		right, ok := b.(bool)
		return ok && left == right
	// POLYRUST-BEGIN math-float-equality
	case float64:
		right, ok := b.(float64)
		if !ok {
			return false
		}
		if exactFloat {
			return (math.IsNaN(left) && math.IsNaN(right)) || math.Float64bits(left) == math.Float64bits(right)
		}
		return left == right
	// POLYRUST-END math-float-equality
	case []any:
		right, ok := b.([]any)
		if !ok || len(left) != len(right) {
			return false
		}
		for index := range left {
			if !equalImpl(left[index], right[index], exactFloat) {
				return false
			}
		}
		return true
	case map[string]any:
		right, ok := b.(map[string]any)
		if !ok || len(left) != len(right) {
			return false
		}
		for key, value := range left {
			other, exists := right[key]
			if !exists || !equalImpl(value, other, exactFloat) {
				return false
			}
		}
		return true
	case PolyBytes:
		right, ok := b.(PolyBytes)
		if !ok || len(left.items) != len(right.items) {
			return false
		}
		for index := range left.items {
			if left.items[index] != right.items[index] {
				return false
			}
		}
		return true
	}
	return false
}

func equal(a, b any) bool { return equalImpl(a, b, false) }
func testEqual(a, b any) bool { return equalImpl(a, b, true) }
func (r *runtime) sequence(expressions []any, environment map[string]any, self any) PolyResult[any] {
	values := make([]any, 0, len(expressions))
	for _, raw := range expressions {
		value := r.expression(raw.(map[string]any), environment, self)
		if !value.Ok {
			return value
		}
		values = append(values, value.Value)
	}
	return polyOk(any(values))
}
func (r *runtime) construct(data map[string]any, variant any, environment map[string]any, self any) PolyResult[any] {
	fields := make([]any, 0, len(data["fields"].([]any)))
	for _, raw := range data["fields"].([]any) {
		field := raw.(map[string]any)
		value := r.expression(field["value"].(map[string]any), environment, self)
		if !value.Ok {
			return value
		}
		fields = append(fields, map[string]any{"field": field["field"], "value": value.Value})
	}
	aggregate := map[string]any{"declaration": data["declaration"], "fields": []any{}}
	if variant != nil {
		aggregate["variant"] = variant
	}
	result := r.aggregate(aggregate, variant)
	declaration := r.declarations[number(data["declaration"])]["data"].(map[string]any)
	var members []any
	if raw, ok := declaration["fields"].([]any); ok {
		members = raw
	}
	for _, raw := range fields {
		field := raw.(map[string]any)
		result[r.memberName(members, number(field["field"]))] = field["value"]
	}
	return polyOk(any(result))
}
func (r *runtime) constant(id int64) PolyResult[any] {
	if value, ok := r.constants[id]; ok {
		return value
	}
	declaration := r.declarations[id]
	if declaration == nil {
		return polyFail[any]("invalid_constant", "unknown constant")
	}
	data := declaration["data"].(map[string]any)
	expression := data["value"].(map[string]any)
	if expression["kind"] == "literal" {
		result := polyOk(r.value(expression["data"].(map[string]any)["value"].(map[string]any)))
		r.constants[id] = result
		return result
	}
	return polyFail[any]("unsupported", "constant form not implemented")
}
func (r *runtime) field(value any, name string) any {
	if record, ok := value.(polyRecord); ok {
		return record.polyValue()[name]
	}
	return value.(map[string]any)[name]
}
func (r *runtime) memberName(members []any, id int64) string {
	for _, raw := range members {
		member := raw.(map[string]any)
		header := member["header"].(map[string]any)
		if number(header["node"].(map[string]any)["id"]) == id {
			return header["name"].(string)
		}
	}
	return "field"
}
func (r *runtime) fieldName(id int64) string {
	for _, declaration := range r.declarations {
		data := declaration["data"].(map[string]any)
		if fields, ok := data["fields"].([]any); ok {
			if name := r.memberName(fields, id); name != "field" {
				return name
			}
		}
	}
	return "field"
}
func (r *runtime) findImplementation(contract, record int64) int64 {
	for id, declaration := range r.declarations {
		if declaration["kind"] == "implementation" {
			data := declaration["data"].(map[string]any)
			if number(data["contract"]) == contract && number(data["record"]) == record {
				return id
			}
		}
	}
	return -1
}
