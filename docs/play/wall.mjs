// THE WALL. `docs/play/` is a toy and the observatory is an instrument, and the whole value
// of the instrument is that its numbers trace. This gate fails the build if the toy ever
// starts to look like evidence, in either direction.
//
// It is deliberately blunt. A toy that cites a record is a toy whose screenshots can be
// mistaken for a reading, and a record that cites a toy is a record that has stopped being
// one. Neither is a thing to catch by review.
//
//   node docs/play/wall.mjs        exit 0 clean, 1 with the breach named
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
let bad = 0;
const ok = (s) => console.log(`  ok   ${s}`);
const no = (s) => { console.log(`  FAIL ${s}`); bad++; };

const files = readdirSync(here).filter((f) => /\.(js|mjs|html|css|json|md)$/.test(f));

// 1. THE TOY MAY NOT CITE THE RECORD. No path into conformance/, no record filename, no
//    campaign name used as a source. It may LINK to the workbench, which is a page, not a record.
for (const f of files) {
  if (f === "wall.mjs") continue;
  const t = readFileSync(join(here, f), "utf8");
  for (const pat of [/conformance\//, /_RESULTS\.md/, /_PREREG/, /_AMENDMENT/, /GANTT2?\.md/]) {
    if (pat.test(t)) no(`${f} cites the record (${pat}) — a toy may not point at evidence`);
  }
}
ok("no file in the toy cites conformance/, a results file, a prereg, an amendment or the plan");

// 2. THE RECORD MAY NOT CITE THE TOY. Nothing under conformance/ or the workbench page may
//    mention this directory — including as an example, which is how it would start.
const scan = (dir, depth = 0) => {
  if (depth > 3 || !existsSync(dir)) return;
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) { if (!/^(target|node_modules|\.git)$/.test(e.name)) scan(p, depth + 1); }
    else if (/\.(md|json|js|rs)$/.test(e.name)) {
      const t = readFileSync(p, "utf8");
      if (/docs\/play|acuity[- ]sandbox/i.test(t)) no(`${p} mentions the toy — the record may not cite it`);
    }
  }
};
scan(join(repo, "conformance"));
scan(join(repo, "docs", "workbench"));
ok("nothing under conformance/ or the workbench page mentions the toy");

// 3. THE TOY MUST SAY SO, ON THE FACE, ABOVE THE FOLD. A disclaimer in a comment is a
//    disclaimer nobody reads.
const html = readFileSync(join(here, "index.html"), "utf8");
if (!/A TOY\./.test(html)) no("index.html does not carry the TOY banner");
else if (html.indexOf("A TOY.") > html.indexOf("<canvas")) no("the TOY banner is below the canvas");
else ok("the TOY banner is on the face, above the canvas");
if (!/No claim to physics is made/.test(html)) no("index.html does not disclaim a physics claim in words");
else ok("the page disclaims a physics claim in words a reader will see");

// 4. THE TOY MAY NOT PRETEND ITS INVENTED LAYER IS MEASURED. The blob kernel's constants are
//    invented and the file must say so where they are declared.
const app = readFileSync(join(here, "app.js"), "utf8");
if (!/invented/.test(app)) no("app.js never calls its invented layer invented");
else ok("app.js calls its invented layer invented, where the constants are declared");

console.log(bad === 0 ? "\nthe wall holds: all checks passed" : `\nthe wall is breached: ${bad} failure(s)`);
process.exit(bad === 0 ? 0 : 1);
