# neomux

**A Cursor-style AI coding lane for Neovim.**

Neomux launches your normal Neovim setup in the left tmux pane and a compact EdgeOne-backed coding agent in the right pane. It is for people who want the “AI pair programmer beside the editor” workflow without leaving their terminal, replacing their dotfiles, or installing a heavy editor.

Think: **Cursor for Neovim, built as a tiny Rust CLI.**

## What Works Today

- Opens a deterministic tmux workspace: Neovim left, agent right.
- Uses your existing Neovim config and tries `rose-pine` if available.
- Gives the agent repo context with `/find`, `/read`, and `/context`.
- Runs local commands with `/run`, including exit code and duration.
- Streams chat from Tencent EdgeOne Makers Models.
- Keeps model controls local: `/model`, `/temperature`, `/max-tokens`.
- Checks the machine with `neomux doctor`.

Current MVP boundary: the agent suggests code and commands, searches files, reads files, and runs commands. It does not yet apply edits directly into buffers like a full IDE assistant.

## 30-Second Install

From this repo:

```bash
cargo install --path .
```

Npm wrapper install:

```bash
npm install -g .
```

If npm blocks install scripts, the wrapper builds the Rust binary on first run. To allow the install-time build explicitly:

```bash
npm install -g --allow-scripts=neomux .
```

Then launch it from any project:

```bash
neomux .
```

## Prerequisites

- Rust stable
- `tmux`
- `nvim`
- `rg`
- `curl`
- `MAKERS_MODELS_KEY`

Check setup:

```bash
neomux doctor
```

## Local Usage

Start the cockpit:

```bash
neomux .
```

Use a stable session name for demos:

```bash
neomux . --session neomux-demo
```

The session opens with:

- left pane: `nvim .` using your normal Neovim dotfiles, then `rose-pine` if available
- right pane: a compact pink/purple `neomux agent`

Set your EdgeOne Makers Models key before chatting:

```bash
export MAKERS_MODELS_KEY="..."
```

Optional model override:

```bash
export EDGEONE_MODEL="@makers/deepseek-v4-flash"
```

Optional EdgeOne API base URL:

```bash
export EDGEONE_BASE_URL="https://ai-gateway.edgeone.link/v1"
```

## Agent Commands

```text
/help                 show agent commands
/find <text>          search workspace with rg
/read <file>          load a file into model context
/context              show loaded files and recent command summaries
/forget [file|all]    remove loaded context
/run <cmd>            run a shell command from the workspace
/model <id>           switch model
/temperature <n>      set model temperature
/max-tokens <n>       set max output tokens
/clear                clear chat history
/exit                 quit the agent pane
```

## Why It Exists

Cursor proved that an editor-adjacent coding agent is the right shape. Neomux keeps that shape but moves it into the terminal:

- no editor migration
- no Neovim plugin lifecycle
- no hidden project indexing service
- no runtime npm dependency for the local CLI
- plain tmux, plain Neovim, plain commands

The design goal is not to clone every IDE feature. It is to make the tight loop useful fast: inspect code, load context, ask the model, run commands, edit in Neovim.

## Build

```bash
cargo build --release
npm run check
```

Package a local archive:

```bash
npm run package
```

## EdgeOne Hosting

Tencent EdgeOne build command:

```bash
npm run edgeone:build
```

Console settings:

```json
{
  "installCommand": "npm install",
  "buildCommand": "npm run edgeone:build",
  "outputDirectory": "dist",
  "nodeVersion": "22.11.0"
}
```

Deploy:

```bash
edgeone makers deploy ./dist -n neomux -t $EDGEONE_API_TOKEN -e production
```

Hosted routes:

- `/api/chat`: optional EdgeOne model proxy
- `/api/models`: static model metadata
- `/api/usage`: honest usage/pricing placeholder until official data is wired
- `/`: install and demo page

See `docs/edgeone.md` for details.

## Next Features

- Apply model-proposed diffs directly from the agent pane.
- Send selected Neovim ranges to the agent.
- Add a repo map command for fast high-level context.
- Add first-class hosted proxy configuration.
- Replace static model metadata when EdgeOne exposes an official catalog/pricing API.

