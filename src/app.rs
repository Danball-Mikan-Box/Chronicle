use dioxus::prelude::*;
use dioxus_desktop::use_window;

use crate::components::dialog::ProjectDialog;
use crate::components::dialog::RenameDialog;
use crate::components::editor::Editor;
use crate::components::preview::Preview;
use crate::components::sidebar::Sidebar;
use crate::components::status_bar::StatusBar;
use crate::components::toolbar::Toolbar;
use crate::fs;
use crate::model::project::{Project, WritingMode};
use crate::styles;

fn do_save(
    project: &Signal<Option<Project>>,
    content: &Signal<String>,
    selected_chapter: &Signal<String>,
    is_saved: &mut Signal<bool>,
    save_notification: &mut Signal<Option<String>>,
) {
    if let Some(ref p) = project.read().clone() {
        let chapter = selected_chapter.read().clone();
        if !chapter.is_empty() {
            let content_val = content.read().clone();
            match fs::chapter::save_chapter(p, &chapter, &content_val) {
                Ok(_) => {
                    *save_notification.write() = Some("保存しました".to_string());
                    is_saved.set(true);
                }
                Err(e) => {
                    *save_notification.write() = Some(format!("保存エラー: {}", e));
                }
            }
        }
    }
}

fn do_export(project: &Signal<Option<Project>>, content: &Signal<String>, selected_chapter: &Signal<String>) {
    let proj = project.read().clone();
    let chapter = selected_chapter.read().clone();
    if let Some(ref p) = proj {
        let content_val = content.read().clone();
        if !chapter.is_empty() {
            let dir = rfd::FileDialog::new()
                .set_title("保存先を選択")
                .set_file_name(&chapter.replace(".md", ".txt"))
                .save_file();
            if let Some(path) = dir {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
                let output = if ext == "html" {
                    let html = crate::markdown::renderer::render_to_html(&content_val);
                    format!("<!DOCTYPE html><html lang=ja><meta charset=utf-8><title>{}</title><body>{}</body></html>",
                        chapter.replace(".md", ""), html)
                } else {
                    content_val.clone()
                };
                let _ = std::fs::write(&path, &output);
            }
        } else {
            // export all chapters
            let txt = p.chapters.iter().filter_map(|ch| {
                let c = fs::chapter::load_chapter(p, &ch.file_name).ok()?;
                Some(format!("# {}\n\n{}\n\n", ch.title, c))
            }).collect::<Vec<_>>().join("---\n\n");
            let dir = rfd::FileDialog::new()
                .set_title("保存先を選択")
                .set_file_name("全章.txt")
                .save_file();
            if let Some(path) = dir {
                let _ = std::fs::write(&path, &txt);
            }
        }
    }
}

