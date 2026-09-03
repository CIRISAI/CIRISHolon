// The entry points and the bridge ABI, checked on the FINISHED artifact.
const fs = require("fs");
const need = [
  "holon3d_run_owned", "holon3d_run_hosted",
  "holon3d_frame_begin", "holon3d_frame_atom", "holon3d_frame_bond", "holon3d_frame_commit",
  "holon3d_hand_len", "holon3d_hand_kind", "holon3d_hand_arg", "holon3d_hand_clear",
  "holon3d_dropped_frames", "holon3d_refused_frames",
];
const m = new WebAssembly.Module(fs.readFileSync(process.argv[2]));
const have = new Set(WebAssembly.Module.exports(m).map((e) => e.name));
const missing = need.filter((n) => !have.has(n));
if (missing.length) {
  console.error("  ARTIFACT IS MISSING ENTRY POINTS: " + missing.join(", "));
  console.error("  The page calls these by name. A module without them loads and never draws.");
  process.exit(1);
}
console.log("  entry points present: " + need.length + " (page-callable)");
