#include <cstdint>
#include <optional>

int main() {
  const std::optional<std::int32_t> value{42};
  return value == 42 ? 0 : 1;
}
