# NeoPad CLI 与 Agent 使用指南

适用于 **NeoPad 0.7.0 及以后版本**。

NeoPad 为本地 Agent 提供原生 `neopad-cli`。CLI 直接使用共享 Rust 核心，
可独立于桌面运行，不需要配置 MCP、HTTP 地址、端口、令牌或 Node.js。

## 找到 CLI

安装并启动 NeoPad 一次，桌面会将随安装包提供的 CLI 部署到：

| 系统 | 默认路径 |
| --- | --- |
| Windows | `%USERPROFILE%\.neopad\.agents\bin\neopad-cli.exe` |
| macOS / Linux | `~/.neopad/.agents/bin/neopad-cli` |

“设置 > Agent 工具”可以查看状态、修复文件或复制完整路径。每次桌面启动会
对比安装包中的二进制内容并更新部署；不会覆盖 `.agents/` 中的用户指南。
本阶段不自动生成用户数据目录的 `AGENTS.md` 或安装 skills。

把本指南和 CLI 完整路径交给 Agent 即可。使用完整路径调用，无需修改 PATH。
开发者可先执行 `cargo build -p neopad-cli --release`，然后使用
`target/release/neopad-cli[.exe]`，或启动桌面完成部署。

```powershell
$cli = "$env:USERPROFILE\.neopad\.agents\bin\neopad-cli.exe"
& $cli --version
& $cli doctor
& $cli list
```

```sh
"$HOME/.neopad/.agents/bin/neopad-cli" doctor
"$HOME/.neopad/.agents/bin/neopad-cli" list
```

工作区按 `--workspace PATH`、`NEOPAD_WORKSPACE` 环境变量、默认 `~/.neopad`
的顺序选择。自定义路径应使用完整路径。`doctor`、`--help` 和 `--version`
不会初始化工作区；其他命令会通过核心初始化工作区。

## 保存、读取与更新

输入文件须为 UTF-8（允许 BOM），最大 2 MiB。Windows Agent 建议使用
`--content-file`，避免旧版 PowerShell 管道按系统代码页编码中文。
`--stdin` 会读取到输入结束；不要同时指定这两个参数。

```powershell
& $cli create --title "本次讨论结论" --content-file .\conclusion.md
& $cli append-inbox --content-file .\conclusion.md
```

`create` 在正文前加入标题 H1；`update` 完整替换 Markdown，调用者应自行
保留需要的标题。优先使用 `append` 保存增量内容。必须替换整篇笔记时，
先 `read`，再传入返回的 `updatedAt`，不允许省略或猜测版本。

```powershell
$created = & $cli create --title "Agent 示例" | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw $created.error.message }
$id = $created.data.id
$note = & $cli read $id | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw $note.error.message }
& $cli update $id --expected-updated-at $note.data.updatedAt --content-file .\edited.md
if ($LASTEXITCODE -eq 3) {
    # 重新读取最新内容，与修改内容合并，再决定是否提交。
    # 不要只换一个新时间戳就重试原来的覆盖内容。
}
```

## 命令

全局选项：`--workspace PATH`、`--json`（默认已经输出 JSON）。

| 命令 | 用法 |
| --- | --- |
| 检查 | `doctor` |
| 列表 | `list [--scope active\|archive\|trash\|all]` |
| 读取 | `read ID` |
| 搜索 | `search QUERY [--limit 1..100]` |
| 创建 | `create [--title TITLE] [--content-file PATH\|--stdin]` |
| 追加 | `append ID --content-file PATH`，或 `--stdin` |
| 收集到 Inbox | `append-inbox --content-file PATH`，或 `--stdin` |
| 收集到 Clipboard | `append-clipboard --content-file PATH`，或 `--stdin` |
| 完整替换 | `update ID --expected-updated-at N --content-file PATH`，或 `--stdin` |
| 重命名 | `rename ID --title TITLE`（同步首个 H1） |
| 归档与撤销 | `archive ID`、`unarchive ID` |
| 回收与恢复 | `trash ID`、`restore ID` |

ID 必须来自列表、搜索或创建结果，不要将文件路径当作 ID。
`all` 包含活动笔记和归档笔记，排除回收站；搜索也排除回收站和用户提示词。
`append-clipboard` 只追加提供的文本，不读取系统剪贴板。

## 结果与安全边界

成功格式为 `{"ok":true,"version":"…","data":…}`；失败格式为
`{"ok":false,"error":{"code":"…","message":"…"}}`。
`list` 的内容在 `data.pages`，`search` 在 `data.matches`；其他笔记命令
返回相应的笔记或操作结果。`--help` 和 `--version` 返回纯文本。

| 退出码 | 含义 |
| --- | --- |
| 0 | 成功 |
| 1 | 操作失败（`operation_failed`） |
| 2 | 参数错误（`invalid_arguments`） |
| 3 | 内容或版本冲突（`conflict`） |

所有笔记命令通过共享核心执行路径校验、跨进程锁和原子写入。
不提供物理删除接口；Inbox、Clipboard 等固定笔记不可丢入回收站。
Agent 更新后，桌面会自动刷新已经保存的笔记。编辑中的内容会保留；
遇到冲突时应读取最新版本并合并。

部署拒绝 `.agents/`、`bin/` 或 CLI 文件上的符号链接及 Windows junction。
这些措施约束 CLI 的行为；拥有同一用户文件权限的 Agent 仍可以绕过 CLI
直接访问文件。请在 Agent 工具自身的权限设置中限定操作范围，并指导它
通过 CLI 操作笔记，不直接改动 `config.json`、`meta/` 或程序文件。

## Migrate from MCP

NeoPad 0.7.0 移除了此前 0.x 版本提供的 MCP 服务。这是 Agent 接口的不兼容变更，
不会改变现有 Markdown 笔记、归档、提示词或提醒数据的格式和位置。

1. 保存并退出旧版 NeoPad；如有手动启动的旧版 `neopad-mcp` 服务，也应停止。
2. 安装并启动 NeoPad 0.7.0，从“设置 > Agent 工具”确认 CLI 已就绪并复制路径。
3. 在 Agent 工具中移除旧的 NeoPad MCP 连接，将本指南和 CLI 完整路径交给 Agent。
   保留该工具中的其他 MCP 连接。
4. 先运行 `--version` 和 `doctor`，再运行 `list` 与 `read ID` 验证旧笔记可访问。

不再需要 NeoPad HTTP 地址、Bearer token 或 `NEOPAD_MCP_TOKEN`。旧的
`config.json` 中 `mcp` 字段会被忽略，并在下次保存配置时移除，无需手动
编辑元数据。旧 MCP 配置文档仅供查看 0.7.0 之前的历史行为，不适用于 0.7.0。
