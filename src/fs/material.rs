use std::fs;

use crate::model::project::Project;

pub fn save_material(project: &Project, file_name: &str, content: &str) -> Result<(), String> {
    let path = project.material_path(file_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_material(project: &Project, file_name: &str) -> Result<String, String> {
    let path = project.material_path(file_name);
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

pub fn rename_material_file(project: &Project, old_file: &str, new_file: &str) -> Result<(), String> {
    let old_path = project.material_path(old_file);
    let new_path = project.material_path(new_file);
    if old_path.exists() {
        fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn delete_material_file(project: &Project, file_name: &str) -> Result<(), String> {
    let path = project.material_path(file_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
