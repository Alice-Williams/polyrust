interface JavaVerifierCloneReturn {
  int clone();
}

final class JavaVerifierCloneImplementation implements JavaVerifierCloneReturn {
  @Override
  public int clone() {
    return 1;
  }
}
