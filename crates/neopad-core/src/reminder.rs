use crate::{
    atomic_write::write_atomic, list_notes, note::read_note, write_note_atomic, Workspace,
};
use anyhow::{bail, Context, Result};
use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs};

const REMINDER_MARKER: &str = "@remind";
const LEGACY_REMINDER_MARKER: &str = "@提醒";
const DATE_TIME_LENGTH: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub file_name: String,
    pub line_number: usize,
    pub due_at: i64,
    pub due_text: String,
    pub content: String,
    pub completed: bool,
    pub status: ReminderStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReminderStatus {
    Pending,
    Due,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReminder {
    pub due_at: i64,
    pub due_text: String,
    pub content: String,
    pub completed: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReminderDeliveryState {
    #[serde(default = "delivery_state_version")]
    version: u32,
    #[serde(default)]
    delivered: Vec<String>,
}

fn delivery_state_version() -> u32 {
    1
}

pub fn parse_reminder_line(line: &str) -> Option<ParsedReminder> {
    let line = line.trim_start();
    let (completed, remainder) = if let Some(value) = line.strip_prefix("- [ ]") {
        (false, value)
    } else if let Some(value) = line
        .strip_prefix("- [x]")
        .or_else(|| line.strip_prefix("- [X]"))
    {
        (true, value)
    } else {
        return None;
    };

    let remainder = remainder.trim_start();
    let remainder = remainder
        .strip_prefix(REMINDER_MARKER)
        .or_else(|| remainder.strip_prefix(LEGACY_REMINDER_MARKER))?
        .trim_start();
    if remainder.len() <= DATE_TIME_LENGTH || !remainder.is_char_boundary(DATE_TIME_LENGTH) {
        return None;
    }

    let due_text = &remainder[..DATE_TIME_LENGTH];
    let content = remainder[DATE_TIME_LENGTH..].trim();
    if content.is_empty() {
        return None;
    }

    let naive = NaiveDateTime::parse_from_str(due_text, "%Y-%m-%d %H:%M").ok()?;
    let due_at = Local
        .from_local_datetime(&naive)
        .earliest()?
        .timestamp_millis();
    Some(ParsedReminder {
        due_at,
        due_text: due_text.to_owned(),
        content: content.to_owned(),
        completed,
    })
}

pub fn list_reminders(workspace: &Workspace) -> Result<Vec<Reminder>> {
    let now = Utc::now().timestamp_millis();
    let mut reminders = Vec::new();

    for tab in list_notes(workspace)? {
        let note = read_note(workspace, &tab.id)?;
        for (index, line) in note.content.lines().enumerate() {
            let Some(parsed) = parse_reminder_line(line) else {
                continue;
            };
            let status = if parsed.completed {
                ReminderStatus::Completed
            } else if parsed.due_at <= now {
                ReminderStatus::Due
            } else {
                ReminderStatus::Pending
            };
            let id = reminder_signature(&tab.id, parsed.due_at, &parsed.content);
            reminders.push(Reminder {
                id,
                note_id: tab.id.clone(),
                title: tab.title.clone(),
                file_name: tab.file_name.clone(),
                line_number: index + 1,
                due_at: parsed.due_at,
                due_text: parsed.due_text,
                content: parsed.content,
                completed: parsed.completed,
                status,
            });
        }
    }

    reminders.sort_by(|left, right| {
        left.completed
            .cmp(&right.completed)
            .then(left.due_at.cmp(&right.due_at))
            .then(left.title.cmp(&right.title))
            .then(left.line_number.cmp(&right.line_number))
    });
    Ok(reminders)
}

pub fn claim_due_reminders(workspace: &Workspace) -> Result<Vec<Reminder>> {
    let reminders = list_reminders(workspace)?;
    let mut state = load_delivery_state(workspace)?;
    let previous_delivered = state.delivered.clone();
    let active = reminders
        .iter()
        .filter(|reminder| !reminder.completed)
        .map(|reminder| reminder.id.as_str())
        .collect::<HashSet<_>>();
    state
        .delivered
        .retain(|signature| active.contains(signature.as_str()));

    let delivered = state.delivered.iter().cloned().collect::<HashSet<_>>();
    let due = reminders
        .into_iter()
        .filter(|reminder| {
            reminder.status == ReminderStatus::Due && !delivered.contains(&reminder.id)
        })
        .collect::<Vec<_>>();
    state
        .delivered
        .extend(due.iter().map(|reminder| reminder.id.clone()));
    if state.delivered != previous_delivered {
        save_delivery_state(workspace, &state)?;
    }
    Ok(due)
}

pub fn complete_reminder(
    workspace: &Workspace,
    note_id: &str,
    line_number: usize,
    reminder_id: &str,
) -> Result<()> {
    set_reminder_completion(workspace, note_id, line_number, reminder_id, true)
}

pub fn reopen_reminder(
    workspace: &Workspace,
    note_id: &str,
    line_number: usize,
    reminder_id: &str,
) -> Result<()> {
    set_reminder_completion(workspace, note_id, line_number, reminder_id, false)
}

fn set_reminder_completion(
    workspace: &Workspace,
    note_id: &str,
    line_number: usize,
    reminder_id: &str,
    completed: bool,
) -> Result<()> {
    if line_number == 0 {
        bail!("reminder line number must be positive");
    }

    let note = read_note(workspace, note_id)?;
    let mut found = false;
    let mut next_content = String::with_capacity(note.content.len());
    for (index, line) in note.content.split_inclusive('\n').enumerate() {
        if index + 1 != line_number {
            next_content.push_str(line);
            continue;
        }

        let Some(parsed) = parse_reminder_line(line.trim_end_matches(['\r', '\n'])) else {
            bail!("reminder no longer exists at line {line_number}");
        };
        if reminder_signature(note_id, parsed.due_at, &parsed.content) != reminder_id {
            bail!("reminder changed before it could be updated");
        }
        if parsed.completed == completed {
            bail!("reminder is already in the requested completion state");
        }
        if completed {
            next_content.push_str(&line.replacen("- [ ]", "- [x]", 1));
        } else if line.trim_start().starts_with("- [X]") {
            next_content.push_str(&line.replacen("- [X]", "- [ ]", 1));
        } else {
            next_content.push_str(&line.replacen("- [x]", "- [ ]", 1));
        }
        found = true;
    }

    if !found {
        bail!("reminder line {line_number} does not exist");
    }
    write_note_atomic(workspace, note_id, &next_content)?;
    Ok(())
}

pub fn complete_due_reminders(workspace: &Workspace) -> Result<usize> {
    let now = Utc::now().timestamp_millis();
    let mut completed = 0;

    for tab in list_notes(workspace)? {
        let note = read_note(workspace, &tab.id)?;
        let mut changed = false;
        let mut next_content = String::with_capacity(note.content.len());
        for line in note.content.split_inclusive('\n') {
            let parsed = parse_reminder_line(line.trim_end_matches(['\r', '\n']));
            if parsed.is_some_and(|reminder| !reminder.completed && reminder.due_at <= now) {
                next_content.push_str(&line.replacen("- [ ]", "- [x]", 1));
                changed = true;
                completed += 1;
            } else {
                next_content.push_str(line);
            }
        }
        if changed {
            write_note_atomic(workspace, &tab.id, &next_content)?;
        }
    }
    Ok(completed)
}

fn reminder_signature(note_id: &str, due_at: i64, content: &str) -> String {
    format!("{note_id}\u{1f}{due_at}\u{1f}{content}")
}

fn load_delivery_state(workspace: &Workspace) -> Result<ReminderDeliveryState> {
    let contents = fs::read_to_string(&workspace.reminders_path).with_context(|| {
        format!(
            "failed to read reminder state {}",
            workspace.reminders_path.display()
        )
    })?;
    serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse reminder state {}",
            workspace.reminders_path.display()
        )
    })
}

