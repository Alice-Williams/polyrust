// POLYRUST-BEGIN runtime.model
namespace polyrust_generated {
struct poly_error;
template <typename T> struct poly_result;
template <typename T, typename E> struct value_result;
}  // namespace polyrust_generated

namespace poly_runtime {

using any = std::any;
using any_list = std::vector<any>;
using bytes_value = std::vector<std::uint8_t>;
using json_map = std::map<std::string, any>;
using environment = std::map<std::string, any>;

struct error {
  std::string code;
  std::string message;
  bool operator==(const error&) const = default;
};

struct any_result {
  bool ok;
  any value;
  error failure;
};

struct option_value {
  bool some;
  any value;
};

struct value_result {
  bool is_ok;
  any value;
  any failure;
};

struct aggregate {
  std::int64_t declaration;
  std::string tag;
  std::map<std::string, any> fields;
};

struct test_outcome {
  any_result actual;
  any expected;
  bool expects_error;
};

inline any_result succeed(any value) {
  return {true, std::move(value), {}};
}

inline any_result fail(std::string code, std::string message) {
  return {false, {}, {std::move(code), std::move(message)}};
}

template <typename T> struct is_vector : std::false_type {};
template <typename T, typename A>
struct is_vector<std::vector<T, A>> : std::true_type {
  using item = T;
};

template <typename T> struct is_optional : std::false_type {};
template <typename T> struct is_optional<std::optional<T>> : std::true_type {
  using item = T;
};

template <typename T> struct is_value_result : std::false_type {};
template <typename T, typename E>
struct is_value_result<polyrust_generated::value_result<T, E>> : std::true_type {
  using ok_type = T;
  using error_type = E;
};

inline std::string encode_scalar(char32_t scalar) {
  std::string output;
  if (scalar <= 0x7F) output.push_back(static_cast<char>(scalar));
  else if (scalar <= 0x7FF) {
    output.push_back(static_cast<char>(0xC0 | (scalar >> 6)));
    output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
  } else if (scalar <= 0xFFFF) {
    output.push_back(static_cast<char>(0xE0 | (scalar >> 12)));
    output.push_back(static_cast<char>(0x80 | ((scalar >> 6) & 0x3F)));
    output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
  } else {
    output.push_back(static_cast<char>(0xF0 | (scalar >> 18)));
    output.push_back(static_cast<char>(0x80 | ((scalar >> 12) & 0x3F)));
    output.push_back(static_cast<char>(0x80 | ((scalar >> 6) & 0x3F)));
    output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
  }
  return output;
}

inline char32_t decode_scalar(const std::string& value) {
  if (value.empty()) throw std::runtime_error("empty Unicode scalar");
  const auto first = static_cast<std::uint8_t>(value[0]);
  if (first < 0x80) return first;
  int length = first < 0xE0 ? 2 : first < 0xF0 ? 3 : 4;
  char32_t scalar = first & (0x7F >> length);
  if (value.size() != static_cast<std::size_t>(length)) throw std::runtime_error("invalid Unicode scalar");
  for (int index = 1; index < length; ++index) {
    const auto byte = static_cast<std::uint8_t>(value[static_cast<std::size_t>(index)]);
    if ((byte & 0xC0) != 0x80) throw std::runtime_error("invalid Unicode scalar");
    scalar = (scalar << 6) | (byte & 0x3F);
  }
  return scalar;
}

template <typename T> any to_any(const T& value);
template <typename T> T from_any(const any& value);

template <typename T> any to_any(const T& value) {
  if constexpr (std::is_same_v<T, std::monostate>) {
    return any{};
  } else if constexpr (std::is_same_v<T, bytes_value>) {
    return value;
  } else if constexpr (is_vector<T>::value) {
    any_list values;
    values.reserve(value.size());
    for (const auto& item : value) values.push_back(to_any(item));
    return values;
  } else if constexpr (is_optional<T>::value) {
    return option_value{
        value.has_value(),
        value.has_value() ? to_any(*value) : any{},
    };
  } else if constexpr (is_value_result<T>::value) {
    return value_result{
        value.is_ok,
        value.value.has_value() ? to_any(*value.value) : any{},
        value.error.has_value() ? to_any(*value.error) : any{},
    };
  } else if constexpr (std::is_same_v<T, char32_t>) {
    return encode_scalar(value);
  } else {
    return value;
  }
}

template <typename T> T from_any(const any& value) {
  if constexpr (std::is_same_v<T, std::monostate>) {
    return {};
  } else if constexpr (std::is_same_v<T, bytes_value>) {
    return std::any_cast<const bytes_value&>(value);
  } else if constexpr (is_vector<T>::value) {
    T result;
    for (const auto& item : std::any_cast<const any_list&>(value)) {
      result.push_back(from_any<typename is_vector<T>::item>(item));
    }
    return result;
  } else if constexpr (is_optional<T>::value) {
    const auto& option = std::any_cast<const option_value&>(value);
    if (!option.some) return std::nullopt;
    return from_any<typename is_optional<T>::item>(option.value);
  } else if constexpr (is_value_result<T>::value) {
    const auto& result = std::any_cast<const value_result&>(value);
    if (result.is_ok) {
      return {true, from_any<typename is_value_result<T>::ok_type>(result.value), std::nullopt};
    }
    return {false, std::nullopt, from_any<typename is_value_result<T>::error_type>(result.failure)};
  } else if constexpr (std::is_same_v<T, char32_t>) {
    return decode_scalar(std::any_cast<const std::string&>(value));
  } else {
    return std::any_cast<T>(value);
  }
}

template <typename T>
polyrust_generated::poly_result<T> convert_result(const any_result& result) {
  if (!result.ok) {
    return {
        false,
        std::nullopt,
        polyrust_generated::poly_error{result.failure.code, result.failure.message},
    };
  }
  return {true, from_any<T>(result.value), std::nullopt};
}

inline std::int64_t number(const any& value) {
  if (value.type() == typeid(std::int64_t)) return std::any_cast<std::int64_t>(value);
  if (value.type() == typeid(std::int32_t)) return std::any_cast<std::int32_t>(value);
  throw std::runtime_error("JSON number expected");
}

inline const std::string& string(const any& value) {
  return std::any_cast<const std::string&>(value);
}

inline const json_map& object(const any& value) {
  return std::any_cast<const json_map&>(value);
}

inline const any_list& list(const any& value) {
  return std::any_cast<const any_list&>(value);
}

// POLYRUST-END runtime.model
// POLYRUST-BEGIN runtime.json
class json_parser {
 public:
  explicit json_parser(std::string_view source) : source_(source) {}

