import {
  trimNewlines as upstreamBoth,
  trimNewlinesStart as upstreamStart,
  trimNewlinesEnd as upstreamEnd,
} from "./upstream.mjs";
import {
  trim_newlines as generatedBoth,
  trim_newlines_start as generatedStart,
  trim_newlines_end as generatedEnd,
} from "./generated/typescript/dist/index.js";

const corpus = new Set([""]);
const alphabet = ["\r", "\n", "x", " ", "\t", "🦄", "\u2028"];
let frontier = [""];
for (let length = 1; length <= 5; length += 1) {
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

const boundaries = [""];
frontier = [""];
for (let length = 1; length <= 6; length += 1) {
  const next = [];
  for (const prefix of frontier) {
    for (const newline of ["\r", "\n"]) {
      const value = prefix + newline;
      boundaries.push(value);
      next.push(value);
    }
  }
  frontier = next;
}
for (const left of boundaries) {
  for (const right of boundaries) {
    for (const body of ["", "x", "🦄", " x ", "\t", "\u2028"]) {
      corpus.add(left + body + right);
    }
  }
}
corpus.add("\r\n".repeat(45_000));
corpus.add("\r\n".repeat(45_000) + "x");
corpus.add("x" + "\r\n".repeat(45_000));

const functions = [
  ["both", upstreamBoth, generatedBoth],
  ["start", upstreamStart, generatedStart],
  ["end", upstreamEnd, generatedEnd],
];
let comparisons = 0;
for (const input of corpus) {
  for (const [name, upstream, generated] of functions) {
    const expected = upstream(input);
    const result = generated(input);
    if (!result.ok || result.value !== expected) {
      throw new Error(
        `${name} mismatch for ${JSON.stringify(input)}: upstream=${JSON.stringify(expected)} generated=${JSON.stringify(result)}`,
      );
    }
    comparisons += 1;
  }
}

console.log(`differential oracle passed ${corpus.size} inputs and ${comparisons} function comparisons`);
