# AI collaboration

NeoPad includes optional, user-initiated AI collaboration inside the editor.
It combines a lightweight note chat opened with `Ctrl+K` and quick Slash
commands. It does not add a permanent chat sidebar or on-disk chat history.

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

## Prompt library

Reusable user prompts live as UTF-8 Markdown files in:

```text
~/.neopad/prompts/*.md
```

The file name is the prompt name and the Markdown body is the instruction. Use
the `+` button in the `Ctrl+K` composer to search and attach one prompt. The
prompt remains visible as a removable chip and is sent only when the user sends
a message. The picker can open the prompts folder and refresh after files are
changed externally.

## Slash commands

Typing `/` at the start of a line or after whitespace opens the AI command
menu. The initial commands are:

- `/rewrite`
- `/summarize`
- `/translate`
- `/continue`
- `/ask`

Slash completion does not activate in Markdown code nodes. Selecting a command
removes its command token before opening the AI surface.

## Privacy and boundaries

- NeoPad sends note text only after the user explicitly runs an AI request.
- Requests are sent by the Rust shell; the API key is not exposed to frontend
  network code or stored in logs.
- Redirects are disabled so bearer credentials cannot be forwarded to another
  host.
- **All notes** searches locally first and sends only relevant excerpts, with
  their source note titles shown below the response.
- Conversations and selected prompts are kept in memory only and are discarded
  when NeoPad exits.
- NeoPad does not provide accounts, hosted inference, persistent AI chat
  history, RAG, vector search, or autonomous background agents.

The AI client is separate from `neopad-mcp`: the AI client calls a model service
from NeoPad, while MCP lets external agents call NeoPad note tools.
