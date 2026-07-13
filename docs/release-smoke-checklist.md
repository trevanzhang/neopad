# Release Smoke Checklist

Run this short manual check against the packaged Windows build before creating
or publishing a release.

1. Install over the previous NeoPad version, then launch NeoPad.
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
