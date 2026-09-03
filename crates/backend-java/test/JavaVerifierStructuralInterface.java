sealed interface JavaVerifierStructuralInterface permits JavaVerifierStructuralRecord {
  int hidden();
}

record JavaVerifierStructuralRecord() implements JavaVerifierStructuralInterface {}
