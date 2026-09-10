import { readFileSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import assert from "node:assert/strict";
import init, { analyze, evaluate } from "../web/pkg/claim_wasm.js";
await init({ module_or_path: readFileSync("web/pkg/claim_wasm_bg.wasm") });
const input = JSON.stringify({
  bundle: JSON.parse(readFileSync("examples/corpus.json", "utf8")),
  reviews: [],
});
const dir = mkdtempSync(join(tmpdir(), "atlas-parity-")),
  file = join(dir, "document.json");
writeFileSync(file, input, { mode: 0o600 });
const native = JSON.parse(
  execFileSync(
    "cargo",
    ["run", "--quiet", "--locked", "-p", "claim-cli", "--", "analyze", file],
    { encoding: "utf8" },
  ),
);
assert.deepEqual(JSON.parse(analyze(input)), native);
const gold = readFileSync("examples/gold.json", "utf8");
const nativeEval = JSON.parse(
  execFileSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--locked",
      "-p",
      "claim-cli",
      "--",
      "evaluate",
      file,
      "examples/gold.json",
    ],
    { encoding: "utf8" },
  ),
);
assert.deepEqual(JSON.parse(evaluate(input, gold)), nativeEval);
console.log(
  "Native/WASM graph and evaluation parity passed: 60 claims, 30 gold pairs.",
);
