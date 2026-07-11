import type { AppLanguage } from './i18n'
import { formatShortcutLabel } from './shortcut'

export type HelpTopic = 'software' | 'markdown' | 'shortcuts' | 'expression' | 'about'

interface HelpContext {
  appVersion: string
  shortcutBaseKey: string
  shortcutModifiers: string[]
  clipboardShortcutBaseKey: string
  clipboardShortcutModifiers: string[]
}

export function getHelpContent(topic: HelpTopic | null, language: AppLanguage, context: HelpContext) {
  const zh = language === 'zh'
  if (topic === 'shortcuts') {
    return {
      title: zh ? '快捷键列表' : 'Shortcut List',
      lines: [
        `${formatShortcutLabel(context.shortcutBaseKey, context.shortcutModifiers)} - ${zh ? '显示/隐藏窗口' : 'Show/hide window'}`,
        `${formatShortcutLabel(context.clipboardShortcutBaseKey, context.clipboardShortcutModifiers)} - ${zh ? '保存剪贴板' : 'Save clipboard'}`,
        `F1 - ${zh ? '打开快捷键帮助' : 'Open shortcut help'}`,
        `Alt+Enter - ${zh ? '最大化/还原窗口' : 'Maximize/restore window'}`,
        `Ctrl+N - ${zh ? '新建标签页' : 'New tab'}`,
        `F2 - ${zh ? '重命名标签页' : 'Rename tab'}`,
        `Alt+Del - ${zh ? '将当前标签页移至回收站' : 'Move current tab to Trash'}`,
        `Ctrl+W - ${zh ? '关闭标签页' : 'Close tab'}`,
        `Ctrl+O - ${zh ? '从文件载入' : 'Load from file'}`,
        `Ctrl+Tab / Ctrl+Shift+Tab - ${zh ? '切换下一个/上一个标签页' : 'Switch next/previous tab'}`,
        `Ctrl+F - ${zh ? '查找' : 'Find'}`,
        `Ctrl+Shift+F - ${zh ? '全局搜索' : 'Global search'}`,
        `Ctrl+D - ${zh ? '插入日期时间' : 'Insert date time'}`,
        `Ctrl+- - ${zh ? '插入分隔行' : 'Insert separator'}`,
        `Ctrl+Shift+- - ${zh ? '插入日期时间分隔行' : 'Insert date time separator'}`,
        `Ctrl+E - ${zh ? '插入提醒' : 'Insert reminder'}`,
        `F4 - ${zh ? '循环切换编辑器模式' : 'Cycle editor mode'}`,
        `F5 - ${zh ? '打开/关闭提醒列表' : 'Open/close reminder list'}`,
        `F6 - ${zh ? '切换窗口置顶' : 'Toggle window on top'}`,
        `F7 - ${zh ? '切换预览主题' : 'Toggle preview theme'}`,
        `F8 - ${zh ? '打开设置' : 'Open settings'}`,
        `F9 - ${zh ? '切换日间/夜间模式' : 'Toggle day/night mode'}`,
        `F11 - ${zh ? '切换沉浸式全屏' : 'Toggle immersive fullscreen'}`,
        `F12 - ${zh ? '归档当前标签页' : 'Archive current tab'}`,
        `F10 - ${zh ? '切换标签栏方向' : 'Toggle tab bar orientation'}`,
        `Esc - ${zh ? '隐藏窗口' : 'Hide window'}`,
      ],
    }
  }
  if (topic === 'markdown') {
    return {
      title: zh ? 'Markdown 简明指南' : 'Markdown Quick Guide',
      lines: zh ? [
        '# 一级标题；## 二级标题；### 三级标题',
        '**粗体**；*斜体*；~~删除线~~',
        '- 无序列表；1. 有序列表；- [ ] 待办；- [x] 已完成',
        '[链接文字](https://example.com)；![图片说明](图片地址)',
        '> 引用文字；`行内代码`；三个反引号包裹代码块',
        '--- 单独一行可插入分隔线。段落之间空一行。',
      ] : [
        '# Heading 1; ## Heading 2; ### Heading 3',
        '**bold**; *italic*; ~~strikethrough~~',
        '- Bulleted list; 1. numbered list; - [ ] task; - [x] done',
        '[link text](https://example.com); ![image description](image-url)',
        '> Quote; `inline code`; wrap code blocks in three backticks',
        'Use --- on its own line for a divider. Leave a blank line between paragraphs.',
      ],
    }
  }
  if (topic === 'expression') {
    return {
      title: zh ? '表达式计算指南' : 'Expression Guide',
      lines: zh ? [
        '在编辑模式下，输入一行数学表达式后按 Ctrl+Enter，NeoPad 会在行尾追加计算结果。',
        '支持 +, -, *, /, %, ^ 和括号，也支持 × 和 ÷ 符号。',
        '示例：899*565-451 按 Ctrl+Enter 后变为 899*565-451 = 507484。',
        '如果行内包含非表达式文字，会尽量计算可识别的前缀部分。',
      ] : [
        'In edit mode, type a math expression on one line and press Ctrl+Enter. NeoPad appends the result to that line.',
        'Supported operators: +, -, *, /, %, ^, parentheses, ×, and ÷.',
        'Example: 899*565-451 becomes 899*565-451 = 507484.',
        'If the line contains non-expression text, NeoPad tries to calculate the recognizable expression prefix.',
      ],
    }
  }
  if (topic === 'about') {
    return {
      title: zh ? '关于 NeoPad' : 'About NeoPad',
      lines: zh ? [
        'NeoPad - 轻量、本地优先的 Markdown 桌面便笺。',
        ...(context.appVersion ? [`版本：${context.appVersion}`] : []),
        '作者：TrevanZhang',
        '开源项目：https://github.com/trevanzhang/neopad',
        '开源协议：MIT License',
        '技术栈：Tauri 2, Vue 3, TypeScript, Rust',
      ] : [
        'NeoPad - a lightweight, local-first Markdown desktop note pad.',
        ...(context.appVersion ? [`Version: ${context.appVersion}`] : []),
        'Author: TrevanZhang',
        'Open source: https://github.com/trevanzhang/neopad',
        'License: MIT License',
        'Built with Tauri 2, Vue 3, TypeScript, and Rust.',
      ],
    }
  }
  return {
    title: zh ? '软件说明' : 'Software Help',
    lines: zh ? [
      'NeoPad 是一款轻量的本地优先桌面便笺，专注于快速记录和查找。',
      '笔记以 Markdown 文件自动保存在本地，无需账号，不依赖云服务。',
      '支持多标签页、全文搜索、剪贴板采集、Markdown 预览、Vim 键位和行内计算。',
      '独立 MCP 服务器可供本地 AI 工具访问同一笔记工作区，默认只读。',
    ] : [
      'NeoPad is a lightweight, local-first desktop note pad focused on fast capture and retrieval.',
      'Notes are autosaved locally as Markdown files. No account or cloud service is required.',
      'It supports tabs, full-text search, clipboard capture, Markdown preview, Vim keys, and inline calculations.',
      'A standalone, read-only-by-default MCP server lets local AI tools access the same note workspace.',
    ],
  }
}
