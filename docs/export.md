# Markdown Preview and Export

NeoPad uses one Markdown rendering pipeline for split/preview mode and for the
current-note PNG and PDF commands. Keeping the pipeline shared prevents a note
from looking materially different after export.

## Using the Feature

Export the active note from either location:

- File > Export Current Note > As PNG Image, then choose Export PNG to File /
  Export PNG to Clipboard / Export PNG to Clipboard (Mobile), or choose As PDF
  Document.
- Right-click a tab and choose the equivalent PNG or PDF action.

All exports use the active preview theme plus the current preview font family,
size, and line spacing. After NeoPad reports that a PNG was copied, paste it
directly into a chat, document, image editor, or social app that accepts
clipboard images. The mobile variant uses a narrower 540 CSS pixel layout,
larger type, and 2x capture for an approximately 1080-pixel-wide image that
remains readable on phones.

Choose a light preview theme before exporting for white-background printing, or
choose any other preview theme for sharing. Mobile output keeps the selected
appearance while enforcing the mobile layout's minimum 18 px type and 1.75 line
spacing for readability.

The tab context menu can also copy the absolute path of its Markdown file. This
is useful when handing a note to a terminal command or local agent. Internal
notes resolve inside the NeoPad workspace; external Markdown tabs retain their
approved original paths. File > Open NeoPad Data Folder opens the complete
`~/.neopad/` workspace in the system file manager.

## Markdown Rendering

The shared renderer provides:

- Markdown source highlighting in CodeMirror 6.
- Fenced-code highlighting in preview and export.
- Curated editor language support for JavaScript/JSX, TypeScript/TSX, JSON,
  HTML, CSS, Python, Rust, and SQL.
- KaTeX inline, display, and fenced math.
- Mermaid diagrams rendered with strict security settings.

Raw HTML in Markdown is disabled. Invalid Mermaid input remains visible as a
code block instead of replacing the note with an error surface.

## Output Rules

- PNG is a content-only long image at a fixed reading width. It can be saved to
  a file or copied directly to the operating-system clipboard.
- PDF uses A4 portrait pages with 15 mm margins. Pagination prefers top-level
  Markdown block boundaries and falls back to a regular page cut for blocks
  taller than one page.
- Output uses the active preview palette and typography. PDF pages fill the
  full A4 page with the theme background, including the margins.
- Every PNG and PDF ends with a centered `Powered by NeoPad` footer. The footer
  is added only to the rendered export and never changes the Markdown note. Its
  muted text and divider colors follow the selected theme when applicable.
- The suggested file name comes from the tab title and is sanitized before the
  native Save dialog opens.

Before exporting the active tab, NeoPad completes the normal save barrier. A
save conflict or failed save stops the export. Exporting a different tab reads
that tab from its validated source path rather than using stale editor content;
this includes note, prompt, and external Markdown tabs.

## Performance and Limits

Mermaid, `html2canvas`, and `jsPDF` are loaded only when needed. Export waits
for fonts, images, formulas, and diagrams, with a 30-second overall timeout.
Canvas dimensions and total area are capped to avoid WebView crashes. Notes
that cannot fit within the safe capture scale report that they are too long to
export safely.

Remote images depend on their server's cross-origin policy. A blocked or broken
image does not stop the remaining Markdown from rendering, but it may be absent
from the exported file.

Mermaid diagrams follow the preview renderer's light/dark mode. Their palette is
therefore coordinated with the selected theme, but it is not customized
separately for every preview theme.

## Implementation Map

```text
app/src/lib/markdown.ts              Shared Markdown/KaTeX/Mermaid renderer
app/src/lib/editor-code-languages.ts Curated CodeMirror fence languages
app/src/lib/note-export.ts           Canvas capture and A4 pagination
app/src/composables/useNoteExport.ts  Save barrier and tab-source selection
app/src-tauri/src/commands.rs         Native dialogs, PNG clipboard, and reveal commands
crates/neopad-core/src/lib.rs         Atomic binary export writes
```

Relevant checks:

```powershell
pnpm test:frontend
cargo test --workspace
pnpm build
pnpm tauri:build
```
