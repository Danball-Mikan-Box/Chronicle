use std::fs;

use crate::model::project::Project;

pub fn save_chapter(project: &Project, file_name: &str, content: &str) -> Result<(), String> {
    let path = project.chapter_path(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_chapter(project: &Project, file_name: &str) -> Result<String, String> {
    let path = project.chapter_path(file_name);
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}
