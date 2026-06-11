use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub root_dir: PathBuf,
    pub chapters: Vec<ChapterEntry>,
    pub settings: ProjectSettings,
}

impl Project {
    pub fn new(name: &str, root_dir: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            root_dir,
            chapters: Vec::new(),
            settings: ProjectSettings::default(),
        }
    }

    pub fn add_chapter(&mut self, title: &str) -> ChapterEntry {
        let order = self.chapters.len();
        let safe_name = title.chars().take(20).collect::<String>()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "");
        let file_name = format!("{:02}-{}.md", order + 1, safe_name.replace(' ', "-"));
        let entry = ChapterEntry {
            title: title.to_string(),
            file_name,
            order,
            children: Vec::new(),
        };
        self.chapters.push(entry.clone());
        entry
    }

    pub fn remove_chapter(&mut self, file_name: &str) {
        self.chapters.retain(|c| c.file_name != file_name);
        for (i, c) in self.chapters.iter_mut().enumerate() {
            c.order = i;
        }
    }

    pub fn chapter_path(&self, file_name: &str) -> PathBuf {
        self.root_dir.join("chapters").join(file_name)
    }

    pub fn project_file_path(&self) -> PathBuf {
        self.root_dir.join("chronicle.json")
    }
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
    pub daily_goal: usize,
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
            daily_goal: 1000,
        }
    }
}
