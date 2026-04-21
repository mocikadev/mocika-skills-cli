# skm 技术设计文档

> 版本：0.1  
> 更新：2026-04-21

---

## 一、技术选型

### 1.1 语言 & 工具链

| 项目 | 选择 | 理由 |
|------|------|------|
| 语言 | Rust 2021 edition | 性能、跨平台、二进制分发 |
| 包管理 | Cargo | 标准工具链 |
| Rust 版本 | MSRV 1.80（稳定） | 覆盖主流发行版自带 Rust 版本 |

### 1.2 关键选型决策

#### 同步 vs 异步
**选择：同步（blocking）+ 标准库线程**

- CLI 工具无高并发需求，每次只执行一条命令
- `tokio` 增加编译时间和二进制体积
- 并行场景（批量 relink、批量 update check）用 `std::thread::spawn` + `rayon` 处理
- 与 skilly 核心层保持一致（skilly 使用 `reqwest::blocking`）

#### Git 操作
**选择：shell out 到系统 `git` 命令**

- 避免引入 `git2`（libgit2 静态链接 +400KB 以上）
- 与 skilly 一致（`Command::new("git")`）
- 前置检查：启动时验证 `git` 是否在 PATH，不在则报错退出

---

## 二、依赖清单

### 2.1 Cargo.toml 依赖

```toml
[dependencies]
# CLI 框架
clap = { version = "4", features = ["derive", "color"] }

# 错误处理
anyhow  = "1"
thiserror = "2"

# 序列化
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
toml       = "0.8"
serde_yaml = "0.9"          # SKILL.md frontmatter 解析

# HTTP（blocking，不引入 tokio）
reqwest = { version = "0.12", default-features = false, features = [
  "blocking",
  "json",
  "rustls-tls",             # 不依赖系统 OpenSSL，静态链接 TLS
] }

# 路径/系统
dirs = "5"                  # home_dir / config_dir 跨平台

# 输出体验
indicatif = "0.17"          # 进度条（install / update）
console   = "0.15"          # 终端颜色、样式

# 并行（批量操作）
rayon = "1"

# 正则（Registry HTML scraping）
regex = "1"

# 时间
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile   = "3"            # 测试用临时目录
assert_cmd = "2"            # CLI 集成测试
predicates = "3"            # 断言
```

### 2.2 依赖选型说明

| 依赖 | 作用 | 备注 |
|------|------|------|
| `clap` derive | CLI 参数解析 | v4 derive 宏，编译时生成 |
| `anyhow` | 应用层错误处理 | 二进制不需要 `thiserror` 的细粒度 |
| `thiserror` | 核心模块的结构化错误类型 | 给 `core/` 模块用 |
| `reqwest` blocking + rustls | HTTP，无系统 OpenSSL 依赖 | 跨平台静态 TLS |
| `serde_yaml` | SKILL.md frontmatter（YAML）解析 | 比 `gray_matter` 更轻量 |
| `rayon` | 并行迭代（批量链接创建） | 替代 tokio 的并行方案 |
| `indicatif` | git clone/pull 进度展示 | 参考 skilly 无进度条的痛点 |

---

## 三、模块架构

遵循 `proj-lib-main-split`：`main.rs` 只负责入口，所有逻辑在 `lib.rs` + 子模块。

