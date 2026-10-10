# 工作区 Agent 指南

NeoPad 在工作区根目录提供 `AGENTS.md`，让本地 Agent 快速理解笔记用途、
CLI 入口、项目上下文获取方式、写入边界和用户协作约定。
它与源码仓库的开发者 `AGENTS.md` 用途不同。

## 生成与保留

桌面启动或 CLI 笔记命令初始化工作区时，缺失的指南会自动创建。
`doctor`、`--help` 和 `--version` 不创建文件。
自定义工作区在自己的根目录获得指南，不固定写入默认用户目录。

已有指南逐字节保留，包括空文件；升级 NeoPad、修复 CLI 或再次初始化
不会替换它。现有同名目录和符号链接也保留，不向链接目标写入默认内容。
打开指南时只接受普通文件，拒绝目录和链接。缺失时重新创建默认文件。

默认模板位于
[`crates/neopad-core/resources/workspace-AGENTS.md`](../crates/neopad-core/resources/workspace-AGENTS.md)。
需要参考新版默认指南时可查看模板，自行合并个人定制。
NeoPad 不自动安装 skills 或其他 Agent 配置。

## 打开与定制

在 **设置 > Agent 工具 > 打开 Agent 指南** 中打开文件。
它作为外部 Markdown 标签页编辑和保存，不加入笔记列表、笔记搜索或项目。
当前笔记的未保存编辑受到原有保存屏障保护，外部文件修改仍检查内容版本。

可在“用户补充”中写入语言偏好、整理方式等稳定约定。
具体项目的目标、结论和下一步建议保存为该项目的普通“项目说明”笔记，
通过 CLI 创建；不把所有项目进展或聊天记录不断追加到根目录指南。

## 让 Agent 读取

支持 `AGENTS.md` 的工具可将 NeoPad 数据根目录作为工作目录新开会话。
以 Codex 为例，没有项目根目录时只检查当前目录，指引在会话启动时加载；
已有会话中仅切换目录不代表重新加载。
参见 [官方加载规则](https://learn.chatgpt.com/docs/agent-configuration/agents-md)。

从其他项目使用 NeoPad 时，可明确交代：

> 请先读取我的 NeoPad 工作区根目录下的 AGENTS.md，再按指南查找相关笔记。
> CLI 操作要显式指定这份指南所在的工作区，不改变当前源码项目的工作目录。

Windows 默认文件在 `%USERPROFILE%\.neopad\AGENTS.md`；macOS / Linux 在
`~/.neopad/AGENTS.md`。将实际绝对路径交给 Agent；其他工具需确认其文件识别方式。
检查 CLI 的 `doctor` JSON 可获得实际 `guidePath` 和 `guideAvailable`。

## 默认约定

- 按当前任务搜索和读取，不在每次启动时扫描所有笔记。
- 用户要求保存才写入；保存原文时保留原文。
- 使用 CLI 和稳定笔记 ID，增量补充优先追加。
- 完整替换先读取 `updatedAt`；冲突时重新读取并合并。
- 通过 CLI 移动、归档和回收，不直接改元数据或物理删除笔记。
- 已有授权内直接行动，只澄清实质歧义；检查操作结果再反馈成功。

指引是行为约定。CLI 提供路径检查、共享锁和原子笔记写入，但
Agent 的实际文件权限由其工具与操作系统控制。
