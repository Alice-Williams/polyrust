// Execute the unchanged vendored numeric API, providing its sole constant dependency.
import {readFileSync} from "node:fs";
import {createInterface} from "node:readline";
const module = {exports: {}};
const source = readFileSync(process.argv[2], "utf8");
new Function("module", "exports", "require", source)(module, module.exports, (name) => {
  if (name === "@stdlib/constants-float64-ninf") return -Infinity;
  throw new Error("Unexpected upstream dependency: " + name);
});
const buffer = new ArrayBuffer(8);
const view = new DataView(buffer);
for await (const line of createInterface({input: process.stdin})) {
  view.setBigUint64(0, BigInt("0x" + line), false);
  console.log(Number(module.exports(view.getFloat64(0, false))));
}