  any parse() {
    any result = parse_value();
    whitespace();
    if (offset_ != source_.size()) throw std::runtime_error("trailing JSON");
    return result;
  }

 private:
  any parse_value() {
    whitespace();
    if (offset_ >= source_.size()) throw std::runtime_error("unexpected JSON end");
    switch (source_[offset_]) {
      case '{': return parse_object();
      case '[': return parse_array();
      case '"': return parse_string();
      case 't': return keyword("true", true);
      case 'f': return keyword("false", false);
      case 'n': return keyword("null", std::nullptr_t{});
      default: return parse_number();
    }
  }

  any parse_object() {
    ++offset_;
    json_map result;
    whitespace();
    if (take('}')) return result;
    for (;;) {
      std::string key = parse_string();
      whitespace();
      expect(':');
      result.emplace(std::move(key), parse_value());
      whitespace();
      if (take('}')) return result;
      expect(',');
      whitespace();
    }
  }

  any parse_array() {
    ++offset_;
    any_list result;
    whitespace();
    if (take(']')) return result;
    for (;;) {
      result.push_back(parse_value());
      whitespace();
      if (take(']')) return result;
      expect(',');
    }
  }

  std::string parse_string() {
    whitespace();
    expect('"');
    std::string result;
    while (offset_ < source_.size()) {
      char character = source_[offset_++];
      if (character == '"') return result;
      if (character != '\\') {
        result.push_back(character);
        continue;
      }
      if (offset_ >= source_.size()) throw std::runtime_error("invalid JSON escape");
      const char escaped = source_[offset_++];
      switch (escaped) {
        case '"': result.push_back('"'); break;
        case '\\': result.push_back('\\'); break;
        case '/': result.push_back('/'); break;
        case 'b': result.push_back('\b'); break;
        case 'f': result.push_back('\f'); break;
        case 'n': result.push_back('\n'); break;
        case 'r': result.push_back('\r'); break;
        case 't': result.push_back('\t'); break;
        case 'u': append_unicode_escape(result); break;
        default: throw std::runtime_error("invalid JSON escape");
      }
    }
    throw std::runtime_error("unterminated JSON string");
  }

  void append_unicode_escape(std::string& output) {
    std::uint32_t scalar = hex_quad();
    if (scalar >= 0xD800 && scalar <= 0xDBFF) {
      if (offset_ + 2 > source_.size() || source_[offset_] != '\\' || source_[offset_ + 1] != 'u') {
        throw std::runtime_error("unpaired JSON surrogate");
      }
      offset_ += 2;
      const std::uint32_t low = hex_quad();
      if (low < 0xDC00 || low > 0xDFFF) throw std::runtime_error("invalid JSON surrogate");
      scalar = 0x10000 + ((scalar - 0xD800) << 10) + (low - 0xDC00);
    }
    append_utf8(output, scalar);
  }

  std::uint32_t hex_quad() {
    if (offset_ + 4 > source_.size()) throw std::runtime_error("short JSON escape");
    std::uint32_t value = 0;
    for (int index = 0; index < 4; ++index) {
      const char character = source_[offset_++];
      value <<= 4;
      if (character >= '0' && character <= '9') value += character - '0';
      else if (character >= 'a' && character <= 'f') value += character - 'a' + 10;
      else if (character >= 'A' && character <= 'F') value += character - 'A' + 10;
      else throw std::runtime_error("invalid JSON hex");
    }
    return value;
  }

  static void append_utf8(std::string& output, std::uint32_t scalar) {
    if (scalar <= 0x7F) output.push_back(static_cast<char>(scalar));
    else if (scalar <= 0x7FF) {
      output.push_back(static_cast<char>(0xC0 | (scalar >> 6)));
      output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
    } else if (scalar <= 0xFFFF) {
      output.push_back(static_cast<char>(0xE0 | (scalar >> 12)));
      output.push_back(static_cast<char>(0x80 | ((scalar >> 6) & 0x3F)));
      output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
    } else {
      output.push_back(static_cast<char>(0xF0 | (scalar >> 18)));
      output.push_back(static_cast<char>(0x80 | ((scalar >> 12) & 0x3F)));
      output.push_back(static_cast<char>(0x80 | ((scalar >> 6) & 0x3F)));
      output.push_back(static_cast<char>(0x80 | (scalar & 0x3F)));
    }
  }

  any parse_number() {
    const std::size_t start = offset_;
    if (source_[offset_] == '-') ++offset_;
    while (offset_ < source_.size() && source_[offset_] >= '0' && source_[offset_] <= '9') ++offset_;
    bool decimal = false;
    if (offset_ < source_.size() && source_[offset_] == '.') {
      decimal = true;
      ++offset_;
      while (offset_ < source_.size() && source_[offset_] >= '0' && source_[offset_] <= '9') ++offset_;
    }
    if (offset_ < source_.size() && (source_[offset_] == 'e' || source_[offset_] == 'E')) {
      decimal = true;
      ++offset_;
      if (offset_ < source_.size() && (source_[offset_] == '+' || source_[offset_] == '-')) ++offset_;
      while (offset_ < source_.size() && source_[offset_] >= '0' && source_[offset_] <= '9') ++offset_;
    }
    const std::string token(source_.substr(start, offset_ - start));
    if (decimal) return std::stod(token);
    return static_cast<std::int64_t>(std::stoll(token));
  }

  template <typename T> any keyword(std::string_view token, T value) {
    if (source_.substr(offset_, token.size()) != token) throw std::runtime_error("invalid JSON token");
    offset_ += token.size();
    return value;
  }

  void whitespace() {
    while (offset_ < source_.size()
        && (source_[offset_] == ' ' || source_[offset_] == '\n'
            || source_[offset_] == '\r' || source_[offset_] == '\t')) ++offset_;
  }

  bool take(char expected) {
    if (offset_ < source_.size() && source_[offset_] == expected) {
      ++offset_;
      return true;
    }
    return false;
  }

  void expect(char expected) {
    if (!take(expected)) throw std::runtime_error("unexpected JSON token");
  }

