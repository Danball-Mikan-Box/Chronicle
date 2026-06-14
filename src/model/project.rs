use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub last_date: String,
    pub start_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub root_dir: PathBuf,
    pub chapters: Vec<ChapterEntry>,
    #[serde(default)]
    pub materials: Vec<MaterialEntry>,
    pub settings: ProjectSettings,
    #[serde(default)]
    pub daily_stats: DailyStats,
}

impl Project {
    pub fn new(name: &str, root_dir: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            root_dir,
            chapters: Vec::new(),
            materials: Vec::new(),
            settings: ProjectSettings::default(),
            daily_stats: DailyStats::default(),
        }
    }

    pub fn sanitize_name(s: &str, max: usize) -> String {
        s.chars()
            .take(max)
            .collect::<String>()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "")
            .trim()
            .to_string()
    }

    // ── Chapters ──

    pub fn add_chapter(&mut self, title: &str) -> ChapterEntry {
        let order = self.chapters.len();
        let safe = Self::sanitize_name(title, 20).replace(' ', "-");
        let dir_name = format!("{:02}-{}", order + 1, if safe.is_empty() { "無題" } else { &safe });
        let entry = ChapterEntry {
            title: title.to_string(),
            dir_name,
            order,
            tales: Vec::new(),
        };
        self.chapters.push(entry.clone());
        entry
    }

    pub fn rename_chapter(&mut self, old_dir: &str, new_title: &str) -> Option<String> {
        let entry = self.chapters.iter_mut().find(|c| c.dir_name == old_dir)?;
        let safe = Self::sanitize_name(new_title, 20).replace(' ', "-");
        let new_dir = format!("{:02}-{}", entry.order + 1, if safe.is_empty() { "無題" } else { &safe });
        entry.title = new_title.to_string();
        entry.dir_name = new_dir.clone();
        Some(new_dir)
    }

    pub fn remove_chapter(&mut self, dir_name: &str) {
        self.chapters.retain(|c| c.dir_name != dir_name);
        for (i, c) in self.chapters.iter_mut().enumerate() {
            c.order = i;
        }
    }

    // ── Tales ──

    pub fn add_tale(&mut self, chapter_dir: &str, title: &str) -> Option<TaleEntry> {
        let ch = self.chapters.iter_mut().find(|c| c.dir_name == chapter_dir)?;
        let order = ch.tales.len();
        let safe = Self::sanitize_name(title, 30).replace(' ', "-");
        let file_name = format!(
            "{:02}-{}.md",
            order + 1,
            if safe.is_empty() { "無題" } else { &safe }
        );
        let entry = TaleEntry {
            title: title.to_string(),
            file_name,
            order,
            cached_char_count: None,
        };
        ch.tales.push(entry.clone());
        Some(entry)
    }

    pub fn rename_tale(
        &mut self,
        chapter_dir: &str,
        old_file: &str,
        new_title: &str,
    ) -> Option<(String, String)> {
        let ch = self.chapters.iter_mut().find(|c| c.dir_name == chapter_dir)?;
        let entry = ch.tales.iter_mut().find(|t| t.file_name == old_file)?;
        let safe = Self::sanitize_name(new_title, 30).replace(' ', "-");
        let new_file = format!(
            "{:02}-{}.md",
            entry.order + 1,
            if safe.is_empty() { "無題" } else { &safe }
        );
        entry.title = new_title.to_string();
        let old = entry.file_name.clone();
        entry.file_name = new_file.clone();
        Some((old, new_file))
    }

    pub fn remove_tale(&mut self, chapter_dir: &str, file_name: &str) {
        if let Some(ch) = self.chapters.iter_mut().find(|c| c.dir_name == chapter_dir) {
            ch.tales.retain(|t| t.file_name != file_name);
            for (i, t) in ch.tales.iter_mut().enumerate() {
                t.order = i;
            }
        }
    }

    // ── Materials ──

    pub fn add_material(&mut self, title: &str, category: MaterialCategory) -> MaterialEntry {
        let order = self.materials.len();
        let safe = Self::sanitize_name(title, 30).replace(' ', "-");
        let base = if safe.is_empty() { "無題" } else { &safe };
        let mut file_name = format!("{}.md", base);
        let mut counter = 1;
        while self.materials.iter().any(|m| m.file_name == file_name) {
            file_name = format!("{}-{}.md", base, counter);
            counter += 1;
        }
        let entry = MaterialEntry {
            title: title.to_string(),
            file_name,
            category,
            order,
        };
        self.materials.push(entry.clone());
        entry
    }

    pub fn rename_material(&mut self, old_file: &str, new_title: &str) -> Option<(String, String)> {
        let entry = self.materials.iter_mut().find(|m| m.file_name == old_file)?;
        let safe = Self::sanitize_name(new_title, 30).replace(' ', "-");
        let new_file = format!("{}.md", if safe.is_empty() { "無題" } else { &safe });
        entry.title = new_title.to_string();
        let old = entry.file_name.clone();
        entry.file_name = new_file.clone();
        Some((old, new_file))
    }

    pub fn remove_material(&mut self, file_name: &str) {
        self.materials.retain(|m| m.file_name != file_name);
        for (i, m) in self.materials.iter_mut().enumerate() {
            m.order = i;
        }
    }

    // ── Paths ──

    pub fn chapter_dir(&self, dir_name: &str) -> PathBuf {
        self.root_dir.join("chapters").join(dir_name)
    }

    pub fn tale_path(&self, chapter_dir: &str, file_name: &str) -> PathBuf {
        self.root_dir.join("chapters").join(chapter_dir).join(file_name)
    }

    pub fn material_path(&self, file_name: &str) -> PathBuf {
        self.root_dir.join("materials").join(file_name)
    }

    pub fn project_file_path(&self) -> PathBuf {
        self.root_dir.join("chronicle.json")
    }
}

