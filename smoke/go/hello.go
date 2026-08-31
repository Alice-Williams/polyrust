package smoke

import "fmt"

// TargetMessage confirms that the pinned Bazel Go toolchain can build library code.
func TargetMessage(target string) string {
	return fmt.Sprintf("portable code generation target: %s", target)
}