  std::string_view source_;
  std::size_t offset_ = 0;
};

inline bool is_null(const any& value) {
  return value.type() == typeid(std::nullptr_t);
}

inline const any& member(const json_map& value, const std::string& name) {
  return value.at(name);
}

inline const any_list& nullable_list(const json_map& value, const std::string& name) {
  static const any_list empty;
  auto found = value.find(name);
  return found == value.end() || is_null(found->second) ? empty : list(found->second);
}

inline std::int64_t node_id_from_header(const json_map& header) {
  return number(member(object(member(header, "node")), "id"));
}

inline std::int64_t node_id(const json_map& declaration) {
  return node_id_from_header(object(member(object(member(declaration, "data")), "header")));
}

// POLYRUST-END runtime.json
// POLYRUST-BEGIN runtime.engine
inline any_result checked_i32(std::int64_t value) {
  if (value < std::numeric_limits<std::int32_t>::min()
      || value > std::numeric_limits<std::int32_t>::max()) {
    return fail("integer_overflow", "i32 result is out of range");
  }
  return succeed(static_cast<std::int32_t>(value));
}

inline any_result checked_i64(__int128 value) {
  if (value < std::numeric_limits<std::int64_t>::min()
      || value > std::numeric_limits<std::int64_t>::max()) {
    return fail("integer_overflow", "i64 result is out of range");
  }
  return succeed(static_cast<std::int64_t>(value));
}

inline std::size_t utf8_scalar_width(std::uint8_t byte) {
  if (byte < 0x80) return 1;
  if ((byte & 0xE0) == 0xC0) return 2;
  if ((byte & 0xF0) == 0xE0) return 3;
  if ((byte & 0xF8) == 0xF0) return 4;
  return 0;
}

inline bool valid_utf8(const bytes_value& bytes, std::size_t* scalar_count = nullptr,
                       std::size_t* utf16_count = nullptr) {
  std::size_t count = 0;
  std::size_t code_units = 0;
  for (std::size_t offset = 0; offset < bytes.size();) {
    const std::size_t width = utf8_scalar_width(bytes[offset]);
    if (width == 0 || offset + width > bytes.size()) return false;
    char32_t scalar = bytes[offset] & (0x7F >> width);
    for (std::size_t index = 1; index < width; ++index) {
      const auto byte = bytes[offset + index];
      if ((byte & 0xC0) != 0x80) return false;
      scalar = (scalar << 6) | (byte & 0x3F);
    }
    if ((width == 2 && scalar < 0x80) || (width == 3 && scalar < 0x800)
        || (width == 4 && scalar < 0x10000) || scalar > 0x10FFFF
        || (scalar >= 0xD800 && scalar <= 0xDFFF)) return false;
    offset += width;
    ++count;
    code_units += scalar > 0xFFFF ? 2 : 1;
  }
  if (scalar_count != nullptr) *scalar_count = count;
  if (utf16_count != nullptr) *utf16_count = code_units;
  return true;
}

inline bool equal_impl(const any& left, const any& right, bool exact_float) {
  if (left.type() != right.type()) return false;
  if (!left.has_value()) return true;
  if (left.type() == typeid(std::nullptr_t)) return true;
  if (left.type() == typeid(bool)) return std::any_cast<bool>(left) == std::any_cast<bool>(right);
  if (left.type() == typeid(std::int32_t)) return std::any_cast<std::int32_t>(left) == std::any_cast<std::int32_t>(right);
  if (left.type() == typeid(std::int64_t)) return std::any_cast<std::int64_t>(left) == std::any_cast<std::int64_t>(right);
  if (left.type() == typeid(double)) {
    const double a = std::any_cast<double>(left), b = std::any_cast<double>(right);
    return exact_float
        ? (std::isnan(a) && std::isnan(b)) || std::bit_cast<std::uint64_t>(a) == std::bit_cast<std::uint64_t>(b)
        : a == b;
  }
  if (left.type() == typeid(std::string)) return string(left) == string(right);
  if (left.type() == typeid(bytes_value)) return std::any_cast<const bytes_value&>(left) == std::any_cast<const bytes_value&>(right);
  if (left.type() == typeid(any_list)) {
    const auto& a = list(left); const auto& b = list(right);
    if (a.size() != b.size()) return false;
    for (std::size_t index = 0; index < a.size(); ++index) if (!equal_impl(a[index], b[index], exact_float)) return false;
    return true;
  }
  if (left.type() == typeid(option_value)) {
    const auto& a = std::any_cast<const option_value&>(left); const auto& b = std::any_cast<const option_value&>(right);
    return a.some == b.some && (!a.some || equal_impl(a.value, b.value, exact_float));
  }
  if (left.type() == typeid(value_result)) {
    const auto& a = std::any_cast<const value_result&>(left); const auto& b = std::any_cast<const value_result&>(right);
    return a.is_ok == b.is_ok && equal_impl(a.is_ok ? a.value : a.failure, b.is_ok ? b.value : b.failure, exact_float);
  }
  if (left.type() == typeid(aggregate)) {
    const auto& a = std::any_cast<const aggregate&>(left); const auto& b = std::any_cast<const aggregate&>(right);
    if (a.declaration != b.declaration || a.tag != b.tag || a.fields.size() != b.fields.size()) return false;
    for (const auto& [name, value] : a.fields) {
      const auto found = b.fields.find(name);
      if (found == b.fields.end() || !equal_impl(value, found->second, exact_float)) return false;
    }
    return true;
  }
  return false;
}

inline bool deep_equal(const any& left, const any& right) { return equal_impl(left, right, true); }
inline bool semantic_equal(const any& left, const any& right) { return equal_impl(left, right, false); }

struct flow {
  bool returned;
  any_result result;
};

class runtime {
 public:
  explicit runtime(std::string_view source) {
    document_ = object(json_parser(source).parse());
    const auto& module = object(member(document_, "module"));
    for (const auto& raw : list(member(module, "declarations"))) {
      const auto& declaration = object(raw);
      declarations_.emplace(node_id(declaration), declaration);
    }
  }

  any_result invoke(std::int64_t function_id, const any_list& arguments) {
    const auto found = declarations_.find(function_id);
    if (found == declarations_.end() || string(member(found->second, "kind")) != "function") {
      return fail("invalid_call", "unknown function " + std::to_string(function_id));
    }
    return invoke_body(object(member(found->second, "data")), arguments, any{});
  }

  any_result invoke_method(std::int64_t implementation_id, std::int64_t method_id,
                           const any& receiver, const any_list& arguments) {
    const auto found = declarations_.find(implementation_id);
    if (found == declarations_.end() || string(member(found->second, "kind")) != "implementation") {
      return fail("invalid_call", "unknown implementation " + std::to_string(implementation_id));
    }
    for (const auto& raw : list(member(object(member(found->second, "data")), "methods"))) {
      const auto& method = object(raw);
      if (node_id_from_header(object(member(method, "header"))) == method_id
          || number(member(method, "contract_method")) == method_id) {
        return invoke_body(method, arguments, receiver);
      }
    }
    return fail("invalid_call", "unknown method " + std::to_string(method_id));
  }

  any_result read_constant(std::int64_t identifier) { return constant(identifier); }

