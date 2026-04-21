# skm 需求文档

> 版本：0.5  
> 更新：2026-04-21  
> 参考：skilly GUI 工程（`~/WorkSpace/skilly`）

---

## 一、项目定位

**skm**（Skill Manager）是一个本地 AI Agent **技能包生命周期管理 CLI 工具**，用 Rust 编写。

**与 Vercel 官方 `skills` CLI 的关系**：
- 官方 `skills` CLI：从 skills.sh 平台发现和拉取技能包（`npx skills add`）
- `skm`：本地管理层——中央仓库存储、多 Agent 软链接部署、批量同步

类比：官方 `skills` = `npm install`（拉包），`skm` = `mise` / `nvm`（本地环境管理）

**同类参考**：skilly 是同一生态的 GUI 桌面版，共用目录约定（`~/.agents/skills`）和锁文件格式。`skm` 与 skilly 数据兼容。

---

## 二、命名

**二进制名：`skm`**

- 无系统命令冲突、crates.io / npm 均无占用
- `skill-manager` 缩写，业界已有先例
- 来自 MocikaSpace org，工程仓库名 `mocika-skills-cli`

---

## 三、技能包格式（skills.sh 标准）

技能包是一个 **Git 仓库的子目录**，核心文件是 `SKILL.md`：

```
<skill-name>/
├── SKILL.md          # 必需：YAML frontmatter + Markdown 指令正文
├── scripts/          # 可选：sh / py / js 脚本
├── references/       # 可选：参考文档
└── assets/           # 可选：静态资源
```

`SKILL.md` frontmatter 字段：

| 字段 | 必需 | 说明 |
|------|------|------|
| `name` | ✅ | 小写字母/数字/连字符，1-64 字符，必须与目录名一致 |
| `description` | ✅ | 功能描述，1-1024 字符 |
| `version` | 可选 | 语义化版本 |
| `license` | 可选 | SPDX 标识 |
| `compatibility` | 可选 | 环境要求说明 |

**多技能仓库**：一个 Git 仓库可以包含多个技能子目录（如 `vercel-labs/skills` 仓库内有数十个技能），安装时通过 `skillSubpath` 指定具体子目录。

---

## 四、核心存储设计

### 目录结构

```
~/.agents/
├── skills/                    # 中央技能仓库（唯一存储位置）
│   ├── android-cli/
│   │   └── SKILL.md
│   └── git-master/
│       └── SKILL.md
├── .skill-lock.json           # 锁文件：与 skilly 共用，记录安装元数据
├── sources.toml               # 技能源配置
└── agents.toml                # 已扫描/注册的 Agent 路径
```

> **与 skilly 的兼容性**：skm 与 skilly **共用同一锁文件** `~/.agents/.skill-lock.json`，字段名完全一致，实现双向数据互通（skm 装的技能 skilly 能即时感知，反之亦然）。

### 锁文件格式（`~/.agents/.skill-lock.json`）

**与 skilly 共用同一锁文件**，skm 读写该文件时字段名与 skilly 完全一致，实现双向数据互通（skm 装的技能 skilly 能感知，反之亦然）。

```json
{
  "skills": {
    "android-cli": {
      "source": "owner/repo",
      "sourceType": "github",
      "sourceUrl": "https://github.com/owner/repo.git",
      "skillSubpath": "android-cli",
      "skillyCommitHash": "7f4d2b8...",
      "installedAt": "2026-04-21T10:00:00Z",
      "updatedAt": "2026-04-21T10:00:00Z"
    }
  }
}
```

> 注意：字段名 `skillyCommitHash` 来自 skilly，skm 沿用以保持兼容。

原子写入：先写 `.skill-lock.json.tmp`，再 rename，防止写入中断损坏文件。

### agents.toml

```toml
# 由 `skm scan` 自动生成，可手动追加自定义条目
[agents]
claude-code = "~/.claude/skills"             # auto-detected: 2026-04-21
opencode    = "~/.config/opencode/skills"    # auto-detected: 2026-04-21
my-agent    = "~/.my-agent/skills"           # 手动添加
```

---

## 五、Agent 检测策略

**检测来源（任一为真即认为已安装）**：

```
command in PATH (which)
  || 配置目录存在 (e.g. ~/.claude/)
  || skills 目录存在 (e.g. ~/.claude/skills/)
  || skills 目录内有技能包 (含 SKILL.md 的子目录 > 0)
```

**内置 Agent 预设列表**（来自 skilly agent.rs，经过补充）：

