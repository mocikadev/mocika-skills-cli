use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" => Some(Self::En),
            "zh" => Some(Self::Zh),
            _ => None,
        }
    }
}

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn init(lang: Lang) {
    let _ = LANG.set(lang);
}

pub fn current() -> &'static Lang {
    LANG.get().unwrap_or(&Lang::En)
}

pub fn t(key: &str) -> &'static str {
    match *current() {
        Lang::En => match key {
            "installed" => "installed",
            "uninstalled" => "uninstalled",
            "linked" => "linked",
            "unlinked" => "unlinked",
            "updated" => "updated",
            "ok" => "ok",
            "scan" => "scan",
            "info" => "info",
            "warn" => "warn",
            "error" => "error",
            "no new agents detected" => "no new agents detected",
            "no changes detected" => "no changes detected",
            "no skills found" => "no skills found",
            "no installed skills" => "no installed skills",
            "no sources configured" => "no sources configured",
            "no agents registered — run `skm scan` to detect installed agents" => {
                "no agents registered — run `skm scan` to detect installed agents"
            }
            "no outdated skills" => "no outdated skills",
            "no backups found for" => "no backups found for",
            "no backups found" => "no backups found",
            "linked to" => "linked to",
            "id" => "id",
            "path" => "path",
            "scope" => "scope",
            "installed_on" => "installed_on",
            "frontmatter" => "frontmatter",
            "lock" => "lock",
            "source added" => "source added",
            "source removed" => "source removed",
            "agent added" => "agent added",
            "restored" => "restored",
            "deleted backup" => "deleted backup",
            "relink" => "relink",
            "enabled" => "enabled",
            "skills" => "skills",
            "not_tracked" => "not_tracked",
            "unsupported" => "unsupported",
            "has_update" => "has_update",
            "up_to_date" => "up_to_date",
            "cloning repository" => "cloning repository",
            "scanning repository" => "scanning repository",
            "copying skill files" => "copying skill files",
            "creating symlinks" => "creating symlinks",
            "writing lock file" => "writing lock file",
            "checking for updates" => "checking for updates",
            "already up to date" => "already up to date",
            "downloading update" => "downloading update",
            "verifying checksum" => "verifying checksum",
            "unsupported platform for self-update" => "unsupported platform for self-update",
            "update available" => "update available",
            "heading.usage" => "Usage",
            "heading.commands" => "Commands",
            "heading.arguments" => "Arguments",
            "heading.options" => "Options",
            "flag.help" => "Print help",
            "flag.version" => "Print version",
            "cmd.help" => "Print this message or the help of the given subcommand(s)",
            "cmd.skm" => "AI Agent skill package manager",
            "cmd.install" => "Install a skill from registry or Git repository",
            "cmd.search" => "Search skills in configured registries",
            "cmd.scan" => "Detect installed AI agents and register them",
            "cmd.relink" => "Re-link installed skills to agent skill directories",
            "cmd.update" => "Update installed skills to latest version",
            "cmd.list" => "List all installed skills",
            "cmd.info" => "Show detailed information about a skill",
            "cmd.uninstall" => "Uninstall a skill and remove all its symlinks",
            "cmd.link" => "Create a symlink for a skill in an agent directory",
            "cmd.unlink" => "Remove a skill symlink from an agent directory",
            "cmd.source" => "Manage skill registries",
            "cmd.agent" => "Manage registered AI agents",
            "cmd.backup" => "Manage skill backups",
            "cmd.config" => "Configure skm settings (language, etc.)",
            "cmd.self-update" => "Update skm itself to the latest release",
            "cmd.doctor" => "Check environment and diagnose common issues",
            "cmd.source.add" => "Add a new skill registry source",
            "cmd.source.remove" => "Remove a registry source by name",
            "cmd.source.list" => "List all configured registry sources",
            "cmd.agent.list" => "List all registered agents and their status",
            "cmd.agent.add" => "Register a custom agent manually",
            "cmd.backup.list" => "List backup snapshots (all skills when no name given)",
            "cmd.backup.restore" => "Restore a skill from a backup snapshot",
            "cmd.backup.delete" => "Delete a specific backup snapshot",
            "cmd.config.lang" => "Show or set the UI language",
            "arg.install.name" => "Skill name, owner/repo[:subpath], or full Git URL",
            "arg.install.link-to" => r#"Link installed skill to agent(s): agent ID or "all""#,
            "arg.search.keyword" => "Keyword to search for",
            "arg.search.limit" => "Maximum number of results to show",
            "arg.scan.dry-run" => "Preview detected agents without writing to agents.toml",
            "arg.relink.agent" => "Target agent ID (omit to relink all agents)",
            "arg.relink.skill" => "Only relink this specific skill",
            "arg.relink.force" => "Overwrite conflicting paths (non-skm symlinks or files)",
            "arg.relink.backup" => {
                "Back up conflicting paths before overwriting (requires --force)"
            }
            "arg.relink.dry-run" => "Show what would be done without making any changes",
            "arg.update.name" => "Skill name to update (omit or use --all to update all)",
            "arg.update.all" => "Update all installed skills",
            "arg.update.check" => "Only check for updates, do not apply them",
            "arg.self-update.check" => "Only check for a newer version without downloading",
            "doctor.env" => "Environment",
            "doctor.agents" => "Agents",
            "doctor.links" => "Links",
            "doctor.shared_dir" => "shared skills dir",
            "doctor.lock_file" => "lock file",
            "doctor.agents_toml" => "agents.toml",
            "doctor.exists" => "exists",
            "doctor.readable" => "readable",
            "doctor.missing" => "missing",
            "doctor.unreadable" => "unreadable",
            "doctor.installed" => "installed",
            "doctor.not_installed" => "not installed",
            "doctor.linked" => "linked",
            "doctor.not_linked" => "not linked",
            "doctor.conflict" => "conflict",
            "doctor.stale_hint" => "run `skm scan` to clean up",
            "doctor.no_agents" => "no agents registered — run `skm scan` to detect",
            "doctor.all_ok" => "all checks passed",
            "doctor.issues" => "issue(s)",
            "arg.info.name" => "Skill name",
            "arg.uninstall.name" => "Skill name",
            "arg.link.name" => "Skill name",
            "arg.link.agent" => "Agent ID (e.g. opencode, claude-code)",
            "arg.unlink.name" => "Skill name",
            "arg.unlink.agent" => "Agent ID (e.g. opencode, claude-code)",
            "arg.source.add.name" => "Display name for this source",
            "arg.source.add.url" => "Registry URL (https://skills.sh or GitHub repo URL)",
            "arg.source.remove.name" => "Source name to remove",
            "arg.agent.add.id" => "Agent ID (e.g. my-agent)",
            "arg.agent.add.path" => {
                "Path to the agent's skills directory (e.g. ~/.my-agent/skills)"
            }
            "arg.backup.list.name" => "Skill name",
            "arg.backup.restore.name" => "Skill name",
            "arg.backup.restore.snapshot-id" => "Snapshot ID to restore (defaults to latest)",
            "arg.backup.delete.name" => "Skill name",
            "arg.backup.delete.snapshot-id" => "Snapshot ID to delete",
            "arg.config.lang.lang" => "Language code (en or zh)",
            "arg.config.lang.reset" => "Reset language to auto-detect from environment",
            "arg.list.outdated" => "Only show skills with available updates",
            _ => "unknown translation key",
        },
        Lang::Zh => match key {
            "installed" => "已安装",
            "uninstalled" => "已卸载",
            "linked" => "已链接",
            "unlinked" => "已取消链接",
            "updated" => "已更新",
            "ok" => "完成",
            "scan" => "扫描",
            "info" => "信息",
            "warn" => "警告",
            "error" => "错误",
            "no new agents detected" => "未检测到新 Agent",
            "no changes detected" => "未检测到变化",
            "no skills found" => "未找到技能包",
            "no installed skills" => "暂未安装技能包",
            "no sources configured" => "未配置来源",
            "no agents registered — run `skm scan` to detect installed agents" => {
                "未注册 Agent — 运行 `skm scan` 检测已安装的 Agent"
            }
            "no outdated skills" => "所有技能均为最新版本",
            "no backups found for" => "未找到备份：",
            "no backups found" => "暂无备份快照",
            "linked to" => "已链接到",
            "id" => "ID",
            "path" => "路径",
            "scope" => "范围",
            "installed_on" => "已安装到",
            "frontmatter" => "元数据",
            "lock" => "锁",
            "source added" => "已添加来源",
            "source removed" => "已移除来源",
            "agent added" => "已添加 Agent",
            "restored" => "已恢复",
            "deleted backup" => "已删除备份",
            "relink" => "重链接",
            "enabled" => "启用",
            "skills" => "技能数",
            "not_tracked" => "未跟踪",
            "unsupported" => "不支持",
            "has_update" => "可更新",
            "up_to_date" => "已是最新",
            "cloning repository" => "克隆仓库",
            "scanning repository" => "扫描仓库",
            "copying skill files" => "复制技能文件",
            "creating symlinks" => "创建软链接",
            "writing lock file" => "写入锁文件",
            "checking for updates" => "正在检查更新",
            "already up to date" => "已是最新版本",
            "downloading update" => "正在下载更新",
            "verifying checksum" => "正在校验文件",
            "unsupported platform for self-update" => "当前平台不支持自动升级",
            "update available" => "有新版本可用",
            "heading.usage" => "用法",
            "heading.commands" => "命令",
            "heading.arguments" => "参数",
            "heading.options" => "选项",
            "flag.help" => "显示帮助信息",
            "flag.version" => "显示版本信息",
            "cmd.help" => "显示帮助信息或指定子命令的帮助信息",
            "cmd.skm" => "AI Agent 技能包管理器",
            "cmd.install" => "从注册表或 Git 仓库安装技能",
            "cmd.search" => "从注册表搜索技能",
            "cmd.scan" => "检测本机已安装的 AI Agent 并注册",
            "cmd.relink" => "将已安装技能重新软链接到 Agent 目录",
            "cmd.update" => "更新已安装技能到最新版本",
            "cmd.list" => "列出所有已安装技能",
            "cmd.info" => "查看技能详细信息",
            "cmd.uninstall" => "卸载技能并移除所有软链接",
            "cmd.link" => "在 Agent 目录中为技能创建软链接",
            "cmd.unlink" => "移除 Agent 目录中的技能软链接",
            "cmd.source" => "管理技能注册表来源",
            "cmd.agent" => "管理已注册的 AI Agent",
            "cmd.backup" => "管理技能备份",
            "cmd.config" => "配置 skm 设置（语言等）",
            "cmd.self-update" => "升级 skm 到最新版本",
            "cmd.doctor" => "检测环境并诊断常见问题",
            "cmd.source.add" => "添加新的技能注册表来源",
            "cmd.source.remove" => "按名称移除注册表来源",
            "cmd.source.list" => "列出所有已配置的注册表来源",
            "cmd.agent.list" => "列出所有已注册 Agent 及其状态",
            "cmd.agent.add" => "手动注册自定义 Agent",
            "cmd.backup.list" => "列出备份快照（不指定技能名则列出全部）",
            "cmd.backup.restore" => "从备份快照恢复技能",
            "cmd.backup.delete" => "删除指定的备份快照",
            "cmd.config.lang" => "查看或设置界面语言",
            "arg.install.name" => "技能名、owner/repo[:子路径] 或完整 Git URL",
            "arg.install.link-to" => "安装后链接到 Agent：Agent ID 或 \"all\"",
            "arg.search.keyword" => "搜索关键词",
            "arg.search.limit" => "最多显示结果数",
            "arg.scan.dry-run" => "预览检测结果，不写入 agents.toml",
            "arg.relink.agent" => "目标 Agent ID（省略则重链接所有 Agent）",
            "arg.relink.skill" => "仅重链接此技能",
            "arg.relink.force" => "覆盖冲突路径（非 skm 软链接或文件）",
            "arg.relink.backup" => "覆盖前备份冲突路径（需要 --force）",
            "arg.relink.dry-run" => "预览操作，不实际修改",
            "arg.update.name" => "要更新的技能名（省略或使用 --all 则更新全部）",
            "arg.update.all" => "更新所有已安装技能",
            "arg.update.check" => "仅检查更新，不执行",
            "arg.self-update.check" => "仅检查新版本，不下载",
            "doctor.env" => "环境",
            "doctor.agents" => "Agent",
            "doctor.links" => "链接",
            "doctor.shared_dir" => "共享技能目录",
            "doctor.lock_file" => "锁文件",
            "doctor.agents_toml" => "agents.toml",
            "doctor.exists" => "存在",
            "doctor.readable" => "可读",
            "doctor.missing" => "缺失",
            "doctor.unreadable" => "不可读",
            "doctor.installed" => "已安装",
            "doctor.not_installed" => "未安装",
            "doctor.linked" => "已链接",
            "doctor.not_linked" => "未链接",
            "doctor.conflict" => "冲突",
            "doctor.stale_hint" => "运行 `skm scan` 清理",
            "doctor.no_agents" => "未注册 Agent — 运行 `skm scan` 检测",
            "doctor.all_ok" => "所有检查通过",
            "doctor.issues" => "个问题",
            "arg.info.name" => "技能名",
            "arg.uninstall.name" => "技能名",
            "arg.link.name" => "技能名",
            "arg.link.agent" => "Agent ID（如 opencode、claude-code）",
            "arg.unlink.name" => "技能名",
            "arg.unlink.agent" => "Agent ID（如 opencode、claude-code）",
            "arg.source.add.name" => "来源的显示名称",
            "arg.source.add.url" => "注册表 URL（skills.sh 或 GitHub 仓库地址）",
            "arg.source.remove.name" => "要移除的来源名称",
            "arg.agent.add.id" => "Agent ID（如 my-agent）",
            "arg.agent.add.path" => "Agent 技能目录路径（如 ~/.my-agent/skills）",
            "arg.backup.list.name" => "技能名",
            "arg.backup.restore.name" => "技能名",
            "arg.backup.restore.snapshot-id" => "要恢复的快照 ID（默认为最新）",
            "arg.backup.delete.name" => "技能名",
            "arg.backup.delete.snapshot-id" => "要删除的快照 ID",
            "arg.config.lang.lang" => "语言代码（en 或 zh）",
            "arg.config.lang.reset" => "重置语言为自动检测",
            "arg.list.outdated" => "只显示有可用更新的技能",
            _ => "未知翻译键",
        },
    }
}

