---
name: skm
displayName: skm — AI Agent 技能包管理器
description: 使用 skm CLI 管理本地 AI Agent 技能包：安装、卸载、搜索、链接、更新和备份。
version: 0.1.0
author: mocikadev
tags: [cli, skill-manager, agent, tooling]
---

# skm — AI Agent 技能包管理器

`skm` 是一个本地 CLI 工具，统一管理 AI Agent 技能包（skill）的完整生命周期：安装、卸载、跨 Agent 软链接部署、更新和备份。

## 安装 skm

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

安装后运行 `skm --help` 验证。

## 数据目录

| 路径 | 说明 |
|------|------|
| `~/.agents/skills/` | 中央技能仓库（所有技能的唯一存储位置） |
| `~/.agents/.skill-lock.json` | 安装元数据锁文件（与 skilly 共用） |
| `~/.agents/sources.toml` | 注册表源配置 |
| `~/.agents/agents.toml` | 已注册 Agent 路径配置 |
| `~/.agents/.skm-backups/` | 技能备份目录 |

Agent 目录下的技能文件均为指向中央仓库的软链接，不复制文件。

## 快速工作流

### 第一次使用：检测已安装 Agent

```bash
skm scan            # 检测本机所有 AI Agent，写入 agents.toml
skm agent list      # 查看已注册的 Agent 列表
```

### 安装并部署技能

```bash
# 从注册表安装
skm install mobile-android-design --link-to all

# GitHub 简写（owner/repo，自动补全 github.com）
skm install mocikadev/skm-skill --link-to all

# GitHub 简写 + 子目录（支持多级路径）
skm install wshobson/agents:mobile-android-design --link-to all
skm install myorg/skills:tools/formatter --link-to opencode

# 完整 Git URL（GitLab / Gitee 等非 GitHub 平台使用此格式）
skm install https://gitlab.com/myorg/skills.git --link-to all

# 完整 Git URL + 子目录
skm install https://github.com/myorg/skills.git#tools/formatter

# 直接粘贴 GitHub 网页地址
skm install https://github.com/myorg/skills/tree/main/formatter --link-to all
```

### 新增 Agent 后补齐链接

```bash
skm scan            # 检测新 Agent（如 cursor）
skm relink cursor   # 将所有已安装技能链接到 cursor
```

## 完整命令参考

### 技能管理

#### `skm install <NAME> [--link-to <AGENT|all>]`

从注册表或 Git 仓库安装技能。NAME 支持以下格式：
- `skill-name` — 注册表名称
- `owner/repo` — GitHub 仓库根目录（固定解析为 github.com）
- `owner/repo:subpath` — GitHub 子目录，支持多级路径
- `<git-url>` — 完整 Git URL（适用于 GitLab/Gitee 等）
- `<git-url>#subpath` — 完整 URL + 子目录
- GitHub 网页 URL — 自动解析仓库和路径

```bash
skm install mobile-android-design
skm install mobile-android-design --link-to opencode
skm install mobile-android-design --link-to all
skm install wshobson/agents:mobile-android-design
skm install https://github.com/wshobson/agents.git --link-to opencode
```

#### `skm uninstall <NAME>`

卸载技能：删除中央仓库目录并移除所有 Agent 的软链接。

```bash
skm uninstall mobile-android-design
```

#### `skm search <KEYWORD> [--limit <N>]`

在 skills.sh 注册表中搜索技能（默认显示最多 20 条）。

```bash
skm search android
skm search android --limit 5
```

输出格式：`<name>  <安装量>  <描述>`

#### `skm list`

列出所有已安装技能及其软链接状态。

```bash
skm list
```

#### `skm info <NAME>`

显示技能详细信息：frontmatter 元数据、锁文件条目、各 Agent 链接状态。

```bash
skm info mobile-android-design
```

#### `skm update [NAME] [--check]`

更新技能到最新版本（更新前自动备份）。省略 NAME 则更新全部。

```bash
skm update
skm update mobile-android-design
skm update --check                        # 仅检查，不执行更新
skm update --check mobile-android-design
```

#### `skm link <NAME> <AGENT>`

为已安装技能在指定 Agent 目录下创建软链接。

```bash
skm link mobile-android-design opencode
```

#### `skm unlink <NAME> <AGENT>`

移除指定 Agent 目录下的技能软链接（不删除中央仓库文件）。

```bash
skm unlink mobile-android-design opencode
```

#### `skm relink [AGENT] [--skill <NAME>] [--force] [--backup] [--dry-run]`

批量重新链接技能。典型场景：新装 Agent 后同步所有技能。

```bash
skm relink                        # 对所有 Agent 重新链接所有技能
skm relink cursor                 # 仅对 cursor 重新链接
skm relink --skill mobile-android-design  # 仅重新链接指定技能
skm relink cursor --dry-run       # 预览，不实际执行
skm relink cursor --force         # 覆盖冲突路径
```

### 注册表管理

```bash
skm source list                   # 列出所有注册表源
skm source add my-org https://github.com/my-org/skills  # 添加自定义源
skm source remove my-org          # 移除注册表源
```

配置文件：`~/.agents/sources.toml`

### Agent 管理

```bash
skm scan                          # 自动检测并注册本机 AI Agent
skm scan --dry-run                # 预览检测结果，不写入配置
skm agent list                    # 列出所有已注册 Agent 及其技能数量
skm agent add my-agent ~/.my-agent/skills  # 手动注册自定义 Agent
```

配置文件：`~/.agents/agents.toml`

内置支持的 Agent：`claude-code`、`codex`、`gemini-cli`、`copilot-cli`、`opencode`、`antigravity`、`cursor`、`kiro`、`codebuddy`、`openclaw`、`trae`、`junie`、`qoder`、`trae-cn`

### 备份管理

```bash
skm backup list mobile-android-design                    # 列出所有备份快照
skm backup restore mobile-android-design                 # 恢复最新快照
skm backup restore mobile-android-design 1776758731056   # 恢复指定快照
skm backup delete mobile-android-design 1776758731056    # 删除指定快照
```

备份目录：`~/.agents/.skm-backups/<skill-name>/<snapshot-id>/`

## 常见场景速查

| 场景 | 命令 |
|------|------|
| 初次配置，检测所有 Agent | `skm scan` |
| 安装技能到所有 Agent | `skm install <name> --link-to all` |
| 只安装，稍后手动链接 | `skm install <name>` 然后 `skm link <name> <agent>` |
| 新装了一个 Agent | `skm scan && skm relink <new-agent>` |
| 查看某技能链接到哪些 Agent | `skm info <name>` |
| 检查是否有可用更新 | `skm update --check` |
| 升级技能（自动备份） | `skm update <name>` |
| 升级出问题，回滚 | `skm backup list <name>` 后 `skm backup restore <name>` |
