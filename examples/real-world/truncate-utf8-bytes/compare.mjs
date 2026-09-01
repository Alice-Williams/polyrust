import {readFileSync} from "node:fs";
import {createRequire} from "node:module";
import {truncate as generatedTruncate} from "./generated/typescript/dist/index.js";

const require = createRequire(import.meta.url);
const upstreamTruncate = require("./upstream/index.js");
const naughtyStrings = JSON.parse(
  readFileSync(new URL("./upstream/blns.json", import.meta.url), "utf8"),
);

const corpus = new Set([
  "",
  "a☃",
  "a🦀e\u0301\0z",
  "中é𐀀",
  "a".repeat(300),
  "a".repeat(252) + "𐀀",
  ...naughtyStrings,
]);

let checked = 0;
for (const input of corpus) {
  const byteLength = Buffer.byteLength(input);
  const budgets = [];
  for (let budget = 0; budget <= byteLength; budget += 1) budgets.push(budget);
  budgets.push(-Infinity, -1, -0, 0.5, 1.5, byteLength - 0.5, byteLength + 0.5, Infinity, NaN);
  for (const budget of budgets) {
    const expected = upstreamTruncate(input, budget);
    const generated = generatedTruncate(input, budget);
    if (!generated.ok || generated.value !== expected) {
      throw new Error(
        `mismatch for input=${JSON.stringify(input)} budget=${String(budget)}: upstream=${JSON.stringify(expected)} generated=${JSON.stringify(generated)}`,
      );
    }
    if (!input.startsWith(generated.value)) {
      throw new Error(`output is not a prefix for ${JSON.stringify(input)} at ${String(budget)}`);
    }
    if (Number.isFinite(budget) && budget >= 0 && Buffer.byteLength(generated.value) > budget) {
      throw new Error(`output exceeds budget for ${JSON.stringify(input)} at ${String(budget)}`);
    }
    checked += 1;
  }
}

for (const input of ["🦀".repeat(20_000), "a☃🦀é".repeat(12_000)]) {
  for (const budget of [0, 1, 3, 4, 5, 4095, 4096, 4097, Buffer.byteLength(input), Infinity, NaN]) {
    const expected = upstreamTruncate(input, budget);
    const generated = generatedTruncate(input, budget);
    if (!generated.ok || generated.value !== expected) {
      throw new Error(`large-input mismatch at budget ${String(budget)}`);
    }
    checked += 1;
  }
}

console.log(`differential oracle passed ${checked} input/budget comparisons over ${corpus.size + 2} strings`);
