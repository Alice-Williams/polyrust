import upstreamNormalizeNewline from "./upstream/index.js";
import {
  normalize_newline as generatedText,
  normalize_newline_bytes as generatedBytes,
} from "./generated/typescript/dist/index.js";

function assertText(input) {
  const expected = upstreamNormalizeNewline(input);
  const result = generatedText(input);
  if (!result.ok || result.value !== expected) {
    throw new Error(
      `text mismatch input=${JSON.stringify(input)} upstream=${JSON.stringify(expected)} generated=${JSON.stringify(result)}`,
    );
  }
}

function assertBytes(input) {
  const expected = upstreamNormalizeNewline(new Uint8Array(input));
  const result = generatedBytes(input);
  const actual = result.ok ? Array.from(result.value) : result;
  if (
    !result.ok ||
    actual.length !== expected.length ||
    actual.some((value, index) => value !== expected[index])
  ) {
    throw new Error(
      `bytes mismatch input=${JSON.stringify(input)} upstream=${JSON.stringify(Array.from(expected))} generated=${JSON.stringify(actual)}`,
    );
  }
}

const textCorpus = new Set([
  "",
  "foo\r\nbar\r\nbaz",
  "foo\nbar\nbaz\r\n",
  "foo\nbar\n",
  "no crlf here",
  "\r\n\r\n",
  "lone\rcarriage\nreturn",
]);
const textAlphabet = ["\r", "\n", "x", "\0", " ", "🦀"];
let textFrontier = [""];
for (let length = 1; length <= 5; length += 1) {
  const next = [];
  for (const prefix of textFrontier) {
    for (const scalar of textAlphabet) {
      const value = prefix + scalar;
      textCorpus.add(value);
      next.push(value);
    }
  }
  textFrontier = next;
}
textCorpus.add("\r\n".repeat(45_000));
textCorpus.add("🦀\r\n".repeat(20_000));

const byteCorpus = [];
const byteAlphabet = [0, 10, 13, 255];
let byteFrontier = [[]];
byteCorpus.push([]);
for (let length = 1; length <= 7; length += 1) {
  const next = [];
  for (const prefix of byteFrontier) {
    for (const byte of byteAlphabet) {
      const value = [...prefix, byte];
      byteCorpus.push(value);
      next.push(value);
    }
  }
  byteFrontier = next;
}

let state = 0x9e3779b9;
function randomUnit() {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}
for (let index = 0; index < 10_000; index += 1) {
  const length = randomUnit() % 129;
  const value = [];
  for (let offset = 0; offset < length; offset += 1)
    value.push(randomUnit() & 0xff);
  if (index % 4 === 0 && value.length >= 2) {
    const offset = randomUnit() % (value.length - 1);
    value[offset] = 13;
    value[offset + 1] = 10;
  }
  byteCorpus.push(value);
}
byteCorpus.push(
  Array.from({ length: 90_000 }, (_, index) => (index % 2 === 0 ? 13 : 10)),
);
byteCorpus.push(Array.from({ length: 65_536 }, (_, index) => index & 0xff));

for (const input of textCorpus) assertText(input);
for (const input of byteCorpus) assertBytes(input);
console.log(
  `normalize-newline differential oracle passed ${textCorpus.size} text inputs and ${byteCorpus.length} byte inputs`,
);
