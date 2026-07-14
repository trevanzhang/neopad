use crate::{list_searchable_notes, note::read_note, Workspace};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelevantNoteExcerpt {
    pub note_id: String,
    pub title: String,
    pub file_name: String,
    pub line_number: usize,
    pub excerpt: String,
    pub score: usize,
}

pub fn search_notes(workspace: &Workspace, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for tab in list_searchable_notes(workspace)? {
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

pub fn find_relevant_note_excerpts(
    workspace: &Workspace,
    query: &str,
    exclude_note_id: Option<&str>,
    limit: usize,
) -> Result<Vec<RelevantNoteExcerpt>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let terms = query_terms(query);
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut excerpts = Vec::new();
    for tab in list_searchable_notes(workspace)? {
        if exclude_note_id.is_some_and(|note_id| note_id == tab.id) {
            continue;
        }
        let note = read_note(workspace, &tab.id)?;
        let lines = note.content.lines().collect::<Vec<_>>();
        let title_lower = tab.title.to_lowercase();
        let title_score = terms
            .iter()
            .filter(|term| title_lower.contains(term.as_str()))
            .map(|term| 8 + term.chars().count())
            .sum::<usize>();

        let best = lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                let line_lower = line.to_lowercase();
                let score = title_score
                    + terms
                        .iter()
                        .filter(|term| line_lower.contains(term.as_str()))
                        .map(|term| 3 + term.chars().count())
                        .sum::<usize>();
                (score > 0).then_some((index, score))
            })
            .max_by_key(|(_, score)| *score);

        if let Some((index, score)) = best {
            let start = index.saturating_sub(1);
            let end = (index + 2).min(lines.len());
            excerpts.push(RelevantNoteExcerpt {
                note_id: tab.id,
                title: tab.title,
                file_name: tab.file_name,
                line_number: index + 1,
                excerpt: lines[start..end].join("\n"),
                score,
            });
        }
    }

    excerpts.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
    });
    excerpts.truncate(limit);
    Ok(excerpts)
}

fn query_terms(query: &str) -> Vec<String> {
    const STOP_WORDS: &[&str] = &[
        "about", "current", "help", "note", "please", "this", "what", "which", "with", "一下",
        "什么", "内容", "哪些", "如何", "当前", "帮我", "怎么", "笔记", "这篇",
    ];
    let mut segments = Vec::new();
    let mut current = String::new();
    for character in query.to_lowercase().chars() {
        if character.is_alphanumeric() || is_cjk(character) {
            current.push(character);
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }

    let stop_words = STOP_WORDS.iter().copied().collect::<HashSet<_>>();
    let mut terms = HashSet::new();
    for segment in segments {
        let characters = segment.chars().collect::<Vec<_>>();
        if characters.iter().all(|character| is_cjk(*character)) {
            if characters.len() <= 6 && !stop_words.contains(segment.as_str()) {
                terms.insert(segment.clone());
            }
            for pair in characters.windows(2) {
                let term = pair.iter().collect::<String>();
                if !stop_words.contains(term.as_str()) {
                    terms.insert(term);
                }
            }
        } else if characters.len() >= 2 && !stop_words.contains(segment.as_str()) {
            terms.insert(segment);
        }
    }
    let mut terms = terms.into_iter().collect::<Vec<_>>();
    terms.sort_by_key(|term| std::cmp::Reverse(term.chars().count()));
    terms.truncate(24);
    terms
}

fn is_cjk(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
    )
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

    #[test]
    fn relevant_excerpts_rank_matching_notes_and_exclude_current_note() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let first = create_note(&workspace, Some("Project risks".to_owned())).expect("first");
        write_note_atomic(
            &workspace,
            &first.id,
            "Budget is stable.\nThe schedule risk is high.",
        )
        .expect("write first");
        let second = create_note(&workspace, Some("Other".to_owned())).expect("second");
        write_note_atomic(&workspace, &second.id, "No relevant material here.")
            .expect("write second");

        let results = find_relevant_note_excerpts(
            &workspace,
            "What are the project schedule risks?",
            Some(&second.id),
            5,
        )
        .expect("relevant excerpts");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].note_id, first.id);
        assert!(results[0].excerpt.contains("schedule risk"));
    }

    #[test]
    fn relevant_excerpts_support_chinese_query_terms() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp_dir.path().join("workspace"))).expect("workspace");
        let note = create_note(&workspace, Some("项目复盘".to_owned())).expect("note");
        write_note_atomic(&workspace, &note.id, "主要问题是进度风险和沟通成本。")
            .expect("write note");

        let results = find_relevant_note_excerpts(&workspace, "项目有哪些风险？", None, 5)
            .expect("relevant excerpts");

        assert_eq!(results[0].note_id, note.id);
        assert!(results[0].excerpt.contains("风险"));
    }
}