```
src/
├── main.rs              # 入口：解析参数，调用 run()，处理顶层错误输出
├── lib.rs               # pub use 重导出，wire CLI → core
│
├── cli/                 # 命令处理层（薄层，负责 I/O 呈现）
│   ├── mod.rs           # Commands enum + dispatch
│   ├── install.rs       # skm install
│   ├── search.rs        # skm search
│   ├── scan.rs          # skm scan
│   ├── relink.rs        # skm relink
│   ├── update.rs        # skm update
│   ├── list.rs          # skm list / info
│   ├── source.rs        # skm source add/remove/list
│   ├── agent.rs         # skm agent list/add
│   ├── backup.rs        # skm backup list/restore/delete
│   └── self_update.rs   # skm self-update（Backlog）
│
├── core/                # 业务逻辑层（纯逻辑，不直接写终端）
│   ├── mod.rs
│   ├── agent.rs         # Agent 检测（4-signal）+ agents.toml 读写
│   ├── lock.rs          # .skill-lock.json 原子读写（对齐 skilly）
│   ├── registry.rs      # skills.sh API 集成 + 内存缓存
│   ├── skill.rs         # 技能扫描、SKILL.md 解析、链接状态检测
│   ├── operations.rs    # install / link / unlink / uninstall 核心逻辑
│   ├── update.rs        # git-based 更新检查与执行
│   ├── backup.rs        # 快照创建、恢复、清理
│   ├── config.rs        # sources.toml + agents.toml 结构体与 I/O
│   └── git.rs           # git 命令封装（clone / pull / rev-parse 等）
│
├── models.rs            # 跨层共享数据类型（SkillSummary、AgentStatus 等）
└── error.rs             # 自定义错误类型（thiserror）
```

### 3.1 层级职责

```
main.rs
  └── cli/          ← 解析 clap 参数，调用 core，格式化输出（颜色/进度条）
        └── core/   ← 纯业务逻辑，返回 Result<T>，不直接 println!
              └── models.rs / error.rs
```

**CLI 层**负责：
- `indicatif` 进度条启停
- `console` 颜色渲染
- 错误信息人性化展示

**Core 层**负责：
- 所有文件 I/O、网络请求、git 操作
- 返回结构化 `Result<T, SkmError>`
- 不依赖终端输出，方便测试

---

## 四、核心模块设计要点

### 4.1 `core/agent.rs`

直接参考 skilly `src-tauri/src/core/agent.rs` 的实现：

```rust
pub struct AgentDefinition {
    pub id:             &'static str,
    pub display_name:   &'static str,
    pub detect_command: &'static str,
}

// 4-signal 检测：任一为真即视为已安装
fn is_installed(cmd: &str, config_dir: &Path, skills_dir: &Path) -> bool {
    which::which(cmd).is_ok()
        || config_dir.exists()
        || skills_dir.exists()
        || count_skill_dirs(skills_dir) > 0
}
```

内置 14 个 Agent 定义（同 skilly，含 junie / qoder / trae-cn 补充项）。

### 4.2 `core/lock.rs`

与 skilly `~/.agents/.skill-lock.json` **完全兼容**：
- 文件路径、JSON 结构、字段名（`skillyCommitHash`）一致
- 原子写入：`write(tmp_path)` → `rename(tmp_path, lock_path)`

### 4.3 `core/registry.rs`

复用 skilly 实现思路：
- `/api/search?q=...&limit=N` → JSON 直接反序列化
- `/`、`/trending`、`/hot` → HTML scraping（同 skilly 的 regex 方案）
- 内存缓存：`std::sync::OnceLock<Mutex<HashMap<String, CacheEntry>>>`
  - 榜单 TTL：300 秒
  - 技能内容 TTL：600 秒

### 4.4 `core/git.rs`

封装 `std::process::Command` 调用 git：

```rust
pub fn clone(url: &str, dest: &Path) -> Result<()>
pub fn pull(repo_dir: &Path) -> Result<()>
pub fn current_commit(repo_dir: &Path) -> Result<String>   // rev-parse HEAD
pub fn remote_commit(repo_dir: &Path) -> Result<String>    // ls-remote HEAD
pub fn is_git_repo(dir: &Path) -> bool
```

前置检测：`which::which("git").is_err()` → 提前报错，给出安装提示。

### 4.5 `core/skill.rs`

SKILL.md 解析（YAML frontmatter + Markdown body）：

```rust
pub struct SkillFrontmatter {
    pub name:          String,
    pub description:   String,
    pub version:       Option<String>,
    pub license:       Option<String>,
    pub compatibility: Option<String>,
}

// 解析逻辑：按 "---\n" 切割，取第一个块用 serde_yaml 反序列化
pub fn parse_skill_md(content: &str) -> Result<(SkillFrontmatter, String)>
```

