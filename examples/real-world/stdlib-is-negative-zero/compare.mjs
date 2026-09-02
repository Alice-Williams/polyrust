import { readFileSync } from "node:fs";
import { is_negative_zero as generatedIsNegativeZero } from "./generated/typescript/dist/index.js";

function loadCommonJs(source, requireDependency) {
  const module = { exports: {} };
  const evaluate = new Function("module", "exports", "require", source);
  evaluate(module, module.exports, requireDependency);
  return module.exports;
}

const mainSource = readFileSync(
  new URL("./upstream/main.js", import.meta.url),
  "utf8",
);
const upstreamMain = loadCommonJs(mainSource, (identifier) => {
  if (identifier === "@stdlib/constants-float64-ninf") return -Infinity;
  throw new Error(`unexpected main.js dependency: ${identifier}`);
});
const indexSource = readFileSync(
  new URL("./upstream/index.js", import.meta.url),
  "utf8",
);
const upstreamIsNegativeZero = loadCommonJs(indexSource, (identifier) => {
  if (identifier === "./main.js") return upstreamMain;
  throw new Error(`unexpected index.js dependency: ${identifier}`);
});

const SIGN = 1n << 63n;
const MANTISSA_MASK = (1n << 52n) - 1n;
const U64_MASK = (1n << 64n) - 1n;
const corpus = new Map();

function hex(bits) {
  return bits.toString(16).padStart(16, "0");
}

function add(bits) {
  const normalized = bits & U64_MASK;
  corpus.set(hex(normalized), normalized);
}

for (const bits of [
  0n,
  SIGN,
  1n,
  SIGN | 1n,
  MANTISSA_MASK,
  SIGN | MANTISSA_MASK,
  0x0010000000000000n,
  0x8010000000000000n,
  0x3ff0000000000000n,
  0xbff0000000000000n,
  0x7fefffffffffffffn,
  0xffefffffffffffffn,
  0x7ff0000000000000n,
  0xfff0000000000000n,
  0x7ff0000000000001n,
  0xfff0000000000001n,
  0x7ff8000000000001n,
  0xfff8000000000001n,
  0x7fffffffffffffffn,
  0xffffffffffffffffn,
]) {
  add(bits);
}

const mantissas = [0n, 1n, 1n << 51n, MANTISSA_MASK - 1n, MANTISSA_MASK];
for (let exponent = 0n; exponent <= 0x7ffn; exponent += 1n) {
  for (const sign of [0n, SIGN]) {
    for (const mantissa of mantissas) add(sign | (exponent << 52n) | mantissa);
  }
}

let state = 0x4d595df4d0f33173n;
for (let index = 0; index < 65_536; index += 1) {
  state = (state * 6364136223846793005n + 1442695040888963407n) & U64_MASK;
  add(state);
}

function fromBits(bits) {
  const bytes = new ArrayBuffer(8);
  const view = new DataView(bytes);
  view.setBigUint64(0, bits, false);
  return view.getFloat64(0, false);
}

for (const [inputHex, bits] of corpus) {
  const input = fromBits(bits);
  const upstream = upstreamIsNegativeZero(input);
  const generated = generatedIsNegativeZero(input);
  if (!generated.ok) {
    throw new Error(`generated predicate failed for bits=${inputHex}`);
  }
  if (generated.value !== upstream) {
    throw new Error(
      `mismatch bits=${inputHex}: upstream=${String(upstream)} generated=${String(generated.value)}`,
    );
  }
}

console.log(
  `stdlib is-negative-zero differential oracle passed ${corpus.size} exact-bit inputs`,
);
