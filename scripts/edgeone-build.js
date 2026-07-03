import { cpSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Packs exactly what EdgeOne needs, nothing else.
const outputDir = "dist";
rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });

cpSync("edgeone/cloud-functions", join(outputDir, "cloud-functions"), { recursive: true });
cpSync("edgeone/public", join(outputDir, "public"), { recursive: true });
cpSync("edgeone/releases", join(outputDir, "releases"), { recursive: true });
cpSync("edgeone/install.sh", join(outputDir, "install.sh"));

writeFileSync(
  join(outputDir, "package.json"),
  `${JSON.stringify({ name: "neomux-edgeone", version: "0.1.0", type: "module", private: true }, null, 2)}\n`,
);

console.log("built dist");
