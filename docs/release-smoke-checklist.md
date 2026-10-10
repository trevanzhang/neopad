# Release Smoke Checklist

Run this short manual check against the packaged Windows build before creating
or publishing a release.

1. On a clean install, verify the language page defaults to Chinese, then test
   both Chinese and English selections against NeoPad's first launch. Install
   over the previous NeoPad version and verify the existing app language is not
   overwritten.
2. Verify the version shown in Settings > About matches the installer version.
3. Verify the global show/hide shortcut and clipboard-capture shortcut.
4. Verify F2 through F10, plus F11 and F12, from an editor-focused note.
5. Verify tray Show, Hide, New Note, Save Clipboard, Settings, and Quit.
6. Right-click an internal tab and a library entry, then confirm Show in
   Explorer selects the corresponding Markdown file in its actual directory.
   Repeat with project folders containing Chinese, spaces, and commas, and
   with an archived note.
7. With File Explorer preview enabled for `clipboard.md`, trigger clipboard
   capture repeatedly and confirm the capture succeeds or retries without
   losing content.
8. If a simulated save remains blocked, restart NeoPad and verify the recovery
   prompt can restore the preserved note content.
9. Preview a note containing a fenced JavaScript block, a KaTeX formula, and a
   Mermaid diagram, then export it from both the File menu and tab context menu.
   Open the PNG and PDF and verify all three rendered elements are present and
   `Powered by NeoPad` appears at the end. Copy the mobile PNG to the clipboard,
   paste it into an image editor, and verify it is approximately 1080 pixels
   wide with larger text and the same footer. Select a dark preview theme and
   repeat the file, clipboard, mobile, and PDF exports; verify colors and
   typography match the preview, the footer remains readable, and every PDF
   page and margin uses the dark background. Select a light preview theme and
   confirm the same commands produce light output without separate Print or
   Preview menu branches.
10. Use File > Open NeoPad Data Folder and verify the workspace root opens,
    then copy the file path from an internal and an external Markdown tab and
    verify each absolute path.
11. Press F4, create a project and a note, edit and rename it, move the project
    into another folder, archive it, restore the note, delete and restore it.
    Confirm Ctrl+Shift+F search and reminders include project notes. Verify old
    nested and trashed prompts migrate once with unchanged bodies.

12. Select multi-line editor text, right-click it, and open it in a new tab;
    verify the new note keeps the selection beneath `# Untitled` and leaves the
    clipboard unchanged. Then run AI Rename on that tab and verify the tab title
    and first `#` heading match.
13. Open Settings > Agent tools and verify the deployed CLI is Ready. Copy its
    path, run `doctor` and `append-inbox --content-file PATH` from a terminal,
    and confirm saved editor content refreshes. Alter the deployed CLI in an
    isolated test workspace, repair it from Settings, and verify it runs again.
    Confirm a stale `update --expected-updated-at` fails without losing content.
