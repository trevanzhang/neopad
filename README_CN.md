# NeoPad

NeoPad 是一款轻量、本地优先的桌面便签工具，适合快速记录、快速查找，
也方便本地 AI Agent 通过原生 CLI 访问同一批 Markdown 笔记。

NeoPad 的定位是捕捉工具，不是知识库套件。笔记默认保存为本地
Markdown 文件，用户数据不需要账号，也不依赖云服务。

## 当前状态

当前源码版本 **0.7.2** 正在准备发行构建，包含 Windows 文件定位修复、依赖更新
和可编辑的工作区 Agent 指南。参见 [0.7.2 发行说明](docs/releases/v0.7.2.md)。
源码版本不代表 GitHub Release 已公开发布；安装包需完成校验与签名检查后发布。

**0.7.1** 将提示词库替换为普通项目笔记，旧提示词正文和分类会自动迁移。
**0.7.0** 将 MCP 接入替换为随安装包提供的原生 CLI。已有 Agent 配置需要
迁移，笔记存储格式不变。参见 [MCP 到 CLI 的迁移说明](docs/cli.md#migrate-from-mcp)
及 [更新日志](CHANGELOG.md)。

当前版本是面向 Windows 的可用 MVP，主要能力包括：

- 基于 Tauri 2、Vue 3、TypeScript、Rust 和 CodeMirror 6 的桌面应用。
- 捕获笔记保存为 `~/.neopad/notes/*.md`，项目笔记保存在
  `~/.neopad/projects/**/*.md`，归档笔记保存在 `~/.neopad/archive/**/*.md`。
- 多标签页、自动保存、全文搜索、剪贴板捕获、Markdown 预览。
- 页面切换和替换内容前会经过保存屏障；保存失败时保留待保存内容，不会继续
  执行可能丢失编辑的操作。
- 全局搜索按笔记分组显示匹配行，展示每篇笔记的匹配数，重复匹配可按需展开。
- 编辑、混合和预览三种编辑器模式，可通过菜单、状态栏和 `F5` 切换。
- 预览模式支持多套主题预设：Light、One Dark、Nord、Solarized、
  Monokai、GitHub Light、Dracula。
- Markdown 源码和围栏代码块支持语法高亮；混合与预览模式支持 KaTeX
  数学公式和 Mermaid 图表。
- 当前标签页可从文件菜单或标签页右键菜单导出为 PNG 长图或 A4
  多页 PDF；PNG 可保存到文件或直接复制到系统剪贴板，代码高亮、公式和
  图表会按预览效果写入导出内容。导出使用当前预览主题与字体，支持移动端长图；
  深色 PDF 的页面与页边距也使用主题背景，PNG 和 PDF 末尾显示 Powered by NeoPad。
- 预览外观支持字体、字号、行高和内容宽度设置。
- 编辑器字体设置提供预设字体、字号滑块和实时预览，不需要手写 CSS 字体串。
- 应用支持日间/夜间模式，`F9` 可快速切换。
- `F10` 可循环切换预览主题，`F7` 切换窗口置顶。
- `F4` 可打开紧凑的文件浏览器，按笔记、项目、归档、回收站排列；支持项目文件夹、
  笔记移动与整项目归档，可拖动分隔线调整宽度并记住设置。
- 标签页右键菜单可以在资源管理器中定位 Markdown 文件，也可以复制当前页面的
  绝对文件地址，便于交给终端或本地 Agent。文件菜单中的“打开 NeoPad 数据文件夹”
  会直接打开完整的 `~/.neopad/` 工作区。
- 重组后的设置中心将通用、编辑器、预览、Vim、快捷键、AI 和 Agent 工具设置集中在一个聚焦的
  对话框中；窗口显示与剪贴板捕获快捷键均可配置。
- 系统托盘图标支持左键恢复并聚焦主窗口，右键打开显示、隐藏、新建笔记、
  保存剪贴板、设置和退出菜单。
- 可选 AI 协作提供紧凑的 `Ctrl+K` 笔记对话、受限的全笔记
  相关内容搜索，以及独立的 `//` 续写、润色、总结和中英互译命令；结果只有在用户
  选择命令后才会以单次可撤销编辑直接落入正文，聊天结果仍由用户选择如何使用。
  API Key 保存在系统凭据管理器中。
- 可选 Vim 编辑模式，支持 Normal、Insert、Visual 状态显示；帮助菜单提供涵盖
  常用编辑操作与 NeoPad 标签切换的 Vim 简明指南。
- Markdown 原生提醒，格式为
  `- [ ] @remind YYYY-MM-DD HH:mm 提醒内容`。
- 原生 `neopad-cli` 随安装包提供，桌面启动时自动部署到
  `~/.neopad/.agents/bin/`；设置中的 Agent 工具页面可检查、修复及复制路径。
  工作区初始化只在根目录 `AGENTS.md` 不存在时创建默认指南，已有文件（包括空文件）
  会保留。可从 Agent 工具页面打开编辑，CLI 修复不会覆盖指南。
  Agent 更新笔记后，编辑器仅在没有待保存内容时自动刷新。
- Windows MSI 安装包提供英文应用语言选择页，其中中文默认选项为双语，所选语言会用于
  NeoPad 首次启动；安装包同时内置 `neopad-cli.exe` 作为 sidecar，并声明 `.md`
  和 `.markdown` 文件关联，供用户在系统中选择 NeoPad 作为默认打开方式。
- `Ctrl+O`、将 `.md` 或 `.markdown` 文件拖入窗口、或从系统打开关联文件，都会
  就地打开外部 Markdown 文件，并使用 SHA-256 内容 revision 检测外部修改，避免
  静默覆盖。

## 用户数据

默认工作区位于：

```text
~/.neopad/
```

常见目录和文件：

```text
~/.neopad/
  notes/              Markdown 笔记正文
  projects/           项目 Markdown 笔记
  archive/            已归档笔记，仍参与全文搜索
  meta/tabs.json      标签页元数据
  meta/reminders.json 提醒投递状态
  config.json         应用设置
  trash/              删除的笔记会移动到这里
```

捕获笔记正文保存在 `notes/*.md`，项目笔记保存在 `projects/**/*.md` 中。
旧提示词库迁移说明见 [项目与迁移](docs/projects.md)。清空 NeoPad 回收站时，Markdown
文件会先移至操作系统回收站，再从 NeoPad 的笔记库中移除对应条目。通过
操作系统恢复到 NeoPad `trash/` 目录的文件，会在刷新时重新出现在 NeoPad
回收站中。元数据文件不会保存笔记正文。

## 常用快捷键

```text
F1              打开快捷键帮助
F2              上一个标签页
F3              下一个标签页
F4              打开或关闭笔记浏览器
F5              循环切换编辑、混合、预览模式
F6              打开或关闭提醒列表
F7              切换窗口置顶
F8              重命名当前页面
F9              切换日间/夜间模式
F10             循环切换预览主题
Ctrl+,          打开设置
F11             切换沉浸式全屏
F12             归档当前页面
Alt+Enter       最大化或还原主窗口
Esc             关闭浮层、退出全屏或隐藏窗口
Ctrl+O          打开外部 Markdown 文件
Ctrl+W          关闭当前非固定标签页，保留文件
Ctrl+Z          撤销当前笔记中的编辑
Ctrl+Y          重做当前笔记中的编辑
Ctrl+Shift+Z    备用重做快捷键
Ctrl+E          插入 Markdown 提醒
Ctrl+Tab        下一个标签页
Ctrl+Shift+Tab  上一个标签页
```

可配置的全局快捷键：

```text
Alt+Z           显示或隐藏 NeoPad
Ctrl+Shift+V    将当前剪贴板文本追加到 clipboard.md
```

## 开发

安装依赖：

```powershell
pnpm install
```

启动开发版：

```powershell
pnpm tauri:dev
```

构建前端：

```powershell
pnpm build
```

运行 Rust 测试：

```powershell
cargo test
```

构建 Windows MSI：

```powershell
pnpm tauri:build
```

该命令会按平台显式生成 Windows MSI、macOS DMG，或 Linux DEB 与
AppImage；Windows 不会额外生成 NSIS。

安装包输出位置：

```text
target/release/bundle/msi/NeoPad_<version>_x64_en-US.msi
```

构建后运行 `node scripts/write-release-checksums.mjs`，生成
`target/SHA256SUMS-win32.txt`。GitHub 上推送与版本一致的 `v*` 标签才会触发
三平台安装包构建并上传至草稿 Release。构建通过后仍需验证校验和、Windows 签名、
macOS 签名与公证，并完成安装包冒烟检查。完整步骤见 [发行准备](docs/release.md)。

## 原生 CLI

启动 NeoPad 一次后，从“设置 > Agent 工具”复制 CLI 路径。
CLI 可独立于桌面运行，命令结果以 JSON 返回，无需配置服务、端口或令牌。

```powershell
$cli = "$env:USERPROFILE\.neopad\.agents\bin\neopad-cli.exe"
& $cli doctor
& $cli list
& $cli create --title "Agent 笔记" --content-file .\content.md
```

开发构建：`cargo build -p neopad-cli --release`。
完整命令及冲突处理见 [CLI 与 Agent 使用指南](docs/cli.md)。

## 更多文档

- [English README](README.md)
- [Changelog](CHANGELOG.md)
- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [发行准备](docs/release.md)
- [项目与迁移](docs/projects.md)
- [工作区 Agent 指南](docs/agent-guide.md)
- [Markdown 预览与导出](docs/export.md)
- [CLI 与 Agent 使用指南](docs/cli.md)
- [AI 协作](docs/ai.md)
