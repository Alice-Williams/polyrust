import upstream from "./upstream.mjs";
import {slash as generated} from "./generated/typescript/dist/index.js";

const corpus = new Set([
  "",
  "c:/aaaa\\bbbb",
  "c:\\aaaa\\bbbb",
  "\\\\?\\c:\\aaaa\\bbbb",
  "\\\\server\\share",
  "🦄\\🐐",
]);
const alphabet = ["\\", "/", "?", "c", ":", "★"];
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
corpus.add("\\".repeat(90_000));
corpus.add("\\\\?\\" + "\\".repeat(90_000));

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

console.log(`differential oracle passed ${checked} unique paths`);