  bool run_tests() {
    const auto& module = object(member(document_, "module"));
    for (const auto& raw : list(member(module, "declarations"))) {
      const auto& declaration = object(raw);
      if (string(member(declaration, "kind")) != "test") continue;
      const auto& data = object(member(declaration, "data"));
      const auto& invocation = object(member(data, "invocation"));
      const auto& invocation_data = object(member(invocation, "data"));
      any_list arguments;
      for (const auto& argument : list(member(invocation_data, "arguments"))) arguments.push_back(decode(argument));
      any_result actual = string(member(invocation, "kind")) == "function"
          ? invoke(number(member(invocation_data, "function")), arguments)
          : invoke_method(number(member(invocation_data, "implementation")),
                          number(member(invocation_data, "method")),
                          decode(member(invocation_data, "receiver")), arguments);
      const auto& expected = object(member(data, "expected"));
      const bool expects_error = string(member(expected, "kind")) == "error";
      if (expects_error) {
        if (actual.ok) return false;
        const auto& expected_error = object(member(expected, "data"));
        if (actual.failure.code != string(member(expected_error, "code"))) return false;
      } else if (!actual.ok || !deep_equal(actual.value, decode(member(expected, "data")))) {
        return false;
      }
    }
    return true;
  }

 private:
  any decode(const any& typed) { return value(object(member(object(typed), "value"))); }

  any_result invoke_body(const json_map& callable, const any_list& arguments, const any& self) {
    environment values;
    const auto& parameters = list(member(callable, "parameters"));
    for (std::size_t index = 0; index < parameters.size(); ++index) {
      const auto& header = object(member(object(parameters[index]), "header"));
      values.emplace(string(member(header, "name")), arguments.at(index));
    }
    return block(object(member(callable, "body")), std::move(values), self).result;
  }

  any value(const json_map& item) {
    const std::string kind = string(member(item, "kind"));
    const auto found = item.find("data");
    const any data = found == item.end() ? any{} : found->second;
    if (kind == "unit") return any{};
    if (kind == "bool" || kind == "string" || kind == "char") return data;
    if (kind == "i32") return static_cast<std::int32_t>(number(data));
    if (kind == "i64") return static_cast<std::int64_t>(std::stoll(string(data)));
    if (kind == "f64") return std::bit_cast<double>(static_cast<std::uint64_t>(std::stoull(string(data))));
    if (kind == "bytes") {
      bytes_value result;
      for (const auto& byte : list(data)) result.push_back(static_cast<std::uint8_t>(number(byte)));
      return result;
    }
    if (kind == "list") {
      any_list result;
      for (const auto& raw : list(data)) result.push_back(value(object(raw)));
      return result;
    }
    if (kind == "none") return option_value{false, {}};
    if (kind == "some") return option_value{true, value(object(data))};
    if (kind == "ok") return value_result{true, value(object(data)), {}};
    if (kind == "err") return value_result{false, {}, value(object(data))};
    if (kind == "record" || kind == "enum") {
      const auto& values = object(data);
      return make_aggregate(values, kind == "enum" ? std::optional(number(member(values, "variant"))) : std::nullopt,
                            false, environment{}, any{}).value;
    }
    return any{};
  }

  any_result expression(const json_map& expression_value, environment values, const any& self) {
    const std::string kind = string(member(expression_value, "kind"));
    const auto& data = object(member(expression_value, "data"));
    if (kind == "literal") return succeed(value(object(member(data, "value"))));
    if (kind == "local") return succeed(values.at(string(member(data, "name"))));
    if (kind == "self_value") return succeed(self);
    if (kind == "constant") return constant(number(member(data, "declaration")));
    if (kind == "construct_none") return succeed(option_value{false, {}});
    if (kind == "construct_some" || kind == "construct_ok" || kind == "construct_err") {
      auto item = expression(object(member(data, "value")), values, self);
      if (!item.ok) return item;
      if (kind == "construct_some") return succeed(option_value{true, item.value});
      return succeed(kind == "construct_ok" ? any(value_result{true, item.value, {}})
                                               : any(value_result{false, {}, item.value}));
    }
    if (kind == "construct_list") return sequence(list(member(data, "elements")), values, self);
    if (kind == "construct_record" || kind == "construct_enum") {
      return make_aggregate(data,
          kind == "construct_enum" ? std::optional(number(member(data, "variant"))) : std::nullopt,
          true, values, self);
    }
    if (kind == "field") {
      auto base = expression(object(member(data, "base")), values, self);
      return base.ok ? succeed(field(base.value, field_name(number(member(data, "field"))))) : base;
    }
    if (kind == "call" || kind == "intrinsic" || kind == "method_call") {
      auto arguments = sequence(list(member(data, "arguments")), values, self);
      if (!arguments.ok) return arguments;
      const auto& args = std::any_cast<const any_list&>(arguments.value);
      if (kind == "call") return invoke(number(member(data, "function")), args);
      if (kind == "intrinsic") return intrinsic(string(member(data, "operation")), args);
      auto receiver = expression(object(member(data, "receiver")), values, self);
      if (!receiver.ok) return receiver;
      const auto& dispatch = object(member(data, "dispatch"));
      const auto& target = object(member(dispatch, "data"));
      std::int64_t implementation;
      if (string(member(dispatch, "kind")) == "contract") {
        implementation = find_implementation(number(member(target, "contract")),
                                             std::any_cast<const aggregate&>(receiver.value).declaration);
      } else {
        implementation = number(member(target, "implementation"));
      }
      return invoke_method(implementation, number(member(target, "method")), receiver.value, args);
    }
    if (kind == "if") {
      auto condition = expression(object(member(data, "condition")), values, self);
      if (!condition.ok) return condition;
      const char* branch = std::any_cast<bool>(condition.value) ? "then_block" : "else_block";
      return block(object(member(data, branch)), std::move(values), self).result;
    }
    if (kind == "match") return match_expression(data, std::move(values), self);
    if (kind == "block") return block(data, std::move(values), self).result;
    return fail("invalid_expression", "unknown expression " + kind);
  }

