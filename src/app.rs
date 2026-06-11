use dioxus::prelude::*;

use crate::components::dialog::ProjectDialog;
use crate::components::editor::Editor;
use crate::components::preview::Preview;
use crate::components::sidebar::Sidebar;
use crate::components::toolbar::Toolbar;
use crate::fs;
use crate::model::project::{Project, WritingMode};
use crate::styles;

#[component]
pub fn App() -> Element {
    let mut content = use_signal(|| String::new());
    let mut selected_chapter = use_signal(|| String::new());
    let mut project = use_signal(|| Option::<Project>::None);
    let writing_mode = use_signal(|| WritingMode::Horizontal);
    let mut dialog_visible = use_signal(|| false);
    let mut save_notification = use_signal(|| Option::<String>::None);

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
        spawn(async move {
            let dir = rfd::FileDialog::new()
                .set_title("プロジェクトを作成する場所を選択")
                .pick_folder();
            if let Some(dir) = dir {
                match fs::project::create_project(&name, &dir) {
                    Ok(mut p) => {
                        p.settings.author = author;
                        let _ = fs::project::save_project(&p);
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
        spawn(async move {
            let dir = rfd::FileDialog::new()
                .set_title("プロジェクトフォルダを選択")
                .pick_folder();
            if let Some(dir) = dir {
                match fs::project::load_project(&dir) {
                    Ok(p) => {
                        *proj_sig.write() = Some(p);
                    }
                    Err(e) => {
                        *notif.write() = Some(format!("開くエラー: {}", e));
                    }
                }
            }
        });
    };

    let on_save = move |_| {
        let proj = project.read().clone();
        let content_val = content.read().clone();
        let chapter = selected_chapter.read().clone();

        if let Some(ref p) = proj {
            if !chapter.is_empty() {
                match fs::chapter::save_chapter(p, &chapter, &content_val) {
                    Ok(_) => {
                        *save_notification.write() = Some("保存しました".to_string());
                    }
                    Err(e) => {
                        *save_notification.write() = Some(format!("保存エラー: {}", e));
                    }
                }
            }
        }
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

    use_effect(use_reactive(&(project, selected_chapter), move |(proj, ch)| {
        if let Some(ref p) = *proj.read() {
            let fname = ch.read().clone();
            if !fname.is_empty() {
                match fs::chapter::load_chapter(p, &fname) {
                    Ok(text) => {
                        content.set(text);
                    }
                    Err(_) => {}
                }
            }
        }
    }));

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
            }
            div { class: "main-content",
                Sidebar {
                    project: project,
                    selected_chapter: selected_chapter,
                    on_add_chapter: on_add_chapter,
                    on_delete_chapter: on_delete_chapter,
                }
                div { class: "editor-pane",
                    Editor { content: content }
                }
                div { class: "preview-pane",
                    Preview {
                        content: content,
                        writing_mode: writing_mode_str,
                    }
                }
            }
            ProjectDialog {
                visible: dialog_visible,
                title: "新規プロジェクト".to_string(),
                on_confirm: on_confirm_new,
            }
            if let Some(msg) = save_notification.read().as_ref() {
                div { class: "notification", "{msg}" }
            }
        }
    }
}