pub fn fmt_installed(name: &str, id: &str) -> String {
    match *current() {
        Lang::En => format!("installed {name} ({id})"),
        Lang::Zh => format!("已安装 {name} ({id})"),
    }
}

pub fn fmt_agents_detected(count: usize, ids: &str) -> String {
    match *current() {
        Lang::En => format!("{count} agents detected: {ids}"),
        Lang::Zh => format!("检测到 {count} 个 Agent：{ids}"),
    }
}

pub fn fmt_new_agents(count: usize, ids: &str) -> String {
    match *current() {
        Lang::En => format!("{count} new agents detected: {ids}"),
        Lang::Zh => format!("检测到 {count} 个新 Agent：{ids}"),
    }
}

pub fn fmt_removed_agents(count: usize, ids: &str) -> String {
    match *current() {
        Lang::En => format!("{count} stale agent(s) removed from agents.toml: {ids}"),
        Lang::Zh => format!("已从 agents.toml 移除 {count} 个失效 Agent：{ids}"),
    }
}

pub fn fmt_relink_result(linked: usize, conflicts: usize, skipped: usize) -> String {
    match *current() {
        Lang::En => format!("{linked} linked, {conflicts} conflicts, {skipped} skipped"),
        Lang::Zh => format!("已链接 {linked}，冲突 {conflicts}，跳过 {skipped}"),
    }
}

