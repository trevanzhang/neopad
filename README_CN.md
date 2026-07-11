# NeoPad

NeoPad 是一款轻量、本地优先的桌面便签工具，适合快速记录、快速查找，
也方便本地 AI Agent 通过 MCP 访问同一批 Markdown 笔记。

NeoPad 的定位是捕捉工具，不是知识库套件。笔记默认保存为本地
Markdown 文件，用户数据不需要账号，也不依赖云服务。

## 当前状态

当前版本是面向 Windows 的可用 MVP，主要能力包括：

- 基于 Tauri 2、Vue 3、TypeScript、Rust 和 CodeMirror 6 的桌面应用。
- 所有笔记内容保存为 `~/.neopad/notes/*.md`。
- 多标签页、自动保存、全文搜索、剪贴板捕获、Markdown 预览。
- 页面切换和替换内容前会经过保存屏障；保存失败时保留待保存内容，不会继续
  执行可能丢失编辑的操作。
- 全局搜索按笔记分组显示匹配行，展示每篇笔记的匹配数，重复匹配可按需展开。
- 编辑、混合和预览三种编辑器模式，可通过菜单、状态栏和 `F4` 切换。
- 预览模式支持多套主题预设：Light、One Dark、Nord、Solarized、
  Monokai、GitHub Light、Dracula。
- 预览外观支持字体、字号、行高和内容宽度设置。
- 编辑器字体设置提供预设字体、字号滑块和实时预览，不需要手写 CSS 字体串。
- 应用支持日间/夜间模式，`F9` 可快速切换。
- `F7` 可循环切换预览主题。
- 可选 Vim 编辑模式，支持 Normal、Insert、Visual 状态显示。
- Markdown 原生提醒，格式为
  `- [ ] @remind YYYY-MM-DD HH:mm 提醒内容`。
- 本地 HTTP MCP 服务默认关闭，可在设置中启停；启用后通过 bearer token
  授权本地 Agent 读写笔记。
- Windows MSI 安装包会内置 `neopad-mcp.exe` 作为 sidecar。
- `Ctrl+O` 通过原生文件选择器授权外部 Markdown 路径，并使用 SHA-256
  内容 revision 检测外部修改，避免静默覆盖。

## 用户数据

默认工作区位于：

```text
~/.neopad/
```

常见目录和文件：

```text
~/.neopad/
  notes/              Markdown 笔记正文
  archive/            已归档笔记，仍参与全文搜索
  meta/tabs.json      标签页元数据
  meta/reminders.json 提醒投递状态
  config.json         应用设置
  trash/              删除的笔记会移动到这里
  backups/            预留备份目录
```

笔记正文只保存在 `notes/*.md` 中。元数据文件不会保存笔记正文。

## 常用快捷键

```text
F1              打开快捷键帮助
F2              重命名当前页面
F4              循环切换编辑、混合、预览模式
F5              打开或关闭提醒列表
F7              循环切换预览主题
F8              打开设置
F9              切换日间/夜间模式
F10             切换标签栏方向
F11             切换沉浸式全屏
Alt+Enter       最大化或还原主窗口
Esc             关闭浮层、退出全屏或隐藏窗口
Ctrl+O          打开外部 Markdown 文件
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
target/release/bundle/msi/NeoPad_0.4.6_x64_en-US.msi
```

## MCP 服务

NeoPad 可以从设置中启动或停止本地 MCP 服务。复制给 Agent 的配置形如：

```json
{
  "mcpServers": {
    "neopad": {
      "url": "http://127.0.0.1:8765/mcp",
      "headers": {
        "Authorization": "Bearer <local-token>"
      }
    }
  }
}
```

服务默认关闭。启用后，持有 token 的本地 Agent 可以读取和写入 NeoPad 笔记。
服务只允许绑定本机 loopback 地址，不接受局域网或公网监听地址。
桌面端通过子进程环境变量传递 token，不把 token 放入进程命令行。

## 更多文档

- [English README](README.md)
- [Changelog](CHANGELOG.md)
- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [MCP](docs/mcp.md)