| Agent ID | 显示名 | 检测命令 | skills 目录 |
|---------|--------|---------|------------|
| `claude-code` | Claude Code | `claude` | `~/.claude/skills` |
| `codex` | Codex | `codex` | `~/.codex/skills` |
| `gemini-cli` | Gemini CLI | `gemini` | `~/.gemini/skills` |
| `copilot-cli` | Copilot CLI | `gh` | `~/.copilot/skills` |
| `opencode` | OpenCode | `opencode` | `~/.config/opencode/skills`（Linux/macOS）|
| `cursor` | Cursor | `cursor` | `~/.cursor/skills` |
| `kiro` | Kiro | `kiro` | `~/.kiro/skills` |
| `codebuddy` | CodeBuddy | `codebuddy` | `~/.codebuddy/skills` |
| `openclaw` | OpenClaw | `openclaw` | `~/.openclaw/skills` |
| `trae` | Trae | `trae` | `~/.trae/skills` |
| `antigravity` | Antigravity | `antigravity` | `~/.gemini/antigravity/skills` |
| `junie` | Junie | `junie` | `~/.junie/skills` |
| `qoder` | Qoder | `qoder` | `~/.qoder/skills` |
| `trae-cn` | Trae CN | `trae-cn` | `~/.trae-cn/skills` |

> **特殊**：`opencode` 在 Windows 使用 `%APPDATA%/opencode/skills`。

---

## 六、功能需求（Phase 1）

### 6.1 技能源管理

| 功能 | 命令 | 说明 |
|------|------|------|
| 搜索技能 | `skm search <keyword>` | 从所有配置源搜索，展示名称、描述、安装量 |
| 添加源 | `skm source add <name> <url>` | 添加 Git 仓库源（GitHub / GitLab / 私有） |
| 删除源 | `skm source remove <name>` | 删除指定源 |
| 列出源 | `skm source list` | 展示所有技能源及状态 |

**默认源：`https://skills.sh`**（Vercel，9.1万+ 技能包）

### 6.2 Registry API 集成（来自 skilly 实现）

| 端点 | 用途 | 认证 |
|------|------|------|
| `GET {base}/api/search?q=...&limit=N` | 搜索技能，返回 JSON | ✅ 无需 |
| `GET {base}/` | 全时间榜（HTML scraping） | ✅ 无需 |
| `GET {base}/trending` | 趋势榜（HTML scraping） | ✅ 无需 |
| `GET {base}/hot` | 热榜（HTML scraping） | ✅ 无需 |
| GitHub raw URL | 获取 SKILL.md 内容预览 | ✅ 无需（公开仓库）|

- **全部端点均无需 API key**（经 skilly 实现验证：仅携带 `Accept` + `User-Agent` header）
- 请求 User-Agent 设置为 `skm/<version>`
- 缓存策略：榜单 5min TTL，技能内容 10min TTL（内存缓存）

### 6.3 Agent 管理

| 功能 | 命令 | 说明 |
|------|------|------|
| 扫描 | `skm scan` | 用四信号检测法扫描所有内置 Agent，追加写入 `agents.toml` |
| 查看 | `skm agent list` | 列出所有已注册 Agent（ID、路径、安装状态、技能数） |
| 手动注册 | `skm agent add <id> <path>` | 注册自定义 Agent |

**`skm scan` 幂等行为**：
- 已在 `agents.toml` 中的条目跳过（不覆盖手动配置）
- 新检测到的追加写入
- 支持 `--dry-run` 预览

### 6.4 安装与部署

| 功能 | 命令 | 说明 |
|------|------|------|
| 安装 | `skm install <name>` | 拉取到中央仓库，不创建软链接 |
| 安装并部署 | `skm install <name> --link-to <agent>` | 安装后软链到指定 Agent |
| 安装到所有 Agent | `skm install <name> --link-to all` | 安装后链到所有已注册 Agent |
| 从 Git 安装 | `skm install <owner/repo>[:<subpath>]` | GitHub 简写，subpath 支持多级路径；完整 URL 格式：`<git-url>[#subpath]`；支持直接粘贴 GitHub 网页地址 |
| 链接 | `skm link <name> <agent>` | 为已安装技能补充软链接 |
| 取消链接 | `skm unlink <name> <agent>` | 移除 Agent 的软链接，保留中央仓库 |

**安装状态三态**（参考 skilly）：
- `installed`：中央仓库存在，软链接有效
- `conflict`：目标路径已存在但不是本工具创建的软链接
- `not_installed`：中央仓库不存在