// ── Entry types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterEntry {
    pub title: String,
    pub dir_name: String,
    pub order: usize,
    #[serde(default)]
    pub tales: Vec<TaleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaleEntry {
    pub title: String,
    pub file_name: String,
    pub order: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_char_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialEntry {
    pub title: String,
    pub file_name: String,
    pub category: MaterialCategory,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialCategory {
    Character,
    World,
    Glossary,
    Timeline,
    Other(String),
}

impl MaterialCategory {
    pub fn label(&self) -> &str {
        match self {
            MaterialCategory::Character => "キャラクター",
            MaterialCategory::World => "世界観",
            MaterialCategory::Glossary => "用語集",
            MaterialCategory::Timeline => "年表",
            MaterialCategory::Other(s) => s,
        }
    }
}

// ── Settings ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub writing_mode: WritingMode,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub daily_goal: usize,
    #[serde(default)]
    pub preview_position: PanelPosition,
    #[serde(default)]
    pub sidebar_position: SidebarPosition,
    #[serde(default)]
    pub indent_paragraphs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub theme_dark: bool,
    pub font_size: u32,
    pub auto_save: bool,
    pub font_family: String,
    pub line_height: f32,
    pub max_width: u32,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            theme_dark: false,
            font_size: 16,
            auto_save: true,
            font_family: "Noto Sans JP".to_string(),
            line_height: 2.0,
            max_width: 800,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WritingMode {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PanelPosition {
    Right,
    Bottom,
}

impl Default for PanelPosition {
    fn default() -> Self { Self::Right }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SidebarPosition {
    Left,
    Right,
}

impl Default for SidebarPosition {
    fn default() -> Self { Self::Left }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ActivityTab {
    Explorer,
    Materials,
}

// ── Document reference ──

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocRef {
    Tale {
        chapter_dir: String,
        tale_file: String,
        chapter_title: String,
        tale_title: String,
    },
    Material {
        file_name: String,
        title: String,
    },
}

impl DocRef {
    pub fn tab_label(&self) -> String {
        match self {
            DocRef::Tale { chapter_title, tale_title, .. } => {
                format!("{} / {}", chapter_title, tale_title)
            }
            DocRef::Material { title, .. } => title.clone(),
        }
    }

    pub fn short_label(&self) -> String {
        match self {
            DocRef::Tale { tale_title, .. } => tale_title.clone(),
            DocRef::Material { title, .. } => title.clone(),
        }
    }
}

/// Open-tab state, not serialised with the project.
#[derive(Debug, Clone)]
pub struct TabState {
    pub open_tabs: Vec<DocRef>,
    pub active_tab: DocRef,
    pub tab_content: HashMap<DocRef, String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            writing_mode: WritingMode::Horizontal,
            author: String::new(),
            description: String::new(),
            daily_goal: 1000,
            preview_position: PanelPosition::Right,
            sidebar_position: SidebarPosition::Left,
            indent_paragraphs: true,
        }
    }
}
