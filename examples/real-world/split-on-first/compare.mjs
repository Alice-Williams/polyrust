import upstreamSplitOnFirst from "./upstream/index.js";
import {split_on_first as generatedSplitOnFirst} from "./generated/typescript/dist/index.js";

let comparisons = 0;

function assertEquivalent(input, separator) {
  const upstream = upstreamSplitOnFirst(input, separator);
  const generated = generatedSplitOnFirst(input, separator);
  if (!generated.ok) {
    throw new Error(`generated split_on_first failed for ${JSON.stringify({input, separator})}`);
  }
  if (
    generated.value.length !== upstream.length
    || generated.value.some((value, index) => value !== upstream[index])
  ) {
    throw new Error(
      `mismatch ${JSON.stringify({input, separator})}: upstream=${JSON.stringify(upstream)} generated=${JSON.stringify(generated.value)}`,
    );
  }
  comparisons += 1;
}

const sources = new Set([
  "", "a", "abc", "a-b-c", "key:value:value2", "aaaa", "a::b::c",
  " ", "\t", "\r\n", "a\0b\0c", "é", "é", "🦀", "🚀", "🦀🚀",
  "a🦀b🦀c", "a🦀🚀b🦀🚀c", "Hello-World", "key=value=tail",
]);
const fixedSeparators = [
  "", "a", "b", "-", "--", ":", "::", "/", "=", "+", " ", "\t", "\r", "\n",
  "\r\n", "\0", "é", "é", "́", "🦀", "🚀", "🦀🚀", "missing", "abc", "aaaa",
];

let state = 0x6a09e667;
function random() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}

function randomScalar() {
  const frequent = ["a", "b", "-", ":", "\0", "\r", "\n", "é", "́", "🦀", "🚀"];
  if ((random() & 3) !== 0) return frequent[random() % frequent.length];
  let value = random() % 0x110000;
  if (value >= 0xD800 && value <= 0xDFFF) value += 0x800;
  if (value > 0x10FFFF) value = 0x10FFFF;
  return String.fromCodePoint(value);
}

for (let index = 0; index < 2500; index += 1) {
  let value = "";
  const length = random() % 9;
  for (let scalar = 0; scalar < length; scalar += 1) value += randomScalar();
  sources.add(value);
}

for (const input of sources) {
  const scalars = Array.from(input);
  const separators = new Set(fixedSeparators);
  separators.add(input);
  separators.add(input + "x");
  if (scalars.length > 0) {
    separators.add(scalars[0]);
    separators.add(scalars.at(-1));
    separators.add(scalars.slice(0, 2).join(""));
    separators.add(scalars.slice(-2).join(""));
  }
  if (scalars.length > 2) {
    const start = random() % scalars.length;
    const end = start + 1 + (random() % (scalars.length - start));
    separators.add(scalars.slice(start, end).join(""));
  }
  for (const separator of separators) assertEquivalent(input, separator);
}

console.log(`split-on-first differential oracle passed ${comparisons} admitted comparisons`);