fn load_recent() -> Vec<String> {
    let path = std::env::temp_dir().join("chronicle_recent.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_recent(list: &[String]) {
    let path = std::env::temp_dir().join("chronicle_recent.json");
    if let Ok(json) = serde_json::to_string(list) {
        let _ = std::fs::write(&path, &json);
    }
}

fn push_recent(list: &mut Vec<String>, dir: String) {
    list.retain(|x| x != &dir);
    list.push(dir);
    if list.len() > 20 {
        list.remove(0);
    }
    save_recent(list);
}

#[component]
pub fn App() -> Element {
    let mut content = use_signal(|| String::new());
    let mut selected_chapter = use_signal(|| String::new());
    let mut project = use_signal(|| Option::<Project>::None);
    let writing_mode = use_signal(|| WritingMode::Horizontal);
    let mut dialog_visible = use_signal(|| false);
    let mut rename_dialog_visible = use_signal(|| false);
    let mut rename_target = use_signal(|| (String::new(), String::new()));
    let mut save_notification = use_signal(|| Option::<String>::None);
    let mut is_saved = use_signal(|| true);
    let mut is_dark = use_signal(|| false);
    let recent_projects = use_signal(|| load_recent());
    let auto_save_enabled = use_signal(|| true);
    let desktop = use_window();

    let writing_mode_str = use_memo(move || match *writing_mode.read() {
        WritingMode::Vertical => "vertical".to_string(),
        WritingMode::Horizontal => "horizontal".to_string(),
    });

    let on_new_project = move |_| {
        dialog_visible.set(true);
    };

    let on_confirm_new = move |(name, author): (String, String)| {
        let mut proj_sig = project.clone();
        let mut notif = save_notification.clone();
        let mut recent = recent_projects.clone();
        spawn(async move {
            let dir = rfd::FileDialog::new()
                .set_title("プロジェクトを作成する場所を選択")
                .pick_folder();
            if let Some(dir) = dir {
                match fs::project::create_project(&name, &dir) {
                    Ok(mut p) => {
                        p.settings.author = author;
                        let _ = fs::project::save_project(&p);
                        push_recent(&mut recent.write(), dir.to_string_lossy().to_string());
                        *proj_sig.write() = Some(p);
                    }
                    Err(e) => {
                        *notif.write() = Some(format!("作成エラー: {}", e));
                    }
                }
            }
        });
    };

    let on_open_project = move |_| {
        let mut proj_sig = project.clone();
        let mut notif = save_notification.clone();
        let mut recent = recent_projects.clone();
        spawn(async move {
            let dir = rfd::FileDialog::new()
                .set_title("プロジェクトフォルダを選択")
                .pick_folder();
            if let Some(dir) = dir {
                match fs::project::load_project(&dir) {
                    Ok(p) => {
                        push_recent(&mut recent.write(), dir.to_string_lossy().to_string());
                        *proj_sig.write() = Some(p);
                    }
                    Err(e) => {
                        *notif.write() = Some(format!("開くエラー: {}", e));
                    }
                }
            }
        });
    };

    let on_open_recent = move |path: String| {
        let mut proj_sig = project.clone();
        let mut notif = save_notification.clone();
        let mut recent = recent_projects.clone();
        spawn(async move {
            let dir = std::path::PathBuf::from(&path);
            if dir.exists() {
                match fs::project::load_project(&dir) {
                    Ok(p) => {
                        push_recent(&mut recent.write(), path);
                        *proj_sig.write() = Some(p);
                    }
                    Err(e) => {
                        *notif.write() = Some(format!("開くエラー: {}", e));
                    }
                }
            } else {
                *notif.write() = Some("プロジェクトが見つかりません".to_string());
                recent.write().retain(|x| x != &path);
            }
        });
    };

    let on_save = move |_| {
        do_save(&project, &content, &selected_chapter, &mut is_saved, &mut save_notification);
    };

    let on_export = move |_| {
        do_export(&project, &content, &selected_chapter);
    };

    let on_toggle_dark = move |_| {
        let new_val = !*is_dark.read();
        is_dark.set(new_val);
        let js = if new_val {
            "document.documentElement.classList.add('dark')"
        } else {
            "document.documentElement.classList.remove('dark')"
        };
        let _ = desktop.webview.evaluate_script(js);
    };

    let on_add_chapter = move |title: String| {
        let mut proj = project.write();
        if let Some(ref mut p) = *proj {
            let entry = p.add_chapter(&title);
            let _ = fs::project::save_project(p);
            let fname = entry.file_name.clone();
            drop(proj);
            selected_chapter.set(fname);
            content.set(String::new());
            is_saved.set(true);
        }
    };

    let on_delete_chapter = move |file_name: String| {
        let mut proj = project.write();
        if let Some(ref mut p) = *proj {
            p.remove_chapter(&file_name);
            let _ = fs::project::save_project(p);
            let _ = std::fs::remove_file(p.chapter_path(&file_name));
            drop(proj);
            if *selected_chapter.read() == file_name {
                selected_chapter.set(String::new());
                content.set(String::new());
            }
        }
    };

    let on_rename_chapter = move |(file_name, current_title): (String, String)| {
        rename_target.set((file_name, current_title));
        rename_dialog_visible.set(true);
    };

    let on_confirm_rename = move |(file_name, new_title): (String, String)| {
        let mut proj = project.write();
        if let Some(ref mut p) = *proj {
            if let Some(old_name) = p.rename_chapter(&file_name, &new_title) {
                let old_path = p.chapter_path(&old_name);
                let new_path = p.chapter_path(&file_name);
                if old_path != new_path {
                    let _ = std::fs::rename(&old_path, &new_path);
                }
                let _ = fs::project::save_project(p);
                if *selected_chapter.read() == old_name || *selected_chapter.read() == file_name {
                    selected_chapter.set(file_name);
                }
            }
        }
    };

    // Load chapter content when switching chapters
    use_effect(use_reactive(&(project, selected_chapter), move |(proj, ch)| {
        if let Some(ref p) = *proj.read() {
            let fname = ch.read().clone();
            if !fname.is_empty() {
                match fs::chapter::load_chapter(p, &fname) {
                    Ok(text) => {
                        content.set(text);
                        is_saved.set(true);
                    }
                    Err(_) => {}
                }
            }
        }
    }));

    // Auto-save with debounce
    let auto_save = auto_save_enabled.clone();
    use_effect(use_reactive(&content, move |_| {
        let mut is_saved = is_saved.clone();
        let mut notif = save_notification.clone();
        let proj = project.clone();
        let ch = selected_chapter.clone();
        let enabled = auto_save.clone();
        if *enabled.read() {
            is_saved.set(false);
            let proj_c = proj.clone();
            let ch_c = ch.clone();
            let c = content.clone();
            let s = is_saved.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                do_save(&proj_c, &c, &ch_c, &mut s.clone(), &mut notif);
            });
        }
    }));

    // Auto-dismiss notification
    let notif_clone = save_notification.clone();
    use_effect(move || {
        if save_notification.read().is_some() {
            let mut n = notif_clone.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                *n.write() = None;
            });
        }
    });

    rsx! {
        style { "{styles::MAIN_CSS}" }
        div { class: "app",
            Toolbar {
                writing_mode: writing_mode,
                content: content,
                project: project,
                on_new_project: on_new_project,
                on_open_project: on_open_project,
                on_save: on_save,
                on_export: on_export,
                on_toggle_dark: on_toggle_dark,
                is_dark: *is_dark.read(),
            }
            div { class: "main-content",
                Sidebar {
                    project: project,
                    selected_chapter: selected_chapter,
                    on_add_chapter: on_add_chapter,
                    on_delete_chapter: on_delete_chapter,
                    on_rename_chapter: on_rename_chapter,
                    recent_projects: recent_projects,
                    on_open_recent: on_open_recent,
                }
                div { class: "editor-pane",
                    Editor {
                        content: content,
                        on_save: on_save,
                    }
                }
                div { class: "preview-pane",
                    Preview {
                        content: content,
                        writing_mode: writing_mode_str,
                    }
                }
            }
            StatusBar {
                project: project,
                selected_chapter: selected_chapter,
                content: content,
                is_saved: is_saved,
                auto_save_enabled: auto_save_enabled,
                writing_mode: writing_mode_str,
            }
            ProjectDialog {
                visible: dialog_visible,
                title: "新規プロジェクト".to_string(),
                on_confirm: on_confirm_new,
            }
            RenameDialog {
                visible: rename_dialog_visible,
                file_name: rename_target,
                on_confirm: on_confirm_rename,
            }
            if let Some(msg) = save_notification.read().as_ref() {
                div { class: "notification", "{msg}" }
            }
        }
    }
}
