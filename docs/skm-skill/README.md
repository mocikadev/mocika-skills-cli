# skm-skill

> 中文 | [English](README.en.md)

AI Agent 技能包，让你的 AI 助手直接操作 `skm` 命令，无需手动执行。

## 安装

通过 skm 安装（推荐）：

```bash
skm install mocikadev/skm-skill --link-to all
```

链接到指定 Agent：

```bash
skm install mocikadev/skm-skill --link-to opencode
```

手动安装（不使用 skm）：

```bash
# 克隆到中央技能目录
git clone https://github.com/mocikadev/skm-skill ~/.agents/skills/skm

# 软链接到目标 Agent（以 opencode 为例）
ln -s ~/.agents/skills/skm/SKILL.md ~/.config/opencode/skills/skm
```

## 使用方式

安装后，直接用自然语言告诉你的 AI Agent：

```
帮我搜索 android 相关的技能包
```

```
安装 mobile-android-design 并链接到所有 Agent
```

```
检测我电脑上装了哪些 AI Agent
```

```
更新所有技能包
```

AI Agent 会自动调用对应的 `skm` 命令完成操作。

## 能力范围

| 能力 | 对应命令 |
|------|----------|
| 搜索技能注册表 | `skm search` |
| 安装 / 卸载技能 | `skm install` / `skm uninstall` |
| 链接 / 取消链接 | `skm link` / `skm unlink` |
| 查看已安装技能 | `skm list` / `skm info` |
| 查看可更新技能 | `skm list --outdated` |
| 更新技能 | `skm update [--all]` |
| 批量重新链接 | `skm relink` |
| 检测本机 Agent | `skm scan` |
| 管理 Agent 列表 | `skm agent list / add` |
| 备份与恢复 | `skm backup list / restore / delete` |
| 诊断环境健康 | `skm doctor` |
| 注册表源管理 | `skm source list / add / remove` |

## 前置条件

- 已安装 `skm`（[安装方式](https://github.com/mocikadev/mocika-skills-cli#安装-skm)）
- 目标 AI Agent 已被 `skm scan` 检测或通过 `skm agent add` 注册

## 相关链接

- [skm CLI 仓库](https://github.com/mocikadev/mocika-skills-cli)
- [完整命令参考](https://github.com/mocikadev/mocika-skills-cli/blob/main/docs/commands.md)
