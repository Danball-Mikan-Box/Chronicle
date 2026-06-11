use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub root_dir: PathBuf,
    pub chapters: Vec<ChapterEntry>,
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEntry {
    pub title: String,
    pub file_name: String,
    pub order: usize,
    pub children: Vec<ChapterEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub writing_mode: WritingMode,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WritingMode {
    Vertical,
    Horizontal,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            writing_mode: WritingMode::Horizontal,
            author: String::new(),
            description: String::new(),
        }
    }
}
