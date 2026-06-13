use std::fs;
use std::path::PathBuf;
use crate::model::project::GlobalSettings;

fn get_settings_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    path.pop();
    path.push("settings.json");
    path
}

pub fn save_global_settings(settings: &GlobalSettings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(get_settings_path(), json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_global_settings() -> GlobalSettings {
    let path = get_settings_path();
    if !path.exists() {
        return GlobalSettings::default();
    }
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|json| serde_json::from_str(&json).map_err(|e| e.to_string()))
        .unwrap_or_else(|_| GlobalSettings::default())
}
