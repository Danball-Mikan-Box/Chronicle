use std::fs;

use crate::model::project::Project;

pub fn create_chapter_dir(project: &Project, dir_name: &str) -> Result<(), String> {
    let path = project.root_dir.join("chapters").join(dir_name);
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

pub fn save_tale(project: &Project, chapter_dir: &str, file_name: &str, content: &str) -> Result<(), String> {
    let path = project.tale_path(chapter_dir, file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_tale(project: &Project, chapter_dir: &str, file_name: &str) -> Result<String, String> {
    let path = project.tale_path(chapter_dir, file_name);
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn rename_chapter_dir(project: &Project, old_dir: &str, new_dir: &str) -> Result<(), String> {
    let old_path = project.root_dir.join("chapters").join(old_dir);
    let new_path = project.root_dir.join("chapters").join(new_dir);
    if old_path.exists() {
        fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn rename_tale_file(project: &Project, chapter_dir: &str, old_file: &str, new_file: &str) -> Result<(), String> {
    let old_path = project.tale_path(chapter_dir, old_file);
    let new_path = project.tale_path(chapter_dir, new_file);
    if old_path.exists() {
        fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn delete_chapter_dir(project: &Project, dir_name: &str) -> Result<(), String> {
    let path = project.root_dir.join("chapters").join(dir_name);
    if path.exists() {
        fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn delete_tale_file(project: &Project, chapter_dir: &str, file_name: &str) -> Result<(), String> {
    let path = project.tale_path(chapter_dir, file_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
