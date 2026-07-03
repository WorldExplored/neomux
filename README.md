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

Agent commands:

```text
/model <id>
/read <file>
/run <cmd>
/clear
/exit
```
