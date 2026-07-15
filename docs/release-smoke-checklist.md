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
   Explorer selects the corresponding Markdown file.
7. With File Explorer preview enabled for `clipboard.md`, trigger clipboard
   capture repeatedly and confirm the capture succeeds or retries without
   losing content.
8. If a simulated save remains blocked, restart NeoPad and verify the recovery
   prompt can restore the preserved note content.
9. Preview a note containing a fenced JavaScript block, a KaTeX formula, and a
   Mermaid diagram, then export it from both the File menu and tab context menu.
   Open the PNG and PDF and verify all three rendered elements are present.
10. Use File > Open NeoPad Data Folder and verify the workspace root opens,
    then copy the file path from an internal and an external Markdown tab and
    verify each absolute path.
11. Press F4, create and edit a prompt, switch away with F2/F3, rename it with
    F8, and attach it from Ctrl+K. Then close and reopen its prompt tab, move it
    to Trash, restore it, and confirm its Markdown content remains intact.
