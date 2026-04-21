# AGENTS.md

> 此文件是 AI 代理的项目导航地图。详细规范查阅 `docs/` 目录。

## 项目概览

**skm** — AI Agent 技能包本地管理 CLI，Rust 编写。  
仓库：`MocikaSpace/mocika-skills-cli`  
当前状态：**需求讨论阶段，尚未开始实现**

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
| 提交规范 | `~/.config/opencode/docs/process/commit-convention.md` |
| 全局规则 | `~/.config/opencode/AGENTS.md` |

## 参考工程

- **skilly**（`~/WorkSpace/skilly`）：同生态 GUI 桌面版，Rust 核心层可直接参考
  - `src-tauri/src/core/agent.rs`：Agent 检测逻辑（4-signal 检测）
  - `src-tauri/src/core/lock.rs`：锁文件读写（原子 tmp+rename）
  - `src-tauri/src/core/registry.rs`：Registry API 集成（search / leaderboard / skill content）
  - `src-tauri/src/core/operations.rs`：安装、更新、备份核心操作

## 核心设计速查

```
skm install <name> [--link-to <agent|all>]   # 安装 + 软链接部署
skm scan                                      # 检测已安装 Agent → agents.toml
skm relink [agent]                            # 新装 Agent 后批量补链
skm update [name]                             # Git-based 更新（自动备份）
skm search <keyword>                          # 从 skills.sh 搜索
skm self-update                               # 自我升级（Backlog）
```

Agent 检测四信号（任一为真即认为已安装）：
`which <cmd>` || 配置目录存在 || skills目录存在 || skills目录有技能包

## 配套产物

- **skm-skill**：CLI 命令参考 skill，供 AI Agent 学习并代替用户操作 skm。  
  仓库规划：`mocikadev/skm-skill`，CLI 命令稳定后输出。
