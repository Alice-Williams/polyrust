# strip-bom upstream oracle

- Repository: https://github.com/sindresorhus/strip-bom
- Revision: `b80d7bc94e79b4744d92a2dc6328c91d9afe9775`
- Version: 5.0.0
- License: MIT; see `LICENSE`
- `upstream.js`, `upstream.d.ts`, and `upstream.test.js` correspond
  to upstream `index.js`, `index.d.ts`, and `test/test.js`
- The two official fixtures are represented byte-for-byte as portable vectors:
  UTF-8 BOM + `Unicorn\n` and `Unicorn ` + UTF-8 BOM + `Unicorn\n`

Generated PolyRust packages do not contain or depend on these artifacts.
