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
export EDGEONE_BASE_URL="https://<your-edgeone-domain>"
```

For a deployed EdgeOne domain, neomux calls `/api/chat`. For the direct Makers Models gateway, it calls `/v1/chat/completions`.

This repo includes `edgeone/cloud-functions/api/chat.js`, which proxies the same chat payload the CLI sends. Set `MAKERS_MODELS_KEY` in the deployed EdgeOne function environment to use the hosted proxy without a local key.

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

Function environment:

```bash
MAKERS_MODELS_KEY=...
EDGEONE_GATEWAY_BASE_URL=https://ai-gateway.edgeone.link/v1
```

Deploy:

```bash
edgeone makers deploy ./dist -n neomux -t $EDGEONE_API_TOKEN -e production
```

See `docs/edgeone.md` for details.
