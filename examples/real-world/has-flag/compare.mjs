import upstreamHasFlag from "./upstream/index.js";
import {has_flag as generatedHasFlag} from "./generated/typescript/dist/index.js";

let comparisons = 0;

function assertEquivalent(flag, argv) {
  const upstream = upstreamHasFlag(flag, argv);
  const generated = generatedHasFlag(flag, argv);
  if (!generated.ok) throw new Error(`generated has_flag failed for ${JSON.stringify({flag, argv})}`);
  if (generated.value !== upstream) {
    throw new Error(
      `mismatch ${JSON.stringify({flag, argv})}: upstream=${upstream} generated=${generated.value}`,
    );
  }
  comparisons += 1;
}

const flags = [
  "", "-", "--", "a", "ab", "-a", "--alpha", "x=y", "é", "é", "🦀", "🦀x", "\0",
];

let state = 0x9e3779b9;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}

const scalar = () => {
  let value = random() % 0x110000;
  if (value >= 0xD800 && value <= 0xDFFF) value += 0x800;
  if (value > 0x10FFFF) value = 0x10FFFF;
  return String.fromCodePoint(value);
};

for (let index = 0; index < 2000; index += 1) {
  let value = "";
  const length = random() % 5;
  for (let unit = 0; unit < length; unit += 1) value += scalar();
  if (index % 5 === 0) value = `-${value}`;
  if (index % 11 === 0) value += "=value";
  flags.push(value);
}

for (const flag of flags) {
  const candidates = [flag, `-${flag}`, `--${flag}`];
  for (const candidate of candidates) {
    for (const argv of [
      [],
      [candidate],
      ["--", candidate],
      [candidate, "--"],
      ["noise", candidate, "tail"],
      ["noise", "--", candidate, candidate],
      [candidate, candidate, "--"],
    ]) {
      assertEquivalent(flag, argv);
    }
  }
}

console.log(`has-flag differential oracle passed ${comparisons} admitted comparisons`);
