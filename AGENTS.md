# AGENTS.md

> 此文件是 AI 代理的项目导航地图。详细规范查阅 `docs/` 目录。

## 项目概览

**skm** — AI Agent 技能包本地管理 CLI，Rust 编写。  
仓库：`MocikaSpace/mocika-skills-cli`  
当前状态：**Phase 1 主体功能已实现，i18n / help 文本中英双语已完成**

## 关键约束

- **语言**：纯 Rust CLI，无 GUI
- **二进制名**：`skm`
- **中央仓库**：`~/.agents/skills/`（与 skilly GUI 共用目录约定）
- **锁文件**：`~/.agents/.skill-lock.json`（与 skilly 共用，字段兼容）
- **配置**：`~/.agents/sources.toml`、`~/.agents/agents.toml`
- **提交格式**：`<英文类型>: <中文描述>`，类型限 `feat/fix/docs/refactor/test/chore` 等
- **不可提交**：不得在未明确要求时自动提交；不得使用 `as any` / `unwrap()` 无错误处理

## 导航

| 文档 | 路径 |
|------|------|
| 需求文档 | `docs/requirements.md` |
| 技术设计 | `docs/design.md` |
| 命令参考 | `docs/commands.md` |
| 提交规范 | `~/.config/opencode/docs/process/commit-convention.md` |
| 全局规则 | `~/.config/opencode/AGENTS.md` |

## 核心设计速查

```
skm install <name> [--link-to <agent|all>]   # 安装 + 软链接部署
skm uninstall <name>                          # 卸载技能及所有软链接
skm search <keyword> [--limit <N>]            # 从 skills.sh 搜索
skm list                                      # 列出已安装技能及链接状态
skm info <name>                               # 查看技能详情
skm update [name] [--check]                   # Git-based 更新（自动备份）
skm link <name> <agent>                       # 为技能补链到 Agent
skm unlink <name> <agent>                     # 移除 Agent 的软链接
skm relink [agent] [--skill <name>] [--force] [--dry-run]  # 批量重新链接
skm scan [--dry-run]                          # 检测已安装 Agent → agents.toml
skm source list/add/remove                    # 注册表源管理
skm agent list/add                            # Agent 列表 / 手动注册
skm backup list/restore/delete <name>         # 备份快照管理
skm config lang [code|--reset]                # 查看 / 设置界面语言
skm self-update [--check]                     # 自我升级（从 GitHub Releases）
```

Agent 检测四信号（任一为真即认为已安装）：
`which <cmd>` || 配置目录存在 || skills目录存在 || skills目录有技能包

## i18n 说明

help 文本支持中英双语，运行时动态注入（不是静态编译）。语言优先级：
1. `~/.agents/skm.toml` 中的 `lang` 配置
2. 系统环境变量 `$LANG`（`zh_*` → 中文，其余 → 英文）

切换命令：`skm config lang zh` / `skm config lang en` / `skm config lang --reset`

## 发布规范

### 产物命名

Release 产物统一使用用户友好格式，**不使用 Rust target triple**：

| 平台 | 产物文件名 |
|------|-----------|
| Linux x86_64 | `skm-linux-amd64` |
| Linux aarch64 | `skm-linux-arm64` |
| macOS x86_64 | `skm-macos-amd64` |
| macOS Apple Silicon | `skm-macos-arm64` |

⚠️ **`install.sh` 中的 `detect_target()` 返回值必须与上表一一对应**。  
修改 `release.yml` 的 `matrix.artifact` 时，必须同步修改 `install.sh`，反之亦然。

### MSRV

当前最低支持 Rust 版本：**1.88**（由 `home` crate 依赖决定）。  
升级依赖前先确认新的 MSRV，并同步更新 `Cargo.toml` 中的 `rust-version` 和 `ci.yml` 中的 MSRV job。

---

## 仓库配置规范

适用于本 org（`mocikadev`）下所有仓库，新建仓库时参照执行。

### About Description

格式：`中文描述 · English description`（用 ` · ` 分隔，单行）

| 仓库 | description |
|------|-------------|
| `mocika-skills-cli` | `AI Agent 技能包本地管理 CLI · Local skill manager for AI Agents` |
| `skm-skill` | `skm 命令参考技能包 · skm command reference skill for AI Agents` |

### Homepage URL

| 仓库 | homepage |
|------|----------|
| `mocika-skills-cli` | `https://github.com/mocikadev/mocika-skills-cli/releases/latest` |
| `skm-skill` | `https://github.com/mocikadev/mocika-skills-cli` |

### Topics

| 仓库 | topics |
|------|--------|
| `mocika-skills-cli` | `ai-agent` `cli` `rust` `skill-manager` `opencode` `claude-code` |
| `skm-skill` | `ai-agent` `skill` `skm` `opencode` `claude-code` |

---

## 配套产物

- **skm-skill**：CLI 命令参考 skill，供 AI Agent 学习并代替用户操作 skm。  
  仓库规划：`mocikadev/skm-skill`，CLI 命令稳定后输出。

### skm-skill 同步规范

`docs/skm-skill/` 目录是本仓库内的 skm-skill **唯一源文件**，外部仓库 `mocikadev/skm-skill` 是其发布镜像。

| 文件 | 说明 |
|------|------|
| `docs/skm-skill/SKILL.md` | 完整命令参考（AI Agent 使用的核心文件） |
| `docs/skm-skill/README.md` | 技能包说明文档（对人类用户） |

**同步时机**（凡以下情形发生，必须同步更新 `mocikadev/skm-skill`）：
- 新增、删除或改名任何 `skm` 子命令或参数
- 修改命令行为、默认值或输出格式
- 更新示例、安装脚本地址等面向用户的信息

**同步流程**：
1. 先在本仓库修改 `docs/skm-skill/SKILL.md` / `README.md`
2. CLI 稳定后，将 `docs/skm-skill/` 内容推送至 `mocikadev/skm-skill` 仓库
