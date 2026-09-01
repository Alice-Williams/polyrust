import upstream from "./upstream.mjs";
import {escape_string_regexp as generated} from "./generated/typescript/dist/index.js";

const escaped = ["|", "\\", "{", "}", "(", ")", "[", "]", "^", "$", "+", "*", "?", ".", "-"];
const corpus = new Set([
  "",
  "\\ ^ $ * + ? . ( ) | { } [ ]",
  "foo - bar",
  "How much $ for a 🦄?",
  "e\u0301",
  "\0\n\r\t",
  "ordinary text",
]);

for (let scalar = 0; scalar < 128; scalar += 1) {
  corpus.add(String.fromCodePoint(scalar));
}
for (const first of escaped) {
  corpus.add(first.repeat(8));
  for (const second of escaped) {
    corpus.add(first + second);
    for (const third of escaped) {
      corpus.add(first + second + third);
    }
  }
}

let checked = 0;
for (const input of corpus) {
  const expected = upstream(input);
  const result = generated(input);
  if (!result.ok) {
    throw new Error(`generated function failed for ${JSON.stringify(input)}: ${JSON.stringify(result)}`);
  }
  if (result.value !== expected) {
    throw new Error(
      `mismatch for ${JSON.stringify(input)}: upstream=${JSON.stringify(expected)} generated=${JSON.stringify(result.value)}`,
    );
  }
  const exact = new RegExp(`^(?:${expected})$`, "u");
  if (!exact.test(input)) {
    throw new Error(`escaped result is not an exact Unicode regexp for ${JSON.stringify(input)}`);
  }
  checked += 1;
}

console.log(`differential oracle passed ${checked} unique inputs`);
