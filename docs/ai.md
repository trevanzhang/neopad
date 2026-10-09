# AI collaboration

The same workflow is available inside NeoPad from **Help > AI Collaboration
Guide**.

NeoPad includes optional, user-initiated AI collaboration inside the editor.
It combines a lightweight note chat opened with `Ctrl+K` and independent
one-shot editing commands opened with `//`. It does not add a permanent chat
sidebar or on-disk chat history.

## Configure

Open `Settings -> AI` and provide:

- An OpenAI-compatible service URL.
- A model name understood by that service.
- An optional API key. Local services may not require one.

AI is off by default. Remote services must use HTTPS. HTTP is accepted only
for `localhost`, `127.0.0.1`, and `::1` so local model servers remain usable.
Use **Test connection** to check the configured `/models` endpoint.

The enabled state, URL, and model name are stored in `config.json`. The API key
is stored in the operating system credential manager and is never returned to
the Vue webview after it has been saved.

## Note chat

Press `Ctrl+K` while the editor is focused. A compact composer opens near the
bottom of the note and expands upward after the first message. The default
context is the current note. Turn on **All notes** to search the local workspace
and add a small set of relevant note excerpts to the request. NeoPad does not
send every note as one large context.

Each note keeps one in-memory conversation while NeoPad is running. Closing the
panel hides it; use **Clear** to start over. Conversations are discarded when
NeoPad exits. Each response offers actions to:

- Copy the result.
- Insert at the current cursor.
- Insert below the captured context.
- Replace the captured selection when one exists.

Replacement and insertion are applied as one CodeMirror transaction, so a
single undo restores the previous text. Replacement is refused when its
captured text has changed while the request was running.

## Project notes

Reusable prompt management and the Ctrl+K attachment picker have been removed.
Projects contain ordinary Markdown notes under `~/.neopad/projects/`; they use
normal editing, search, reminders, export, and native CLI operations. Existing
prompt files migrate without changing their bodies. See [Projects](projects.md).

## Quick editing commands

Type `//` anywhere in ordinary note text to open the command menu; no leading
space is required. With Chinese punctuation enabled, `、、` is accepted as the
same trigger. A single `/` or `、` has no AI behavior. The focused command set is:

- `//continue`
- `//polish`
- `//summarize`
- `//translate`

Right-click a text selection to polish, summarize, or translate that exact
selection without typing a Slash command. **Open Selection in New Tab** opens
the exact selection as the body of a new note; that local operation does not
call AI and does not change the system clipboard.

## AI note rename

Right-click an ordinary, non-default note tab and choose **AI Rename**. NeoPad
saves the note, sends its current content to the configured provider, and
requests one concise plain-text title. The response is reduced to one bounded
line before NeoPad updates both the tab metadata and the first `# title`
heading. If the note has no level-one heading, NeoPad prepends one without
discarding the existing first line. Inbox, Clipboard, and external
Markdown tabs are excluded.

These commands are separate from `Ctrl+K` chat: they do not use chat history,
all-note context. Polish and translate target the
selection or nearby paragraph. Summarize targets the selection or current
note. Continue uses the nearby paragraph. Translation defaults to automatic
Chinese-English switching.

The operation target and model reference context are separate. Continue,
polish, and translate keep their precise paragraph or selection write range,
while the model also receives the current note as read-only reference context.
Ordinary notes are included in full. Long notes keep the opening and a window
around the operation target, with omitted sections marked explicitly. Summarize
continues to use the complete note as its operation target.

Completion does not activate inside Markdown code nodes, URLs, paths, longer
slash runs, or ordinary single-slash text. Selecting a command removes its
token and shows a small animated marker at the captured cursor. The result is
then applied directly as one undoable CodeMirror transaction: continue and
summarize insert at the marker, while polish and translate replace their
unchanged source text. `Esc` cancels a pending command. Switching notes also
cancels it, and stale source text is never overwritten.

## Privacy and boundaries

- NeoPad sends note text only after the user explicitly runs an AI request.
- Requests are sent by the Rust shell; the API key is not exposed to frontend
  network code or stored in logs.
- Redirects are disabled so bearer credentials cannot be forwarded to another
  host.
- **All notes** searches locally first and sends only relevant excerpts, with
  their source note titles shown below the response.
- Conversations are kept in memory only and are discarded
  when NeoPad exits.
- NeoPad does not provide accounts, hosted inference, persistent AI chat
  history, RAG, vector search, or autonomous background agents.

The AI client is separate from `neopad-cli`: the AI client calls a model service
from NeoPad, while the native CLI lets external agents call NeoPad note tools.
