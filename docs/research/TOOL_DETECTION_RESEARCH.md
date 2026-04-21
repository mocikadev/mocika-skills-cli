# 工具链自动检测与重链接设计研究

**研究日期**: 2026年4月21日  
**研究范围**: rustup、mise、proto、chezmoi、heimdal、stau 等成熟工具链管理器  
**目标**: 为 `skills scan` 和 `skills relink` 命令设计最佳实现方案

---

## 📋 核心发现

### 1. 检测策略的分层优先级系统

```
最高优先级 → 最低优先级
1. 环境变量 (SKILLS_<AGENT>_PATH)
2. 项目配置文件 (.skillsrc, .skills.toml)
3. 标准位置 (~/.config/agents, ~/.local/share/agents)
4. 全局配置 (~/.config/skills/config.toml)
```

### 2. 高效扫描的三个关键要素

1. **只扫描已知位置** - 避免递归扫描整个 home 目录
2. **缓存机制** - 使用 mtime 检查是否需要重新扫描
3. **并行处理** - 使用 rayon/tokio 并行处理多个目录

### 3. Symlink 管理的标准流程

```
1. 冲突检测 → 2. 用户确认 → 3. 备份 → 4. 创建链接 → 5. 验证
```

---

## 🎯 推荐的实现方案

### `skills scan` 命令

```bash
skills scan [OPTIONS]

选项：
  --home <PATH>          # 指定 home 目录（默认 $HOME）
  --config-dir <PATH>    # 指定配置目录
  --output <FORMAT>      # 输出格式 (json, toml, yaml)
  --cache                # 使用缓存结果
  --no-cache             # 忽略缓存，强制重新扫描
  -v, --verbose          # 详细输出
```

### `skills relink` 命令

```bash
skills relink [OPTIONS] [AGENT_NAME]

选项：
  --dry-run              # 预览操作，不实际执行
  -f, --force            # 强制覆盖现有链接
  -b, --backup           # 覆盖前备份现有文件
  -v, --verbose          # 详细输出
  --config <PATH>        # 指定配置文件路径
```

---

## ✅ 最佳实践清单

- ✅ 使用分层优先级系统
- ✅ 只扫描已知位置
- ✅ 实现缓存机制
- ✅ 支持并行处理
- ✅ 冲突检测和备份
- ✅ 支持 `--dry-run` 模式
- ✅ 详细的操作报告

---

## 🔗 参考资源

- **Proto 检测实现**: https://github.com/moonrepo/proto/blob/73900c436068c4126998f6563ee092c6a20470c7/crates/core/src/flow/detect.rs
- **Mise Link 命令**: https://github.com/jdx/mise/blob/f4d20a87a4d1223f981e0355cf25f01c16394a6a/src/cli/link.rs
- **Chezmoi Apply 命令**: https://github.com/twpayne/chezmoi/blob/7615c16dd0d8c4775dcd1337512c9b1cc6586db2/internal/cmd/applycmd.go

---

**研究完成日期**: 2026年4月21日  
**研究状态**: ✅ 完成
