package smoke

import "testing"

func TestTargetMessage(t *testing.T) {
	got := TargetMessage("go")
	want := "portable code generation target: go"
	if got != want {
		t.Fatalf("TargetMessage() = %q, want %q", got, want)
	}
}