fn save_delivery_state(workspace: &Workspace, state: &ReminderDeliveryState) -> Result<()> {
    let contents =
        serde_json::to_string_pretty(state).context("failed to serialize reminder state")?;
    write_atomic(&workspace.reminders_path, &contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_note, init_workspace, write_note_atomic};

    #[test]
    fn parses_pending_and_completed_reminders() {
        let pending =
            parse_reminder_line("- [ ] @remind 2030-07-03 16:05 buy milk").expect("pending");
        assert!(!pending.completed);
        assert_eq!(pending.due_text, "2030-07-03 16:05");
        assert_eq!(pending.content, "buy milk");

        let completed = parse_reminder_line("  - [X]   @remind   2030-07-03 16:05   meeting ")
            .expect("completed");
        assert!(completed.completed);
        assert_eq!(completed.content, "meeting");
    }

    #[test]
    fn parses_legacy_chinese_reminder_marker() {
        let reminder =
            parse_reminder_line("- [ ] @提醒 2030-07-03 16:05 买牛奶").expect("legacy reminder");
        assert_eq!(reminder.due_text, "2030-07-03 16:05");
        assert_eq!(reminder.content, "买牛奶");
    }

    #[test]
    fn rejects_plain_tasks_and_invalid_reminders() {
        assert!(parse_reminder_line("- [ ] ordinary task").is_none());
        assert!(parse_reminder_line("- [ ] @remind 2030-02-30 16:05 invalid date").is_none());
        assert!(parse_reminder_line("- [ ] @remind 2030-07-03 16:05").is_none());
    }

    #[test]
    fn lists_reminders_in_due_order_and_claims_each_once() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Plans".to_owned())).expect("create");
        write_note_atomic(
            &workspace,
            &note.id,
            "- [ ] @remind 2000-01-02 09:00 later\n- [ ] @remind 2000-01-01 09:00 first\n- [x] @remind 1999-01-01 09:00 done",
        )
        .expect("write");

        let reminders = list_reminders(&workspace).expect("list");
        assert_eq!(reminders.len(), 3);
        assert_eq!(reminders[0].content, "first");
        assert_eq!(reminders[2].status, ReminderStatus::Completed);

        let first_claim = claim_due_reminders(&workspace).expect("first claim");
        assert_eq!(first_claim.len(), 2);
        assert!(claim_due_reminders(&workspace)
            .expect("second claim")
            .is_empty());
    }

    #[test]
    fn completes_one_reminder_without_deleting_or_reformatting_other_content() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Plans".to_owned())).expect("create");
        let content = "heading\r\n- [ ] @remind 2000-01-01 09:00 first\r\n- [ ] ordinary\r\n";
        write_note_atomic(&workspace, &note.id, content).expect("write");
        let reminder = list_reminders(&workspace).expect("list").remove(0);

        complete_reminder(&workspace, &note.id, reminder.line_number, &reminder.id)
            .expect("complete");

        assert_eq!(
            read_note(&workspace, &note.id).expect("read").content,
            "heading\r\n- [x] @remind 2000-01-01 09:00 first\r\n- [ ] ordinary\r\n"
        );
    }

    #[test]
    fn reopens_one_completed_reminder_without_reformatting_other_content() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Plans".to_owned())).expect("create");
        let content = format!(
            "heading\r\n- [X] {REMINDER_MARKER} 2000-01-01 09:00 first\r\n- [ ] ordinary\r\n"
        );
        write_note_atomic(&workspace, &note.id, &content).expect("write");
        let reminder = list_reminders(&workspace).expect("list").remove(0);

        reopen_reminder(&workspace, &note.id, reminder.line_number, &reminder.id).expect("reopen");

        assert_eq!(
            read_note(&workspace, &note.id).expect("read").content,
            format!(
                "heading\r\n- [ ] {REMINDER_MARKER} 2000-01-01 09:00 first\r\n- [ ] ordinary\r\n"
            )
        );
    }

    #[test]
    fn completes_all_due_reminders_but_keeps_future_and_plain_tasks_open() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Plans".to_owned())).expect("create");
        write_note_atomic(
            &workspace,
            &note.id,
            "- [ ] @remind 2000-01-01 09:00 due\n- [ ] @remind 2999-01-01 09:00 future\n- [ ] ordinary",
        )
        .expect("write");

        assert_eq!(complete_due_reminders(&workspace).expect("complete due"), 1);
        let content = read_note(&workspace, &note.id).expect("read").content;
        assert!(content.contains("- [x] @remind 2000-01-01 09:00 due"));
        assert!(content.contains("- [ ] @remind 2999-01-01 09:00 future"));
        assert!(content.contains("- [ ] ordinary"));
    }
}
