# skm 命令参考

> 版本：0.3  
> 二进制：`skm`（Windows 为 `skm.exe`）

---

## 安装

**Linux / macOS**

```bash
curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
```

**Windows**（PowerShell，需开启 Developer Mode 或以管理员运行）

```powershell
irm https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.ps1 | iex
```

---

## 全局用法

```
skm <COMMAND> [OPTIONS]

skm --help
skm --version
skm <COMMAND> --help
```

---

## 技能管理

### `skm install`

从注册表或 Git 仓库安装技能。

```
skm install <NAME> [--link-to <AGENT>]
```

| 参数 | 说明 |
|------|------|
| `<NAME>` | 见下方"支持的 NAME 格式" |
| `--link-to <AGENT>` | 安装后软链到指定 Agent，填 `all` 链接所有已注册 Agent（**默认 `all`**，省略即链接全部） |

**支持的 NAME 格式**

| 格式 | 说明 | 示例 |
|------|------|------|
| `<skill-name>` | 从已配置注册表按名称搜索安装 | `mobile-android-design` |
| `owner/repo` | GitHub 仓库根目录（自动补全 `https://github.com/`） | `wshobson/agents` |
| `owner/repo:subpath` | GitHub 仓库子目录，subpath 可含多级 `/` | `wshobson/agents:mobile-android-design` |
| `owner/repo:dir/subdir` | GitHub 仓库多级子目录 | `myorg/skills:tools/formatter` |
| `<git-url>` | 完整 Git URL，安装仓库根目录（支持 https / ssh） | `https://github.com/wshobson/agents.git` |
| `<git-url>#subpath` | 完整 Git URL + `#` 指定子目录 | `https://github.com/myorg/skills.git#tools/formatter` |
| GitHub 网页 URL | 直接粘贴 GitHub 页面地址，自动解析仓库和子目录 | `https://github.com/myorg/skills/tree/main/formatter` |
| `/absolute/path` | 本地绝对路径 | `/home/user/my-skills` |
| `~/path` | 本地 `~` 展开路径 | `~/dev/company-skills` |
| `./relative` / `../relative` | 本地相对路径 | `./skills` |
| `file:///path` | file:// URI | `file:///home/user/my-skills` |

> `owner/repo` 格式固定解析为 GitHub（`github.com`）。  
> GitLab、Gitee 等平台请使用完整 Git URL 格式。  
> 本地路径直接复制文件，无需 git。

**示例**

```bash
# 从注册表安装（默认链接到所有 Agent）
skm install mobile-android-design

# 安装并链接到指定 Agent
skm install mobile-android-design --link-to opencode
skm install mobile-android-design --link-to all

# GitHub 简写（子目录，单级 / 多级均支持）
skm install mocikadev/mocika-skills-cli:skills/skm --link-to all
skm install wshobson/agents:mobile-android-design
skm install myorg/skills:tools/formatter

# 完整 Git URL（仓库根目录）
skm install https://github.com/wshobson/agents.git --link-to opencode
skm install git@github.com:wshobson/agents.git

# 完整 Git URL + 子目录（# 分隔）
skm install https://github.com/myorg/skills.git#tools/formatter

# 直接粘贴 GitHub 网页地址（含 /tree/branch/path）
skm install https://github.com/myorg/skills/tree/main/tools/formatter

# 本地路径（直接复制，无需 git）
skm install ~/dev/my-company-skills
skm install ~/dev/multi-skills#formatter       # 本地多 skill 仓库 + 子目录
skm install /workspace/skills --link-to opencode
skm install ./local-skills                     # 相对路径
```

**成功输出**

```
installed mobile-android-design (mobile-android-design)
linked to: opencode, shared
```

---

### `skm uninstall`

卸载技能，移除中央仓库目录及所有 Agent 软链接。

```
skm uninstall <NAME>
```

**示例**

```bash
skm uninstall mobile-android-design
```

**成功输出**

```
uninstalled mobile-android-design
```

---

### `skm search`

在已配置的所有注册表中搜索技能。`skills.sh` 源通过 HTTP API 搜索；GitHub / Git 源通过本地缓存扫描 SKILL.md 匹配关键词（首次需 `git clone`，之后 5 分钟内命中缓存，超时增量 `git fetch`）；本地源直接扫描本地目录。

```
skm search <KEYWORD> [--limit <N>]
```

| 参数 | 说明 |
|------|------|
| `<KEYWORD>` | 搜索关键词 |
| `--limit <N>` | 最多显示结果数（默认 20） |

**示例**

