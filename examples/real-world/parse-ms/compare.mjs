import upstreamParseMilliseconds from "./upstream/index.js";
import {parse_milliseconds as generatedParseMilliseconds} from "./generated/typescript/dist/index.js";

const fields = [
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
];

function numbersEqual(left, right) {
  return (Number.isNaN(left) && Number.isNaN(right)) || Object.is(left, right);
}

function assertEquivalent(input) {
  const upstream = upstreamParseMilliseconds(input);
  const generated = generatedParseMilliseconds(input);
  if (!generated.ok) throw new Error(`generated parse failed for ${String(input)}`);
  for (const field of fields) {
    if (!numbersEqual(generated.value[field], upstream[field])) {
      throw new Error(
        `mismatch input=${String(input)} field=${field}: upstream=${String(upstream[field])} generated=${String(generated.value[field])}`,
      );
    }
  }
}

const corpus = [
  1400,
  55_000,
  67_000,
  300_000,
  4_020_000,
  43_200_000,
  144_000_000,
  3_596_400_000,
  60_500.345678,
  0.000543,
  0,
  -0,
  NaN,
  Infinity,
  -Infinity,
  Number.MIN_VALUE,
  -Number.MIN_VALUE,
  Number.MAX_VALUE,
  -Number.MAX_VALUE,
];

for (const value of [...corpus]) if (Number.isFinite(value)) corpus.push(-value);
for (const boundary of [1, 1000, 60_000, 3_600_000, 86_400_000]) {
  for (const delta of [-1, -0.5, -Number.EPSILON, 0, Number.EPSILON, 0.5, 1]) {
    corpus.push(boundary + delta, -boundary - delta);
  }
}

let state = 0x6d2b79f5;
for (let index = 0; index < 10_000; index += 1) {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  const unit = (state >>> 0) / 0x1_0000_0000;
  const magnitude = 10 ** ((index % 27) - 9);
  corpus.push((unit * 2 - 1) * magnitude);
}

for (const input of corpus) assertEquivalent(input);
console.log(
  `parse-ms differential oracle passed ${corpus.length} input records (${corpus.length * fields.length} exact components)`,
);
