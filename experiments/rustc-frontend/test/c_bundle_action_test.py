"""Inspect actual provider-driven Bazel tree artifacts, not handwritten output."""
from pathlib import Path
import sys
import c_bundle_assertions

root, aliases = [Path(value).resolve() for value in sys.argv[1:]]
c_bundle_assertions.check(root, 3, 1)
c_bundle_assertions.check(aliases, 2, 1)
print("Bazel bundles: declared transitive graph and repeated aliases generated exact owning files")
