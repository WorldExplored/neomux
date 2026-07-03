# neomux Demo Script

## Setup

```bash
export MAKERS_MODELS_KEY="..."
cargo install --path .
neomux doctor
```

## Live Demo

```bash
neomux . --session neomux-demo
```

Expected layout:

- left pane: Neovim opened at the current project
- right pane: `neomux agent`

Agent flow:

```text
/find TODO
/read package.json
/context
Ask: summarize this project in 3 bullets
/run npm run check
```

## Fallback

If the EdgeOne key or quota fails:

```text
/find neomux
/read README.md
/context
/run cargo build --release
```

This still demonstrates the tmux cockpit, workspace context, and local command loop.
