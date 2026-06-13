use dioxus::prelude::*;
use dioxus_desktop::use_window;
use std::collections::HashMap;

use crate::components::dialog::ProjectDialog;
use crate::components::dialog::RenameDialog;
use crate::components::editor::Editor;
use crate::components::preview::Preview;
use crate::components::sidebar::Sidebar;
use crate::components::status_bar::StatusBar;
use crate::components::tab_bar::TabBar;
use crate::components::toolbar::Toolbar;
use crate::fs;
use crate::model::{ActivityTab, DocRef, Project, WritingMode};
use crate::styles;

fn load_doc_content(p: &Project, doc: &DocRef) -> Result<String, String> {
    match doc {
        DocRef::Tale { chapter_dir, tale_file, .. } => {
            fs::chapter::load_tale(p, chapter_dir, tale_file).map_err(|e| e.to_string())
        }
        DocRef::Material { file_name, .. } => {
            fs::material::load_material(p, file_name).map_err(|e| e.to_string())
        }
    }
}

fn save_doc_content(p: &Project, doc: &DocRef, content: &str) -> Result<(), String> {
    match doc {
        DocRef::Tale { chapter_dir, tale_file, .. } => {
            fs::chapter::save_tale(p, chapter_dir, tale_file, content).map_err(|e| e.to_string())
        }
        DocRef::Material { file_name, .. } => {
            fs::material::save_material(p, file_name, content).map_err(|e| e.to_string())
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
    let mut project = use_signal(|| Option::<Project>::None);
    let mut open_tabs: Signal<Vec<DocRef>> = use_signal(Vec::new);
    let mut active_tab: Signal<Option<DocRef>> = use_signal(|| None);
    let mut tab_content: Signal<HashMap<DocRef, String>> = use_signal(HashMap::new);

    let content = use_signal(|| String::new());

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

    let mut chapter_version = use_signal(|| 0u32);
    let mut show_sidebar = use_signal(|| true);
    let mut show_editor = use_signal(|| true);
    let mut show_preview = use_signal(|| true);
    let mut focus_mode = use_signal(|| false);
    let mut font_size = use_signal(|| 16u32);
    let mut activity_tab = use_signal(|| ActivityTab::Explorer);

    let writing_mode_str = use_memo(move || match *writing_mode.read() {
        WritingMode::Vertical => "vertical".to_string(),
        WritingMode::Horizontal => "horizontal".to_string(),
    });

    let main_class = use_memo(move || {
        let mut c = "main-content".to_string();
        if *focus_mode.read() {
            c.push_str(" focus-mode");
        }
        match project.read().as_ref().map(|p| p.settings.sidebar_position) {
            Some(crate::model::SidebarPosition::Right) => c.push_str(" sidebar-right"),
            _ => {}
        }
        match project.read().as_ref().map(|p| p.settings.preview_position) {
            Some(crate::model::PanelPosition::Bottom) => c.push_str(" preview-bottom"),
            _ => {}
        }
        c
    });

    let desktop_top = desktop.clone();
    use_effect(move || {
        desktop_top.set_always_on_top(false);
    });

    // Resize JS
    let desktop_ui = desktop.clone();
    use_effect(move || {
        let js = r#"
(function() {
    if (window.__chronicle_init) return;
    window.__chronicle_init = true;

    var DRAG = null;
    var MIN_SB = 150, MAX_SB = 500;
    var MIN_PV = 200, MAX_PV = 800;
    var MIN_ED = 200;
    var HW = 5;

    function vis(el) { return el && el.offsetParent !== null; }

    function apply() {
        var c = document.querySelector('.main-content');
        if (!c) return;
        var pb = c.classList.contains('preview-bottom');
        var sb = document.querySelector('.sidebar');
        var ed = document.querySelector('.editor-pane');
        var pv = document.querySelector('.preview-pane');
        var cw = c.offsetWidth;
        var ch = c.offsetHeight;
        var vsb = vis(sb), ved = vis(ed), vpv = vis(pv);

        if (pb) {
            // sidebar | handle | editor  (preview below)
            // Read current heights
            var pvH = pv ? (parseFloat(pv.style.height) || 300) : 300;
            var sbW = sb ? (parseFloat(sb.style.width) || 240) : 0;
            // Clamp preview height
            pvH = Math.max(MIN_PV, Math.min(MAX_PV, Math.min(pvH, ch - HW - MIN_ED - (vsb ? MIN_SB : 0))));
            if (vpv) { pv.style.height = pvH + 'px'; pv.style.flexBasis = pvH + 'px'; }
            // Clamp sidebar width
            if (vsb) {
                sbW = Math.max(MIN_SB, Math.min(MAX_SB, Math.min(sbW, cw - HW - MIN_ED)));
                sb.style.width = sbW + 'px';
            }
        } else {
            var aw = cw - ((vsb && ved ? 1 : 0) + (ved && vpv ? 1 : 0)) * HW;
            if (aw <= 0) return;
            // Read current widths
            var sbW = sb ? (parseFloat(sb.style.width) || 240) : 0;
            var pvW = pv ? (parseFloat(pv.style.width) || 400) : 0;
            // Clamp sidebar: can't starve editor + preview
            if (vsb) {
                sbW = Math.max(MIN_SB, Math.min(MAX_SB, Math.min(sbW, aw - (vpv ? MIN_PV : 0) - MIN_ED)));
                sb.style.width = sbW + 'px';
            }
            // Clamp preview: can't starve editor + sidebar
            if (vpv) {
                pvW = Math.max(MIN_PV, Math.min(MAX_PV, Math.min(pvW, aw - (vsb ? MIN_SB : 0) - MIN_ED)));
                pv.style.width = pvW + 'px';
            }
        }
    }
    window.__chronicle_apply = apply;

    // Drag
    document.addEventListener('mousedown', function(e) {
        var h = e.target.closest('.resize-handle');
        if (!h) return;
        e.preventDefault();
        var c = document.querySelector('.main-content');
        if (!c) return;
        var pb = c.classList.contains('preview-bottom');
        var isV = pb && h.id === 'resize-preview';
        var el = h.id === 'resize-sidebar' ? document.querySelector('.sidebar') : document.querySelector('.preview-pane');
        if (!el) return;
        DRAG = {
            id: h.id,
            startPos: isV ? e.clientY : e.clientX,
            startSize: isV ? el.offsetHeight : el.offsetWidth,
            isV: isV,
        };
    });

    document.addEventListener('mousemove', function(e) {
        if (!DRAG) return;
        var delta = (DRAG.isV ? e.clientY : e.clientX) - DRAG.startPos;
        var c = document.querySelector('.main-content');
        if (!c) return;
        var pb = c.classList.contains('preview-bottom');
        var cSize = DRAG.isV ? c.offsetHeight : c.offsetWidth;
        var ed = document.querySelector('.editor-pane');
        var sb = document.querySelector('.sidebar');
        var pv = document.querySelector('.preview-pane');

        if (DRAG.id === 'resize-sidebar') {
            if (!sb) return;
            var vsb = vis(sb), ved = vis(ed), vpv = pv && vis(pv);
            var maxW = cSize - HW - (vpv ? MIN_PV : 0) - MIN_ED;
            var newW = Math.max(MIN_SB, Math.min(MAX_SB, DRAG.startSize + delta, maxW));
            sb.style.width = newW + 'px';
        } else if (DRAG.id === 'resize-preview') {
            if (!pv) return;
            var vsb = sb && vis(sb), ved = vis(ed), vpv = vis(pv);
            if (DRAG.isV) {
                // Drag up = preview smaller, so negative delta
                var maxH = cSize - HW - MIN_ED;
                var newH = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxH));
                pv.style.height = newH + 'px';
                pv.style.flexBasis = newH + 'px';
            } else {
                // Drag right = preview smaller, so negative delta
                var maxW = cSize - HW - (vsb ? MIN_SB : 0) - MIN_ED;
                var newW = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxW));
                pv.style.width = newW + 'px';
            }
        }
    });

    document.addEventListener('mouseup', function() { DRAG = null; });
    document.addEventListener('mouseleave', function() { DRAG = null; });

    // ResizeObserver for reliable resize handling
    if (window.ResizeObserver) {
        var mc = document.querySelector('.main-content');
        if (mc) {
            var ro = new ResizeObserver(function() { apply(); });
            ro.observe(mc);
        }
    }

    // Preview scroll follow
    var p = document.querySelector('.preview');
    if (p && !p._ps) {
        p._ps = true;
        new MutationObserver(function() {
            requestAnimationFrame(function() {
                var e = document.querySelector('.editor');
                var p2 = document.querySelector('.preview');
                if (!e || !p2) return;
                var r = e.value.length ? e.selectionStart / e.value.length : 0;
                if (p2.classList.contains('vertical')) {
                    p2.scrollLeft = (1 - r) * (p2.scrollWidth - p2.clientWidth);
                } else {
                    p2.scrollTop = r * (p2.scrollHeight - p2.clientHeight);
                }
            });
        }).observe(p, { childList: true, subtree: true, characterData: true });
    }

    apply();
})();
"#;
        let _ = desktop_ui.webview.evaluate_script(js);
    });

    // ── Helpers ──

    let mut switch_to_doc = {
        let mut active_tab = active_tab.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut chapter_version = chapter_version.clone();
        let project = project.clone();
        let mut content_sig = content.clone();
        let mut is_saved = is_saved.clone();
        move |doc: DocRef| {
            let tabs = open_tabs.read().clone();
            if !tabs.contains(&doc) {
                if let Some(ref p) = project.read().clone() {
                    match load_doc_content(p, &doc) {
                        Ok(text) => {
                            tab_content.write().insert(doc.clone(), text);
                        }
                        Err(e) => {
                            tab_content.write().insert(doc.clone(), format!("読み込みエラー: {}", e));
                        }
                    }
                } else {
                    tab_content.write().insert(doc.clone(), String::new());
                }
                open_tabs.write().push(doc.clone());
            }

            if let Some(text) = tab_content.read().get(&doc).cloned() {
                content_sig.set(text);
            }
            active_tab.set(Some(doc));
            is_saved.set(true);
            chapter_version += 1;
        }
    };

    let on_open_doc = {
        let mut switch_to_doc = switch_to_doc.clone();
        move |doc: DocRef| {
            switch_to_doc(doc);
        }
    };

    let on_close_tab = {
        let mut active_tab = active_tab.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut content_sig = content.clone();
        let mut chapter_version = chapter_version.clone();
        let mut is_saved = is_saved.clone();
        move |doc: DocRef| {
            tab_content.write().remove(&doc);
            let mut tabs = open_tabs.write();
            let idx = tabs.iter().position(|t| t == &doc);
            tabs.retain(|t| t != &doc);
            if tabs.is_empty() {
                active_tab.set(None);
                content_sig.set(String::new());
                is_saved.set(true);
            } else if Some(&doc) == active_tab.read().as_ref() {
                let new_idx = idx.unwrap_or(0).min(tabs.len().saturating_sub(1));
                if let Some(new_doc) = tabs.get(new_idx).cloned() {
                    if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                        content_sig.set(text);
                    }
                    active_tab.set(Some(new_doc));
                }
            }
            chapter_version += 1;
        }
    };

    let on_content_change = {
        let mut tab_content = tab_content.clone();
        let mut is_saved = is_saved.clone();
        let active_tab = active_tab.clone();
        let mut content_sig = content.clone();
        move |text: String| {
            content_sig.set(text.clone());
            if let Some(ref doc) = *active_tab.read() {
                tab_content.write().insert(doc.clone(), text);
            }
            is_saved.set(false);
        }
    };

    let mut do_save_current = {
        let project = project.clone();
        let active_tab = active_tab.clone();
        let tab_content = tab_content.clone();
        let mut is_saved = is_saved.clone();
        let mut save_notification = save_notification.clone();
        let mut content_sig = content.clone();
        move || {
            let doc = active_tab.read().clone();
            let p = project.read().clone();
            if let (Some(d), Some(ref proj)) = (doc, p) {
                if let Some(text) = tab_content.read().get(&d).cloned() {
                    match save_doc_content(proj, &d, &text) {
                        Ok(_) => {
                            *save_notification.write() = Some("保存しました".to_string());
                            is_saved.set(true);
                            content_sig.set(text);
                        }
                        Err(e) => {
                            *save_notification.write() = Some(format!("保存エラー: {}", e));
                        }
                    }
                }
            }
        }
    };

    let do_export = {
        let project = project.clone();
        let active_tab = active_tab.clone();
        let tab_content = tab_content.clone();
        move || {
            let doc = active_tab.read().clone();
            if let Some(d) = doc {
                if let Some(ref proj) = *project.read() {
                    if let Some(text) = tab_content.read().get(&d) {
                        let default_name = match &d {
                            DocRef::Tale { tale_file, .. } => tale_file.replace(".md", ".txt"),
                            DocRef::Material { file_name, .. } => file_name.replace(".md", ".txt"),
                        };
                        let dir = rfd::FileDialog::new()
                            .set_title("保存先を選択")
                            .set_file_name(&default_name)
                            .save_file();
                        if let Some(path) = dir {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
                            let output = if ext == "html" {
                                let html = crate::markdown::renderer::render_to_html(text);
                                format!(
                                    "<!DOCTYPE html><html lang=ja><meta charset=utf-8><title>{}</title><body>{}</body></html>",
                                    default_name.replace(".txt", ""),
                                    html
                                )
                            } else {
                                text.clone()
                            };
                            let _ = std::fs::write(&path, &output);
                        }
                    }
                }
            } else if let Some(ref proj) = *project.read() {
                // No active doc — export all chapters as single file
                let txt = proj.chapters.iter().flat_map(|ch| {
                    ch.tales.iter().filter_map(|t| {
                        fs::chapter::load_tale(proj, &ch.dir_name, &t.file_name).ok()
                            .map(|c| format!("# {} / {}\n\n{}\n\n", ch.title, t.title, c))
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>().join("---\n\n");
                let dir = rfd::FileDialog::new()
                    .set_title("保存先を選択")
                    .set_file_name("全話.txt")
                    .save_file();
                if let Some(path) = dir {
                    let _ = std::fs::write(&path, &txt);
                }
            }
        }
    };

    // ── Project callbacks ──

    let on_new_project = move |_| dialog_visible.set(true);

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

    let on_save = move |_| {
        do_save_current();
    };

    let on_export = move |_| {
        do_export();
    };

    let desktop_apply_visibility = desktop.clone();
    let desktop_toggle_dark = desktop.clone();
    let on_toggle_dark = move |_| {
        let new_val = !*is_dark.read();
        is_dark.set(new_val);
        let js = if new_val {
            "document.documentElement.classList.add('dark')"
        } else {
            "document.documentElement.classList.remove('dark')"
        };
        let _ = desktop_toggle_dark.webview.evaluate_script(js);
    };

    let on_toggle_sidebar = move |_| { let v = !*show_sidebar.read(); show_sidebar.set(v); };
    let on_toggle_editor = move |_| { let v = !*show_editor.read(); show_editor.set(v); };
    let on_toggle_preview = move |_| { let v = !*show_preview.read(); show_preview.set(v); };
    let on_toggle_focus_mode = move |_| { let v = !*focus_mode.read(); focus_mode.set(v); };
    let on_increase_font = move |_| { let mut f = font_size.write(); if *f < 32 { *f += 1; } };
    let on_decrease_font = move |_| { let mut f = font_size.write(); if *f > 8 { *f -= 1; } };

    // ── Chapter / Tale / Material CRUD ──

    let on_add_chapter = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        move |title: String| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                p.add_chapter(&title);
                let _ = fs::project::save_project(p);
                let _ = fs::chapter::create_chapter_dir(p, &title);
                *save_notification.write() = Some("章を追加しました".to_string());
            }
        }
    };

    let on_delete_chapter = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut active_tab = active_tab.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut content_sig = content.clone();
        let mut chapter_version = chapter_version.clone();
        let mut is_saved = is_saved.clone();
        move |dir_name: String| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                if let Some(ch) = p.chapters.iter().find(|c| c.dir_name == dir_name) {
                    // Close all tabs for this chapter
                    let close_docs: Vec<DocRef> = ch.tales.iter().map(|t| DocRef::Tale {
                        chapter_dir: dir_name.clone(),
                        tale_file: t.file_name.clone(),
                        chapter_title: ch.title.clone(),
                        tale_title: t.title.clone(),
                    }).collect();
                    for doc in &close_docs {
                        tab_content.write().remove(doc);
                    }
                    let mut tabs = open_tabs.write();
                    tabs.retain(|t| !close_docs.contains(t));
                    if tabs.is_empty() {
                        active_tab.set(None);
                        content_sig.set(String::new());
                        is_saved.set(true);
                    } else {
                        let needs_switch = {
                            let active = active_tab.read();
                            active.as_ref().map_or(false, |a| close_docs.contains(a))
                        };
                        if needs_switch {
                            let new_doc = tabs[0].clone();
                            if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                                content_sig.set(text);
                            }
                            active_tab.set(Some(new_doc));
                        }
                    }
                }

                let _ = fs::chapter::delete_chapter_dir(p, &dir_name);
                p.remove_chapter(&dir_name);
                let _ = fs::project::save_project(p);
                chapter_version += 1;
            }
        }
    };

    let on_rename_chapter = {
        let mut rename_target = rename_target.clone();
        let mut rename_dialog_visible = rename_dialog_visible.clone();
        move |(dir_name, current_title): (String, String)| {
            rename_target.set((dir_name, current_title));
            rename_dialog_visible.set(true);
        }
    };

    let mut on_confirm_rename = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        move |(old_dir, new_title): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                let old_ch_title = p.chapters.iter()
                    .find(|c| c.dir_name == old_dir)
                    .map(|c| c.title.clone())
                    .unwrap_or_default();
                if let Some(old_dir_name) = p.rename_chapter(&old_dir, &new_title) {
                    let _ = fs::chapter::rename_chapter_dir(p, &old_dir_name, &old_dir);
                    let _ = fs::project::save_project(p);

                    // Update tabs referencing this chapter
                    let mut tabs = open_tabs.write();
                    for t in tabs.iter_mut() {
                        if let DocRef::Tale { chapter_dir, chapter_title, .. } = t {
                            if *chapter_dir == old_dir {
                                *chapter_dir = old_dir.clone();
                                *chapter_title = new_title.clone();
                            }
                        }
                    }
                    let mut tc = tab_content.write();
                    let old_tabs: Vec<DocRef> = tc.keys().filter(|k| {
                        matches!(k, DocRef::Tale { chapter_dir, .. } if *chapter_dir == old_dir)
                    }).cloned().collect();
                    for old_t in old_tabs {
                        if let DocRef::Tale { chapter_dir: cd, tale_file, tale_title, .. } = &old_t {
                            let new_t = DocRef::Tale {
                                chapter_dir: old_dir.clone(),
                                tale_file: tale_file.clone(),
                                chapter_title: new_title.clone(),
                                tale_title: tale_title.clone(),
                            };
                            if let Some(v) = tc.remove(&old_t) {
                                tc.insert(new_t, v);
                            }
                        }
                    }
                    if let Some(ref mut active) = *active_tab.write() {
                        if let DocRef::Tale { chapter_dir, chapter_title, .. } = active {
                            if *chapter_dir == old_dir {
                                *chapter_dir = old_dir.clone();
                                *chapter_title = new_title.clone();
                            }
                        }
                    }
                    chapter_version += 1;
                }
            }
        }
    };

    let on_add_tale = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        move |(chapter_dir, title): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                if let Some(entry) = p.add_tale(&chapter_dir, &title) {
                    let ch_title = p.chapters.iter()
                        .find(|c| c.dir_name == chapter_dir)
                        .map(|c| c.title.clone())
                        .unwrap_or_default();
                    let doc = DocRef::Tale {
                        chapter_dir: chapter_dir.clone(),
                        tale_file: entry.file_name.clone(),
                        chapter_title: ch_title,
                        tale_title: entry.title.clone(),
                    };
                    match fs::chapter::save_tale(p, &chapter_dir, &entry.file_name, "") {
                        Ok(_) => {}
                        Err(e) => {
                            *save_notification.write() = Some(format!("作成エラー: {}", e));
                        }
                    }
                    let _ = fs::project::save_project(p);
                    chapter_version += 1;
                }
            }
        }
    };

    let on_delete_tale = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        let mut content_sig = content.clone();
        let mut is_saved = is_saved.clone();
        move |(chapter_dir, tale_file): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                let doc = DocRef::Tale {
                    chapter_dir: chapter_dir.clone(),
                    tale_file: tale_file.clone(),
                    chapter_title: String::new(),
                    tale_title: String::new(),
                };
                tab_content.write().remove(&doc);
                let mut tabs = open_tabs.write();
                tabs.retain(|t| t != &doc);
                if tabs.is_empty() {
                    active_tab.set(None);
                    content_sig.set(String::new());
                    is_saved.set(true);
                } else if Some(&doc) == active_tab.read().as_ref() {
                    let new_doc = tabs[0].clone();
                    if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                        content_sig.set(text);
                    }
                    active_tab.set(Some(new_doc));
                }

                let _ = fs::chapter::delete_tale_file(p, &chapter_dir, &tale_file);
                p.remove_tale(&chapter_dir, &tale_file);
                let _ = fs::project::save_project(p);
                chapter_version += 1;
            }
        }
    };

    let on_rename_tale = {
        let mut rename_target = rename_target.clone();
        let mut rename_dialog_visible = rename_dialog_visible.clone();
        move |(chapter_dir, tale_file, current_title): (String, String, String)| {
            rename_target.set((format!("{}|{}", chapter_dir, tale_file), current_title));
            rename_dialog_visible.set(true);
        }
    };

    // Modified on_confirm_rename to handle tale format
    let on_confirm_rename_tale = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        move |(key, new_title): (String, String)| {
            // Split key: if contains '|' it's chapter_dir|tale_file, else plain
            if let Some(sep) = key.find('|') {
                let chapter_dir = key[..sep].to_string();
                let old_file = key[sep + 1..].to_string();
                let mut proj = project.write();
                if let Some(ref mut p) = *proj {
                    if let Some((old_f, new_f)) = p.rename_tale(&chapter_dir, &old_file, &new_title) {
                        let ch_title = p.chapters.iter()
                            .find(|c| c.dir_name == chapter_dir)
                            .map(|c| c.title.clone())
                            .unwrap_or_default();
                        let _ = fs::chapter::rename_tale_file(p, &chapter_dir, &old_f, &new_f);
                        let _ = fs::project::save_project(p);

                        // Update tabs
                        let old_doc = DocRef::Tale {
                            chapter_dir: chapter_dir.clone(),
                            tale_file: old_f.clone(),
                            chapter_title: ch_title.clone(),
                            tale_title: new_title.clone(),
                        };
                        let new_doc = DocRef::Tale {
                            chapter_dir: chapter_dir.clone(),
                            tale_file: new_f.clone(),
                            chapter_title: ch_title,
                            tale_title: new_title,
                        };
                        let mut tabs = open_tabs.write();
                        for t in tabs.iter_mut() {
                            if *t == old_doc {
                                *t = new_doc.clone();
                            }
                        }
                        let mut tc = tab_content.write();
                        if let Some(v) = tc.remove(&old_doc) {
                            tc.insert(new_doc.clone(), v);
                        }
                        if let Some(ref mut active) = *active_tab.write() {
                            if *active == old_doc {
                                *active = new_doc;
                            }
                        }
                        chapter_version += 1;
                    }
                }
            } else {
                (on_confirm_rename)((key, new_title));
            }
        }
    };

    let on_add_material = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        move |(title, category): (String, crate::model::MaterialCategory)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                let entry = p.add_material(&title, category);
                let _ = fs::material::save_material(p, &entry.file_name, "");
                let _ = fs::project::save_project(p);
                chapter_version += 1;
            }
        }
    };

    let on_delete_material = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        let mut content_sig = content.clone();
        let mut is_saved = is_saved.clone();
        move |file_name: String| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                let doc = DocRef::Material {
                    file_name: file_name.clone(),
                    title: String::new(),
                };
                tab_content.write().remove(&doc);
                let mut tabs = open_tabs.write();
                tabs.retain(|t| t != &doc);
                if tabs.is_empty() {
                    active_tab.set(None);
                    content_sig.set(String::new());
                    is_saved.set(true);
                } else if Some(&doc) == active_tab.read().as_ref() {
                    let new_doc = tabs[0].clone();
                    if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                        content_sig.set(text);
                    }
                    active_tab.set(Some(new_doc));
                }

                let _ = fs::material::delete_material_file(p, &file_name);
                p.remove_material(&file_name);
                let _ = fs::project::save_project(p);
                chapter_version += 1;
            }
        }
    };

    let on_rename_material = {
        let mut rename_target = rename_target.clone();
        let mut rename_dialog_visible = rename_dialog_visible.clone();
        move |(file_name, current_title): (String, String)| {
            rename_target.set((file_name, current_title));
            rename_dialog_visible.set(true);
        }
    };

    let on_confirm_rename_material = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        move |(old_file, new_title): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                if let Some(old_f) = p.rename_material(&old_file, &new_title) {
                    let _ = fs::material::rename_material_file(p, &old_f, &old_file);
                    let _ = fs::project::save_project(p);

                    let old_doc = DocRef::Material {
                        file_name: old_f.clone(),
                        title: new_title.clone(),
                    };
                    let new_doc = DocRef::Material {
                        file_name: old_file.clone(),
                        title: new_title,
                    };
                    let mut tabs = open_tabs.write();
                    for t in tabs.iter_mut() {
                        if *t == old_doc {
                            *t = new_doc.clone();
                        }
                    }
                    let mut tc = tab_content.write();
                    if let Some(v) = tc.remove(&old_doc) {
                        tc.insert(new_doc.clone(), v);
                    }
                    if let Some(ref mut active) = *active_tab.write() {
                        if *active == old_doc {
                            *active = new_doc;
                        }
                    }
                    chapter_version += 1;
                }
            }
        }
    };

    // ── Recalculate panel widths on toggle ──
    use_effect(use_reactive(&(show_sidebar, show_editor, show_preview), move |_| {
        let _ = desktop_apply_visibility.webview.evaluate_script("if(window.__chronicle_apply) window.__chronicle_apply();");
    }));

    // ── Sync content changes back to tab_content ──

    use_effect(use_reactive(&content, move |_| {
        let text = content.read().clone();
        if let Some(ref doc) = *active_tab.read() {
            tab_content.write().insert(doc.clone(), text);
            is_saved.set(false);
        }
    }));

    // ── Auto-save ──

    let auto_save = auto_save_enabled.clone();
    use_effect(use_reactive(&tab_content, move |_| {
        let auto_save = auto_save.clone();
        if *auto_save.read() {
            let proj = project.clone();
            let active = active_tab.clone();
            let tc = tab_content.clone();
            let mut notif = save_notification.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let doc = active.read().clone();
                let p = proj.read().clone();
                if let (Some(d), Some(ref proj)) = (doc, p) {
                    if let Some(text) = tc.read().get(&d).cloned() {
                        let _ = save_doc_content(proj, &d, &text);
                    }
                }
            });
        }
    }));

    // Clear notification after 3s
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

    // ── Render ──

    let sidebar_visible = *show_sidebar.read();
    let preview_visible = *show_preview.read();
    let editor_visible = *show_editor.read();

    // Derive current content from active tab for editor
    let editor_placeholder = match active_tab.read().as_ref() {
        Some(DocRef::Tale { tale_title, .. }) => format!("「{}」を書き始めましょう...", tale_title),
        Some(DocRef::Material { title, .. }) => format!("「{}」の内容を入力...", title),
        None => "章・話を選択してください".to_string(),
    };

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
                show_sidebar: sidebar_visible,
                show_editor: editor_visible,
                show_preview: preview_visible,
                focus_mode: *focus_mode.read(),
                font_size: *font_size.read(),
                on_toggle_sidebar: on_toggle_sidebar,
                on_toggle_editor: on_toggle_editor,
                on_toggle_preview: on_toggle_preview,
                on_toggle_focus_mode: on_toggle_focus_mode,
                on_increase_font: on_increase_font,
                on_decrease_font: on_decrease_font,
            }
            div { class: main_class,
                // Sidebar
                if sidebar_visible {
                    Sidebar {
                        project: project,
                        active_tab: active_tab,
                        activity_tab: activity_tab,
                        on_add_chapter: on_add_chapter,
                        on_delete_chapter: on_delete_chapter,
                        on_rename_chapter: on_rename_chapter,
                        on_add_tale: on_add_tale,
                        on_delete_tale: on_delete_tale,
                        on_rename_tale: on_rename_tale,
                        on_add_material: on_add_material,
                        on_delete_material: on_delete_material,
                        on_rename_material: on_rename_material,
                        on_open_doc: on_open_doc,
                    }
                }
                if sidebar_visible {
                    div { id: "resize-sidebar", class: "resize-handle" }
                }
                // Editor area
                div { class: if *show_editor.read() { "editor-pane" } else { "editor-pane hidden" },
                    // Tab bar
                    TabBar {
                        open_tabs: open_tabs,
                        active_tab: active_tab,
                        on_close_tab: on_close_tab,
                        on_open_doc: on_open_doc,
                    }
                    // Editor
                    Editor {
                        content: content,
                        on_save: on_save,
                        chapter_version: *chapter_version.read(),
                        font_size: *font_size.read(),
                        placeholder: editor_placeholder,
                    }
                }
                if preview_visible {
                    div { id: "resize-preview", class: "resize-handle" }
                }
                div { class: if *show_preview.read() { "preview-pane" } else { "preview-pane hidden" },
                    Preview {
                        content: content,
                        writing_mode: writing_mode_str,
                    }
                }
            }
            StatusBar {
                project: project,
                active_tab: active_tab,
                tab_content: tab_content,
                is_saved: is_saved,
                auto_save_enabled: auto_save_enabled,
                writing_mode: writing_mode_str,
                font_size: *font_size.read(),
                on_increase_font: on_increase_font,
                on_decrease_font: on_decrease_font,
            }
            ProjectDialog {
                visible: dialog_visible,
                title: "新規プロジェクト".to_string(),
                on_confirm: on_confirm_new,
            }
            RenameDialog {
                visible: rename_dialog_visible,
                file_name: rename_target,
                on_confirm: {
                    let mut on_confirm_rename_tale = on_confirm_rename_tale.clone();
                    let mut on_confirm_rename_material = on_confirm_rename_material.clone();
                    move |(key, title): (String, String)| {
            if key.contains('|') || key.contains(".md") {
                    let p = project.read().clone();
                    let is_material = p.as_ref().map(|p| {
                        p.materials.iter().any(|m| m.file_name == key)
                    }).unwrap_or(false);
                    if is_material {
                        (on_confirm_rename_material)((key, title));
                    } else {
                        (on_confirm_rename_tale)((key, title));
                    }
                } else {
                    (on_confirm_rename)((key, title));
                }
                    }
                },
            }
            if let Some(msg) = save_notification.read().as_ref() {
                div { class: "notification", "{msg}" }
            }
        }
    }
}