```bash
skm search android
skm search android --limit 5
```

**输出格式**：`<name>  <安装量>  <描述>`

---

### `skm list`

列出所有已安装技能及其链接状态。

```
skm list [--outdated]
```

| 参数 | 说明 |
|------|------|
| `--outdated` | 只显示有可用更新的技能（会发起网络请求） |

**示例**

```bash
skm list
skm list --outdated
```

**输出格式**：`<id>  <displayName>  <链接的Agent列表>`

---

### `skm info`

显示技能详细信息（frontmatter、lock 条目、链接状态）。

```
skm info <NAME>
```

**示例**

```bash
skm info mobile-android-design
```

---

### `skm update`

更新已安装技能到最新版本（更新前自动创建备份）。

```
skm update [NAME] [--all] [--check]
```

| 参数 | 说明 |
|------|------|
| `NAME` | 技能名称（与 `--all` 互斥） |
| `--all` | 显式更新全部已安装技能 |
| `--check` | 仅检查是否有更新，不实际执行 |

**示例**

```bash
skm update                          # 更新全部（等同 --all）
skm update --all                    # 更新全部（显式）
skm update mobile-android-design    # 更新指定技能
skm update --check                  # 检查全部更新
skm update --check mobile-android-design
```

**成功输出**

```
✓ updated mobile-android-design
updated: 1, failed: 0
```

---

### `skm link`

为已安装的技能在指定 Agent 目录下创建软链接。

```
skm link <NAME> <AGENT>
```

**示例**

```bash
skm link mobile-android-design opencode
```

**成功输出**

```
linked mobile-android-design → opencode
```

---

### `skm unlink`

移除指定 Agent 目录下的技能软链接（不删除中央仓库）。

```
skm unlink <NAME> <AGENT>
```

**示例**

```bash
skm unlink mobile-android-design opencode
```

**成功输出**

```
unlinked mobile-android-design from opencode
```

---

### `skm relink`

将所有已安装技能重新软链接到 Agent 目录，适用于新装 Agent 后的批量同步。

```
skm relink [AGENT] [--skill <NAME>] [--force] [--backup] [--dry-run]
```

| 参数 | 说明 |
|------|------|
| `AGENT` | 目标 Agent ID（省略则处理所有已注册 Agent） |
| `--skill <NAME>` | 仅重新链接指定技能 |
| `--force` | 覆盖冲突路径（非 skm 管理的软链接或文件） |
| `--backup` | 覆盖前备份冲突文件（需配合 `--force`） |
| `--dry-run` | 预览操作，不实际执行 |

**典型场景**

```bash
skm scan                   # 新检测到 cursor
skm relink cursor          # 将所有已安装技能链到 ~/.cursor/skills/
```

**成功输出**

```
relink 12 linked, 0 conflicts, 1 skipped
```

冲突警告（默认跳过）：
```
warn conflict: mobile-android-design in cursor (skipped, use --force to overwrite)
```

---

## 注册表管理

### `skm source list`

列出所有配置的技能注册表，输出格式：`<name>  <url>  enabled=<true|false>  type=<skills.sh|github|git>`

```
skm source list
```

### `skm source add`

添加自定义技能注册表。URL 会自动检测类型：

| URL / 路径形式 | 类型 | 搜索方式 |
|----------------|------|----------|
| `https://skills.sh` | `skills.sh` | HTTP API |
| `https://github.com/…` / `git@github.com:…` / `owner/repo` | `github` | git clone + 扫描（本地缓存） |
| 其他 `https://` / `git@` URL（GitLab、私有 Git 等） | `git` | git clone + 扫描（本地缓存） |
| `/绝对路径`、`~/路径`、`./相对路径`、`../..`、`file://` | `local` | 直接扫描本地目录 |

```
skm source add <NAME> <URL|PATH>
```

**示例**

```bash
skm source add my-org https://github.com/my-org/my-skills
skm source add private git@github.com:myorg/private-skills
skm source add gitlab-skills https://gitlab.com/myorg/skills.git

# 本地目录作为源（支持 ~、绝对路径、相对路径）
skm source add dev ~/dev/my-company-skills
skm source add local-ws /workspace/shared-skills
```

### `skm source remove`

移除已配置的注册表。

```
skm source remove <NAME>
```

**配置文件**：`~/.agents/sources.toml`

---

## Agent 管理

### `skm scan`

自动检测本机已安装的 AI Agent，将结果写入 `agents.toml`（已存在的条目不覆盖）。

```
skm scan [--dry-run]
```