### 6.5 重新链接（核心场景：新装 Agent 后同步）

| 功能 | 命令 | 说明 |
|------|------|------|
| 全量重链接 | `skm relink` | 所有技能 → 所有已注册 Agent |
| 指定 Agent | `skm relink <agent>` | 所有技能 → 指定 Agent |
| 指定技能 | `skm relink --skill <name>` | 指定技能 → 所有 Agent |

**冲突处理（已确认行为）**：
- **默认**：遇到冲突（`conflict` 状态）跳过并报告，继续处理其余链接
- `--force`：强制覆盖所有冲突项
- `--backup`：覆盖前备份原文件（备份目录：`~/.agents/.skm-backups/`）
- `--dry-run`：预览所有操作，不实际执行

**典型使用场景**：
```bash
skm scan           # 新检测到 cursor → 追加到 agents.toml
skm relink cursor  # 所有已安装技能软链到 ~/.cursor/skills/
```

### 6.6 本地管理

| 功能 | 命令 | 说明 |
|------|------|------|
| 查看已安装 | `skm list` | 列出所有已安装技能及链接状态（链到哪些 Agent） |
| 查看详情 | `skm info <name>` | 显示技能详情：frontmatter、安装元数据、链接状态 |
| 更新检查 | `skm update --check [name]` | 对比本地/远程 commit hash，报告是否有更新 |
| 更新 | `skm update [name]` | 更新指定技能或全部技能（更新前自动备份） |
| 卸载 | `skm uninstall <name>` | 移除技能及所有 Agent 软链接 |

**更新机制**（来自 skilly）：
- 基于 Git：对比本地 commit hash 与远程最新 commit hash
- 更新状态：`up_to_date` | `has_update` | `unsupported` | `not_tracked` | `error`
- 未通过 Git 安装的技能（`local` sourceType）：报 `unsupported`

### 6.7 备份与回滚（参考 skilly）

| 功能 | 命令 | 说明 |
|------|------|------|
| 列出备份 | `skm backup list <name>` | 查看某技能的所有快照 |
| 恢复备份 | `skm backup restore <name> [snapshot-id]` | 恢复到指定快照，默认最新 |
| 删除备份 | `skm backup delete <name> <snapshot-id>` | 删除指定快照 |

备份目录：`~/.agents/.skm-backups/<skill-name>/<snapshot-id>/`

---

## 七、非功能需求

| 维度 | 要求 |
|------|------|
| **语言** | Rust |
| **平台** | Linux / macOS（Phase 1）；Windows 暂不支持（软链接需管理员权限，Phase 2 再议） |
| **性能** | 并行下载；scan/list 响应迅速；Registry 响应本地缓存 |
| **可靠性** | 文件操作事务化（tmp + rename）；失败自动清理 |
| **安全** | 下载包哈希校验；支持全局 `--dry-run` |
| **兼容性** | 兼容 skills.sh / SKILL.md 标准；与 skilly 锁文件完全兼容 |
| **可扩展** | 技能源协议、包解析器、Agent 定义模块化 |

---

## 八、核心工作流

### install + link
```
skm install android-cli --link-to opencode
  ├─ 1. 从 skills.sh /api/search 查询 android-cli 元数据
  ├─ 2. 解析 GitHub 仓库 URL 和 skillSubpath
  ├─ 3. git clone / git pull → ~/.agents/skills/android-cli/
  ├─ 4. 验证 SKILL.md 存在且格式合法
  ├─ 5. 写入 .skill-lock.json（source/skillyCommitHash/timestamps）
  └─ 6. opencode → ~/.config/opencode/skills/
         创建软链接：
         ~/.config/opencode/skills/android-cli
           -> ~/.agents/skills/android-cli
```

### scan + relink
```
skm scan
  ├─ 对每个内置 Agent 执行 4-signal 检测
  ├─ 检测到 ~/.cursor/skills 存在 → agents.toml 追加 cursor
  └─ 输出：1 new agent detected: cursor

skm relink cursor
  ├─ 读取 ~/.agents/skills/ 枚举所有技能
  ├─ 对每个技能：conflict check → 创建软链接
  └─ 输出：12 linked, 0 conflicts, 0 skipped
```

