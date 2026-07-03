import { mkdirSync } from "node:fs";
import { basename, join } from "node:path";
import { spawnSync } from "node:child_process";

const build = spawnSync("cargo", ["build", "--release"], { stdio: "inherit" });
if (build.status !== 0) process.exit(build.status ?? 1);

const platform = `${process.platform}-${process.arch}`;
const outDir = "dist/releases";
const archiveName = `neomux-0.1.0-${platform}.tar.gz`;
mkdirSync(outDir, { recursive: true });

const binary = process.platform === "win32" ? "neomux.exe" : "neomux";
const source = join("target", "release", binary);
const archive = join(outDir, archiveName);

const result = spawnSync("tar", ["-czf", archive, "-C", join("target", "release"), basename(source)], {
  stdio: "inherit",
});
if (result.status !== 0) process.exit(result.status ?? 1);

console.log(archive);
