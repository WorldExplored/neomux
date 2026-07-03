#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { basename, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const cwd = resolve(process.argv[2] ?? process.cwd());
const session = process.env.FORGE_SESSION ?? `forge-${basename(cwd).replace(/[^A-Za-z0-9_-]/g, "-")}`;
const root = fileURLToPath(new URL("..", import.meta.url));
const agent = `${root}bin/agent.js`;

function has(command) {
  return spawnSync("sh", ["-lc", `command -v ${command}`], { stdio: "ignore" }).status === 0;
}

function tmux(args) {
  const result = spawnSync("tmux", args, { stdio: "inherit" });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

if (!has("tmux")) {
  console.error("tmux is required.");
  process.exit(1);
}

if (!has("nvim")) {
  console.error("nvim is required.");
  process.exit(1);
}

const exists = spawnSync("tmux", ["has-session", "-t", session], { stdio: "ignore" }).status === 0;
if (exists) {
  tmux(["attach-session", "-t", session]);
}

tmux(["new-session", "-d", "-s", session, "-c", cwd]);
tmux(["rename-window", "-t", `${session}:0`, "forge"]);
tmux(["send-keys", "-t", `${session}:0.0`, "nvim '+silent! colorscheme rose-pine' .", "C-m"]);
tmux(["split-window", "-h", "-t", `${session}:0.0`, "-c", cwd]);
tmux(["send-keys", "-t", `${session}:0.1`, `node ${JSON.stringify(agent)}`, "C-m"]);
tmux(["select-pane", "-t", `${session}:0.0`]);
tmux(["attach-session", "-t", session]);
