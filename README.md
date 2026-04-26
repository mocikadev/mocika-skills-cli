> English version: [docs/README.en.md](docs/README.en.md)

# skm

[![CI](https://github.com/mocikadev/mocika-skills-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/mocikadev/mocika-skills-cli/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/mocikadev/mocika-skills-cli)](https://github.com/mocikadev/mocika-skills-cli/releases/latest)

AI Agent 技能包本地管理 CLI。统一管理多个 AI Agent 的技能包安装、部署、更新与备份。

## 特性

- **统一安装**：一条命令从 [skills.sh](https://skills.sh) 安装技能，Git 仓库直装同样支持
- **多源搜索**：`skm source add` 注册任意 GitHub / GitLab / 私有 Git 仓库为技能源，`skm search` 跨源搜索
- **多 Agent 部署**：软链接机制，一份技能文件同时服务多个 Agent
- **自动检测**：`skm scan` 检测本机已安装的 AI Agent，无需手动配置
- **锁文件共享**：与 skilly GUI 共用 `~/.agents/.skill-lock.json`，数据互通
- **安全更新**：更新前自动备份，支持快照级回滚
- **零 root 权限**：全部数据写入 `~/.agents/`，无需 sudo

## 安装 skm

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

**Windows**（PowerShell）

```powershell
irm https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.ps1 | iex
```

> Windows 需开启 **Developer Mode**（设置 → 系统 → 开发者选项）或以管理员身份运行，以支持目录符号链接。

安装到 `~/.local/bin/skm`（Windows 为 `~\.local\bin\skm.exe`）。自定义路径：

```bash
SKM_INSTALL_DIR=/usr/local/bin bash <(curl -fsSL .../install.sh)
```

安装指定版本：

```bash
SKM_VERSION=v0.2.0 bash <(curl -fsSL .../install.sh)
```

## 快速上手

```bash
# 1. 检测本机已安装的 AI Agent
skm scan

# 2. 搜索技能
skm search android

# 3. 安装技能（默认链接到所有 Agent）
skm install mobile-android-design

# 4. 新装了一个 Agent？一条命令补齐所有链接
skm relink cursor
```

## 安装 skm skill（推荐）

`skm` 内置了配套的 AI Agent 技能包，让你的 AI 助手直接理解并操作 skm 命令：

```bash
skm install mocikadev/mocika-skills-cli:skills/skm --link-to all
```

> 安装后，AI Agent 可以代替你执行所有 `skm` 操作，无需记忆命令细节。

## 命令速查

| 命令 | 说明 |
|------|------|
| `skm install <name> [--link-to <agent\|all>]` | 安装技能，默认链接到所有 Agent（支持注册表名、`owner/repo`、完整 Git URL） |
| `skm uninstall <name>` | 卸载技能 |
| `skm search <keyword>` | 搜索注册表（skills.sh API + Git 源本地扫描） |
| `skm list` | 列出已安装技能 |
| `skm list --outdated` | 只显示有更新可用的技能 |
| `skm info <name>` | 查看技能详情 |
| `skm update [name]` | 更新技能（不带 name 或加 `--all` 更新全部） |
| `skm link <name> <agent>` | 链接到 Agent |
| `skm unlink <name> <agent>` | 移除链接 |
| `skm relink [agent]` | 批量重新链接 |
| `skm scan` | 检测本机 Agent |
| `skm agent list` | 列出已注册 Agent |
| `skm backup list/restore/delete <name>` | 备份管理 |
| `skm doctor` | 检测环境健康状态，诊断链接/Agent 问题 |
| `skm source list/add/remove` | 注册表源管理（支持 skills.sh / GitHub / Git） |

完整文档：[docs/commands.md](docs/commands.md)

## 数据目录

```
~/.agents/
├── skills/              # 中央技能仓库（唯一存储位置）
├── .skill-lock.json     # 安装元数据（与 skilly 共用）
├── .skm-backups/        # 技能备份快照
├── sources.toml         # 注册表源配置
└── agents.toml          # 已注册 Agent 配置
```

## 支持的 Agent

`claude-code` · `codex` · `gemini-cli` · `copilot-cli` · `opencode` · `cursor` · `kiro` · `trae` · `trae-cn` · `junie` · `qoder` · `codebuddy` · `openclaw` · `antigravity`

未列出的 Agent 可通过 `skm agent add` 手动注册。

## 平台支持

| 平台 | 架构 | 状态 |
|------|------|------|
| Linux | x86_64 (musl) | ✅ |
| Linux | aarch64 (musl) | ✅ |
| macOS | x86_64 | ✅ |
| macOS | Apple Silicon | ✅ |
| Windows | x86_64 | ✅ |

## 从源码构建

```bash
git clone https://github.com/mocikadev/mocika-skills-cli
cd mocika-skills-cli
cargo build --release
# 产物：./target/release/skm
```

需要 Rust 1.88+。

## 许可证

本项目采用 **MIT OR Apache-2.0** 双协议授权，你可以选择其中任意一种。

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
