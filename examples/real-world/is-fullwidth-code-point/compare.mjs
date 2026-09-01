import upstreamIsFullwidthCodePoint from "./upstream/index.js";
import {is_fullwidth_code_point as generatedIsFullwidthCodePoint} from "./generated/typescript/dist/index.js";

function assertEquivalent(input) {
  const upstream = upstreamIsFullwidthCodePoint(input);
  const generated = generatedIsFullwidthCodePoint(input);
  if (!generated.ok) throw new Error(`generated classifier failed for ${String(input)}`);
  if (generated.value !== upstream) {
    throw new Error(
      `mismatch input=${String(input)}: upstream=${String(upstream)} generated=${String(generated.value)}`,
    );
  }
}

const intervals = [
  [0x1100, 0x115F],
  [0x2329, 0x232A],
  [0x2E80, 0x3247],
  [0x3250, 0x4DBF],
  [0x4E00, 0xA4C6],
  [0xA960, 0xA97C],
  [0xAC00, 0xD7A3],
  [0xF900, 0xFAFF],
  [0xFE10, 0xFE19],
  [0xFE30, 0xFE6B],
  [0xFF01, 0xFF60],
  [0xFFE0, 0xFFE6],
  [0x1B000, 0x1B001],
  [0x1F200, 0x1F251],
  [0x20000, 0x3FFFD],
];
const corpus = [
  0,
  -0,
  NaN,
  Number.NaN,
  Infinity,
  -Infinity,
  Number.MIN_VALUE,
  -Number.MIN_VALUE,
  Number.MAX_VALUE,
  -Number.MAX_VALUE,
  0x303F,
  0x303F + 0.5,
  0x10FFFF,
  0x110000,
];

for (const [start, end] of intervals) {
  for (const boundary of [start, end]) {
    for (const delta of [-2, -1, -0.5, -0.25, 0, 0.25, 0.5, 1, 2]) {
      corpus.push(boundary + delta);
    }
  }
}
for (const boundary of [0x303E, 0x303F, 0x3040]) {
  for (const delta of [-1, -0.5, -0.25, 0, 0.25, 0.5, 1]) corpus.push(boundary + delta);
}
for (let value = -0x1000; value <= 0x41000; value += 257) {
  corpus.push(value, value + 0.5);
}

let state = 0x6d2b79f5;
for (let index = 0; index < 20_000; index += 1) {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  const unit = (state >>> 0) / 0x1_0000_0000;
  if (index % 3 === 0) {
    corpus.push(Math.floor(unit * 0x50000) - 0x8000);
  } else if (index % 3 === 1) {
    corpus.push(unit * 0x50000 - 0x8000);
  } else {
    corpus.push((unit * 2 - 1) * 10 ** ((index % 25) - 5));
  }
}

for (const input of corpus) assertEquivalent(input);
console.log(`is-fullwidth-code-point differential oracle passed ${corpus.length} inputs`);