链接状态检测：

```rust
pub enum LinkState {
    Linked,     // 软链接存在且指向中央仓库
    Conflict,   // 目标路径存在但非本工具创建的软链接
    NotLinked,  // 目标路径不存在
}
```

---

## 五、Cargo.toml 构建配置

```toml
[package]
name    = "skm"
version = "0.1.0"
edition = "2021"
rust-version = "1.80"

[profile.release]
opt-level      = 3
lto            = "fat"       # 全程序链接优化，最小化二进制
codegen-units  = 1           # 单 codegen unit，允许最大优化
panic          = "abort"     # 移除 unwind 代码，缩小体积
strip          = true        # 剥离调试符号

[profile.dev]
opt-level = 0
debug     = true

[profile.dev.package."*"]
opt-level = 3               # 依赖包在 dev 模式下也优化，加快编译体验

[profile.release-with-debug]   # CI 用：有调试符号方便 profiling
inherits = "release"
strip    = false
debug    = true
```

---

## 六、Lint 配置（`src/lib.rs` 顶部）

```rust
#![deny(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(missing_docs)]
```

`.cargo/config.toml`：
```toml
[build]
rustflags = ["-D", "warnings"]
```

---

## 七、错误处理策略

```
core/ 模块   → thiserror 定义具体错误类型（SkmError）
cli/ 模块    → anyhow::Result，用 .context() 补充上下文
main.rs      → 捕获顶层错误，格式化为人性化输出（红色 error: 前缀）
```

禁止：
- `unwrap()` / `expect()` 出现在 `core/` 任何路径
- 空 catch（`let _ = ...` 吞掉错误）

---

## 八、测试策略

| 层级 | 工具 | 位置 |
|------|------|------|
| 单元测试 | `#[cfg(test)]` + `tempfile` | `src/core/*.rs` 内嵌 |
| CLI 集成测试 | `assert_cmd` + `predicates` | `tests/` |
| 核心逻辑关键路径 | 属性测试（proptest，可选） | `tests/` |

重点测试：
- `lock.rs`：并发读写安全、文件损坏恢复
- `skill.rs`：SKILL.md 各种合法/非法格式
- `agent.rs`：4-signal 检测各组合
- `operations.rs`：冲突检测三态、dry-run 不写磁盘

---

## 九、CI/CD

已实现，见 `.github/workflows/`：

**`ci.yml`**（触发：push to main + PR）
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
4. 四个目标并行 build 验证

**`release.yml`**（触发：`v*` tag push）
- 四个目标并行编译，汇聚后创建 GitHub Release
- 附带 `SHA256SUMS.txt` 校验文件，自动生成 release notes

**编译目标**（Phase 1，Linux/macOS）：

| 目标 triple | Runner |
|-------------|--------|
| `x86_64-unknown-linux-musl` | ubuntu-latest + cross |
| `aarch64-unknown-linux-musl` | ubuntu-latest + cross |
| `x86_64-apple-darwin` | macos-latest |
| `aarch64-apple-darwin` | macos-latest |

Windows 目标（`x86_64-pc-windows-msvc`）待 Phase 2 补充。

交叉编译工具：`cross`（通过 `taiki-e/install-action` 安装，比 `cargo install cross` 更快）

---

## 十、与 skilly 核心层复用对照

| skm 模块 | 参考 skilly 文件 | 复用程度 |
|---------|----------------|---------|
| `core/agent.rs` | `src-tauri/src/core/agent.rs` | 直接移植，去掉 Tauri 依赖 |
| `core/lock.rs` | `src-tauri/src/core/lock.rs` | 直接移植 |
| `core/registry.rs` | `src-tauri/src/core/registry.rs` | 直接移植，去掉 Tauri 依赖 |
| `core/operations.rs` | `src-tauri/src/core/operations.rs` | 大量参考，重写安装/链接流程 |
| `models.rs` | `src-tauri/src/core/models.rs` | 部分复用，去掉 GUI 相关字段 |
