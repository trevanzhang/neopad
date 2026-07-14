use crate::Workspace;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

const MAX_PROMPT_BYTES: u64 = 128 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptEntry {
    pub id: String,
    pub name: String,
    pub file_name: String,
    pub content: String,
}

pub fn list_prompts(workspace: &Workspace) -> Result<Vec<PromptEntry>> {
    let mut paths = fs::read_dir(&workspace.prompts_dir)
        .with_context(|| {
            format!(
                "failed to read prompts directory at {}",
                workspace.prompts_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });

    let mut prompts = Vec::new();
    for path in paths {
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to inspect prompt at {}", path.display()))?;
        if !metadata.file_type().is_file() {
            continue;
        }
        if metadata.len() > MAX_PROMPT_BYTES {
            bail!(
                "prompt file exceeds the 128 KiB limit: {}",
                path.file_name()
                    .map(|name| name.to_string_lossy())
                    .unwrap_or_default()
            );
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("prompt file name is not valid UTF-8")?
            .to_owned();
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .context("prompt name is not valid UTF-8")?
            .trim()
            .to_owned();
        if name.is_empty() {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read prompt at {}", path.display()))?;
        if content.trim().is_empty() {
            continue;
        }
        prompts.push(PromptEntry {
            id: file_name.clone(),
            name,
            file_name,
            content,
        });
    }
    Ok(prompts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_workspace;

    #[test]
    fn lists_markdown_prompts_in_name_order() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        fs::write(
            workspace.prompts_dir.join("Weekly.md"),
            "Create a weekly report.",
        )
        .expect("weekly prompt");
        fs::write(workspace.prompts_dir.join("Review.md"), "Review this note.")
            .expect("review prompt");
        fs::write(workspace.prompts_dir.join("ignore.txt"), "ignored").expect("ignored file");

        let prompts = list_prompts(&workspace).expect("list prompts");

        assert_eq!(prompts.len(), 2);
        assert_eq!(prompts[0].name, "Review");
        assert_eq!(prompts[1].id, "Weekly.md");
        assert_eq!(prompts[1].content, "Create a weekly report.");
    }

    #[test]
    fn skips_empty_prompt_files() {
        let temp = tempfile::tempdir().expect("temp dir");
        let workspace = init_workspace(Some(temp.path().join("workspace"))).expect("workspace");
        fs::write(workspace.prompts_dir.join("Empty.md"), "  \n").expect("empty prompt");

        assert!(list_prompts(&workspace).expect("list prompts").is_empty());
    }
}
