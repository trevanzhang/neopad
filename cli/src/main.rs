use anyhow::{bail, Context, Result};
use neopad_core::*;
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Read, path::PathBuf};

const HELP: &str = "NeoPad native CLI
Usage: neopad-cli [--workspace PATH] [--json] COMMAND [OPTIONS]
Workspace: --workspace, then NEOPAD_WORKSPACE, then ~/.neopad.
All command results are JSON: {ok, version, data} or {ok, error}.

Commands:
  doctor                           Inspect workspace and CLI without creating files
  list [--scope active|projects|archive|trash|all] [--project DIRECTORY]
  projects                         List project directories
  project-create DIRECTORY         Create a project directory
  project-rename DIRECTORY --name NAME
  project-move DIRECTORY --parent DIRECTORY
  project-archive DIRECTORY         Archive a project tree
  project-trash DIRECTORY           Trash every project note
  move ID --project DIRECTORY       Move a note into a project
  capture ID                       Move a note back to capture
  read ID                          Read a note and its updatedAt
  search QUERY [--limit 1..100]      Search active and archived notes
  create [--project DIRECTORY] [--title TITLE] [--content-file PATH | --stdin]
  append ID --content-file PATH | --stdin
  append-inbox --content-file PATH | --stdin
  append-clipboard --content-file PATH | --stdin  (does not read system clipboard)
  update ID --expected-updated-at N --content-file PATH | --stdin
  rename ID --title TITLE
  archive ID                       Move to archive
  unarchive ID                     Restore an archived note to active notes
  trash ID                         Move to trash; never physically delete
  restore ID                       Restore from trash

Content is UTF-8, limited to 2 MiB. --stdin reads standard input to EOF.
Exit codes: 0 success, 1 operation failed, 2 invalid arguments, 3 update conflict.
Use --help or --version without touching the workspace.
";
const MAX_CONTENT: usize = 2 * 1024 * 1024;

struct Args {
    workspace: Option<PathBuf>,
    command: String,
    positional: Vec<String>,
    options: BTreeMap<String, String>,
}

fn parse_args(arguments: Vec<String>) -> Result<Args> {
    let mut result = Args {
        workspace: std::env::var_os("NEOPAD_WORKSPACE").map(PathBuf::from),
        command: String::new(),
        positional: Vec::new(),
        options: BTreeMap::new(),
    };
    let mut args = arguments.into_iter();
    let mut literal = false;
    while let Some(arg) = args.next() {
        if arg == "--" && !literal {
            literal = true;
            continue;
        }
        if !literal && arg == "--json" {
            continue;
        }
        if !literal && arg == "--workspace" {
            result.workspace = Some(PathBuf::from(
                args.next().context("--workspace needs a path")?,
            ));
        } else if !literal && arg.starts_with('-') {
            let value = if arg == "--stdin" {
                String::new()
            } else {
                args.next()
                    .with_context(|| format!("{arg} needs a value"))?
            };
            if result.options.insert(arg, value).is_some() {
                bail!("duplicate option");
            }
        } else if result.command.is_empty() {
            result.command = arg;
        } else {
            result.positional.push(arg);
        }
    }
    let (count, allowed): (usize, &[&str]) = match result.command.as_str() {
        "doctor" => (0, &[]),
        "list" => (0, &["--scope", "--project"]),
        "projects" => (0, &[]),
        "project-create" | "project-archive" | "project-trash" | "capture" => (1, &[]),
        "project-rename" => (1, &["--name"]),
        "project-move" => (1, &["--parent"]),
        "move" => (1, &["--project"]),
        "read" | "archive" | "unarchive" | "trash" | "restore" => (1, &[]),
        "search" => (1, &["--limit"]),
        "rename" => (1, &["--title"]),
        "create" => (0, &["--project", "--title", "--content-file", "--stdin"]),
        "append" => (1, &["--content-file", "--stdin"]),
        "append-inbox" | "append-clipboard" => (0, &["--content-file", "--stdin"]),
        "update" => (1, &["--content-file", "--stdin", "--expected-updated-at"]),
        _ => bail!("unknown or missing command; use --help"),
    };
    if result.positional.len() != count {
        bail!("{} expects {count} positional argument(s)", result.command);
    }
    for key in result.options.keys() {
        if !allowed.contains(&key.as_str()) {
            bail!("unsupported option {key} for {}", result.command);
        }
    }
    if result.options.contains_key("--stdin") && result.options.contains_key("--content-file") {
        bail!("choose --stdin or --content-file, not both");
    }
    if matches!(
        result.command.as_str(),
        "append" | "append-inbox" | "append-clipboard" | "update"
    ) && !result.options.contains_key("--stdin")
        && !result.options.contains_key("--content-file")
    {
        bail!("content requires --content-file PATH or --stdin");
    }
    for (command, option) in [
        ("move", "--project"),
        ("project-rename", "--name"),
        ("project-move", "--parent"),
    ] {
        if result.command == command {
            result.required(option)?;
        }
    }
    if result.command == "rename" {
        result.required("--title")?;
    }
    if result.command == "update" {
        result
            .required("--expected-updated-at")?
            .parse::<i64>()
            .context("invalid expected timestamp")?;
    }
    if result.command == "list"
        && !matches!(
            result.option("--scope", "active"),
            "active" | "projects" | "archive" | "trash" | "all"
        )
    {
        bail!("scope must be active, projects, archive, trash, or all");
    }
    if result.command == "search" {
        let limit = result
            .option("--limit", "20")
            .parse::<usize>()
            .context("invalid search limit")?;
        if !(1..=100).contains(&limit) {
            bail!("search limit must be between 1 and 100");
        }
    }
    Ok(result)
}

