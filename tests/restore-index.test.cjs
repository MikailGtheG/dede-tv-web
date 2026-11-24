"use strict";
// Standalone regression test for persisted channel restoration; run with node.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const source = fs.readFileSync("prototype/app.js", "utf8");
const match = source.match(/function restoreIndex\(\) \{[\s\S]*?\n\}/);
assert.ok(match, "restoreIndex function must exist");
for (const [stored, expected] of [
  [null, 2], ["0", 0], ["5", 5], ["2", 2],
  ["2junk", 2], ["2.0", 2], ["-1", 2], [" 1", 2],
  ["6", 2], ["999999999999999999999", 2], ["", 2]
]) {
  const context = {
    channels: Array(6),
    localStorage: { getItem: () => stored }
  };
  vm.createContext(context);
  const actual = vm.runInContext(match[0] + "\nrestoreIndex()", context);
  assert.equal(actual, expected, `stored ${JSON.stringify(stored)}`);
}
const brokenStorage = { channels: Array(6), localStorage: { getItem() { throw Error("denied"); } } };
vm.createContext(brokenStorage);
assert.equal(vm.runInContext(match[0] + "\nrestoreIndex()", brokenStorage), 2);
console.log("restoreIndex regression cases passed");
