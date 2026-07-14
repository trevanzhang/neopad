import type { AppLanguage } from './i18n'
import { formatShortcutLabel } from './shortcut'

export type HelpTopic = 'software' | 'ai' | 'markdown' | 'shortcuts' | 'expression' | 'about'

export interface HelpContext {
  appVersion: string
  shortcutBaseKey: string
  shortcutModifiers: string[]
  clipboardShortcutBaseKey: string
  clipboardShortcutModifiers: string[]
}

export interface ShortcutHelpRow {
  keys: string
  description: string
}

export interface ShortcutHelpGroup {
  title: string
  rows: ShortcutHelpRow[]
}

export interface ReferenceHelpRow {
  value: string
  description: string
}

export interface ReferenceHelpGroup {
  title: string
  rows: ReferenceHelpRow[]
}

export interface ReferenceHelp {
  intro: string
  valueLabel: string
  descriptionLabel: string
  groups: ReferenceHelpGroup[]
}

export function getShortcutHelpGroups(language: AppLanguage, context: HelpContext): ShortcutHelpGroup[] {
  const zh = language === 'zh'
  return [
    {
      title: zh ? '\u5168\u5c40\u4e0e\u7a97\u53e3' : 'Global and window',
      rows: [
        { keys: formatShortcutLabel(context.shortcutBaseKey, context.shortcutModifiers), description: zh ? '\u663e\u793a/\u9690\u85cf\u7a97\u53e3' : 'Show or hide the window' },
        { keys: formatShortcutLabel(context.clipboardShortcutBaseKey, context.clipboardShortcutModifiers), description: zh ? '\u4fdd\u5b58\u5f53\u524d\u526a\u8d34\u677f' : 'Save the current clipboard' },
        { keys: 'F1', description: zh ? '\u6253\u5f00\u5feb\u6377\u952e\u8bf4\u660e' : 'Open shortcut help' },
        { keys: 'Ctrl+,', description: zh ? '\u6253\u5f00\u8bbe\u7f6e' : 'Open Settings' },
        { keys: 'Alt+Enter', description: zh ? '\u6700\u5927\u5316/\u8fd8\u539f\u7a97\u53e3' : 'Maximize or restore the window' },
        { keys: 'Esc', description: zh ? '\u5173\u95ed\u5f53\u524d\u9762\u677f\uff1b\u65e0\u9762\u677f\u65f6\u9690\u85cf\u7a97\u53e3' : 'Close the current panel; hide the window when no panel is open' },
      ],
    },
    {
      title: zh ? '\u7b14\u8bb0\u4e0e\u6807\u7b7e\u9875' : 'Notes and tabs',
      rows: [
        { keys: 'Ctrl+N', description: zh ? '\u65b0\u5efa\u6807\u7b7e\u9875' : 'Create a new tab' },
        { keys: 'Ctrl+O', description: zh ? '\u6253\u5f00 Markdown \u6587\u4ef6' : 'Open a Markdown file' },
        { keys: 'F2 / F3', description: zh ? '\u5207\u6362\u5230\u4e0a\u4e00\u4e2a/\u4e0b\u4e00\u4e2a\u6807\u7b7e\u9875' : 'Switch to the previous or next tab' },
        { keys: 'Ctrl+Tab / Ctrl+Shift+Tab', description: zh ? '\u5207\u6362\u5230\u4e0b\u4e00\u4e2a/\u4e0a\u4e00\u4e2a\u6807\u7b7e\u9875' : 'Switch to the next or previous tab' },
        { keys: 'Ctrl+W', description: zh ? '\u5173\u95ed\u5f53\u524d\u6807\u7b7e\u9875\uff08\u4e0d\u5220\u9664\u6587\u4ef6\uff09' : 'Close the current tab without deleting its file' },
        { keys: 'Alt+Del', description: zh ? '\u5c06\u5f53\u524d\u7b14\u8bb0\u6216\u63d0\u793a\u8bcd\u79fb\u5165\u56de\u6536\u7ad9' : 'Move the current note or prompt to Trash' },
        { keys: 'F4', description: zh ? '\u663e\u793a/\u9690\u85cf\u7b14\u8bb0\u4e0e\u63d0\u793a\u8bcd\u6587\u4ef6\u6d4f\u89c8\u5668' : 'Show or hide the note and prompt file browser' },
        { keys: 'F8', description: zh ? '\u91cd\u547d\u540d\u5f53\u524d\u6807\u7b7e\u9875' : 'Rename the current tab' },
        { keys: 'F12', description: zh ? '\u5f52\u6863\u5f53\u524d\u7b14\u8bb0' : 'Archive the current note' },
      ],
    },
    {
      title: zh ? '\u7f16\u8f91\u3001\u67e5\u627e\u4e0e\u63d2\u5165' : 'Edit, find, and insert',
      rows: [
        { keys: 'Ctrl+F / Ctrl+R', description: zh ? '\u67e5\u627e/\u66ff\u6362\u5f53\u524d\u7b14\u8bb0' : 'Find or replace in the current note' },
        { keys: 'Ctrl+G', description: zh ? '\u67e5\u627e\u4e0b\u4e00\u4e2a' : 'Find the next match' },
        { keys: 'Ctrl+Shift+F', description: zh ? '\u5168\u5c40\u641c\u7d22' : 'Search all NeoPad notes' },
        { keys: 'Ctrl+K', description: zh ? '\u6253\u5f00 AI \u7b14\u8bb0\u5bf9\u8bdd' : 'Open AI note chat' },
        { keys: 'Ctrl+D', description: zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4' : 'Insert date and time' },
        { keys: 'Ctrl+-', description: zh ? '\u63d2\u5165\u5206\u9694\u884c' : 'Insert a divider' },
        { keys: 'Ctrl+Shift+-', description: zh ? '\u63d2\u5165\u65e5\u671f\u65f6\u95f4\u5206\u9694\u884c' : 'Insert a dated divider' },
        { keys: 'Ctrl+E', description: zh ? '\u63d2\u5165\u63d0\u9192' : 'Insert a reminder' },
        { keys: 'Ctrl+Enter', description: zh ? '\u8ba1\u7b97\u5f53\u524d\u884c\u8868\u8fbe\u5f0f' : 'Calculate the current line expression' },
      ],
    },
    {
      title: zh ? '\u89c6\u56fe\u4e0e\u5de5\u5177' : 'View and tools',
      rows: [
        { keys: 'F5', description: zh ? '\u5faa\u73af\u5207\u6362\u7f16\u8f91\u3001\u5206\u5c4f\u548c\u9884\u89c8\u6a21\u5f0f' : 'Cycle Edit, Split, and Preview' },
        { keys: 'F6', description: zh ? '\u6253\u5f00/\u5173\u95ed\u63d0\u9192\u5217\u8868' : 'Open or close the reminder list' },
        { keys: 'F7', description: zh ? '\u5207\u6362\u7a97\u53e3\u7f6e\u9876' : 'Toggle always on top' },
        { keys: 'F9', description: zh ? '\u5207\u6362\u65e5\u95f4/\u591c\u95f4\u6a21\u5f0f' : 'Toggle day or night mode' },
        { keys: 'F10', description: zh ? '\u5207\u6362\u9884\u89c8\u4e3b\u9898' : 'Cycle the preview theme' },
        { keys: 'F11', description: zh ? '\u5207\u6362\u6c89\u6d78\u5f0f\u5168\u5c4f' : 'Toggle immersive fullscreen' },
      ],
    },
  ]
}

export function getReferenceHelp(topic: 'markdown' | 'expression', language: AppLanguage): ReferenceHelp {
  const zh = language === 'zh'
  if (topic === 'expression') {
    return {
      intro: zh
        ? '\u5728\u7f16\u8f91\u6a21\u5f0f\u4e0b\uff0c\u5c06\u5149\u6807\u505c\u5728\u5305\u542b\u7b97\u5f0f\u7684\u884c\u4e0a\u5e76\u6309 Ctrl+Enter\u3002NeoPad \u4f1a\u5728\u884c\u5c3e\u8ffd\u52a0\u7ed3\u679c\uff0c\u7136\u540e\u79fb\u5230\u4e0b\u4e00\u884c\u3002'
        : 'In Edit mode, place the cursor on a line containing an expression and press Ctrl+Enter. NeoPad appends the result, then moves to the next line.',
      valueLabel: zh ? '\u8f93\u5165' : 'Input',
      descriptionLabel: zh ? '\u8bf4\u660e' : 'Meaning',
      groups: [
        {
          title: zh ? '\u652f\u6301\u7684\u8fd0\u7b97\u7b26' : 'Supported operators',
          rows: [
            { value: '+  -', description: zh ? '\u52a0\u6cd5\u4e0e\u51cf\u6cd5\uff0c\u4e5f\u53ef\u8868\u793a\u6b63\u8d1f\u6570' : 'Addition and subtraction, including signed numbers' },
            { value: '*  \u00d7', description: zh ? '\u4e58\u6cd5\uff0c\u661f\u53f7\u548c\u4e58\u53f7\u5747\u53ef' : 'Multiplication using an asterisk or multiplication sign' },
            { value: '/  \u00f7', description: zh ? '\u9664\u6cd5\uff0c\u659c\u6760\u548c\u9664\u53f7\u5747\u53ef' : 'Division using a slash or division sign' },
            { value: '%', description: zh ? '\u53d6\u4f59' : 'Remainder' },
            { value: '^', description: zh ? '\u4e58\u65b9\uff0c\u4f8b\u5982 2^3 = 8' : 'Exponentiation, for example 2^3 = 8' },
            { value: '( )', description: zh ? '\u6539\u53d8\u8fd0\u7b97\u987a\u5e8f' : 'Control evaluation order' },
          ],
        },
        {
          title: zh ? '\u8ba1\u7b97\u793a\u4f8b' : 'Examples',
          rows: [
            { value: '899*565-451', description: '= 507484' },
            { value: '\u9884\u7b97\uff1a(12.5 + 7.5) \u00d7 3', description: zh ? '= 60\uff08\u53ef\u5e26\u6587\u5b57\u524d\u7f00\uff09' : '= 60 (a text prefix is allowed)' },
            { value: '2 ^ 3 ^ 2', description: zh ? '= 512\uff08\u4e58\u65b9\u6309\u53f3\u7ed3\u5408\u8ba1\u7b97\uff09' : '= 512 (exponents are right-associative)' },
            { value: '10 \u00f7 4', description: '= 2.5' },
          ],
        },
        {
          title: zh ? '\u8bc6\u522b\u89c4\u5219\u4e0e\u8fb9\u754c' : 'Recognition rules and limits',
          rows: [
            { value: zh ? '\u6df7\u5408\u6587\u672c' : 'Mixed text', description: zh ? '\u4ece\u884c\u5185\u7b2c\u4e00\u4e2a\u53ef\u8bc6\u522b\u7684\u6570\u5b57\u6216\u7b97\u5f0f\u5f00\u59cb\u8ba1\u7b97' : 'Evaluation starts at the first recognizable number or expression' },
            { value: '1 + (2 * 3', description: zh ? '\u7f3a\u5c11\u7684\u53f3\u62ec\u53f7\u4f1a\u81ea\u52a8\u8865\u9f50\uff0c\u7ed3\u679c\u4e3a 7' : 'A missing closing parenthesis is completed; the result is 7' },
            { value: '2 ** 3  /  1 / 0', description: zh ? '\u65e0\u6548\u6216\u975e\u6709\u9650\u7ed3\u679c\u4e0d\u4f1a\u5199\u5165\u7b14\u8bb0' : 'Invalid or non-finite results are not inserted' },
            { value: zh ? '\u51fd\u6570\u4e0e\u53d8\u91cf' : 'Functions and variables', description: zh ? '\u6682\u4e0d\u652f\u6301 sin\u3001sqrt\u3001\u53d8\u91cf\u548c\u5355\u4f4d\u6362\u7b97' : 'Functions such as sin or sqrt, variables, and unit conversion are not supported' },
          ],
        },
      ],
    }
  }

  return {
    intro: zh
      ? 'NeoPad \u4f7f\u7528\u6807\u51c6 Markdown \u5b8c\u6210\u5feb\u901f\u6392\u7248\uff0c\u5e76\u5728\u5206\u5c4f\u3001\u9884\u89c8\u548c\u5bfc\u51fa\u4e2d\u6e32\u67d3\u4ee3\u7801\u3001KaTeX \u516c\u5f0f\u4e0e Mermaid \u56fe\u8868\u3002'
      : 'NeoPad uses standard Markdown for fast formatting and renders code, KaTeX math, and Mermaid diagrams in Split, Preview, and export output.',
    valueLabel: zh ? '\u8bed\u6cd5' : 'Syntax',
    descriptionLabel: zh ? '\u6548\u679c' : 'Result',
    groups: [
      {
        title: zh ? '\u7ed3\u6784\u4e0e\u6bb5\u843d' : 'Structure and paragraphs',
        rows: [
          { value: '# / ## / ###', description: zh ? '\u4e00\u7ea7\u3001\u4e8c\u7ea7\u548c\u4e09\u7ea7\u6807\u9898' : 'Level 1, 2, and 3 headings' },
          { value: zh ? '\u7a7a\u767d\u884c' : 'Blank line', description: zh ? '\u5206\u9694\u6bb5\u843d' : 'Separate paragraphs' },
          { value: '---', description: zh ? '\u6c34\u5e73\u5206\u9694\u7ebf\uff08\u5355\u72ec\u4e00\u884c\uff09' : 'Horizontal divider when used on its own line' },
          { value: '> \u5f15\u7528', description: zh ? '\u5f15\u7528\u5757' : 'Block quote' },
        ],
      },
      {
        title: zh ? '\u6587\u5b57\u3001\u94fe\u63a5\u4e0e\u56fe\u7247' : 'Text, links, and images',
        rows: [
          { value: '**\u7c97\u4f53**', description: zh ? '\u7c97\u4f53' : 'Bold' },
          { value: '*\u659c\u4f53*', description: zh ? '\u659c\u4f53' : 'Italic' },
          { value: '~~\u5220\u9664\u7ebf~~', description: zh ? '\u5220\u9664\u7ebf' : 'Strikethrough' },
          { value: '`\u884c\u5185\u4ee3\u7801`', description: zh ? '\u884c\u5185\u4ee3\u7801' : 'Inline code' },
          { value: '[\u6587\u5b57](https://example.com)', description: zh ? '\u53ef\u70b9\u51fb\u94fe\u63a5' : 'Clickable link' },
          { value: '![\u8bf4\u660e](\u8def\u5f84\u6216 URL)', description: zh ? '\u56fe\u7247' : 'Image' },
        ],
      },
      {
        title: zh ? '\u5217\u8868\u4e0e\u4efb\u52a1' : 'Lists and tasks',
        rows: [
          { value: '- \u9879\u76ee', description: zh ? '\u65e0\u5e8f\u5217\u8868' : 'Bulleted list' },
          { value: '1. \u9879\u76ee', description: zh ? '\u6709\u5e8f\u5217\u8868' : 'Numbered list' },
          { value: '- [ ] \u4efb\u52a1', description: zh ? '\u672a\u5b8c\u6210\u4efb\u52a1' : 'Open task' },
          { value: '- [x] \u4efb\u52a1', description: zh ? '\u5df2\u5b8c\u6210\u4efb\u52a1' : 'Completed task' },
          { value: '- [ ] @remind YYYY-MM-DD HH:mm \u5185\u5bb9', description: zh ? 'NeoPad \u63d0\u9192' : 'NeoPad reminder' },
        ],
      },
      {
        title: zh ? '\u4ee3\u7801\u3001\u516c\u5f0f\u4e0e\u56fe\u8868' : 'Code, math, and diagrams',
        rows: [
          { value: '```js  ...  ```', description: zh ? '\u5e26\u8bed\u8a00\u7684\u4ee3\u7801\u5757\u4e0e\u8bed\u6cd5\u9ad8\u4eae' : 'Fenced code block with language highlighting' },
          { value: '$E = mc^2$', description: zh ? '\u884c\u5185 KaTeX \u516c\u5f0f' : 'Inline KaTeX formula' },
          { value: '$$  ...  $$ / ```math  ...  ```', description: zh ? '\u72ec\u7acb KaTeX \u516c\u5f0f' : 'Display or fenced KaTeX formula' },
          { value: '```mermaid  ...  ```', description: zh ? 'Mermaid \u56fe\u8868' : 'Mermaid diagram' },
        ],
      },
      {
        title: zh ? 'NeoPad \u6e32\u67d3\u8bf4\u660e' : 'NeoPad rendering notes',
        rows: [
          { value: 'F5', description: zh ? '\u5728\u7f16\u8f91\u3001\u5206\u5c4f\u548c\u9884\u89c8\u6a21\u5f0f\u4e4b\u95f4\u5207\u6362' : 'Cycle Edit, Split, and Preview' },
          { value: '<div>...</div>', description: zh ? '\u539f\u59cb HTML \u4e3a\u4e86\u5b89\u5168\u4e0d\u4f1a\u6e32\u67d3' : 'Raw HTML is not rendered for safety' },
          { value: zh ? '\u5bfc\u51fa PNG / PDF' : 'Export PNG / PDF', description: zh ? '\u4e0e\u9884\u89c8\u5171\u7528\u540c\u4e00\u6e32\u67d3\u6d41\u7a0b' : 'Uses the same rendering pipeline as Preview' },
        ],
      },
    ],
  }
}

export function getHelpContent(topic: HelpTopic | null, language: AppLanguage, context: HelpContext) {
  const zh = language === 'zh'
  if (topic === 'ai') {
    return {
      title: zh ? 'AI \u534f\u4f5c\u6307\u5357' : 'AI Collaboration Guide',
      lines: zh ? [
        '## 1. \u542f\u7528\u4e0e\u914d\u7f6e',
        '\u6253\u5f00\u8bbe\u7f6e\uff08Ctrl+,\uff09\u2192 AI\uff0c\u542f\u7528 AI \u534f\u4f5c\uff0c\u586b\u5199 OpenAI \u517c\u5bb9\u670d\u52a1 URL\u3001\u6a21\u578b\u540d\u79f0\u548c API Key\uff0c\u7136\u540e\u4f7f\u7528\u201c\u6d4b\u8bd5\u8fde\u63a5\u201d\u786e\u8ba4\u914d\u7f6e\u3002',
        '## 2. // \u5feb\u6377\u547d\u4ee4',
        '\u5728\u7f16\u8f91\u533a\u8f93\u5165 //\uff0c\u9009\u62e9\u7eed\u5199\u3001\u6da6\u8272\u3001\u603b\u7ed3\u6216\u7ffb\u8bd1\u3002\u7eed\u5199\u3001\u6da6\u8272\u548c\u7ffb\u8bd1\u4ee5\u5f53\u524d\u6bb5\u843d\u4e3a\u64cd\u4f5c\u76ee\u6807\uff1b\u603b\u7ed3\u9ed8\u8ba4\u5904\u7406\u6574\u7bc7\u7b14\u8bb0\u3002\u7ffb\u8bd1\u9ed8\u8ba4\u4e2d\u82f1\u4e92\u8bd1\u3002',
        '\u547d\u4ee4\u8fd4\u56de\u540e\u4f1a\u4f5c\u4e3a\u4e00\u6b21\u53ef\u64a4\u9500\u7684\u7f16\u8f91\u81ea\u52a8\u5e94\u7528\u3002\u6574\u7bc7\u7b14\u8bb0\u4ec5\u4f5c\u4e3a\u6a21\u578b\u53c2\u8003\u4e0a\u4e0b\u6587\uff0c\u957f\u7b14\u8bb0\u4f1a\u81ea\u52a8\u88c1\u526a\u3002',
        '## 3. \u5904\u7406\u9009\u4e2d\u6587\u5b57',
        '\u7528\u9f20\u6807\u62d6\u9009\u6216 Shift+\u65b9\u5411\u952e\u9009\u4e2d\u6587\u5b57\uff0c\u7136\u540e\u5728\u9009\u533a\u5185\u53f3\u51fb\uff0c\u6253\u5f00\u201cAI \u5904\u7406\u9009\u4e2d\u6587\u5b57\u201d\u5b50\u83dc\u5355\u3002\u6da6\u8272\u548c\u7ffb\u8bd1\u4f1a\u66ff\u6362\u9009\u533a\uff0c\u603b\u7ed3\u7ed3\u679c\u4f1a\u63d2\u5165\u5728\u9009\u533a\u672b\u5c3e\u3002',
        '## 4. Ctrl+K \u7b14\u8bb0\u5bf9\u8bdd',
        '\u6309 Ctrl+K \u53ef\u56f4\u7ed5\u5f53\u524d\u7b14\u8bb0\u8fde\u7eed\u63d0\u95ee\u3002\u201c\u5f53\u524d\u7b14\u8bb0\u201d\u53ea\u53d1\u9001\u672c\u9875\u5185\u5bb9\uff1b\u201c\u5b8c\u5168\u8bbf\u95ee\u201d\u4f1a\u5148\u5728\u672c\u5730\u641c\u7d22\u7b14\u8bb0\u5e93\uff0c\u53ea\u9644\u52a0\u76f8\u5173\u7247\u6bb5\u3002AI \u7ed3\u679c\u4e0d\u4f1a\u81ea\u52a8\u4fee\u6539\u7b14\u8bb0\uff0c\u53ef\u4ee5\u590d\u5236\u3001\u63d2\u5165\u5149\u6807\u5904\u6216\u63d2\u5165\u4e0b\u65b9\u3002',
        '## 5. \u63d0\u793a\u8bcd\u5e93',
        '\u70b9\u51fb Ctrl+K \u8f93\u5165\u6846\u5de6\u4e0b\u89d2\u7684 +\uff0c\u53ef\u5f15\u7528\u63d0\u793a\u8bcd\u5e93\u4e2d\u7684 Markdown \u6587\u4ef6\u3002\u9009\u62e9\u5668\u4f1a\u6309\u63d0\u793a\u8bcd\u6807\u9898\u3001\u76ee\u5f55\u5206\u7c7b\u548c\u4e00\u884c\u5185\u5bb9\u9884\u89c8\u663e\u793a\u3002\u6309 F4 \u53ef\u65b0\u5efa\u76ee\u5f55\u3001\u62d6\u62fd\u6587\u4ef6\u6216\u6574\u4e2a\u76ee\u5f55\u8fdb\u884c\u5206\u7c7b\uff0c\u53f3\u952e\u76ee\u5f55\u53ef\u91cd\u547d\u540d\u6216\u5c06\u5176\u5185\u5bb9\u79fb\u5165\u56de\u6536\u7ad9\uff1b\u70b9\u51fb\u63d0\u793a\u8bcd\u540e\u4f1a\u5728\u4e3b\u7f16\u8f91\u5668\u4e2d\u4f5c\u4e3a\u7279\u6b8a\u6807\u7b7e\u9875\u6253\u5f00\u3002\u63d0\u793a\u8bcd\u4fdd\u5b58\u5728 ~/.neopad/prompts/**/*.md\uff0c\u4e0e\u7b14\u8bb0\u5185\u5bb9\u5206\u5f00\u7ba1\u7406\u3002',
        '## 6. \u9690\u79c1\u4e0e\u6570\u636e\u8303\u56f4',
        'AI \u9ed8\u8ba4\u5173\u95ed\uff0c\u53ea\u6709\u5728\u7528\u6237\u6267\u884c\u547d\u4ee4\u6216\u53d1\u9001\u5bf9\u8bdd\u65f6\u624d\u4f1a\u5411\u914d\u7f6e\u7684\u6a21\u578b\u670d\u52a1\u53d1\u9001\u6587\u672c\u3002API Key \u4fdd\u5b58\u5728\u7cfb\u7edf\u51ed\u636e\u7ba1\u7406\u5668\uff0c\u5bf9\u8bdd\u8bb0\u5fc6\u53ea\u5b58\u5728\u4e8e\u5f53\u524d\u8fd0\u884c\u671f\u95f4\uff0c\u4e0d\u5199\u5165\u7b14\u8bb0\u6216\u65e5\u5fd7\u3002',
        '\u5efa\u8bae\uff1a\u5feb\u901f\u5904\u7406\u5f53\u524d\u6bb5\u843d\u7528 //\uff0c\u7cbe\u786e\u5904\u7406\u9009\u533a\u7528\u53f3\u952e\uff0c\u9700\u8981\u591a\u8f6e\u8ba8\u8bba\u6216\u53cd\u590d\u8c03\u6574\u65f6\u7528 Ctrl+K\u3002',
      ] : [
        '## 1. Enable and configure',
        'Open Settings (Ctrl+,) -> AI, enable AI collaboration, enter an OpenAI-compatible service URL, model name, and API key, then use Test connection to verify the configuration.',
        '## 2. // quick commands',
        'Type // in the editor and choose Continue, Polish, Summarize, or Translate. Continue, polish, and translate operate on the current paragraph; summarize defaults to the whole note. Translation automatically switches between Chinese and English.',
        'The result is applied as one undoable edit. The full note is reference context only, and long notes are automatically bounded.',
        '## 3. Process selected text',
        'Select text with the mouse or Shift+Arrow, then right-click inside the selection and open AI actions for selection. Polish and translate replace the selection; summarize inserts its result after the selection.',
        '## 4. Ctrl+K note chat',
        'Press Ctrl+K for an ongoing conversation about the current note. Current note sends only this page; Full access first searches locally and adds only relevant excerpts. Chat never changes a note automatically: copy or explicitly insert a result.',
        '## 5. Prompt library',
        'Use the + button in Ctrl+K to attach a Markdown prompt. The picker shows each prompt title, folder category, and a one-line content preview. Press F4 to create folders and organize prompts or complete folder trees by drag and drop. Right-click a folder to rename it or move its contents to Trash. Prompts can also be duplicated, renamed, trashed, restored, or opened as marked tabs in the main editor. Prompt files stay under ~/.neopad/prompts/**/*.md and remain separate from notes.',
        '## 6. Privacy and data scope',
        'AI is off by default. Text is sent only when you run a command or send a chat message. API keys stay in the system credential manager, and chat memory is kept only for the current app session without being written to notes or logs.',
        'Tip: use // for the current paragraph, right-click for an exact selection, and Ctrl+K for multi-turn discussion or refinement.',
      ],
    }
  }
  if (topic === 'shortcuts') {
    const groups = getShortcutHelpGroups(language, context)
    return {
      title: zh ? '\u5feb\u6377\u952e\u8bf4\u660e' : 'Keyboard Shortcuts',
      lines: groups.flatMap((group) => group.rows.map((row) => `${row.keys} - ${row.description}`)),
    }
  }
  if (topic === 'markdown') {
    const reference = getReferenceHelp('markdown', language)
    return {
      title: zh ? 'Markdown 简明指南' : 'Markdown Quick Guide',
      lines: [reference.intro, ...reference.groups.flatMap((group) => [
        `## ${group.title}`,
        ...group.rows.map((row) => `${row.value} - ${row.description}`),
      ])],
    }
  }
  if (topic === 'expression') {
    const reference = getReferenceHelp('expression', language)
    return {
      title: zh ? '表达式计算指南' : 'Expression Guide',
      lines: [reference.intro, ...reference.groups.flatMap((group) => [
        `## ${group.title}`,
        ...group.rows.map((row) => `${row.value} - ${row.description}`),
      ])],
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
    title: zh ? '\u8f6f\u4ef6\u8bf4\u660e' : 'Software Help',
    lines: zh ? [
      '## 1. NeoPad \u662f\u4ec0\u4e48',
      'NeoPad \u662f\u4e00\u6b3e\u8f7b\u91cf\u3001\u672c\u5730\u4f18\u5148\u7684 Markdown \u684c\u9762\u4fbf\u7b3a\uff0c\u4e13\u6ce8\u5feb\u901f\u8bb0\u5f55\u3001\u968f\u624b\u6574\u7406\u548c\u53ca\u65f6\u627e\u56de\u3002\u5b83\u662f\u4e00\u4e2a\u9ad8\u6548\u7684\u6355\u6349\u5de5\u5177\uff0c\u4e0d\u8bd5\u56fe\u6210\u4e3a\u590d\u6742\u7684\u77e5\u8bc6\u5e93\u5957\u4ef6\u3002',
      '## 2. \u7b14\u8bb0\u3001\u6807\u7b7e\u9875\u4e0e\u672c\u5730\u6587\u4ef6',
      '\u6536\u4ef6\u7bb1\u548c\u526a\u8d34\u677f\u662f\u7cfb\u7edf\u9ed8\u8ba4\u9875\uff1b\u5176\u4ed6\u7b14\u8bb0\u53ef\u65b0\u5efa\u3001\u91cd\u547d\u540d\u3001\u5173\u95ed\u3001\u5f52\u6863\u6216\u79fb\u5165\u56de\u6536\u7ad9\u3002\u5173\u95ed\u6807\u7b7e\u9875\u4e0d\u4f1a\u5220\u9664\u7b14\u8bb0\uff1b\u5185\u5bb9\u4f1a\u81ea\u52a8\u4fdd\u5b58\u4e3a ~/.neopad \u4e0b\u7684\u666e\u901a Markdown \u6587\u4ef6\u3002',
      '## 3. \u7f16\u8f91\u4e0e\u9884\u89c8',
      '\u6309 F5 \u5728\u7f16\u8f91\u3001\u5206\u5c4f\u548c\u9884\u89c8\u6a21\u5f0f\u4e4b\u95f4\u5207\u6362\u3002NeoPad \u652f\u6301 Markdown \u6e90\u7801\u4e0e\u4ee3\u7801\u5757\u9ad8\u4eae\u3001KaTeX \u516c\u5f0f\u3001Mermaid \u56fe\u8868\u3001\u65e5\u95f4/\u591c\u95f4\u4e3b\u9898\u3001\u9884\u89c8\u6392\u7248\u4ee5\u53ca\u53ef\u9009\u7684 Vim \u952e\u4f4d\u3002',
      '## 4. \u67e5\u627e\u3001\u6574\u7406\u4e0e\u63d0\u9192',
      'Ctrl+F \u67e5\u627e\u5f53\u524d\u6587\u6863\uff0cCtrl+Shift+F \u641c\u7d22 NeoPad \u7b14\u8bb0\uff0cF4 \u6253\u5f00\u6587\u4ef6\u6d4f\u89c8\u5668\u7ba1\u7406\u7b14\u8bb0\u548c\u63d0\u793a\u8bcd\u3002Ctrl+E \u63d2\u5165\u63d0\u9192\uff0cF6 \u67e5\u770b\u63d0\u9192\u5217\u8868\uff1bNeoPad \u8fd0\u884c\u65f6\u53ef\u53d1\u9001\u7cfb\u7edf\u901a\u77e5\u3002',
      '## 5. \u5feb\u901f\u91c7\u96c6\u4e0e\u6587\u4ef6\u4ea4\u6362',
      '\u53ef\u5728\u8bbe\u7f6e\u4e2d\u914d\u7f6e\u5168\u5c40\u663e\u793a/\u9690\u85cf\u548c\u526a\u8d34\u677f\u91c7\u96c6\u5feb\u6377\u952e\uff0c\u4e5f\u53ef\u914d\u5408\u7cfb\u7edf\u6258\u76d8\u4f7f\u7528\u3002NeoPad \u53ef\u6253\u5f00\u6216\u62d6\u5165\u5916\u90e8 Markdown \u6587\u4ef6\uff0c\u5e76\u53ef\u5bfc\u51fa\u5f53\u524d\u7b14\u8bb0\u4e3a Markdown\u3001PNG \u6216 PDF\uff0c\u4e5f\u53ef\u5c06\u6240\u6709\u7b14\u8bb0\u5bfc\u51fa\u4e3a ZIP\u3002',
      '## 6. AI \u534f\u4f5c\u4e0e MCP',
      'AI \u534f\u4f5c\u9ed8\u8ba4\u5173\u95ed\uff1b\u542f\u7528\u5e76\u914d\u7f6e\u6a21\u578b\u540e\uff0c\u53ef\u7528 // \u5feb\u6377\u5904\u7406\u3001\u9009\u533a\u53f3\u952e AI \u64cd\u4f5c\u548c Ctrl+K \u7b14\u8bb0\u5bf9\u8bdd\u3002MCP \u662f\u4e0e AI \u5ba2\u6237\u7aef\u5206\u79bb\u7684\u672c\u5730\u670d\u52a1\uff0c\u4e5f\u9ed8\u8ba4\u5173\u95ed\uff1b\u542f\u7528\u540e\uff0c\u6301\u6709 Bearer Token \u7684\u672c\u5730\u5de5\u5177\u53ef\u8bfb\u5199\u7b14\u8bb0\u3002',
      '## 7. \u6570\u636e\u5b89\u5168\u4e0e\u4ea7\u54c1\u8fb9\u754c',
      'NeoPad \u65e0\u9700\u8d26\u53f7\uff0c\u4e0d\u5185\u7f6e\u4e91\u540c\u6b65\uff1b\u7b14\u8bb0\u4fdd\u6301\u4e3a\u672c\u5730\u53ef\u8bfb\u6587\u4ef6\uff0c\u5199\u5165\u4f7f\u7528\u539f\u5b50\u4fdd\u5b58\u548c\u51b2\u7a81\u68c0\u67e5\u3002\u5b83\u4e0d\u81ea\u52a8\u8bb0\u5f55\u526a\u8d34\u677f\u5386\u53f2\uff0c\u4e0d\u6301\u4e45\u5316 AI \u5bf9\u8bdd\uff0c\u4e5f\u4e0d\u63d0\u4f9b\u77e5\u8bc6\u56fe\u8c31\u3001\u53cc\u94fe\u6216 RAG \u7d22\u5f15\u3002',
      '\u5efa\u8bae\uff1a\u6309 F1 \u67e5\u770b\u5b8c\u6574\u5feb\u6377\u952e\uff1b\u9700\u8981\u5177\u4f53\u8bed\u6cd5\u6216\u529f\u80fd\u65f6\uff0c\u7ee7\u7eed\u67e5\u770b\u201cMarkdown \u7b80\u660e\u6307\u5357\u201d\u3001\u201c\u8868\u8fbe\u5f0f\u8ba1\u7b97\u6307\u5357\u201d\u548c\u201cAI \u534f\u4f5c\u6307\u5357\u201d\u3002',
    ] : [
      '## 1. What NeoPad is',
      'NeoPad is a lightweight, local-first Markdown desktop note pad for fast capture, quick organization, and timely retrieval. It is designed as an efficient capture tool, not a full knowledge-base suite.',
      '## 2. Notes, tabs, and local files',
      'Inbox and Clipboard are protected default pages. Other notes can be created, renamed, closed, archived, or moved to Trash. Closing a tab does not delete its note, and content is autosaved as ordinary Markdown files under ~/.neopad.',
      '## 3. Editing and preview',
      'Press F5 to cycle through Edit, Split, and Preview. NeoPad supports Markdown and fenced-code highlighting, KaTeX formulas, Mermaid diagrams, day and night themes, preview typography, and optional Vim keys.',
      '## 4. Find, organize, and remember',
      'Use Ctrl+F in the current document, Ctrl+Shift+F across NeoPad notes, and F4 to manage notes and prompts. Ctrl+E inserts a reminder and F6 opens the reminder list; NeoPad can notify you while it is running.',
      '## 5. Quick capture and file exchange',
      'Configure global show/hide and clipboard-capture shortcuts, or keep NeoPad close through the system tray. Open or drop external Markdown files, export the current note as Markdown, PNG, or PDF, and export all notes as a ZIP archive.',
      '## 6. AI collaboration and MCP',
      'AI collaboration is off by default. After configuring a model, use // quick actions, right-click AI actions for a selection, or Ctrl+K note chat. MCP is a separate local service and is also off by default; when enabled, local tools with its bearer token can read and write notes.',
      '## 7. Data safety and product boundaries',
      'NeoPad requires no account and includes no cloud sync. Notes stay readable local files protected by atomic saves and conflict checks. It does not automatically keep clipboard history, persist AI chats, or provide a knowledge graph, backlinks, or a RAG index.',
      'Tip: press F1 for the complete shortcut list, then use the Markdown, Expression, and AI Collaboration guides when you need feature-specific help.',
    ],
  }
}
