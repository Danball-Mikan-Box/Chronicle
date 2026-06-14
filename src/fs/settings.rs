use std::fs;
use std::path::PathBuf;
use crate::model::project::GlobalSettings;

fn get_settings_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    let app_dir = config_dir.join("chronicle");
    let _ = std::fs::create_dir_all(&app_dir);
    app_dir.join("settings.json")
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