  flow block(const json_map& block_value, environment values, const any& self) {
    for (const auto& raw : list(member(block_value, "statements"))) {
      const auto& statement = object(raw);
      const std::string kind = string(member(statement, "kind"));
      const auto& data = object(member(statement, "data"));
      if (kind == "let" || kind == "expression") {
        auto item = expression(object(member(data, "value")), values, self);
        if (!item.ok) return {true, item};
        if (kind == "let") values[string(member(data, "name"))] = item.value;
      } else if (kind == "return") {
        const auto found = data.find("value");
        return {true, found == data.end() || is_null(found->second)
                          ? succeed(any{}) : expression(object(found->second), values, self)};
      } else if (kind == "for_each") {
        auto items = expression(object(member(data, "iterable")), values, self);
        if (!items.ok) return {true, items};
        for (const auto& item : std::any_cast<const any_list&>(items.value)) {
          environment inner = values;
          inner[string(member(data, "binding"))] = item;
          auto item_flow = block(object(member(data, "body")), std::move(inner), self);
          if (item_flow.returned || !item_flow.result.ok) return item_flow;
        }
      }
    }
    const auto found = block_value.find("result");
    return {false, found == block_value.end() || is_null(found->second)
                       ? succeed(any{}) : expression(object(found->second), values, self)};
  }

  any_result match_expression(const json_map& data, environment values, const any& self) {
    auto matched = expression(object(member(data, "value")), values, self);
    if (!matched.ok) return matched;
    for (const auto& raw : list(member(data, "arms"))) {
      const auto& arm = object(raw);
      auto bindings = pattern(object(member(arm, "pattern")), matched.value);
      if (bindings.has_value()) {
        values.insert(bindings->begin(), bindings->end());
        return block(object(member(arm, "body")), std::move(values), self).result;
      }
    }
    return fail("non_exhaustive_match", "checked match had no matching arm");
  }

  std::optional<environment> pattern(const json_map& pattern_value, const any& value) {
    const std::string kind = string(member(pattern_value, "kind"));
    const auto& data = object(member(pattern_value, "data"));
    environment result;
    if (kind == "wildcard") return result;
    if (kind == "bool") return std::any_cast<bool>(value) == std::any_cast<bool>(member(data, "value"))
                                   ? std::optional(result) : std::nullopt;
    if (kind == "none" || kind == "some") {
      const auto& option = std::any_cast<const option_value&>(value);
      if (option.some != (kind == "some")) return std::nullopt;
      if (option.some) result.emplace(string(member(data, "binding")), option.value);
      return result;
    }
    if (kind == "ok" || kind == "err") {
      const auto& item = std::any_cast<const value_result&>(value);
      if (item.is_ok != (kind == "ok")) return std::nullopt;
      result.emplace(string(member(data, "binding")), item.is_ok ? item.value : item.failure);
      return result;
    }
    if (kind == "enum_variant") {
      const auto& item = std::any_cast<const aggregate&>(value);
      const auto& declaration = object(member(declarations_.at(number(member(data, "declaration"))), "data"));
      const auto* variant = find_variant(declaration, number(member(data, "variant")));
      if (variant == nullptr || item.tag != string(member(object(member(*variant, "header")), "name"))) return std::nullopt;
      const auto& members = list(member(*variant, "fields"));
      for (const auto& raw : list(member(data, "bindings"))) {
        const auto& binding = object(raw);
        result.emplace(string(member(binding, "binding")), item.fields.at(member_name(members, number(member(binding, "field")))));
      }
      return result;
    }
    return std::nullopt;
  }

