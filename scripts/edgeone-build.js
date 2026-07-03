import { cpSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const dist = "dist";
rmSync(dist, { recursive: true, force: true });
mkdirSync(dist, { recursive: true });

cpSync("edgeone/cloud-functions", join(dist, "cloud-functions"), { recursive: true });
cpSync("edgeone/public", join(dist, "public"), { recursive: true });
cpSync("edgeone/releases", join(dist, "releases"), { recursive: true });
cpSync("edgeone/install.sh", join(dist, "install.sh"));

writeFileSync(
  join(dist, "package.json"),
  `${JSON.stringify({ name: "neomux-edgeone", version: "0.1.0", type: "module", private: true }, null, 2)}\n`,
);

console.log("built dist");
