> 中文版本：[README.md](../README.md)

# skm

[![CI](https://github.com/mocikadev/mocika-skills-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-skills-cli/actions/workflows/ci.yml)

A local skill package manager CLI for AI Agents. Manage installation, deployment, updates, and backups of skill packages across multiple AI Agents from a single tool.

## Features

- **Unified install**: One command to install skills from [skills.sh](https://skills.sh), with direct Git repo installs also supported
- **Multi-source search**: Register any GitHub / GitLab / private Git repo or **local directory** as a skill source via `skm source add`; search across all sources with `skm search`
- **Local source support**: Register a local filesystem directory as a source, or install directly from a local path — no git required
- **Multi-agent deployment**: Symlink mechanism lets a single skill file serve multiple Agents at once
- **Export & import**: `skm export` bundles installed skills into a shareable file; `skm import` batch-installs from a bundle for quick environment restore
- **Auto-detection**: `skm scan` detects AI Agents installed on your machine, no manual config needed
- **Shared lock file**: Shares `~/.agents/.skill-lock.json` with the skilly GUI, keeping data in sync
- **Safe updates**: Automatic backup before every update, with snapshot-level rollback support
- **Zero root required**: All data written to `~/.agents/`, no sudo needed

## Install skm

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

Installs to `~/.local/bin/skm`. To use a custom path:

```bash
SKM_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL .../install.sh)
```

To install a specific version:

```bash
SKM_VERSION=v0.2.0 bash <(curl -fsSL .../install.sh)
```

## Quick Start

```bash
# 1. Detect AI Agents installed on your machine
skm scan

# 2. Search for a skill
skm search android

# 3. Install a skill and link it to all Agents
skm install mobile-android-design --link-to all

# 4. Just installed a new Agent? Re-link everything in one command
skm relink cursor
```

## Install skm skill (recommended)

`skm` ships with a companion AI Agent skill that lets your AI assistant understand and run skm commands directly:

```bash
skm install mocikadev/mocika-skills-cli:skills/skm --link-to all
```

> Once installed, your AI Agent can handle all `skm` operations on your behalf, so you don't need to memorize the commands.

## Command Reference

| Command | Description |
|---------|-------------|
| `skm install <name> [--link-to <agent\|all>]` | Install a skill (supports registry name, `owner/repo`, or full Git URL) |
| `skm uninstall <name>` | Uninstall a skill |
| `skm search <keyword>` | Search across all registered sources |
| `skm list` | List installed skills |
| `skm list --outdated` | Show only skills with available updates |
| `skm info <name>` | Show skill details |
| `skm update [name]` | Update a skill (omit name or add `--all` to update everything) |
| `skm link <name> <agent>` | Link a skill to an Agent |
| `skm unlink <name> <agent>` | Remove a link |
| `skm relink [agent]` | Re-link all skills in bulk |
| `skm export [--output <file>]` | Export installed skills as a shareable `skills.bundle` |
| `skm import <file> [--link-to <agent\|all>] [--force]` | Batch-install skills from a bundle file |
| `skm scan [--import <file>]` | Detect Agents on the local machine (optionally import a bundle after scan) |
| `skm agent list` | List registered Agents |
| `skm backup list/restore/delete <name>` | Manage backups |
| `skm doctor` | Check environment health and diagnose link/Agent issues |
| `skm doctor fix-skills [--dry-run]` | Detect and auto-fix SKILL.md frontmatter issues |
| `skm doctor sync [--agent <id>] [--dry-run]` | Rebuild symlinks, repair broken or missing links |
| `skm source list/add/remove` | Manage registry sources (skills.sh / GitHub / Git / local dir) |
| `skm config lang [zh\|en\|--reset]` | View or switch help language |
| `skm self-update [--check]` | Upgrade skm itself to the latest version |

Full documentation: [docs/commands.md](commands.md)

## Data Directory

```
~/.agents/
├── skills/              # Central skill store (single source of truth)
├── .skill-lock.json     # Install metadata (shared with skilly)
├── .skm-backups/        # Skill backup snapshots
├── sources.toml         # Registry source config
└── agents.toml          # Registered Agent config
```

## Supported Agents

`claude-code` · `codex` · `gemini-cli` · `copilot-cli` · `opencode` · `cursor` · `kiro` · `trae` · `trae-cn` · `junie` · `qoder` · `codebuddy` · `openclaw` · `antigravity` · `windsurf` · `augment` · `kilocode` · `ob1` · `amp` · `hermes` · `factory-droid` · `qwen` · `cline` · `roo` · `goose` · `continue` · `warp` · `openhands` · `firebender` · `zencoder` · `cortex` · `deepagents` · `crush` · `kimi-cli` · `mux` · `neovate` · `mistral-vibe` · `pochi` · `openclaude-ide` · `kode` · `mcpjam` · `bob` · `adal` · `pi` · `iflow-cli` · `command-code`

Agents not listed above can be registered manually with `skm agent add`.

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | x86_64 | ✅ |

## Build from Source

```bash
git clone https://github.com/mocikadev/mocika-skills-cli
cd mocika-skills-cli
cargo build --release
# Output: ./target/release/skm
```

Requires Rust 1.88+.

## License

Licensed under either of [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE) at your option.