pub fn fmt_progress(step_key: &str, done: usize, total: usize) -> String {
    format!("{} ({}/{total})", t(step_key), done + 1)
}

pub fn fmt_unlinked_from(name: &str, agent_id: &str) -> String {
    match *current() {
        Lang::En => format!("{name} from {agent_id}"),
        Lang::Zh => format!("{name} 从 {agent_id}"),
    }
}

pub fn fmt_lang_status(lang_code: &str, configured: bool) -> String {
    match *current() {
        Lang::En => format!(
            "lang: {lang_code} ({})",
            if configured {
                "configured"
            } else {
                "auto-detected"
            }
        ),
        Lang::Zh => format!(
            "语言: {lang_code}（{}）",
            if configured {
                "已配置"
            } else {
                "自动检测"
            }
        ),
    }
}

pub fn fmt_lang_set(lang_code: &str) -> String {
    match *current() {
        Lang::En => format!("lang set to: {lang_code}"),
        Lang::Zh => format!("语言已设置为: {lang_code}"),
    }
}

pub fn fmt_lang_reset_to_auto_detect() -> String {
    match *current() {
        Lang::En => "lang reset to auto-detect".to_string(),
        Lang::Zh => "语言已重置为自动检测".to_string(),
    }
}

pub fn fmt_no_lock_file_entry() -> String {
    match *current() {
        Lang::En => "skill has no lock file entry".to_string(),
        Lang::Zh => "技能没有锁文件记录".to_string(),
    }
}

