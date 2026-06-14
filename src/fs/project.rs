use std::fs;
use std::path::Path;

use crate::model::project::Project;

pub fn create_project(name: &str, dir: &Path) -> Result<Project, String> {
    let safe_name = crate::model::project::Project::sanitize_name(name, 50);
    let safe_name = if safe_name.is_empty() { "project".to_string() } else { safe_name };
    let root_dir = dir.join(&safe_name);
    if root_dir.exists() {
        return Err(format!("ディレクトリ '{}' は既に存在します", safe_name));
    }
    fs::create_dir_all(&root_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(root_dir.join("chapters")).map_err(|e| e.to_string())?;
    fs::create_dir_all(root_dir.join("materials")).map_err(|e| e.to_string())?;

    let project = Project::new(name, root_dir.clone());
    save_project(&project)?;
    Ok(project)
}

pub fn save_project(project: &Project) -> Result<(), String> {
    let json = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
    fs::write(project.project_file_path(), json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_project(dir: &Path) -> Result<Project, String> {
    let project_file = dir.join("chronicle.json");
    if !project_file.exists() {
        return Err("chronicle.json が見つかりません".to_string());
    }
    let json = fs::read_to_string(&project_file).map_err(|e| e.to_string())?;
    let mut project: Project = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    project.root_dir = dir.to_path_buf();
    Ok(project)
}
