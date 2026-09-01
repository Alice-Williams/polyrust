# Pinned truncate-utf8-bytes fixture

- Upstream: `parshap/truncate-utf8-bytes`
- Package version: `1.0.2`
- Implementation/license revision: `4212839ea184e74fb81f1e4e633e1db794ebe4f4`
- Type declaration revision: `451dc8fc19383bc12af59522020e571957f1684e`
- Naughty-string submodule revision: `5f5a11b34b86f811e9888e32f3053d8cb1466325`
- License: MIT (dual-licensed upstream)

The implementation revision is the published 1.0.2 code plus the upstream
commit that explicitly added the MIT option; the comparison from the published
revision changes only license files and the package license expression.

`index.js`, `lib/truncate.js`, `upstream.test.js`, `package.json`, and
`LICENSE.MIT.txt` are exact copies from the implementation revision.
`upstream.d.ts` is the exact DefinitelyTyped declaration. `blns.json` is the
exact corpus at the submodule revision pinned by upstream. Bazel tests execute
the retained CommonJS entry point directly and never fetch the network.