impl Args {
    fn required(&self, name: &str) -> Result<&str> {
        self.options
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.trim().is_empty())
            .with_context(|| format!("{name} is required"))
    }
    fn option<'a>(&'a self, name: &str, fallback: &'a str) -> &'a str {
        self.options.get(name).map_or(fallback, String::as_str)
    }
    fn content(&self) -> Result<String> {
        let input: Box<dyn Read> = if let Some(path) = self.options.get("--content-file") {
            Box::new(std::fs::File::open(path).context("could not open content file")?)
        } else if self.options.contains_key("--stdin") {
            Box::new(std::io::stdin())
        } else {
            return Ok(String::new());
        };
        let mut bytes = Vec::new();
        input
            .take((MAX_CONTENT + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CONTENT {
            bail!("content exceeds 2 MiB");
        }
        let content = String::from_utf8(bytes).context("content must be UTF-8")?;
        Ok(content
            .strip_prefix('\u{feff}')
            .unwrap_or(&content)
            .to_owned())
    }
}

fn execute(args: Args) -> Result<Value> {
    let path = match args.workspace.as_ref() {
        Some(path) => path.clone(),
        None => default_workspace_dir()?,
    };
    if args.command == "doctor" {
        let workspace = Workspace::new(path);
        let mut diagnostics = serde_json::to_value(inspect_agent_workspace(&workspace))?;
        diagnostics["executable"] = json!(std::env::current_exe()?);
        diagnostics["version"] = json!(env!("CARGO_PKG_VERSION"));
        return Ok(diagnostics);
    }
    // Read potentially slow stdin before acquiring the shared workspace lock.
    let content = args.content()?;
    let workspace = init_workspace(Some(path))?;
    let _lock = lock_workspace_for_write(&workspace)?;
    reconcile_note_metadata(&workspace)?;
    let id = args.positional.first().map_or("", String::as_str);
    Ok(match args.command.as_str() {
        "list" => {
            let mut pages = match args.option("--scope", "active") {
                "projects" => list_project_notes(&workspace)?,
                "archive" => list_archived_notes(&workspace)?,
                "trash" => list_trashed_notes(&workspace)?,
                "all" => list_searchable_notes(&workspace)?,
                _ => list_notes(&workspace)?,
            };
            if let Some(project) = args.options.get("--project") {
                neopad_core::path::project_directory_path(&workspace, project)?;
                pages.retain(|page| {
                    page.project_relative_path
                        .as_deref()
                        .is_some_and(|path| path.starts_with(&format!("{project}/")))
                });
            }
            json!({ "pages": pages })
        }
        "read" => json!(read_note(&workspace, id)?),
        "search" => {
            json!({ "matches": search_notes(&workspace, id, args.option("--limit", "20").parse()?)? })
        }
        "create" => json!(if let Some(project) = args.options.get("--project") {
            create_project_note(
                &workspace,
                project,
                args.options.get("--title").cloned(),
                &content,
            )?
        } else {
            create_note_with_body(&workspace, args.options.get("--title").cloned(), &content)?
        }),
        "projects" => json!({"directories": list_project_directories(&workspace)?}),
        "project-create" => json!(create_project_directory(&workspace, id)?),
        "project-rename" => json!(rename_project_directory(
            &workspace,
            id,
            args.required("--name")?
        )?),
        "project-move" => json!(move_project_directory(
            &workspace,
            id,
            args.required("--parent")?
        )?),
        "project-archive" => json!(archive_project_directory(&workspace, id)?),
        "project-trash" => json!(delete_project_directory_to_trash(&workspace, id)?),
        "move" => json!(move_note_to_project(
            &workspace,
            id,
            args.required("--project")?
        )?),
        "capture" => json!(move_note_to_capture(&workspace, id)?),
        "append" => json!(append_to_note(&workspace, id, &content)?),
        "append-inbox" => json!(append_to_note(&workspace, "inbox", &content)?),
        "append-clipboard" => json!(append_to_clipboard_note(&workspace, &content)?),
        "update" => json!(write_note_atomic_checked(
            &workspace,
            id,
            &content,
            args.required("--expected-updated-at")?.parse()?
        )?),
        "rename" => json!(rename_note_with_heading(
            &workspace,
            id,
            args.required("--title")?.to_owned()
        )?),
        "archive" => json!(archive_note(&workspace, id)?),
        "unarchive" => json!(unarchive_note(&workspace, id)?),
        "trash" => json!(delete_note_to_trash(&workspace, id)?),
        "restore" => json!(restore_note_from_trash(&workspace, id)?),
        _ => unreachable!("validated command"),
    })
}

fn failure(error: anyhow::Error, code: &str, exit: i32) -> ! {
    println!(
        "{}",
        json!({"ok": false, "error": {"code": code, "message": format!("{error:#}")}})
    );
    std::process::exit(exit);
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments == ["--help"] || arguments == ["-h"] {
        print!("{HELP}");
        return;
    }
    if arguments == ["--version"] {
        println!("neopad-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let args = parse_args(arguments).unwrap_or_else(|error| failure(error, "invalid_arguments", 2));
    match execute(args) {
        Ok(data) => println!(
            "{}",
            json!({"ok": true, "version": env!("CARGO_PKG_VERSION"), "data": data})
        ),
        Err(error) if error.downcast_ref::<NoteWriteConflict>().is_some() => {
            failure(error, "conflict", 3)
        }
        Err(error) => failure(error, "operation_failed", 1),
    }
}
