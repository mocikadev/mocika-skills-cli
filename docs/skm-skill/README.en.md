# skm-skill

> English | [中文](README.md)

AI Agent skill that lets your AI assistant operate `skm` commands directly — no manual execution needed.

## Installation

Via skm (recommended):

```bash
skm install mocikadev/skm-skill --link-to all
```

Link to a specific agent only:

```bash
skm install mocikadev/skm-skill --link-to opencode
```

Manual installation (without skm):

```bash
# Clone into the central skills directory
git clone https://github.com/mocikadev/skm-skill ~/.agents/skills/skm

# Symlink to your agent (opencode example)
ln -s ~/.agents/skills/skm/SKILL.md ~/.config/opencode/skills/skm
```

## Usage

After installation, just tell your AI agent in natural language:

```
Search for android-related skill packages
```

```
Install mobile-android-design and link it to all agents
```

```
Detect which AI agents are installed on my machine
```

```
Update all skill packages
```

Your AI agent will automatically run the corresponding `skm` commands.

## Capabilities

| Capability | Command |
|------------|---------|
| Search skill registry | `skm search` |
| Install / uninstall skills | `skm install` / `skm uninstall` |
| Link / unlink skills | `skm link` / `skm unlink` |
| List installed skills | `skm list` / `skm info` |
| Update skills | `skm update` |
| Re-link all skills | `skm relink` |
| Detect installed agents | `skm scan` |
| Manage agent list | `skm agent list / add` |
| Backup & restore | `skm backup list / restore / delete` |
| Registry source management | `skm source list / add / remove` |

## Prerequisites

- `skm` must be installed ([installation guide](https://github.com/mocikadev/mocika-skills-cli#installing-skm))
- Target AI agent must be detected via `skm scan` or registered via `skm agent add`

## Links

- [skm CLI repository](https://github.com/mocikadev/mocika-skills-cli)
- [Full command reference](https://github.com/mocikadev/mocika-skills-cli/blob/main/docs/commands.md)
