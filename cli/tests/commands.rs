use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn command(workspace: &Path, args: &[&str], input: Option<&str>) -> std::process::Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_neopad-cli"))
        .arg("--workspace")
        .arg(workspace)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child
}

fn run(workspace: &Path, args: &[&str], input: Option<&str>) -> Value {
    let output = command(workspace, args, input).wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let result = parse(&output);
    assert_eq!(result["ok"], true);
    result["data"].clone()
}

fn parse(output: &Output) -> Value {
    assert!(output.stderr.is_empty(), "unexpected diagnostics");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn project_commands_share_note_ids_search_conflicts_and_recovery() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    run(&workspace, &["project-create", "项目/Research"], None);
    let note = run(
        &workspace,
        &[
            "create",
            "--project",
            "项目/Research",
            "--title",
            "设计",
            "--stdin",
        ],
        Some("unique project keyword"),
    );
    let id = note["id"].as_str().unwrap();
    assert_eq!(
        run(
            &workspace,
            &["list", "--scope", "projects", "--project", "项目"],
            None
        )["pages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        run(&workspace, &["search", "keyword"], None)["matches"][0]["noteId"],
        id
    );
    run(
        &workspace,
        &["project-rename", "项目", "--name", "Renamed"],
        None,
    );
    let current = run(&workspace, &["read", id], None);
    let old_version = note["updatedAt"].to_string();
    let failed = command(
        &workspace,
        &[
            "update",
            id,
            "--expected-updated-at",
            &old_version,
            "--stdin",
        ],
        Some("stale"),
    )
    .wait_with_output()
    .unwrap();
    assert_eq!(failed.status.code(), Some(3));
    assert_eq!(
        run(&workspace, &["read", id], None)["content"],
        current["content"]
    );
    run(
        &workspace,
        &["project-move", "Renamed/Research", "--parent", "Other"],
        None,
    );
    run(&workspace, &["project-archive", "Other"], None);
    assert_eq!(
        run(&workspace, &["list", "--scope", "archive"], None)["pages"][0]["id"],
        id
    );
    run(&workspace, &["unarchive", id], None);
    run(&workspace, &["project-trash", "Other"], None);
    assert_eq!(
        run(&workspace, &["list", "--scope", "trash"], None)["pages"][0]["id"],
        id
    );
    run(&workspace, &["restore", id], None);
    run(&workspace, &["capture", id], None);
    run(&workspace, &["move", id, "--project", "Final"], None);
    assert_eq!(
        run(&workspace, &["read", id], None)["content"],
        current["content"]
    );
    assert!(run(&workspace, &["projects"], None)["directories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "Final"));
}

#[test]
fn doctor_and_invalid_arguments_do_not_create_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("absent");
    let diagnostics = run(&workspace, &["doctor", "--json"], None);
    assert_eq!(diagnostics["workspaceExists"], false);
    assert_eq!(diagnostics["configValid"], false);
    assert_eq!(diagnostics["guideAvailable"], false);
    assert_eq!(
        diagnostics["guidePath"],
        workspace.join("AGENTS.md").to_string_lossy().as_ref()
    );
    for option in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_neopad-cli"))
            .env("NEOPAD_WORKSPACE", &workspace)
            .arg(option)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    let output = command(&workspace, &["update", "inbox", "--stdin"], Some("text"))
        .wait_with_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(parse(&output)["error"]["code"], "invalid_arguments");
    assert!(!workspace.exists());
}

#[test]
fn cli_initializes_root_guide_and_preserves_user_customizations() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let listed = run(&workspace, &["list"], None);
    assert_eq!(listed["pages"].as_array().unwrap().len(), 2);
    let guide = workspace.join("AGENTS.md");
    let default = std::fs::read_to_string(&guide).unwrap();
    assert!(default.contains("NeoPad 工作指南"));
    assert_eq!(run(&workspace, &["doctor"], None)["guideAvailable"], true);
    std::fs::write(&guide, "# Custom instructions\n").unwrap();
    run(&workspace, &["projects"], None);
    assert_eq!(
        std::fs::read_to_string(guide).unwrap(),
        "# Custom instructions\n"
    );
}

#[test]
fn unicode_content_archive_trash_restore_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace with spaces");
    let file = temp.path().join("中文 草稿.md");
    std::fs::write(&file, "\u{feff}你好，NeoPad。\nsecond line").unwrap();
    let created = run(
        &workspace,
        &[
            "create",
            "--title",
            "中文笔记",
            "--content-file",
            file.to_str().unwrap(),
        ],
        None,
    );
    let id = created["id"].as_str().unwrap();
    assert_eq!(
        created["content"],
        "# 中文笔记\n\n你好，NeoPad。\nsecond line"
    );
    run(&workspace, &["append", id, "--stdin"], Some("appended"));
    assert!(run(&workspace, &["read", id], None)["content"]
        .as_str()
        .unwrap()
        .ends_with("appended"));
    assert!(!run(&workspace, &["search", "NeoPad"], None)["matches"]
        .as_array()
        .unwrap()
        .is_empty());
    run(&workspace, &["rename", id, "--title", "Renamed"], None);
    assert!(run(&workspace, &["read", id], None)["content"]
        .as_str()
        .unwrap()
        .starts_with("# Renamed"));
    run(&workspace, &["archive", id], None);
    assert_eq!(
        run(&workspace, &["list", "--scope", "archive"], None)["pages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    run(&workspace, &["unarchive", id], None);
    run(&workspace, &["trash", id], None);
    assert_eq!(
        run(&workspace, &["list", "--scope", "trash"], None)["pages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    run(&workspace, &["restore", id], None);
    assert!(run(&workspace, &["read", id], None)["content"]
        .as_str()
        .unwrap()
        .contains("你好"));
    let output = command(&workspace, &["trash", "inbox"], None)
        .wait_with_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(workspace.join("notes/inbox.md").is_file());
}

#[test]
fn concurrent_updates_have_one_winner_and_keep_conflict_content() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let note = run(&workspace, &["create", "--title", "Concurrency"], None);
    let id = note["id"].as_str().unwrap();
    let version = note["updatedAt"].as_i64().unwrap().to_string();
    let args = ["update", id, "--expected-updated-at", &version, "--stdin"];
    let first = command(&workspace, &args, Some("first"));
    let second = command(&workspace, &args, Some("second"));
    let results = [
        first.wait_with_output().unwrap(),
        second.wait_with_output().unwrap(),
    ];
    assert_eq!(results.iter().filter(|r| r.status.success()).count(), 1);
    let conflict = results.iter().find(|r| !r.status.success()).unwrap();
    assert_eq!(conflict.status.code(), Some(3));
    assert_eq!(parse(conflict)["error"]["code"], "conflict");
    let winner = parse(results.iter().find(|r| r.status.success()).unwrap());
    assert_eq!(
        run(&workspace, &["read", id], None)["content"],
        winner["data"]["content"]
    );

    let current = run(&workspace, &["read", id], None);
    std::fs::write(
        workspace
            .join("notes")
            .join(current["fileName"].as_str().unwrap()),
        "changed outside",
    )
    .unwrap();
    let version = current["updatedAt"].as_i64().unwrap().to_string();
    let output = command(
        &workspace,
        &["update", id, "--expected-updated-at", &version, "--stdin"],
        Some("stale"),
    )
    .wait_with_output()
    .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        run(&workspace, &["read", id], None)["content"],
        "changed outside"
    );
}

#[test]
fn concurrent_appends_preserve_every_entry() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    run(&workspace, &["list"], None);
    let children = (0..8)
        .map(|i| {
            command(
                &workspace,
                &["append-inbox", "--stdin"],
                Some(&format!("entry-{i}\n")),
            )
        })
        .collect::<Vec<_>>();
    for child in children {
        assert!(child.wait_with_output().unwrap().status.success());
    }
    let note = run(&workspace, &["read", "inbox"], None);
    let body = note["content"].as_str().unwrap();
    for i in 0..8 {
        assert_eq!(body.matches(&format!("entry-{i}\n")).count(), 1);
    }
}

#[test]
fn deployed_cli_runs_without_desktop_or_http() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = neopad_core::init_workspace(Some(temp.path().join("workspace"))).unwrap();
    let status =
        neopad_core::install_agent_cli(&workspace, Path::new(env!("CARGO_BIN_EXE_neopad-cli")))
            .unwrap();
    assert!(status.matches_bundle);
    let output = Command::new(status.cli_path)
        .arg("--workspace")
        .arg(&workspace.root)
        .arg("list")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(parse(&output)["data"]["pages"].as_array().unwrap().len(), 2);
}

#[test]
fn unsafe_ids_and_oversized_input_do_not_replace_notes() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    run(&workspace, &["list"], None);
    let output = command(&workspace, &["read", "../config.json"], None)
        .wait_with_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let file = temp.path().join("large.md");
    std::fs::write(&file, vec![b'a'; 2 * 1024 * 1024 + 1]).unwrap();
    let output = command(
        &workspace,
        &["append-inbox", "--content-file", file.to_str().unwrap()],
        None,
    )
    .wait_with_output()
    .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        run(&workspace, &["read", "inbox"], None)["content"],
        "# Inbox\n\n"
    );
}
