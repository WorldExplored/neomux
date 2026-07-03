#!/usr/bin/env node
import readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";

const makersGateway = "https://ai-gateway.edgeone.link/v1";
const endpoint = normalizeEndpoint(process.env.EDGEONE_BASE_URL);
const usesMakersGateway = endpoint === makersGateway;
let model = process.env.EDGEONE_MODEL ?? (usesMakersGateway ? "@makers/deepseek-v4-flash" : "deepseek");
const messages = [
  {
    role: "system",
    content:
      "You are Agent Forge, a terse coding agent in a tmux pane. Be practical. Prefer simple code, clear diffs, and shell commands the user can inspect.",
  },
];

function header() {
  console.log("\x1b[38;5;175mAgent Forge\x1b[0m");
  console.log(`model: ${model}`);
  console.log("commands: /model <id>, /read <file>, /run <cmd>, /clear, /exit");
}

function requireKey() {
  if (usesMakersGateway && !process.env.MAKERS_MODELS_KEY) {
    console.error("Set MAKERS_MODELS_KEY to use EdgeOne Makers Models.");
    process.exit(1);
  }
}

function normalizeEndpoint(value) {
  const endpoint = (value ?? makersGateway).replace(/\/+$/, "");
  return endpoint.endsWith("/v1") ? endpoint : `${endpoint}/v1`;
}

function requestHeaders() {
  const headers = {
    "Content-Type": "application/json",
  };

  if (process.env.MAKERS_MODELS_KEY) {
    headers.Authorization = `Bearer ${process.env.MAKERS_MODELS_KEY}`;
  }

  return headers;
}

function run(command) {
  return new Promise((resolveRun) => {
    const child = spawn(command, { shell: true, stdio: "inherit" });
    child.on("close", (code) => resolveRun(code));
  });
}

async function ask(prompt) {
  requireKey();
  messages.push({ role: "user", content: prompt });

  const response = await fetch(`${endpoint}/chat/completions`, {
    method: "POST",
    headers: requestHeaders(),
    body: JSON.stringify({ model, messages, stream: true }),
  });

  if (!response.ok || !response.body) {
    const text = await response.text();
    throw new Error(`EdgeOne request failed: ${response.status} ${text}`);
  }

  let answer = "";
  const decoder = new TextDecoder();
  let buffer = "";
  const writeLine = (line) => {
    if (!line.startsWith("data: ")) return;
    const data = line.slice(6).trim();
    if (!data || data === "[DONE]") return;
    const json = JSON.parse(data);
    const delta = json.choices?.[0]?.delta?.content ?? "";
    answer += delta;
    output.write(delta);
  };

  for await (const chunk of response.body) {
    buffer += decoder.decode(chunk, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() ?? "";
    for (const line of lines) writeLine(line);
  }
  buffer += decoder.decode();
  if (buffer) writeLine(buffer);
  output.write("\n");
  messages.push({ role: "assistant", content: answer });
}

header();
const rl = readline.createInterface({ input, output, prompt: "\x1b[38;5;175mforge>\x1b[0m " });

while (true) {
  const line = (await rl.question(rl.getPrompt())).trim();
  if (!line) continue;
  if (line === "/exit") break;
  if (line === "/clear") {
    messages.splice(1);
    console.log("conversation cleared");
    continue;
  }
  if (line.startsWith("/model ")) {
    model = line.slice(7).trim();
    console.log(`model: ${model}`);
    continue;
  }
  if (line.startsWith("/run ")) {
    await run(line.slice(5));
    continue;
  }
  if (line.startsWith("/read ")) {
    const path = line.slice(6).trim();
    const text = await readFile(path, "utf8");
    messages.push({ role: "user", content: `File ${path}:\n\n${text}` });
    console.log(`loaded ${path}`);
    continue;
  }

  try {
    await ask(line);
  } catch (error) {
    console.error(error.message);
  }
}

rl.close();
