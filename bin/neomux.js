#!/usr/bin/env node
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binary = join(root, "target", "release", process.platform === "win32" ? "neomux.exe" : "neomux");

if (!existsSync(binary)) {
  // Npm users get the Rust binary without learning Cargo first.
  const buildResult = spawnSync("cargo", ["build", "--release"], { cwd: root, stdio: "inherit" });
  if (buildResult.status !== 0) process.exit(buildResult.status ?? 1);
}

const cliResult = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
process.exit(cliResult.status ?? 1);