pub fn fmt_source_type_no_remote_update_checks(source_type: &str) -> String {
    match *current() {
        Lang::En => {
            format!("sourceType '{source_type}' does not support remote update checks")
        }
        Lang::Zh => format!("sourceType '{source_type}' 不支持远程更新检查"),
    }
}

pub fn fmt_source_type_no_remote_updates(source_type: &str) -> String {
    match *current() {
        Lang::En => format!("sourceType '{source_type}' does not support remote updates"),
        Lang::Zh => format!("sourceType '{source_type}' 不支持远程更新"),
    }
}

pub fn fmt_invalid_lang(value: &str) -> String {
    match *current() {
        Lang::En => format!("unsupported language: {value}. use 'en' or 'zh'"),
        Lang::Zh => format!("不支持的语言：{value}。请使用 en 或 zh"),
    }
}

pub fn fmt_doctor_summary(issues: usize) -> String {
    match *current() {
        Lang::En => {
            if issues == 0 {
                "all checks passed".to_string()
            } else {
                format!("{issues} issue(s) found")
            }
        }
        Lang::Zh => {
            if issues == 0 {
                "所有检查通过".to_string()
            } else {
                format!("发现 {issues} 个问题")
            }
        }
    }
}

pub fn fmt_update_summary(updated: usize, failed: usize) -> String {
    match *current() {
        Lang::En => format!("updated: {updated}, failed: {failed}"),
        Lang::Zh => format!("已更新：{updated}，失败：{failed}"),
    }
}

pub fn fmt_updated_to(version: &str) -> String {
    match *current() {
        Lang::En => format!("skm updated to v{version}"),
        Lang::Zh => format!("skm 已升级至 v{version}"),
    }
}

pub fn fmt_update_available(tag: &str) -> String {
    match *current() {
        Lang::En => format!("{tag} is available — run `skm self-update` to install"),
        Lang::Zh => format!("{tag} 可用 — 运行 `skm self-update` 安装"),
    }
}
