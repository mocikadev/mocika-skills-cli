# AGENTS.md

> 此文件是 AI 代理的项目导航地图。详细规范查阅 `docs/` 目录。

## 项目概览

**skm** — AI Agent 技能包本地管理 CLI，Rust 编写。  
仓库：`MocikaSpace/mocika-skills-cli`  
当前状态：**Phase 1 + Phase 2 全部功能已实现，i18n 中英双语已完成**

## Rust 提交前检查清单（必须按顺序执行）

```bash
cargo fmt                        # 1. 格式化（CI 用 --check，本地直接跑）
cargo clippy -- -D warnings      # 2. Lint（警告即报错）
cargo test                       # 3. 测试（如有）
```

> ⚠️ `clippy` 通过 ≠ `fmt` 通过，两者独立。CI 会同时检查，本地必须两个都跑。

## 关键约束

- **语言**：纯 Rust CLI，无 GUI
- **二进制名**：`skm`
- **中央仓库**：`~/.agents/skills/`（与 skilly GUI 共用目录约定）
- **锁文件**：`~/.agents/.skill-lock.json`（与 skilly 共用，字段兼容）
- **配置**：`~/.agents/sources.toml`、`~/.agents/agents.toml`
- **提交格式**：`<英文类型>: <中文描述>`，类型限 `feat/fix/docs/refactor/test/chore` 等
- **不可提交**：不得在未明确要求时自动提交；不得使用 `unwrap()` 无错误处理
- **MSRV**：最低支持 Rust **1.88**（由 `home` crate 决定）。升级依赖前确认新 MSRV，并同步更新 `Cargo.toml` 的 `rust-version` 和 `ci.yml` 的 MSRV job
- **release.yml ↔ install.sh 强耦合**：修改 `release.yml` 的 `matrix.artifact` 时必须同步修改 `install.sh` 的 `detect_target()`，反之亦然
- **SKILL.md 同步**：新增/删除/改名任何子命令或参数，必须同步更新 `skills/skm/SKILL.md`

## 导航

| 文档 | 路径 |
|------|------|
| 需求文档 | `docs/requirements.md` |
| 技术设计 | `docs/design.md` |
| 命令参考 | `docs/commands.md` |
| 提交规范 | `~/.config/opencode/docs/process/commit-convention.md` |
| 全局规则 | `~/.config/opencode/AGENTS.md` |

## i18n 说明

help 文本支持中英双语，运行时动态注入（不是静态编译）。语言优先级：
1. `~/.agents/skm.toml` 中的 `lang` 配置
2. 系统环境变量 `$LANG`（`zh_*` → 中文，其余 → 英文）

切换命令：`skm config lang zh` / `skm config lang en` / `skm config lang --reset`
