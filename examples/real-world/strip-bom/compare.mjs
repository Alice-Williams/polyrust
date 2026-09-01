import upstream from "./upstream.mjs";
import {strip_bom as generated} from "./generated/typescript/dist/index.js";

const corpus = new Set([
  "",
  "\uFEFF",
  "\uFEFFUnicorn\n",
  "Unicorn \uFEFFUnicorn\n",
  "\uFEFF\uFEFF",
  "🦄\uFEFF",
]);
const alphabet = ["\uFEFF", "a", "🦄", "\u0301", "\0", "\n"];
let frontier = [""];
for (let length = 1; length <= 6; length += 1) {
  const next = [];
  for (const prefix of frontier) {
    for (const scalar of alphabet) {
      const value = prefix + scalar;
      corpus.add(value);
      next.push(value);
    }
  }
  frontier = next;
}
corpus.add("\uFEFF".repeat(90_000));
corpus.add("a" + "\uFEFF".repeat(90_000));

let checked = 0;
for (const input of corpus) {
  const expected = upstream(input);
  const result = generated(input);
  if (!result.ok || result.value !== expected) {
    throw new Error(
      `mismatch for ${JSON.stringify(input)}: upstream=${JSON.stringify(expected)} generated=${JSON.stringify(result)}`,
    );
  }
  checked += 1;
}

console.log(`differential oracle passed ${checked} unique strings`);
