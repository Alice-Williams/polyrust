import * as upstream from "./upstream.mjs";
import {escape as generatedEscape, unescape as generatedUnescape} from "./generated/typescript/dist/index.js";

const corpus = new Set([
  "",
  "&<>\'\"",
  "&amp;&lt;&gt;&#39;&quot;",
  "&amp;lt;",
  "&#38;lt;",
  "&copy;",
  "🦀\0e\u0301",
]);
const alphabet = ["&", "<", ">", "'", "\"", ";", "#", "38", "amp", "lt", "gt", "apos", "quot", "🦀", "\0"];
let frontier = [""];
for (let length = 1; length <= 4; length += 1) {
  const next = [];
  for (const prefix of frontier) {
    for (const token of alphabet) {
      const value = prefix + token;
      corpus.add(value);
      next.push(value);
    }
  }
  frontier = next;
}
corpus.add("<&>'\"".repeat(18_000));
corpus.add("&amp;&lt;&#60;&#38;".repeat(5_000));

let checked = 0;
for (const input of corpus) {
  for (const [name, expected, generated] of [
    ["escape", upstream.escape(input), generatedEscape(input)],
    ["unescape", upstream.unescape(input), generatedUnescape(input)],
  ]) {
    if (!generated.ok || generated.value !== expected) {
      throw new Error(
        `${name} mismatch for ${JSON.stringify(input)}: upstream=${JSON.stringify(expected)} generated=${JSON.stringify(generated)}`,
      );
    }
    checked += 1;
  }
}

console.log(`differential oracle passed ${checked} function/input pairs over ${corpus.size} unique strings`);
