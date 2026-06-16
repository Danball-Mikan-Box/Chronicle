use std::fs;
use std::io::Read;
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

/// Import a project from a ZIP archive (as produced by `export_project_zip`).
/// The archive must contain a `chronicle.json` at its root.
/// Files are extracted into `projects_dir/<project_name>`.
pub fn import_project_from_zip(zip_bytes: &[u8], projects_dir: &Path) -> Result<Project, String> {
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

    // Read chronicle.json first to get the project name
    let mut chronicle_json = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.name() == "chronicle.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
            chronicle_json = Some(contents);
            break;
        }
    }
    let json = chronicle_json.ok_or("chronicle.json が見つかりません")?;
    let mut project: Project = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    // Determine safe directory name from project name
    let safe_name = Project::sanitize_name(&project.name, 50);
    let safe_name = if safe_name.is_empty() { "imported".to_string() } else { safe_name };
    let root_dir = projects_dir.join(&safe_name);
    if root_dir.exists() {
        return Err(format!("プロジェクト '{}' は既に存在します", safe_name));
    }

    // Extract all entries
    fs::create_dir_all(&root_dir).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        if name.ends_with('/') {
            fs::create_dir_all(root_dir.join(&name)).map_err(|e| e.to_string())?;
            continue;
        }
        let target = root_dir.join(&name);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        fs::write(&target, &buf).map_err(|e| e.to_string())?;
    }

    project.root_dir = root_dir;
    Ok(project)
}