### update
```
skm update android-cli
  ├─ 读取 .skill-lock.json → 获取本地 skillyCommitHash
  ├─ 查询 GitHub API → 获取远程最新 commitHash
  ├─ 有更新 → 备份当前版本到 .skm-backups/
  ├─ git pull
  └─ 更新 .skill-lock.json skillyCommitHash + updatedAt
```

---

## 九、整合策略

### skills.sh Registry
- 搜索 API：`GET /api/search?q=...&limit=N`（无需认证）
- 榜单：HTML scraping（`/`、`/trending`、`/hot`，公开）
- 技能内容预览：GitHub raw URL 直接获取 SKILL.md

### 自定义 Git 源
- `skm source add https://github.com/my-org/my-skills`
- 安装时 clone，支持 `owner/repo:subpath` 语法
- 支持多技能仓库：`skm discover owner/repo` 列出所有子技能

---

## 十、Phase 2 扩展

- `skm create` — 技能打包 & 发布向导
- `skm export / import` — ZIP 归档导出/导入（参考 skilly）
- 技能依赖关系解析
- 版本锁文件（`skm.lock`）
- TUI 交互界面
- Windows 支持（directory junction 方案）

---

## 十一、自我升级（Backlog，主体功能完成后实现）

`skm` 自身支持升级命令：

```bash
skm self-update          # 检查并升级到最新版
skm self-update --check  # 仅检查是否有新版本，不升级
```

升级实现：从 GitHub Releases 下载最新预编译二进制，替换当前可执行文件。  
**优先级**：最后实现，不阻塞其他功能。

---

## 十二、发行与安装

**发行仓库：`https://github.com/mocikadev/mocika-skills-cli`**（源码、预编译二进制、安装脚本均托管于此）

### 分发产物

GitHub Releases 上传各平台预编译二进制（文件名使用 Rust target triple 格式）：

| 目标 | 文件名 |
|------|--------|
| Linux x86_64 (musl 静态) | `skm-x86_64-unknown-linux-musl` |
| Linux aarch64 (musl 静态) | `skm-aarch64-unknown-linux-musl` |
| macOS x86_64 | `skm-x86_64-apple-darwin` |
| macOS aarch64 (Apple Silicon) | `skm-aarch64-apple-darwin` |
| Windows x86_64 | 计划中（Phase 2） |

每次发布同时附带 `SHA256SUMS.txt` 校验文件。

### Linux / macOS — curl 脚本（主推）

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

脚本行为：
- 自动检测 OS + CPU 架构，下载对应二进制
- **默认安装到 `~/.local/bin/skm`（无需 root / sudo）**
- 支持环境变量覆盖：
  - `SKM_INSTALL_DIR=/usr/local/bin` — 自定义安装目录
  - `SKM_VERSION=v0.2.0` — 指定版本，默认 latest
- `~/.local/bin` 不在 `$PATH` 时自动提示追加到 shell rc 文件

### macOS — Homebrew（可选，待 Phase 2）

```bash
brew install mocikadev/tap/skm
```

tap 仓库：`https://github.com/mocikadev/homebrew-tap`

### Windows — PowerShell 脚本（Phase 2）

```powershell
irm https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.ps1 | iex
```

### cargo install（备选，需本地有 Rust 工具链）

```bash
cargo install skm
```

---

## 十三、skm-skill（配套技能包）

### 设计思路

参考 `android-cli` 的模式：CLI 工具本身发布一个配套 skill，AI Agent 装了这个 skill 后，就能理解工具的所有命令，进而代替用户操作。

`skm` 同样提供一个 **`skm-skill`** 技能包，发布在独立仓库（`https://github.com/mocikadev/skm-skill`）：

```
skm-skill/
├── SKILL.md        # 描述 skm 的安装方式 + 完整命令参考
└── references/
    └── commands.md # 子命令详细说明（类比 android-cli/references/）
```

### SKILL.md 内容规划

- **安装 skm 本身**：给出 curl 安装脚本 / cargo install 命令
- **完整命令速查**：`skm install`、`skm scan`、`skm relink`、`skm update` 等全部子命令
- **典型工作流**：初次设置、新装 Agent 后同步、批量更新等场景示例

### 自举循环

```
1. 用户安装 skm：
   curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
2. skm install mocikadev/skm-skill --link-to all
   → AI Agent 学会了所有 skm 命令
3. 用户对 AI 说："帮我把 android-cli 装到所有 Agent"
   → AI 执行 skm install android-cli --link-to all
```

### 实现时机

与 CLI 主体同步开发，CLI 命令基本稳定后输出 `skm-skill` 的 `SKILL.md`。
