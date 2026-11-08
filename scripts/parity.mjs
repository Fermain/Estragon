#!/usr/bin/env node
// Compare Godot's layout of a document with the shared Rust core's layout.
// Usage: node scripts/parity.mjs [schema/fixtures/hud.ui.json]
import { execFileSync } from "node:child_process";
import { basename } from "node:path";
import { existsSync } from "node:fs";

const doc = process.argv[2] ?? "schema/fixtures/hud.ui.json";
const godotBin = process.env.GODOT ?? "godot";
const run = (cmd, args) => execFileSync(cmd, args, { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
const json = (s) => JSON.parse(s.slice(s.indexOf("["), s.lastIndexOf("]") + 1));

const core = json(run("cargo", ["run", "-q", "-p", "estragon-layout", "--example", "dump", "--", doc]));
if (!existsSync("godot/.godot/extension_list.cfg")) {
  // First run: let Godot register the extension. Godot 4.7 may crash on exit after
  // a headless import; the import itself still completes, so ignore the exit code.
  try { execFileSync(godotBin, ["--headless", "--path", "godot", "--import"], { stdio: "ignore" }); } catch {}
}
const godot = json(run(godotBin, ["--headless", "--path", "godot", "-s", "res://tools/dump_layout.gd", "--", `res://fixtures/${basename(doc)}`]));

const EPS = 0.5;
let failures = 0;
for (const c of core) {
  const g = godot.find((n) => n.id === c.id);
  const fields = ["x", "y", "width", "height"];
  const bad = !g || fields.some((f) => Math.abs(g[f] - c[f]) > EPS);
  if (bad) failures++;
  const fmt = (n) => (n ? fields.map((f) => n[f].toFixed(1).padStart(7)).join(" ") : "   (missing in godot)");
  console.log(`${bad ? "✗" : "✓"} ${"  ".repeat(c.depth)}${c.id.padEnd(14 - 2 * c.depth)} core ${fmt(c)} | godot ${fmt(g)}`);
}
console.log(failures ? `\n${failures} of ${core.length} nodes differ` : `\nAll ${core.length} nodes match (±${EPS}px)`);
process.exit(failures ? 1 : 0);