  any_result intrinsic(const std::string& name, const any_list& values) {
    const any empty;
    const any& a = values.empty() ? empty : values[0];
    const any& b = values.size() < 2 ? empty : values[1];
    const any& c = values.size() < 3 ? empty : values[2];
    if (name == "bool_not") return succeed(!std::any_cast<bool>(a));
    if (name == "bool_and") return succeed(std::any_cast<bool>(a) && std::any_cast<bool>(b));
    if (name == "bool_or") return succeed(std::any_cast<bool>(a) || std::any_cast<bool>(b));
    if (name == "equal") return succeed(semantic_equal(a, b));
    if (name == "not_equal") return succeed(!semantic_equal(a, b));
    if (name == "less" || name == "less_equal" || name == "greater" || name == "greater_equal") {
      const int compared = compare(a, b);
      return succeed(name == "less" ? compared < 0 : name == "less_equal" ? compared <= 0
                                     : name == "greater" ? compared > 0 : compared >= 0);
    }
    if (name == "int_neg_checked" || name == "int_add_checked" || name == "int_sub_checked"
        || name == "int_mul_checked" || name == "int_div_checked" || name == "int_rem_checked") {
      if ((name == "int_div_checked" || name == "int_rem_checked") && integer(b) == 0) {
        return fail("division_by_zero", "integer division by zero");
      }
      const bool narrow = a.type() == typeid(std::int32_t);
      const __int128 left = integer(a), right = name == "int_neg_checked" ? 0 : integer(b);
      __int128 result = 0;
      if (name == "int_neg_checked") result = -left;
      else if (name == "int_add_checked") result = left + right;
      else if (name == "int_sub_checked") result = left - right;
      else if (name == "int_mul_checked") result = left * right;
      else if (name == "int_div_checked") result = left / right;
      else result = left % right;
      return narrow ? checked_i32(static_cast<std::int64_t>(result)) : checked_i64(result);
    }
    if (name == "int_neg_wrapping" || name == "int_add_wrapping" || name == "int_sub_wrapping"
        || name == "int_mul_wrapping") {
      if (a.type() == typeid(std::int32_t)) {
        const auto left = static_cast<std::uint32_t>(std::any_cast<std::int32_t>(a));
        const auto right = name == "int_neg_wrapping" ? 0U : static_cast<std::uint32_t>(std::any_cast<std::int32_t>(b));
        const auto result = name == "int_neg_wrapping" ? 0U - left
            : name == "int_add_wrapping" ? left + right
            : name == "int_sub_wrapping" ? left - right : left * right;
        return succeed(std::bit_cast<std::int32_t>(result));
      }
      const auto left = static_cast<std::uint64_t>(std::any_cast<std::int64_t>(a));
      const auto right = name == "int_neg_wrapping" ? UINT64_C(0) : static_cast<std::uint64_t>(std::any_cast<std::int64_t>(b));
      const auto result = name == "int_neg_wrapping" ? UINT64_C(0) - left
          : name == "int_add_wrapping" ? left + right
          : name == "int_sub_wrapping" ? left - right : left * right;
      return succeed(std::bit_cast<std::int64_t>(result));
    }
    if (name == "int_bit_not") return a.type() == typeid(std::int32_t)
        ? succeed(static_cast<std::int32_t>(~std::any_cast<std::int32_t>(a)))
        : succeed(static_cast<std::int64_t>(~std::any_cast<std::int64_t>(a)));
    if (name == "int_bit_and" || name == "int_bit_or" || name == "int_bit_xor") {
      if (a.type() == typeid(std::int32_t)) {
        const auto left = std::any_cast<std::int32_t>(a), right = std::any_cast<std::int32_t>(b);
        return succeed(name == "int_bit_and" ? left & right : name == "int_bit_or" ? left | right : left ^ right);
      }
      const auto left = std::any_cast<std::int64_t>(a), right = std::any_cast<std::int64_t>(b);
      return succeed(name == "int_bit_and" ? left & right : name == "int_bit_or" ? left | right : left ^ right);
    }
    if (name == "int_shift_left_checked") {
      const auto amount = static_cast<unsigned>(std::any_cast<std::int32_t>(b));
      return a.type() == typeid(std::int32_t)
          ? checked_i32(static_cast<std::int64_t>(std::any_cast<std::int32_t>(a)) * (INT64_C(1) << amount))
          : checked_i64(static_cast<__int128>(std::any_cast<std::int64_t>(a)) << amount);
    }
    if (name == "int_shift_right_checked") {
      const auto amount = static_cast<unsigned>(std::any_cast<std::int32_t>(b));
      return a.type() == typeid(std::int32_t)
          ? succeed(static_cast<std::int32_t>(std::any_cast<std::int32_t>(a) >> amount))
          : succeed(static_cast<std::int64_t>(std::any_cast<std::int64_t>(a) >> amount));
    }
    if (name == "float_neg") return succeed(-std::any_cast<double>(a));
    if (name == "float_trunc") return succeed(std::trunc(std::any_cast<double>(a)));
    if (name == "float_is_nan") return succeed(std::isnan(std::any_cast<double>(a)));
    if (name == "float_is_negative_zero") {
      const auto value = std::any_cast<double>(a);
      return succeed(value == 0.0 && std::signbit(value));
    }
    if (name == "float_abs") {
      const auto bits = std::bit_cast<std::uint64_t>(std::any_cast<double>(a));
      return succeed(std::bit_cast<double>(bits & UINT64_C(0x7fffffffffffffff)));
    }
    if (name == "float_add") return succeed(std::any_cast<double>(a) + std::any_cast<double>(b));
    if (name == "float_sub") return succeed(std::any_cast<double>(a) - std::any_cast<double>(b));
    if (name == "float_mul") return succeed(std::any_cast<double>(a) * std::any_cast<double>(b));
    if (name == "float_div") return succeed(std::any_cast<double>(a) / std::any_cast<double>(b));
    if (name == "float_rem_trunc") return succeed(std::fmod(std::any_cast<double>(a), std::any_cast<double>(b)));
    if (name == "string_concat") return succeed(string(a) + string(b));
    if (name == "string_scalar_length") {
      bytes_value bytes(string(a).begin(), string(a).end());
      std::size_t count = 0;
      return valid_utf8(bytes, &count) ? succeed(static_cast<std::int64_t>(count))
                                       : fail("invalid_unicode", "invalid Unicode scalar sequence");
    }
    if (name == "string_utf16_length") {
      bytes_value bytes(string(a).begin(), string(a).end());
      std::size_t count = 0;
      return valid_utf8(bytes, nullptr, &count) ? succeed(static_cast<std::int64_t>(count))
                                                : fail("invalid_unicode", "invalid Unicode scalar sequence");
    }
    if (name == "string_index_of_literal") {
      const auto& source = string(a);
      const auto& needle = string(b);
      const bytes_value source_bytes(source.begin(), source.end());
      const bytes_value needle_bytes(needle.begin(), needle.end());
      if (!valid_utf8(source_bytes) || !valid_utf8(needle_bytes)) {
        return fail("invalid_unicode", "invalid Unicode scalar sequence");
      }
      const std::size_t found = source.find(needle);
      if (found == std::string::npos) return succeed(option_value{false, {}});
      std::int64_t scalar_index = 0;
      for (std::size_t offset = 0; offset < found; ++scalar_index) {
        offset += utf8_scalar_width(static_cast<std::uint8_t>(source[offset]));
      }
      return succeed(option_value{true, scalar_index});
    }
    if (name == "string_slice_scalars") {
      const auto& source = string(a);
      const bytes_value source_bytes(source.begin(), source.end());
      std::size_t scalar_count = 0;
      if (!valid_utf8(source_bytes, &scalar_count)) {
        return fail("invalid_unicode", "invalid Unicode scalar sequence");
      }
      const std::int64_t length = static_cast<std::int64_t>(scalar_count);
      const std::int64_t start =
          std::clamp(std::any_cast<std::int64_t>(b), INT64_C(0), length);
      const std::int64_t end =
          std::clamp(std::any_cast<std::int64_t>(c), INT64_C(0), length);
      if (start >= end) return succeed(std::string{});
      std::size_t byte_start = 0;
      for (std::int64_t index = 0; index < start; ++index) {
        byte_start += utf8_scalar_width(static_cast<std::uint8_t>(source[byte_start]));
      }
      std::size_t byte_end = byte_start;
      for (std::int64_t index = start; index < end; ++index) {
        byte_end += utf8_scalar_width(static_cast<std::uint8_t>(source[byte_end]));
      }
      return succeed(source.substr(byte_start, byte_end - byte_start));
    }
    if (name == "string_is_empty") return succeed(string(a).empty());
    if (name == "string_contains") return succeed(string(a).find(string(b)) != std::string::npos);
    if (name == "string_starts_with") return succeed(string(a).starts_with(string(b)));
    if (name == "string_strip_prefix") return succeed(string(a).starts_with(string(b)) ? string(a).substr(string(b).size()) : string(a));
    if (name == "string_ends_with") return succeed(string(a).ends_with(string(b)));
    if (name == "string_replace_all") return succeed(replace_all(string(a), string(b), string(c)));
    if (name == "string_replace_many") return succeed(replace_many(values));
    if (name == "string_truncate_utf8_bytes") return succeed(truncate_utf8_bytes(string(a), std::any_cast<double>(b)));
    if (name == "string_trim_start") return succeed(trim_scalars(string(a), string(b), true));
    if (name == "string_trim_end") return succeed(trim_scalars(string(a), string(b), false));
    if (name == "bytes_concat") {
      bytes_value result = std::any_cast<const bytes_value&>(a);
      const auto& right = std::any_cast<const bytes_value&>(b);
      result.insert(result.end(), right.begin(), right.end());
      return succeed(result);
    }
    if (name == "bytes_replace_all") {
      return succeed(replace_bytes_all(std::any_cast<const bytes_value&>(a),
                                       std::any_cast<const bytes_value&>(b),
                                       std::any_cast<const bytes_value&>(c)));
    }
    if (name == "bytes_length") return succeed(static_cast<std::int64_t>(std::any_cast<const bytes_value&>(a).size()));
    if (name == "bytes_is_empty") return succeed(std::any_cast<const bytes_value&>(a).empty());
    if (name == "list_concat") {
      any_list result = list(a);
      const auto& right = list(b);
      result.insert(result.end(), right.begin(), right.end());
      return succeed(result);
    }
    if (name == "list_length") return succeed(static_cast<std::int64_t>(list(a).size()));
    if (name == "list_is_empty") return succeed(list(a).empty());
    if (name == "list_get_checked") {
      const std::int64_t index = integer(b);
      return index >= 0 && static_cast<std::size_t>(index) < list(a).size()
          ? succeed(list(a)[static_cast<std::size_t>(index)])
          : fail("index_out_of_bounds", "list index out of bounds");
    }
    if (name == "list_append") {
      any_list result = list(a); result.push_back(b); return succeed(result);
    }
    if (name == "list_contains") {
      return succeed(std::any_of(list(a).begin(), list(a).end(), [&](const any& item) { return semantic_equal(item, b); }));
    }
    if (name == "list_index_of") {
      const auto& values = list(a);
      for (std::size_t index = 0; index < values.size(); ++index) {
        if (semantic_equal(values[index], b)) {
          return succeed(option_value{true, static_cast<std::int64_t>(index)});
        }
      }
      return succeed(option_value{false, {}});
    }
    if (name == "option_is_some") return succeed(std::any_cast<const option_value&>(a).some);
    if (name == "option_is_none") return succeed(!std::any_cast<const option_value&>(a).some);
    if (name == "option_unwrap_or") {
      const auto& option = std::any_cast<const option_value&>(a);
      return succeed(option.some ? option.value : b);
    }
    if (name == "result_is_ok") return succeed(std::any_cast<const value_result&>(a).is_ok);
    if (name == "result_is_err") return succeed(!std::any_cast<const value_result&>(a).is_ok);
    if (name == "widen_i32_to_i64") return succeed(static_cast<std::int64_t>(std::any_cast<std::int32_t>(a)));
    if (name == "narrow_i64_to_i32_checked") return checked_i32(std::any_cast<std::int64_t>(a));
    if (name == "string_to_utf8") return succeed(bytes_value(string(a).begin(), string(a).end()));
    if (name == "string_from_utf8_checked") {
      const auto& bytes = std::any_cast<const bytes_value&>(a);
      return valid_utf8(bytes) ? succeed(std::string(bytes.begin(), bytes.end()))
                               : fail("invalid_utf8", "invalid UTF-8");
    }
    return fail("invalid_intrinsic", "unknown intrinsic " + name);
  }

