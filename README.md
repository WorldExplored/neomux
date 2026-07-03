# neomux

Rose Pine tmux cockpit: Neovim on the left, EdgeOne-backed Rust coding agent on the right.

## 30-second install

From this folder:

```bash
cargo install --path .
```

Npm wrapper install:

```bash
npm install -g .
```

If your npm blocks install scripts, the wrapper will build the Rust binary on first run. To allow the install-time build explicitly:

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

## Local usage

The tmux session opens with:

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

Optional EdgeOne Pages Function endpoint:

```bash
export EDGEONE_BASE_URL="https://<your-edgeone-domain>/v1"
```

This repo includes `functions/v1/chat/completions/index.js`, which accepts the same streaming chat payload the CLI sends. With a Pages Function endpoint, the default model alias is `deepseek`; direct Makers gateway usage keeps the existing `@makers/deepseek-v4-flash` default.

Pages Function model aliases:

```text
deepseek
deepseek-v3
deepseek-v32
deepseek-r1
makers-deepseek
minimax
hy3
codex
codex-frontier
fable
```

The `makers-deepseek`, `minimax`, `hy3`, `codex`, `codex-frontier`, and `fable` aliases require `MAKERS_MODELS_KEY` on the request or in the deployed function environment.

Agent commands:

```text
/help
/find <text>
/model <id>
/read <file>
/context
/forget [file|all]
/run <cmd>
/temperature <n>
/max-tokens <n>
/clear
/exit
```

## Build

```bash
cargo build --release
npm run check
```

## EdgeOne

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

See `docs/edgeone.md` for details.
