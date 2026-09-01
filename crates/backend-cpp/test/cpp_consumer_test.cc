#include "generated.hpp"

#include <string>

int main() {
  const polyrust_generated::Label label{"external"};
  const auto result = polyrust_generated::call_render(label);
  return result.ok && result.value == std::optional<std::string>{"external"}
             && !result.error.has_value()
         ? 0
         : 1;
}