  static __int128 integer(const any& value) {
    return value.type() == typeid(std::int32_t) ? std::any_cast<std::int32_t>(value)
                                                : std::any_cast<std::int64_t>(value);
  }

  static int compare(const any& left, const any& right) {
    if (left.type() == typeid(std::int32_t)) {
      const auto a = std::any_cast<std::int32_t>(left), b = std::any_cast<std::int32_t>(right);
      return (a > b) - (a < b);
    }
    if (left.type() == typeid(std::int64_t)) {
      const auto a = std::any_cast<std::int64_t>(left), b = std::any_cast<std::int64_t>(right);
      return (a > b) - (a < b);
    }
    if (left.type() == typeid(double)) {
      const auto a = std::any_cast<double>(left), b = std::any_cast<double>(right);
      return (a > b) - (a < b);
    }
    return string(left).compare(string(right));
  }

  static std::string replace_all(const std::string& source, const std::string& needle,
                                 const std::string& replacement) {
    if (needle.empty()) {
      std::string result = replacement;
      for (std::size_t offset = 0; offset < source.size();) {
        const std::size_t width = utf8_scalar_width(static_cast<std::uint8_t>(source[offset]));
        result.append(source, offset, width); result += replacement; offset += width;
      }
      return result;
    }
    std::string result;
    for (std::size_t offset = 0;;) {
      const auto found = source.find(needle, offset);
      if (found == std::string::npos) { result.append(source, offset); return result; }
      result.append(source, offset, found - offset); result += replacement; offset = found + needle.size();
    }
  }

  static bytes_value replace_bytes_all(const bytes_value& source, const bytes_value& needle,
                                       const bytes_value& replacement) {
    bytes_value result;
    if (needle.empty()) {
      result.insert(result.end(), replacement.begin(), replacement.end());
      for (const auto value : source) {
        result.push_back(value);
        result.insert(result.end(), replacement.begin(), replacement.end());
      }
      return result;
    }
    for (std::size_t offset = 0; offset < source.size();) {
      const bool matches = offset + needle.size() <= source.size()
          && std::equal(needle.begin(), needle.end(), source.begin() + offset);
      if (matches) {
        result.insert(result.end(), replacement.begin(), replacement.end());
        offset += needle.size();
      } else {
        result.push_back(source[offset++]);
      }
    }
    return result;
  }

  static std::string replace_many(const any_list& values) {
    const auto& source = string(values[0]);
    std::string result;
    for (std::size_t offset = 0;;) {
      const std::string_view remaining(source.data() + offset, source.size() - offset);
      bool matched = false;
      for (std::size_t index = 1; index < values.size(); index += 2) {
        const auto& needle = string(values[index]);
        if (!remaining.starts_with(needle)) continue;
        result += string(values[index + 1]);
        if (!needle.empty()) {
          offset += needle.size();
        } else if (remaining.empty()) {
          return result;
        } else {
          const std::size_t width = utf8_scalar_width(static_cast<std::uint8_t>(source[offset]));
          result.append(source, offset, width);
          offset += width;
        }
        matched = true;
        break;
      }
      if (matched) continue;
      if (remaining.empty()) return result;
      const std::size_t width = utf8_scalar_width(static_cast<std::uint8_t>(source[offset]));
      result.append(source, offset, width);
      offset += width;
    }
  }

  static std::string truncate_utf8_bytes(const std::string& source, double budget) {
    for (std::size_t offset = 0; offset < source.size();) {
      const std::size_t end = offset + utf8_scalar_width(static_cast<std::uint8_t>(source[offset]));
      const double consumed = static_cast<double>(end);
      if (consumed == budget) return source.substr(0, end);
      if (consumed > budget) return source.substr(0, offset);
      offset = end;
    }
    return source;
  }

  static std::vector<std::string_view> scalars(std::string_view value) {
    std::vector<std::string_view> result;
    for (std::size_t offset = 0; offset < value.size();) {
      const std::size_t width = utf8_scalar_width(static_cast<std::uint8_t>(value[offset]));
      result.push_back(value.substr(offset, width)); offset += width;
    }
    return result;
  }

