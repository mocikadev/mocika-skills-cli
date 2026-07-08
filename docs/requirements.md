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
| `atomcode` | AtomCode | `atomcode` | `~/.atomcode/skills` |
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

### 10.1 export / import（技能集分享）

| 功能 | 命令 | 说明 |
|------|------|------|
| 导出 | `skm export [--output <file>]` | 将所有已安装技能（Git/GitHub/注册表来源）导出为 `skills.bundle` 文件 |
| 导入 | `skm import <file> [--link-to <agent\|all>] [--force]` | 从 bundle 文件批量安装技能 |

**设计决策**：

- **文件格式**：TOML（`.bundle` 扩展名），人类可读可手写
- **默认文件名**：`skills.bundle`（当前目录）
- **导出范围**：仅 Git / GitHub / 注册表来源；本地路径来源自动跳过（不报错）
- **导入幂等性**：已安装的技能默认跳过，`--force` 强制覆盖
- **导入默认链接**：`--link-to all`（与 `skm install` 一致）

**bundle 文件格式**：

```toml
[meta]
exported_by = "skm/0.3.2"
exported_at = "2026-05-11"

[[skills]]
name = "mobile-android-design"
source = "wshobson/agents:mobile-android-design"

[[skills]]
name = "rust-skills"
source = "someuser/rust-skills"
```

**典型场景**：

```bash
# 导出当前环境
skm export --output ~/dotfiles/skills.bundle

# 在新机器上一键还原
skm import ~/dotfiles/skills.bundle
```

### 10.2 doctor sync（链接一致性修复）

| 功能 | 命令 | 说明 |
|------|------|------|
| 链接诊断修复 | `skm doctor sync [--agent <id>] [--dry-run]` | 检查所有 agent 目录，找出 broken/orphan 软链接并自动修复 |

**检测的问题类型**：

| 状态 | 描述 | 自动处理 |
|------|------|------|
| `broken` | 软链接目标不存在（中央仓库已删）| 删除悬空软链接 |
| `orphan` | 中央仓库有 skill 但 agent 无链接 | 补建软链接 |
| `conflict` | 目标路径存在但非 skm 创建的软链接 | 报告，不自动处理（需 `--force`）|

`--dry-run`：预览所有操作，不实际执行。

**典型场景**：手动删除了中央仓库某个 skill 后，各 agent 目录还残留悬空软链接；或者锁文件和磁盘状态不一致时。

---

### 10.3 doctor fix-skills（SKILL.md 内容修复）

| 功能 | 命令 | 说明 |
|------|------|------|
| frontmatter 修复 | `skm doctor fix-skills [--dry-run]` | 扫描中央仓库所有 skill，自动修复 SKILL.md frontmatter 格式问题 |

**可修复的问题**：

- 缺少 `---` 分隔符
- frontmatter 不是合法 YAML
- 缺少必需字段（`name`、`description`）
- `name` 字段与目录名不一致

`--dry-run`：仅报告问题，不写文件。

---

### 10.4 scan --import（扫描后批量导入 bundle）

| 功能 | 命令 | 说明 |
|------|------|------|
| 扫描并导入 | `skm scan --import <FILE> [--agent <id>] [--dry-run]` | 先执行 agent 扫描，写入 `agents.toml`，再从指定 bundle 文件批量安装技能并链接 |

**与普通 `skm scan` 的区别**：

- `skm scan`（基础）：仅检测本机已安装的 Agent，写入 `agents.toml`
- `skm scan --import <FILE>`（扩展）：完成扫描后，额外执行 `skm import <FILE>`，自动把 bundle 中的技能安装并链接到所有 Agent

**典型场景**：在全新机器上快速重建环境——先扫描检测 Agent，再从备份 bundle 一次性恢复全部技能。

**行为说明**：
- `--dry-run` 时仅执行扫描，跳过 import 步骤，不写入任何文件
- import 阶段遵循幂等规则：已安装的技能跳过，除非指定 `--force`
- `<FILE>` 为 `skm export` 生成的 TOML bundle 文件（`[[skills]]` 格式）

> **注意**：此功能与"从 agent 目录回收已有 skill 纳入 skm 管理"是两件不同的事，后者规划为 `doctor adopt`（见 10.6）。

---

### 10.5 lock 文件增强（hash 校验字段）

当前 lock 文件缺少内容校验字段，补充以下字段以支持更可靠的更新检测和完整性校验：

| 字段 | 类型 | 说明 |
|------|------|------|
| `computedHash` | `String` | 安装时对 skill 目录内容计算的 SHA256（用于本地内容完整性校验） |
| `remoteTreeSha` | `String` | 安装时记录的远程 Git tree SHA（用于加速更新检测，避免全量 clone） |

**影响的命令**：
- `skm install`：安装后写入两个字段
- `skm update --check`：优先用 `remoteTreeSha` 做远程对比（比 `rev-parse` 更快）
- `skm doctor`：可用 `computedHash` 检测本地内容是否被意外修改

---

### 10.6 其他规划功能

- **`doctor adopt [agent]`**（规划中）：扫描指定（或所有）agent 目录，将目录中已有的 skill 复制到中央仓库并替换为软链接，实现"零重装接管"。适用场景：用户此前手动把 skill 放进 `~/.claude/skills/`，现在想让 skm 统一管理。命令形式：`skm doctor adopt [--agent <id>] [--dry-run]`
- **`skm create`** — 技能打包 & 发布向导
- 技能依赖关系解析
- TUI 交互界面

---

## 十一、自我升级

`skm` 自身支持升级命令：

```bash
skm self-update          # 检查并升级到最新版
skm self-update --check  # 仅检查是否有新版本，不升级
```

升级实现：从 GitHub Releases 下载最新预编译二进制，校验 SHA256 后原子替换当前可执行文件。

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

## 十三、skm skill（配套技能包）

### 设计思路

参考 `android-cli` 的模式：CLI 工具本身发布一个配套 skill，AI Agent 装了这个 skill 后，就能理解工具的所有命令，进而代替用户操作。

`skm` 在本仓库内维护配套 skill，路径为 `skills/skm/`：

```
skills/skm/
├── SKILL.md        # 描述 skm 的安装方式 + 完整命令参考
├── README.md       # 技能包说明（中文）
└── README.en.md    # 技能包说明（英文）
```

### 安装命令

```bash
skm install mocikadev/mocika-skills-cli:skills/skm --link-to all
```

### SKILL.md 内容规划

- **安装 skm 本身**：给出 curl 安装脚本 / cargo install 命令
- **完整命令速查**：`skm install`、`skm scan`、`skm relink`、`skm update` 等全部子命令
- **典型工作流**：初次设置、新装 Agent 后同步、批量更新等场景示例

### 自举循环

```
1. 用户安装 skm：
   curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
2. skm install mocikadev/mocika-skills-cli:skills/skm --link-to all
   → AI Agent 学会了所有 skm 命令
3. 用户对 AI 说："帮我把 android-cli 装到所有 Agent"
   → AI 执行 skm install android-cli --link-to all
```
