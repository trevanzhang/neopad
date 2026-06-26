use crate::{list_notes, note::read_note, Workspace};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub note_id: String,
    pub title: String,
    pub file_name: String,
    pub line_number: usize,
    pub line_text: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

pub fn search_notes(workspace: &Workspace, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for tab in list_notes(workspace)? {
        let note = read_note(workspace, &tab.id)?;
        let lines = note
            .content
            .lines()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        for (index, line) in lines.iter().enumerate() {
            if !line.to_lowercase().contains(&query_lower) {
                continue;
            }

            let before_start = index.saturating_sub(2);
            let after_end = (index + 3).min(lines.len());

            results.push(SearchResult {
                note_id: tab.id.clone(),
                title: tab.title.clone(),
                file_name: tab.file_name.clone(),
                line_number: index + 1,
                line_text: line.clone(),
                before: lines[before_start..index].to_vec(),
                after: lines[index + 1..after_end].to_vec(),
            });

            if results.len() >= limit {
                return Ok(results);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_note, init_workspace, write_note_atomic};

    #[test]
    fn search_notes_returns_case_insensitive_line_matches_with_context() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Searchable".to_owned())).expect("create");
        write_note_atomic(
            &workspace,
            &note.id,
            "first\nbefore\nThe MCP line\nafter\nlast",
        )
        .expect("write");

        let results = search_notes(&workspace, "mcp", 10).expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].note_id, note.id);
        assert_eq!(results[0].line_number, 3);
        assert_eq!(results[0].line_text, "The MCP line");
        assert_eq!(results[0].before, vec!["first", "before"]);
        assert_eq!(results[0].after, vec!["after", "last"]);
    }

    #[test]
    fn search_notes_respects_limit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("Many".to_owned())).expect("create");
        write_note_atomic(&workspace, &note.id, "x\nx\nx").expect("write");

        let results = search_notes(&workspace, "x", 2).expect("search");

        assert_eq!(results.len(), 2);
    }
}
