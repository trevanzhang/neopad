use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoteTab {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub deleted: bool,
    #[serde(default)]
    pub system_title: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TabsState {
    pub version: u32,
    pub active_tab_id: String,
    pub tabs: Vec<NoteTab>,
}

impl TabsState {
    pub fn default_with_timestamp(now_ms: i64) -> Self {
        Self {
            version: 1,
            active_tab_id: "inbox".to_owned(),
            tabs: vec![
                NoteTab {
                    id: "inbox".to_owned(),
                    title: "Inbox".to_owned(),
                    file_name: "inbox.md".to_owned(),
                    created_at: now_ms,
                    updated_at: now_ms,
                    pinned: true,
                    deleted: false,
                    system_title: false,
                },
                NoteTab {
                    id: "clipboard".to_owned(),
                    title: "Clipboard".to_owned(),
                    file_name: "clipboard.md".to_owned(),
                    created_at: now_ms,
                    updated_at: now_ms,
                    pinned: true,
                    deleted: false,
                    system_title: false,
                },
            ],
        }
    }
}