| 参数 | 说明 |
|------|------|
| `--dry-run` | 仅预览检测结果，不写入配置 |

检测采用四信号机制（任一为真即认为已安装）：
1. 命令存在于 `PATH`
2. 配置目录存在（如 `~/.claude/`）
3. Skills 目录存在（如 `~/.claude/skills/`）
4. Skills 目录内有技能包

**内置支持的 Agent**：`claude-code`、`codex`、`gemini-cli`、`copilot-cli`、`opencode`、`antigravity`、`cursor`、`kiro`、`codebuddy`、`openclaw`、`trae`、`junie`、`qoder`、`trae-cn`、`windsurf`、`augment`、`kilocode`、`ob1`、`amp`、`hermes`、`factory-droid`、`qwen`

### `skm agent list`

列出所有已注册 Agent 及其安装状态与技能数量。

```
skm agent list
```

### `skm agent add`

手动注册自定义 Agent。

```
skm agent add <ID> <PATH>
```

**示例**

```bash
skm agent add my-agent ~/.my-agent/skills
```

**配置文件**：`~/.agents/agents.toml`

---

## 备份管理

### `skm backup list`

列出备份快照。不指定技能名则列出全部，按技能名分组显示。

```
skm backup list [NAME]
```

**输出格式**：
- 无参数：按技能名分组，每行 `<snapshot-id>  <创建时间>`
- 指定名称：`<snapshot-id>  <创建时间>  <备份路径>`

### `skm backup restore`

从备份快照恢复技能（默认恢复最新快照）。

```
skm backup restore <NAME> [SNAPSHOT_ID]
```

**示例**

```bash
skm backup restore mobile-android-design
skm backup restore mobile-android-design 1776758731056
```

### `skm backup delete`

删除指定备份快照。

```
skm backup delete <NAME> <SNAPSHOT_ID>
```

**备份目录**：`~/.agents/.skm-backups/<skill-name>/<snapshot-id>/`

---

## 数据目录速查

| 路径 | 用途 |
|------|------|
| `~/.agents/skills/` | 中央技能仓库（唯一存储位置） |
| `~/.agents/.skill-lock.json` | 安装元数据锁文件（与 skilly 共用） |
| `~/.agents/sources.toml` | 注册表源配置 |
| `~/.agents/agents.toml` | 已注册 Agent 路径配置 |
| `~/.agents/.skm-backups/` | 技能备份目录 |
| `~/.agents/.skm-source-cache/` | Git / GitHub 源本地缓存（5 分钟 TTL，超时增量 fetch） |

---

## 配置管理

### `skm config lang`

查看或设置界面显示语言。

```
skm config lang [CODE|--reset]
```

| 参数 | 说明 |
|------|------|
| `CODE` | 语言代码：`zh`（中文）或 `en`（英文） |
| `--reset` | 重置为自动检测（从 `$LANG` 环境变量读取） |

**示例**

```bash
skm config lang          # 查看当前语言设置
skm config lang zh       # 切换为中文
skm config lang en       # 切换为英文
skm config lang --reset  # 重置为随系统语言
```

**配置文件**：`~/.agents/skm.toml`

---

## 自我升级

### `skm self-update`

将 skm 本身升级到最新版本。

```
skm self-update [--check]
```

| 参数 | 说明 |
|------|------|
| `--check` | 仅检查是否有新版本，不实际升级 |

**示例**

```bash
skm self-update          # 检查并升级到最新版
skm self-update --check  # 仅查看是否有可用更新
```

升级流程：从 GitHub Releases 下载对应平台的预编译二进制，校验 SHA256 后原子替换当前可执行文件。

---

## 环境诊断

### `skm doctor`

检测环境健康状态，诊断常见问题。

```
skm doctor
```

检查项：
- **环境**：共享技能目录是否存在、锁文件和 agents.toml 是否可读
- **Agent**：agents.toml 中注册的每个 Agent 是否仍已安装（失效条目提示运行 `skm scan` 清理）
- **链接**：锁文件中的每个技能在每个注册 Agent 下的链接状态（已链接 / 未链接 / 冲突）

**退出码**：所有检查通过返回 0，存在问题返回 1。

**示例**

```bash
skm doctor
```

**示例输出**

```
Environment
  ✓  shared skills dir    exists
  ✓  lock file            readable
  ✓  agents.toml          readable

Agents (2)
  ✓  opencode             installed
  ✗  kiro                 not installed  (run `skm scan` to clean up)

Links (1 issue(s))
  ✓  rust-skills           opencode    linked
  ✗  rust-skills           kiro        not linked
```