  static std::string trim_scalars(const std::string& source, const std::string& characters, bool start) {
    const auto allowed = scalars(characters);
    auto permitted = [&](std::string_view scalar) {
      return std::find(allowed.begin(), allowed.end(), scalar) != allowed.end();
    };
    const auto units = scalars(source);
    std::size_t first = 0, last = units.size();
    if (start) while (first < last && permitted(units[first])) ++first;
    else while (last > first && permitted(units[last - 1])) --last;
    std::string result;
    for (std::size_t index = first; index < last; ++index) result += units[index];
    return result;
  }

  any_result sequence(const any_list& expressions, const environment& values, const any& self) {
    any_list result;
    for (const auto& raw : expressions) {
      auto item = expression(object(raw), values, self);
      if (!item.ok) return item;
      result.push_back(item.value);
    }
    return succeed(result);
  }

  any_result make_aggregate(const json_map& data, std::optional<std::int64_t> variant_id,
                            bool expressions, const environment& values, const any& self) {
    const std::int64_t declaration_id = number(member(data, "declaration"));
    const auto& declaration = object(member(declarations_.at(declaration_id), "data"));
    const auto* variant = variant_id.has_value() ? find_variant(declaration, *variant_id) : nullptr;
    const auto& members = list(member(variant == nullptr ? declaration : *variant, "fields"));
    aggregate result{declaration_id, variant == nullptr ? "" : string(member(object(member(*variant, "header")), "name")), {}};
    for (const auto& raw : list(member(data, "fields"))) {
      const auto& field_value = object(raw);
      any_result item = expressions
          ? expression(object(member(field_value, "value")), values, self)
          : succeed(value(object(member(field_value, "value"))));
      if (!item.ok) return item;
      result.fields.emplace(member_name(members, number(member(field_value, "field"))), item.value);
    }
    return succeed(result);
  }

  any_result constant(std::int64_t identifier) {
    const auto cached = constants_.find(identifier);
    if (cached != constants_.end()) return cached->second;
    const auto found = declarations_.find(identifier);
    if (found == declarations_.end() || string(member(found->second, "kind")) != "constant") {
      return fail("invalid_constant", "unknown constant " + std::to_string(identifier));
    }
    auto result = constant_expression(object(member(object(member(found->second, "data")), "value")));
    constants_.emplace(identifier, result);
    return result;
  }

  any_result constant_expression(const json_map& expression_value) {
    const std::string kind = string(member(expression_value, "kind"));
    const auto& data = object(member(expression_value, "data"));
    if (kind == "literal") return succeed(value(object(member(data, "value"))));
    if (kind == "reference") return constant(number(member(data, "declaration")));
    if (kind == "none") return succeed(option_value{false, {}});
    if (kind == "some" || kind == "ok" || kind == "err") {
      auto item = constant_expression(object(member(data, "value")));
      if (!item.ok) return item;
      if (kind == "some") return succeed(option_value{true, item.value});
      return succeed(kind == "ok" ? any(value_result{true, item.value, {}})
                                    : any(value_result{false, {}, item.value}));
    }
    if (kind == "list") {
      any_list result;
      for (const auto& raw : list(member(data, "elements"))) {
        auto item = constant_expression(object(raw));
        if (!item.ok) return item;
        result.push_back(item.value);
      }
      return succeed(result);
    }
    if (kind == "record" || kind == "enum") {
      const std::int64_t declaration_id = number(member(data, "declaration"));
      const auto& declaration = object(member(declarations_.at(declaration_id), "data"));
      const auto* variant = kind == "enum" ? find_variant(declaration, number(member(data, "variant"))) : nullptr;
      const auto& members = list(member(variant == nullptr ? declaration : *variant, "fields"));
      aggregate result{declaration_id, variant == nullptr ? "" : string(member(object(member(*variant, "header")), "name")), {}};
      for (const auto& raw : list(member(data, "fields"))) {
        const auto& field_value = object(raw);
        auto item = constant_expression(object(member(field_value, "value")));
        if (!item.ok) return item;
        result.fields.emplace(member_name(members, number(member(field_value, "field"))), item.value);
      }
      return succeed(result);
    }
    if (kind == "intrinsic") {
      any_list arguments;
      for (const auto& raw : list(member(data, "arguments"))) {
        auto item = constant_expression(object(raw));
        if (!item.ok) return item;
        arguments.push_back(item.value);
      }
      return intrinsic(string(member(data, "operation")), arguments);
    }
    return fail("invalid_constant", "unknown constant expression " + kind);
  }

  any field(const any& value, const std::string& name) const {
    return std::any_cast<const aggregate&>(value).fields.at(name);
  }

  static std::string member_name(const any_list& members, std::int64_t identifier) {
    for (const auto& raw : members) {
      const auto& header = object(member(object(raw), "header"));
      if (node_id_from_header(header) == identifier) return string(member(header, "name"));
    }
    return "field_" + std::to_string(identifier);
  }

  std::string field_name(std::int64_t identifier) const {
    for (const auto& [unused, declaration] : declarations_) {
      static_cast<void>(unused);
      const auto& data = object(member(declaration, "data"));
      for (const auto& raw : nullable_list(data, "fields")) {
        const auto& header = object(member(object(raw), "header"));
        if (node_id_from_header(header) == identifier) return string(member(header, "name"));
      }
      for (const auto& variant_raw : nullable_list(data, "variants")) {
        for (const auto& field_raw : nullable_list(object(variant_raw), "fields")) {
          const auto& header = object(member(object(field_raw), "header"));
          if (node_id_from_header(header) == identifier) return string(member(header, "name"));
        }
      }
    }
    return "field_" + std::to_string(identifier);
  }

  std::int64_t find_implementation(std::int64_t contract, std::int64_t record) const {
    for (const auto& [identifier, declaration] : declarations_) {
      if (string(member(declaration, "kind")) != "implementation") continue;
      const auto& data = object(member(declaration, "data"));
      if (number(member(data, "contract")) == contract && number(member(data, "record")) == record) return identifier;
    }
    return -1;
  }

  static const json_map* find_variant(const json_map& declaration, std::int64_t variant_id) {
    for (const auto& raw : nullable_list(declaration, "variants")) {
      const auto& variant = object(raw);
      if (node_id_from_header(object(member(variant, "header"))) == variant_id) return &variant;
    }
    return nullptr;
  }

  json_map document_;
  std::map<std::int64_t, json_map> declarations_;
  std::map<std::int64_t, any_result> constants_;
};
}  // namespace poly_runtime
// POLYRUST-END runtime.engine
