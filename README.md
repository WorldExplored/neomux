# Agent Forge Minihack

Minimal tmux cockpit for a coding agent.

```bash
npm start
```

This opens a tmux session with:

- left pane: `nvim .` using your normal Neovim dotfiles, then `rose-pine` if available
- right pane: a tiny EdgeOne-backed coding agent CLI

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
/model <id>
/read <file>
/run <cmd>
/clear
/exit
```
