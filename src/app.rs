#[cfg(target_os = "android")]
use dioxus::document::eval;
use dioxus::prelude::*;
#[cfg(not(target_os = "android"))]
use dioxus_desktop::use_window;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;



use crate::components::dialog::{ConfirmDialog, ExportDialog, PendingDelete, ProjectDialog, ProjectPickerDialog, RenameDialog, SettingsDialog};

#[derive(Clone)]
enum CloseAction {
    CloseProject,
    #[allow(dead_code)]
    CloseWindow,
}
use crate::components::editor::Editor;
use crate::export::ExportFormat;
use crate::components::preview::Preview;
use crate::components::sidebar::Sidebar;
#[cfg(not(target_os = "android"))]
use crate::components::status_bar::StatusBar;
#[cfg(not(target_os = "android"))]
use crate::components::tab_bar::TabBar;
use crate::components::toolbar::Toolbar;
use crate::fs;
use crate::model::{ActivityTab, DocRef, Project, ProjectSettings, WritingMode};
use crate::styles;

#[cfg(not(target_os = "android"))]
fn pick_save_path(default_name: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title("エクスポート先を選択")
        .set_file_name(default_name)
        .add_filter("ZIP Archive", &["zip"])
        .save_file()
}

#[cfg(target_os = "android")]
fn android_storage_dir() -> std::path::PathBuf {
    fs::android_storage_dir()
}

#[cfg(target_os = "android")]
fn pick_save_path(default_name: &str) -> Option<std::path::PathBuf> {
    let export_dir = fs::android_export_dir();
    let _ = std::fs::create_dir_all(&export_dir);
    Some(export_dir.join(default_name))
}

#[cfg(not(target_os = "android"))]
fn pick_folder(title: &str) -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .pick_folder()
}

#[cfg(target_os = "android")]
fn pick_folder(_title: &str) -> Option<std::path::PathBuf> {
    let dir = android_storage_dir().join("projects");
    eprintln!("[chronicle] pick_folder: {}", dir.display());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("[chronicle] pick_folder: create_dir_all error: {}", e);
    }
    Some(dir)
}

/// Scan for available projects in the Android storage directory.
#[cfg(target_os = "android")]
fn scan_android_projects() -> Vec<(String, String)> {
    let dir = android_storage_dir().join("projects");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let proj_file = e.path().join("chronicle.json");
            if !proj_file.exists() {
                return None;
            }
            let path_str = e.path().to_string_lossy().to_string();
            let name = std::fs::read_to_string(&proj_file)
                .ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .and_then(|v| v.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
                .unwrap_or_else(|| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("無題")
                        .to_string()
                });
            Some((name, path_str))
        })
        .collect()
}

/// On Android, scan `dir` for valid project subdirectories and return the most recent.
#[cfg(target_os = "android")]
fn resolve_project_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    if dir.join("chronicle.json").exists() {
        return Some(dir.to_path_buf());
    }
    let entries = std::fs::read_dir(dir).ok()?;
    let mut projects: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().join("chronicle.json").exists())
        .map(|e| e.path())
        .collect();
    projects.sort_by_key(|p| std::fs::metadata(p).ok().and_then(|m| m.modified().ok()));
    projects.into_iter().last()
}

#[cfg(not(target_os = "android"))]
fn resolve_project_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    Some(dir.to_path_buf())
}

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

fn recent_path() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let dir = android_storage_dir();
        let _ = std::fs::create_dir_all(&dir);
        dir.join("recent.json")
    }
    #[cfg(not(target_os = "android"))]
    {
        std::env::temp_dir().join("chronicle_recent.json")
    }
}

fn load_recent() -> Vec<String> {
    let path = recent_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_recent(list: &[String]) {
    let path = recent_path();
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

fn get_total_chars(p: &Project) -> usize {
    p.chapters.iter().flat_map(|ch| {
        ch.tales.iter().filter_map(|t| {
            if let Some(cached) = t.cached_char_count {
                return Some(cached);
            }
            crate::fs::chapter::load_tale(p, &ch.dir_name, &t.file_name).ok()
                .map(|c| c.chars().filter(|ch| !ch.is_whitespace()).count())
        })
    }).sum()
}

fn handle_daily_stats(p: &mut Project) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if p.daily_stats.last_date != today {
        let total = get_total_chars(p);
        p.daily_stats.last_date = today;
        p.daily_stats.start_count = total;
        let _ = crate::fs::project::save_project(p);
    }
}

fn get_other_files_total(p: &Project, active: &Option<DocRef>) -> usize {
    p.chapters.iter().flat_map(|ch| {
        ch.tales.iter().filter_map(|t| {
            let is_active = active.as_ref().map_or(false, |a| match a {
                DocRef::Tale { chapter_dir, tale_file, .. } => chapter_dir == &ch.dir_name && tale_file == &t.file_name,
                _ => false,
            });
            if is_active { return None; }
            if let Some(cached) = t.cached_char_count {
                return Some(cached);
            }
            crate::fs::chapter::load_tale(p, &ch.dir_name, &t.file_name).ok()
                .map(|c| c.chars().filter(|ch| !ch.is_whitespace()).count())
        })
    }).sum()
}

#[component]
pub fn App() -> Element {
    let project = use_signal(|| Option::<Project>::None);
    let open_tabs: Signal<Vec<DocRef>> = use_signal(Vec::new);
    let active_tab: Signal<Option<DocRef>> = use_signal(|| None);
    let mut tab_content: Signal<HashMap<DocRef, String>> = use_signal(HashMap::new);

    let content = use_signal(|| String::new());
    let other_files_total = use_signal(|| 0usize);
    let mut daily_progress = use_signal(|| 0usize);

    use_effect(move || {
        let cur_count = content.read().chars().filter(|c| !c.is_whitespace()).count();
        let other = *other_files_total.read();
        let start = project.read().as_ref().map_or(0, |p| p.daily_stats.start_count);
        daily_progress.set((cur_count + other).saturating_sub(start));
    });

    let writing_mode = use_signal(|| WritingMode::Horizontal);
    let mut dialog_visible = use_signal(|| false);
    let mut project_picker_visible = use_signal(|| false);
    let project_list: Signal<Vec<(String, String)>> = use_signal(Vec::new);
    let rename_dialog_visible = use_signal(|| false);
    let mut export_dialog_visible = use_signal(|| false);
    let rename_target = use_signal(|| (String::new(), String::new()));
    let mut save_notification = use_signal(|| Option::<String>::None);
    let is_saved = use_signal(|| true);
    let mut tab_dirty: Signal<HashSet<DocRef>> = use_signal(|| HashSet::new());
    let mut tab_close_pending: Signal<Option<DocRef>> = use_signal(|| None);
    {
        let is_saved = is_saved.clone();
        let active_tab = active_tab.clone();
        let mut tab_dirty = tab_dirty.clone();
        use_effect(move || {
            if !*is_saved.read() {
                if let Some(doc) = active_tab.read().clone() {
                    tab_dirty.write().insert(doc);
                }
            }
        });
    }
    let mut global_settings = use_signal(|| fs::settings::load_global_settings());
    let is_dark = use_memo(move || global_settings.read().theme_dark);
    let auto_save_enabled = use_memo(move || global_settings.read().auto_save);
    let font_size = use_memo(move || global_settings.read().font_size);
    
    let recent_projects = use_signal(|| load_recent());

    {
        let mut project = project.clone();
        let mut recent = recent_projects.clone();
        let mut other_files_total = other_files_total.clone();
        let mut initialized = use_signal(|| false);
        use_effect(move || {
            let mut init = initialized.write();
            if *init { return; }
            *init = true;
            drop(init);
            let settings = fs::settings::load_global_settings();
            if let Some(ref path) = settings.last_project_path {
                let dir = std::path::Path::new(path);
                if dir.exists() {
                    match fs::project::load_project(dir) {
                        Ok(mut p) => {
                            handle_daily_stats(&mut p);
                            push_recent(&mut recent.write(), path.clone());
                            other_files_total.set(get_other_files_total(&p, &None));
                            *project.write() = Some(p);
                        }
                        Err(_) => {}
                    }
                }
            }
        });
    }

    #[cfg(not(target_os = "android"))]
    let desktop = use_window();

    let pending_delete: Signal<Option<PendingDelete>> = use_signal(|| None);
    let mut close_pending: Signal<Option<CloseAction>> = use_signal(|| None);
    let settings_visible = use_signal(|| false);
    let project_name = use_signal(|| String::new());
    let project_settings = use_signal(|| ProjectSettings::default());
    let chapter_version = use_signal(|| 0u32);
    let mut show_sidebar = use_signal(|| true);
    let mut show_editor = use_signal(|| true);
    let mut show_preview = use_signal(|| true);
    let mut focus_mode = use_signal(|| false);

    let activity_tab = use_signal(|| ActivityTab::Explorer);
    let mut mobile_page = use_signal(|| crate::model::MobilePage::Editor);


    let writing_mode_str = use_memo(move || match *writing_mode.read() {
        WritingMode::Vertical => "vertical".to_string(),
        WritingMode::Horizontal => "horizontal".to_string(),
    });

    #[cfg(not(target_os = "android"))]
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

    #[cfg(not(target_os = "android"))]
    {
        let desktop_top = desktop.clone();
        use_effect(move || {
            desktop_top.set_always_on_top(false);
        });
    }

    // Resize JS (shared between desktop and Android)
    let resize_js = r#"
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
            var pvH = pv ? (parseFloat(pv.style.height) || 300) : 300;
            var sbW = sb ? (parseFloat(sb.style.width) || 240) : 0;
            pvH = Math.max(MIN_PV, Math.min(MAX_PV, Math.min(pvH, ch - HW - MIN_ED - (vsb ? MIN_SB : 0))));
            if (vpv) { pv.style.height = pvH + 'px'; pv.style.flexBasis = pvH + 'px'; }
            if (vsb) {
                sbW = Math.max(MIN_SB, Math.min(MAX_SB, Math.min(sbW, cw - HW - MIN_ED)));
                sb.style.width = sbW + 'px';
            }
        } else {
            var aw = cw - ((vsb && ved ? 1 : 0) + (ved && vpv ? 1 : 0)) * HW;
            if (aw <= 0) return;
            var sbW = sb ? (parseFloat(sb.style.width) || 240) : 0;
            var pvW = pv ? (parseFloat(pv.style.width) || 400) : 0;
            if (vsb) {
                sbW = Math.max(MIN_SB, Math.min(MAX_SB, Math.min(sbW, aw - (vpv ? MIN_PV : 0) - MIN_ED)));
                sb.style.width = sbW + 'px';
            }
            if (vpv) {
                pvW = Math.max(MIN_PV, Math.min(MAX_PV, Math.min(pvW, aw - (vsb ? MIN_SB : 0) - MIN_ED)));
                pv.style.width = pvW + 'px';
            }
        }
    }
    window.__chronicle_apply = apply;

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
        DRAG = { id: h.id, startPos: isV ? e.clientY : e.clientX, startSize: isV ? el.offsetHeight : el.offsetWidth, isV: isV };
    });

    document.addEventListener('mousemove', function(e) {
        if (!DRAG) return;
        var delta = (DRAG.isV ? e.clientY : e.clientX) - DRAG.startPos;
        var c = document.querySelector('.main-content');
        if (!c) return;
        var cSize = DRAG.isV ? c.offsetHeight : c.offsetWidth;
        var sb = document.querySelector('.sidebar');
        var pv = document.querySelector('.preview-pane');

        if (DRAG.id === 'resize-sidebar') {
            if (!sb) return;
            var vsb = vis(sb), vpv = pv && vis(pv);
            var maxW = cSize - HW - (vpv ? MIN_PV : 0) - MIN_ED;
            var newW = Math.max(MIN_SB, Math.min(MAX_SB, DRAG.startSize + delta, maxW));
            sb.style.width = newW + 'px';
        } else if (DRAG.id === 'resize-preview') {
            if (!pv) return;
            var vsb = sb && vis(sb);
            if (DRAG.isV) {
                var maxH = cSize - HW - MIN_ED;
                var newH = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxH));
                pv.style.height = newH + 'px'; pv.style.flexBasis = newH + 'px';
            } else {
                var maxW = cSize - HW - (vsb ? MIN_SB : 0) - MIN_ED;
                var newW = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxW));
                pv.style.width = newW + 'px';
            }
        }
    });

    document.addEventListener('mouseup', function() { DRAG = null; });
    document.addEventListener('mouseleave', function() { DRAG = null; });

    // Touch events for mobile/Android
    document.addEventListener('touchstart', function(e) {
        var h = e.target.closest('.resize-handle');
        if (!h) return;
        e.preventDefault();
        var c = document.querySelector('.main-content');
        if (!c) return;
        var pb = c.classList.contains('preview-bottom');
        var isV = pb && h.id === 'resize-preview';
        var el = h.id === 'resize-sidebar' ? document.querySelector('.sidebar') : document.querySelector('.preview-pane');
        if (!el) return;
        var t = e.touches[0];
        DRAG = { id: h.id, startPos: isV ? t.clientY : t.clientX, startSize: isV ? el.offsetHeight : el.offsetWidth, isV: isV };
    }, { passive: false });

    document.addEventListener('touchmove', function(e) {
        if (!DRAG) return;
        e.preventDefault();
        var t = e.touches[0];
        var delta = (DRAG.isV ? t.clientY : t.clientX) - DRAG.startPos;
        var c = document.querySelector('.main-content');
        if (!c) return;
        var cSize = DRAG.isV ? c.offsetHeight : c.offsetWidth;
        var sb = document.querySelector('.sidebar');
        var pv = document.querySelector('.preview-pane');

        if (DRAG.id === 'resize-sidebar') {
            if (!sb) return;
            var vsb = vis(sb), vpv = pv && vis(pv);
            var maxW = cSize - HW - (vpv ? MIN_PV : 0) - MIN_ED;
            var newW = Math.max(MIN_SB, Math.min(MAX_SB, DRAG.startSize + delta, maxW));
            sb.style.width = newW + 'px';
        } else if (DRAG.id === 'resize-preview') {
            if (!pv) return;
            var vsb = sb && vis(sb);
            if (DRAG.isV) {
                var maxH = cSize - HW - MIN_ED;
                var newH = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxH));
                pv.style.height = newH + 'px'; pv.style.flexBasis = newH + 'px';
            } else {
                var maxW = cSize - HW - (vsb ? MIN_SB : 0) - MIN_ED;
                var newW = Math.max(MIN_PV, Math.min(MAX_PV, DRAG.startSize - delta, maxW));
                pv.style.width = newW + 'px';
            }
        }
    }, { passive: false });

    document.addEventListener('touchend', function() { DRAG = null; });

    if (window.ResizeObserver) {
        var mc = document.querySelector('.main-content');
        if (mc) { var ro = new ResizeObserver(function() { apply(); }); ro.observe(mc); }
    }

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

    #[cfg(not(target_os = "android"))]
    {
        let desktop_ui = desktop.clone();
        use_effect(move || {
            let _ = desktop_ui.webview.evaluate_script(resize_js);
        });
    }

    #[cfg(target_os = "android")]
    {
        use_effect(move || {
            let _ = eval(resize_js);
        });
    }
    // ── Helpers ──

    let switch_to_doc = {
        let mut active_tab = active_tab.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut chapter_version = chapter_version.clone();
        let project = project.clone();
        let mut content_sig = content.clone();
        let mut other_files_total = other_files_total.clone();
        let mut is_saved = is_saved.clone();
        move |doc: DocRef| {
            if let Some(ref p) = *project.read() {
                other_files_total.set(get_other_files_total(p, &Some(doc.clone())));
            }
            
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

            active_tab.set(Some(doc.clone()));
            if let Some(text) = tab_content.read().get(&doc).cloned() {
                content_sig.set(text);
            } else {
                content_sig.set(String::new());
            }
            is_saved.set(true);
            let next = *chapter_version.read() + 1;
            chapter_version.set(next);
        }
    };

    let mut on_open_doc = {
        let mut switch_to_doc = switch_to_doc.clone();
        move |doc: DocRef| {
            switch_to_doc(doc);
        }
    };

    let mut execute_close_tab = {
        let mut active_tab = active_tab.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut content_sig = content.clone();
        let mut chapter_version = chapter_version.clone();
        let mut is_saved = is_saved.clone();
        let mut tab_dirty = tab_dirty.clone();
        move |doc: DocRef| {
            tab_dirty.write().remove(&doc);
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
                    active_tab.set(Some(new_doc.clone()));
                    if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                        content_sig.set(text);
                    }
                }
            }
            let next = *chapter_version.read() + 1;
            chapter_version.set(next);
        }
    };

    #[cfg(not(target_os = "android"))]
    let on_close_tab = {
        let tab_dirty = tab_dirty.clone();
        let mut tab_close_pending = tab_close_pending.clone();
        let mut execute_close_tab = execute_close_tab.clone();
        move |doc: DocRef| {
            if tab_dirty.read().contains(&doc) {
                tab_close_pending.set(Some(doc));
            } else {
                execute_close_tab(doc);
            }
        }
    };

    let mut do_save_current = {
        let mut project = project.clone();
        let active_tab = active_tab.clone();
        let tab_content = tab_content.clone();
        let mut is_saved = is_saved.clone();
        let mut save_notification = save_notification.clone();
        let mut content_sig = content.clone();
        let mut tab_dirty = tab_dirty.clone();
        move || {
            let doc = active_tab.read().clone();
            let mut p = project.write();
            if let (Some(d), Some(ref mut proj)) = (doc, p.as_mut()) {
                if let Some(text) = tab_content.read().get(&d).cloned() {
                    match save_doc_content(proj, &d, &text) {
                        Ok(_) => {
                            if let DocRef::Tale { chapter_dir, tale_file, .. } = &d {
                                let count = text.chars().filter(|c| !c.is_whitespace()).count();
                                for ch in &mut proj.chapters {
                                    if ch.dir_name == *chapter_dir {
                                        for t in &mut ch.tales {
                                            if t.file_name == *tale_file {
                                                t.cached_char_count = Some(count);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            *save_notification.write() = Some("保存しました".to_string());
                            tab_dirty.write().remove(&d);
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

    let mut do_close_project = {
        let mut project = project.clone();
        let mut open_tabs = open_tabs.clone();
        let mut active_tab = active_tab.clone();
        let mut content_sig = content.clone();
        let mut is_saved = is_saved.clone();
        move || {
            project.set(None);
            open_tabs.set(Vec::new());
            active_tab.set(None);
            content_sig.set(String::new());
            is_saved.set(true);
            fs::settings::save_last_project_path(None);
        }
    };

    let on_export = move |_| {
        export_dialog_visible.set(true);
    };

    let on_export_confirm = move |format: ExportFormat| {
        let proj = project.read().clone();
        if let Some(ref proj) = proj {
            let default_name = match format {
                ExportFormat::ProjectZip => format!("{}-backup.zip", proj.name),
                ExportFormat::ManuscriptZipTxt => "原稿_テキスト.zip".to_string(),
                ExportFormat::ManuscriptZipHtml => "原稿_HTML.zip".to_string(),
                ExportFormat::SiteZip => "site.zip".to_string(),
                ExportFormat::NarouZip => "なろう投稿用.zip".to_string(),
                ExportFormat::KakuyomuZip => "カクヨム投稿用.zip".to_string(),
                ExportFormat::HamelnZip => "ハーメルン投稿用.zip".to_string(),
            };

            let path = pick_save_path(&default_name);

            if let Some(path) = path {
                let res = match format {
                    ExportFormat::ProjectZip => crate::export::export_project_zip(proj, &path),
                    ExportFormat::ManuscriptZipTxt | ExportFormat::ManuscriptZipHtml => {
                        crate::export::export_manuscript_zip(proj, format, &path)
                    }
                    ExportFormat::SiteZip => crate::export::export_site_zip(proj, &path),
                    ExportFormat::NarouZip => crate::export::export_platform_zip(proj, "narou", &path),
                    ExportFormat::KakuyomuZip => crate::export::export_platform_zip(proj, "kakuyomu", &path),
                    ExportFormat::HamelnZip => crate::export::export_platform_zip(proj, "hameln", &path),
                };

                match res {
                    Ok(_) => {
                        let msg = format!("出力しました: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
                        #[cfg(target_os = "android")]
                        let msg = format!("出力しました: {}", path.display());
                        *save_notification.write() = Some(msg);
                    }
                    Err(e) => {
                        *save_notification.write() = Some(format!("エクスポートエラー: {}", e));
                    }
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
        let mut other_files_total = other_files_total.clone();
        spawn(async move {
            eprintln!("[chronicle] on_confirm_new: name={}, author={}", name, author);
            let dir = pick_folder("プロジェクトを作成する場所を選択");
            if let Some(dir) = dir {
                eprintln!("[chronicle] on_confirm_new: creating project in dir={}", dir.display());
                match fs::project::create_project(&name, &dir) {
                    Ok(mut p) => {
                        eprintln!("[chronicle] on_confirm_new: created OK, root={}", p.root_dir.display());
                        p.settings.author = author;
                        handle_daily_stats(&mut p);
                        let _ = fs::project::save_project(&p);
                        let project_dir = p.root_dir.to_string_lossy().to_string();
                        push_recent(&mut recent.write(), project_dir.clone());
                        other_files_total.set(0);
                        *proj_sig.write() = Some(p);
                        fs::settings::save_last_project_path(Some(&project_dir));
                    }
                    Err(e) => {
                        eprintln!("[chronicle] on_confirm_new: creation FAILED: {}", e);
                        *notif.write() = Some(format!("作成エラー: {}", e));
                    }
                }
            } else {
                eprintln!("[chronicle] on_confirm_new: pick_folder returned None");
            }
        });
    };

    let on_open_project = move |_| {
        let mut proj_sig = project.clone();
        let mut notif = save_notification.clone();
        let mut recent = recent_projects.clone();
        let mut other_files_total = other_files_total.clone();
        let mut picker_visible = project_picker_visible.clone();
        let mut p_list = project_list.clone();
        spawn(async move {
            #[cfg(target_os = "android")]
            {
                let projects = scan_android_projects();
                if projects.is_empty() {
                    eprintln!("[chronicle] on_open_project: no projects found");
                    *notif.write() = Some("プロジェクトが見つかりませんでした。先に新規プロジェクトを作成してください。".to_string());
                } else if projects.len() == 1 {
                    let (_name, path) = projects.into_iter().next().unwrap();
                    eprintln!("[chronicle] on_open_project: auto-loading single project: {}", path);
                    match fs::project::load_project(std::path::Path::new(&path)) {
                        Ok(mut p) => {
                            handle_daily_stats(&mut p);
                            push_recent(&mut recent.write(), path.clone());
                            other_files_total.set(get_other_files_total(&p, &None));
                            *proj_sig.write() = Some(p);
                            fs::settings::save_last_project_path(Some(&path));
                        }
                        Err(e) => {
                            eprintln!("[chronicle] on_open_project: load FAILED: {}", e);
                            *notif.write() = Some(format!("開くエラー: {}", e));
                        }
                    }
                } else {
                    eprintln!("[chronicle] on_open_project: showing picker with {} projects", projects.len());
                    p_list.set(projects);
                    picker_visible.set(true);
                }
            }
            #[cfg(not(target_os = "android"))]
            {
                eprintln!("[chronicle] on_open_project: start");
                let dir = pick_folder("プロジェクトフォルダを選択");
                if let Some(dir) = dir {
                    eprintln!("[chronicle] on_open_project: dir={}", dir.display());
                    if let Some(project_dir) = resolve_project_dir(&dir) {
                        eprintln!("[chronicle] on_open_project: resolved_project={}", project_dir.display());
                        match fs::project::load_project(&project_dir) {
                            Ok(mut p) => {
                                eprintln!("[chronicle] on_open_project: loaded OK, name={}", p.name);
                                handle_daily_stats(&mut p);
                                let pd = project_dir.to_string_lossy().to_string();
                                push_recent(&mut recent.write(), pd);
                                other_files_total.set(get_other_files_total(&p, &None));
                                *proj_sig.write() = Some(p);
                                fs::settings::save_last_project_path(Some(&project_dir.to_string_lossy()));
                            }
                            Err(e) => {
                                eprintln!("[chronicle] on_open_project: load FAILED: {}", e);
                                *notif.write() = Some(format!("開くエラー: {}", e));
                            }
                        }
                    } else {
                        eprintln!("[chronicle] on_open_project: resolve_project_dir returned None");
                        *notif.write() = Some("プロジェクトが見つかりませんでした".to_string());
                    }
                } else {
                    eprintln!("[chronicle] on_open_project: pick_folder returned None");
                }
            }
        });
    };

    let on_select_project = {
        let mut proj_sig = project.clone();
        let mut notif = save_notification.clone();
        let mut recent = recent_projects.clone();
        let mut other_files_total = other_files_total.clone();
        let mut picker_visible = project_picker_visible.clone();
        move |(_name, path): (String, String)| {
            picker_visible.set(false);
            spawn(async move {
                eprintln!("[chronicle] on_select_project: loading {}", path);
                match fs::project::load_project(std::path::Path::new(&path)) {
                    Ok(mut p) => {
                        eprintln!("[chronicle] on_select_project: loaded OK, name={}", p.name);
                        handle_daily_stats(&mut p);
                        push_recent(&mut recent.write(), path.clone());
                        other_files_total.set(get_other_files_total(&p, &None));
                        *proj_sig.write() = Some(p);
                        fs::settings::save_last_project_path(Some(&path));
                    }
                    Err(e) => {
                        eprintln!("[chronicle] on_select_project: load FAILED: {}", e);
                        *notif.write() = Some(format!("開くエラー: {}", e));
                    }
                }
            });
        }
    };

    let on_save = move |_| {
        do_save_current();
    };

    #[cfg(not(target_os = "android"))]
    let desktop_apply_visibility = desktop.clone();
    #[cfg(not(target_os = "android"))]
    let desktop_toggle_dark = desktop.clone();
    let on_toggle_dark = move |_| {
        let mut gs = global_settings.write();
        gs.theme_dark = !gs.theme_dark;
        let _ = fs::settings::save_global_settings(&gs);
        let js = if gs.theme_dark {
            "document.documentElement.classList.add('dark')"
        } else {
            "document.documentElement.classList.remove('dark')"
        };
        #[cfg(not(target_os = "android"))]
        let _ = desktop_toggle_dark.webview.evaluate_script(js);
        #[cfg(target_os = "android")]
        let _ = eval(js);
    };

    let on_toggle_sidebar = move |_| { let v = !*show_sidebar.read(); show_sidebar.set(v); };
    let on_toggle_editor = move |_| { let v = !*show_editor.read(); show_editor.set(v); };
    let on_toggle_preview = move |_| { let v = !*show_preview.read(); show_preview.set(v); };
    let on_toggle_focus_mode = move |_| { let v = !*focus_mode.read(); focus_mode.set(v); };
    let on_increase_font = move |_| { 
        let mut gs = global_settings.write();
        if gs.font_size < 32 { gs.font_size += 1; }
        let _ = fs::settings::save_global_settings(&gs);
    };
    let on_decrease_font = move |_| { 
        let mut gs = global_settings.write();
        if gs.font_size > 8 { gs.font_size -= 1; }
        let _ = fs::settings::save_global_settings(&gs);
    };

    let on_close_project = {
        let is_saved = is_saved.clone();
        let mut close_pending = close_pending.clone();
        move |_| {
            if !*is_saved.read() {
                close_pending.set(Some(CloseAction::CloseProject));
            } else {
                do_close_project();
            }
        }
    };

    #[cfg(not(target_os = "android"))]
    let close_desktop: Rc<dyn Fn()> = {
        let d = desktop.clone();
        Rc::new(move || d.close())
    };
    #[cfg(target_os = "android")]
    let close_desktop: Rc<dyn Fn()> = Rc::new(|| {});

    let on_settings = {
        let project = project.clone();
        let mut settings_visible = settings_visible.clone();
        let mut project_name = project_name.clone();
        let mut project_settings = project_settings.clone();
        move |_| {
            if let Some(ref p) = *project.read() {
                project_name.set(p.name.clone());
                project_settings.set(p.settings.clone());
            } else {
                project_name.set(String::new());
                project_settings.set(ProjectSettings::default());
            }
            settings_visible.set(true);
        }
    };

    let on_confirm_settings = {
        let mut project = project.clone();
        let mut global_settings = global_settings.clone();
        let mut save_notification = save_notification.clone();
        #[cfg(not(target_os = "android"))]
        let desktop = desktop.clone();
        move |(name, p_settings, g_settings): (String, ProjectSettings, crate::model::project::GlobalSettings)| {
            // Save global settings
            *global_settings.write() = g_settings.clone();
            let _ = fs::settings::save_global_settings(&g_settings);
            // Apply dark mode CSS class
            let js = if g_settings.theme_dark {
                "document.documentElement.classList.add('dark')"
            } else {
                "document.documentElement.classList.remove('dark')"
            };
            #[cfg(not(target_os = "android"))]
            let _ = desktop.webview.evaluate_script(js);
            #[cfg(target_os = "android")]
            let _ = eval(js);

            // Save project settings
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                p.name = name;
                p.settings = p_settings;
                let _ = fs::project::save_project(p);
                *save_notification.write() = Some("設定を保存しました".to_string());
            }
        }
    };

    // ── Chapter / Tale / Material CRUD ──

    let on_add_chapter = {
        let mut project = project.clone();
        let mut save_notification = save_notification.clone();
        move |title: String| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                let entry = p.add_chapter(&title);
                let _ = fs::project::save_project(p);
                let _ = fs::chapter::create_chapter_dir(p, &entry.dir_name);
                *save_notification.write() = Some("章を追加しました".to_string());
            }
        }
    };

    let on_delete_chapter = {
        let mut pending_delete = pending_delete.clone();
        move |dir_name: String| {
            pending_delete.set(Some(PendingDelete::Chapter(dir_name)));
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
        let _save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        move |(old_dir, new_title): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                if let Some(new_dir_name) = p.rename_chapter(&old_dir, &new_title) {
                    let _ = fs::chapter::rename_chapter_dir(p, &old_dir, &new_dir_name);
                    let _ = fs::project::save_project(p);

                    // Update tabs referencing this chapter
                    let mut tabs = open_tabs.write();
                    for t in tabs.iter_mut() {
                        if let DocRef::Tale { chapter_dir, chapter_title, .. } = t {
                            if *chapter_dir == old_dir {
                                *chapter_dir = new_dir_name.clone();
                                *chapter_title = new_title.clone();
                            }
                        }
                    }
                    let mut tc = tab_content.write();
                    let old_tabs: Vec<DocRef> = tc.keys().filter(|k| {
                        matches!(k, DocRef::Tale { chapter_dir, .. } if *chapter_dir == old_dir)
                    }).cloned().collect();
                    for old_t in old_tabs {
                        if let DocRef::Tale { tale_file, tale_title, .. } = &old_t {
                            let new_t = DocRef::Tale {
                                chapter_dir: new_dir_name.clone(),
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
                                *chapter_dir = new_dir_name;
                                *chapter_title = new_title;
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
                    let _doc = DocRef::Tale {
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
        let mut pending_delete = pending_delete.clone();
        move |(chapter_dir, tale_file): (String, String)| {
            pending_delete.set(Some(PendingDelete::Tale(chapter_dir, tale_file)));
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
        let _save_notification = save_notification.clone();
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

                        // Update tabs — find old title from existing tabs
                        let new_doc = DocRef::Tale {
                            chapter_dir: chapter_dir.clone(),
                            tale_file: new_f.clone(),
                            chapter_title: ch_title.clone(),
                            tale_title: new_title,
                        };
                        let mut tabs = open_tabs.write();
                        for t in tabs.iter_mut() {
                            if let DocRef::Tale { chapter_dir: cd, tale_file: tf, .. } = t {
                                if *cd == chapter_dir && *tf == old_f {
                                    *t = new_doc.clone();
                                    break;
                                }
                            }
                        }
                        drop(tabs);
                        let mut tc = tab_content.write();
                        // Find and migrate content by matching chapter_dir + old tale_file
                        let old_key: Option<DocRef> = tc.keys()
                            .find(|k| matches!(k, DocRef::Tale { chapter_dir: cd, tale_file: tf, .. } if *cd == chapter_dir && *tf == old_f))
                            .cloned();
                        if let Some(old_k) = old_key {
                            if let Some(v) = tc.remove(&old_k) {
                                tc.insert(new_doc.clone(), v);
                            }
                        }
                        drop(tc);
                        if let Some(ref mut active) = *active_tab.write() {
                            if let DocRef::Tale { chapter_dir: cd, tale_file: tf, .. } = active {
                                if *cd == chapter_dir && *tf == old_f {
                                    *active = new_doc;
                                }
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
        let _save_notification = save_notification.clone();
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
        let mut pending_delete = pending_delete.clone();
        move |file_name: String| {
            pending_delete.set(Some(PendingDelete::Material(file_name)));
        }
    };

    let on_confirm_delete = {
        let mut project = project.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        let mut content_sig = content.clone();
        let mut is_saved = is_saved.clone();
        let mut chapter_version = chapter_version.clone();
        let mut pending_delete = pending_delete.clone();
        let mut other_files_total = other_files_total.clone();
        move |()| {
            let action = pending_delete.read().clone();
            if let Some(action) = action {
                let mut proj = project.write();
                if let Some(ref mut p) = *proj {
                    match action {
                        PendingDelete::Chapter(dir_name) => {
                            // Remove all tab entries for this chapter
                            let close_keys: Vec<DocRef> = tab_content.read().keys()
                                .filter(|k| matches!(k, DocRef::Tale { chapter_dir: cd, .. } if *cd == dir_name))
                                .cloned().collect();
                            for k in close_keys {
                                tab_content.write().remove(&k);
                            }
                            let was_active = open_tabs.read().iter().any(|t|
                                matches!(t, DocRef::Tale { chapter_dir: cd, .. } if *cd == dir_name)
                                && Some(t.clone()) == active_tab.read().as_ref().cloned()
                            );
                            let mut tabs = open_tabs.write();
                            tabs.retain(|t| !matches!(t, DocRef::Tale { chapter_dir: cd, .. } if *cd == dir_name));
                            if tabs.is_empty() {
                                drop(tabs);
                                active_tab.set(None);
                                content_sig.set(String::new());
                                is_saved.set(true);
                            } else if was_active {
                                let new_doc = tabs[0].clone();
                                drop(tabs);
                                active_tab.set(Some(new_doc.clone()));
                                if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                                    content_sig.set(text);
                                }
                            }
                            let _ = fs::chapter::delete_chapter_dir(p, &dir_name);
                            p.remove_chapter(&dir_name);
                        }
                        PendingDelete::Tale(chapter_dir, tale_file) => {
                            // Remove from tab_content: match by chapter_dir + tale_file
                            let keys_to_remove: Vec<DocRef> = tab_content.read().keys()
                                .filter(|k| matches!(k, DocRef::Tale { chapter_dir: cd, tale_file: tf, .. }
                                    if *cd == chapter_dir && *tf == tale_file))
                                .cloned().collect();
                            for k in keys_to_remove {
                                tab_content.write().remove(&k);
                            }
                            let was_active = open_tabs.read().iter().any(|t|
                                matches!(t, DocRef::Tale { chapter_dir: cd, tale_file: tf, .. }
                                    if *cd == chapter_dir && *tf == tale_file)
                                && Some(t.clone()) == active_tab.read().as_ref().cloned()
                            );
                            let mut tabs = open_tabs.write();
                            tabs.retain(|t| !matches!(t, DocRef::Tale { chapter_dir: cd, tale_file: tf, .. }
                                if *cd == chapter_dir && *tf == tale_file));
                            drop(tabs);
                            if open_tabs.read().is_empty() {
                                active_tab.set(None);
                                content_sig.set(String::new());
                                is_saved.set(true);
                            } else if was_active {
                                let new_doc = open_tabs.read()[0].clone();
                                active_tab.set(Some(new_doc.clone()));
                                if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                                    content_sig.set(text);
                                }
                            }
                            let _ = fs::chapter::delete_tale_file(p, &chapter_dir, &tale_file);
                            p.remove_tale(&chapter_dir, &tale_file);
                        }
                        PendingDelete::Material(file_name) => {
                            // Remove from tab_content: match by file_name
                            let keys_to_remove: Vec<DocRef> = tab_content.read().keys()
                                .filter(|k| matches!(k, DocRef::Material { file_name: fn_, .. }
                                    if *fn_ == file_name))
                                .cloned().collect();
                            for k in keys_to_remove {
                                tab_content.write().remove(&k);
                            }
                            let was_active = open_tabs.read().iter().any(|t|
                                matches!(t, DocRef::Material { file_name: fn_, .. }
                                    if *fn_ == file_name)
                                && Some(t.clone()) == active_tab.read().as_ref().cloned()
                            );
                            let mut tabs = open_tabs.write();
                            tabs.retain(|t| !matches!(t, DocRef::Material { file_name: fn_, .. }
                                if *fn_ == file_name));
                            drop(tabs);
                            if open_tabs.read().is_empty() {
                                active_tab.set(None);
                                content_sig.set(String::new());
                                is_saved.set(true);
                            } else if was_active {
                                let new_doc = open_tabs.read()[0].clone();
                                active_tab.set(Some(new_doc.clone()));
                                if let Some(text) = tab_content.read().get(&new_doc).cloned() {
                                    content_sig.set(text);
                                }
                            }
                            let _ = fs::material::delete_material_file(p, &file_name);
                            p.remove_material(&file_name);
                        }
                    }
                    let _ = fs::project::save_project(p);
                    chapter_version += 1;
                    other_files_total.set(get_other_files_total(p, &*active_tab.read()));
                }
                pending_delete.set(None);
            }
        }
    };

    let on_rename_material = {
        let mut rename_dialog_visible = rename_dialog_visible.clone();
        let mut rename_target = rename_target.clone();
        move |(file_name, current_title): (String, String)| {
            rename_target.set((file_name, current_title));
            rename_dialog_visible.set(true);
        }
    };

    let on_confirm_rename_material = {
        let mut project = project.clone();
        let _save_notification = save_notification.clone();
        let mut chapter_version = chapter_version.clone();
        let mut open_tabs = open_tabs.clone();
        let mut tab_content = tab_content.clone();
        let mut active_tab = active_tab.clone();
        move |(old_file, new_title): (String, String)| {
            let mut proj = project.write();
            if let Some(ref mut p) = *proj {
                if let Some((old_f, new_f)) = p.rename_material(&old_file, &new_title) {
                    let _ = fs::material::rename_material_file(p, &old_f, &new_f);
                    let _ = fs::project::save_project(p);

                    let new_doc = DocRef::Material {
                        file_name: old_file.clone(),
                        title: new_title,
                    };
                    // Update tabs — match by old file_name instead of relying on title
                    let mut tabs = open_tabs.write();
                    for t in tabs.iter_mut() {
                        if let DocRef::Material { file_name: f, .. } = t {
                            if *f == old_f {
                                *t = new_doc.clone();
                                break;
                            }
                        }
                    }
                    drop(tabs);
                    let mut tc = tab_content.write();
                    let old_key: Option<DocRef> = tc.keys()
                        .find(|k| matches!(k, DocRef::Material { file_name: f, .. } if *f == old_f))
                        .cloned();
                    if let Some(old_k) = old_key {
                        if let Some(v) = tc.remove(&old_k) {
                            tc.insert(new_doc.clone(), v);
                        }
                    }
                    drop(tc);
                    if let Some(ref mut active) = *active_tab.write() {
                        if let DocRef::Material { file_name: f, .. } = active {
                            if *f == old_f {
                                *active = new_doc;
                            }
                        }
                    }
                    chapter_version += 1;
                }
            }
        }
    };

    // ── Recalculate panel widths on toggle (desktop only) ──
    #[cfg(not(target_os = "android"))]
    use_effect(use_reactive(&(show_sidebar, show_editor, show_preview), move |_| {
        let _ = desktop_apply_visibility.webview.evaluate_script("if(window.__chronicle_apply) window.__chronicle_apply();");
    }));

    // ── Sync content changes back to tab_content ──
    // (plain use_effect + value comparison, avoids use_reactive reliability issues)

    let mut prev = use_signal(|| String::new());
    use_effect(move || {
        let cur = content.read().clone();
        if cur != *prev.read() {
            prev.set(cur.clone());
            if let Some(ref doc) = *active_tab.read() {
                tab_content.write().insert(doc.clone(), cur);
            }
        }
    });

    // ── Auto-save ──

    let auto_save = auto_save_enabled.clone();
    let mut last_tc = use_signal(|| String::new());
    let mut save_gen: Signal<u64> = use_signal(|| 0);
    let proj = project.clone();
    use_effect(move || {
        let doc = active_tab.read().clone();
        let cur = doc.as_ref()
            .and_then(|d| tab_content.read().get(d).cloned())
            .unwrap_or_default();
        if cur != *last_tc.read() && *auto_save.read() {
            last_tc.set(cur.clone());
            let g = *save_gen.read() + 1;
            *save_gen.write() = g;
            let tc = tab_content.clone();
            let gen_sig = save_gen.clone();
            let mut pclone = proj.clone();
            let mut saved_flag = is_saved.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if *gen_sig.read() != g {
                    return;
                }
                let d = doc.clone();
                let mut p = pclone.write();
                if let (Some(ref doc), Some(ref mut proj)) = (d, p.as_mut()) {
                    if let Some(text) = tc.read().get(doc).cloned() {
                        let _ = save_doc_content(proj, doc, &text);
                        saved_flag.set(true);
                        if let DocRef::Tale { chapter_dir, tale_file, .. } = doc {
                            let count = text.chars().filter(|c| !c.is_whitespace()).count();
                            for ch in &mut proj.chapters {
                                if ch.dir_name == *chapter_dir {
                                    for t in &mut ch.tales {
                                        if t.file_name == *tale_file {
                                            t.cached_char_count = Some(count);
                                            break;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }
    });

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

    let resize_grip = {
        #[cfg(not(target_os = "android"))]
        {
            rsx! { div { class: "resize-grip" } }
        }
        #[cfg(target_os = "android")]
        {
            rsx! {}
        }
    };

    let sidebar_visible = *show_sidebar.read();
    let preview_visible = *show_preview.read();
    let editor_visible = *show_editor.read();
    #[cfg(not(target_os = "android"))]
    let recent_list = recent_projects.read().clone();
    let has_project = project.read().is_some();
    let proj_name_for_welcome = project.read().as_ref().map(|p| p.name.clone()).unwrap_or_default();

    // Derive current content from active tab for editor
    let editor_placeholder = match active_tab.read().as_ref() {
        Some(DocRef::Tale { tale_title, .. }) => format!("「{}」を書き始めましょう...", tale_title),
        Some(DocRef::Material { title, .. }) => format!("「{}」の内容を入力...", title),
        None => "章・話を選択してください".to_string(),
    };
    let mobile_placeholder = editor_placeholder.clone();

    // Desktop main content (side-by-side panels)
    #[cfg(not(target_os = "android"))]
    let desktop_main_content = rsx! {
        div { class: main_class,
            if sidebar_visible {
                Sidebar {
                    project: project,
                    active_tab: active_tab,
                    activity_tab: activity_tab,
                    on_add_chapter: on_add_chapter.clone(),
                    on_delete_chapter: on_delete_chapter.clone(),
                    on_rename_chapter: on_rename_chapter.clone(),
                    on_add_tale: on_add_tale.clone(),
                    on_delete_tale: on_delete_tale.clone(),
                    on_rename_tale: on_rename_tale.clone(),
                    on_add_material: on_add_material.clone(),
                    on_delete_material: on_delete_material.clone(),
                    on_rename_material: on_rename_material.clone(),
                    on_open_doc: on_open_doc.clone(),
                }
            }
            if sidebar_visible {
                div { id: "resize-sidebar", class: "resize-handle" }
            }
            div { class: if *show_editor.read() { "editor-pane" } else { "editor-pane hidden" },
                TabBar {
                    open_tabs: open_tabs,
                    active_tab: active_tab,
                    on_close_tab: on_close_tab,
                    on_open_doc: on_open_doc.clone(),
                }
                if active_tab.read().is_some() {
                    Editor {
                        content: content,
                        project: project,
                        global_settings: global_settings,
                        is_saved: is_saved,
                        on_save: on_save,
                        focus_mode: focus_mode,
                        placeholder: editor_placeholder,
                    }
                } else {
                    if has_project {
                        div { class: "welcome",
                            h2 { "{proj_name_for_welcome}" }
                            p { "サイドバーから章や話を選択してください" }
                        }
                    } else {
                        div { class: "welcome",
                            h1 { "Chronicle" }
                            p { "小説執筆支援アプリケーション" }
                            div { class: "welcome-actions",
                                button { class: "welcome-btn", onclick: move |_| dialog_visible.set(true), "新規プロジェクト" }
                                button { class: "welcome-btn", onclick: move |_| {
                                    let mut proj_sig = project.clone();
                                    let mut notif = save_notification.clone();
                                    let mut recent = recent_projects.clone();
                                    spawn(async move {
                                        let dir = pick_folder("プロジェクトフォルダを選択");
                                        if let Some(dir) = dir {
                                            match crate::fs::project::load_project(&dir) {
                                                Ok(p) => {
                                                    push_recent(&mut recent.write(), dir.to_string_lossy().to_string());
                                                    *proj_sig.write() = Some(p);
                                                    fs::settings::save_last_project_path(Some(&dir.to_string_lossy()));
                                                }
                                                Err(e) => { *notif.write() = Some(format!("開くエラー: {}", e)); }
                                            }
                                        }
                                    });
                                }, "プロジェクトを開く" }
                            }
                            if !recent_list.is_empty() {
                                div { class: "welcome-recent",
                                    h3 { "最近のプロジェクト" }
                                    ul {
                                        {recent_list.iter().rev().map(|dir| {
                                            let d = dir.clone();
                                            let name = std::path::Path::new(&d).file_name().and_then(|n| n.to_str()).unwrap_or(&d).to_string();
                                            rsx! {
                                                li {
                                                    class: "welcome-recent-item",
                                                    onclick: move |_| {
                                                        let mut proj_sig = project.clone();
                                                        let mut notif = save_notification.clone();
                                                        let mut recent = recent_projects.clone();
                                                        let dir = d.clone();
                                                        spawn(async move {
                                                            match crate::fs::project::load_project(std::path::Path::new(&dir)) {
                                                                Ok(p) => { push_recent(&mut recent.write(), dir.clone()); *proj_sig.write() = Some(p); fs::settings::save_last_project_path(Some(&dir)); }
                                                                Err(e) => { *notif.write() = Some(format!("開くエラー: {}", e)); }
                                                            }
                                                        });
                                                    },
                                                    "{name}"
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if preview_visible {
                div { id: "resize-preview", class: "resize-handle" }
            }
            div { class: if *show_preview.read() { "preview-pane" } else { "preview-pane hidden" },
                Preview {
                    content: content,
                    writing_mode: writing_mode_str,
                    project: project,
                    global_settings: global_settings,
                }
            }
        }
    };
    #[cfg(target_os = "android")]
    let desktop_main_content = rsx! {};

    // Mobile main content (page-based, used on Android and narrow desktop)
    let mobile_main_content = {
        let page = *mobile_page.read();
        rsx! {
            div { class: "main-content mobile",
                match page {
                    crate::model::MobilePage::Files => rsx! {
                        Sidebar {
                            project: project,
                            active_tab: active_tab,
                            activity_tab: activity_tab,
                            on_add_chapter: on_add_chapter.clone(),
                            on_delete_chapter: on_delete_chapter.clone(),
                            on_rename_chapter: on_rename_chapter.clone(),
                            on_add_tale: on_add_tale.clone(),
                            on_delete_tale: on_delete_tale.clone(),
                            on_rename_tale: on_rename_tale.clone(),
                            on_add_material: on_add_material.clone(),
                            on_delete_material: on_delete_material.clone(),
                            on_rename_material: on_rename_material.clone(),
                            on_open_doc: {
                                let mut mp = mobile_page.clone();
                                move |doc| { on_open_doc(doc); mp.set(crate::model::MobilePage::Editor); }
                            },
                        }
                    },
                    crate::model::MobilePage::Editor => rsx! {
                        div { class: "editor-pane",
                            if active_tab.read().is_some() {
                                Editor {
                                    content: content,
                                    project: project,
                                    global_settings: global_settings,
                                    is_saved: is_saved,
                                    on_save: on_save,
                                    focus_mode: focus_mode,
                                    placeholder: mobile_placeholder,
                                }
                            } else {
                                if has_project {
                                    div { class: "welcome",
                                        h2 { "{proj_name_for_welcome}" }
                                        p { "サイドバーから章や話を選択してください" }
                                    }
                                } else {
                                    div { class: "welcome",
                                        h1 { "Chronicle" }
                                        p { "小説執筆支援アプリケーション" }
                                        div { class: "welcome-actions",
                                            button { class: "welcome-btn", onclick: move |_| dialog_visible.set(true), "新規プロジェクト" }
                                            button { class: "welcome-btn", onclick: {
                                                let mut picker_visible = project_picker_visible.clone();
                                                let mut p_list = project_list.clone();
                                                let mut proj_sig = project.clone();
                                                let mut notif = save_notification.clone();
                                                let mut recent = recent_projects.clone();
                                                move |_| {
                                                    #[cfg(target_os = "android")]
                                                    {
                                                        let projects = scan_android_projects();
                                                        if projects.is_empty() {
                                                            *notif.write() = Some("プロジェクトが見つかりませんでした。先に新規プロジェクトを作成してください。".to_string());
                                                        } else if projects.len() == 1 {
                                                            let (_name, path) = projects.into_iter().next().unwrap();
                                                            spawn(async move {
                                                                match crate::fs::project::load_project(std::path::Path::new(&path)) {
                                                                    Ok(p) => {
                                                                        let pd = path.clone();
                                                                        push_recent(&mut recent.write(), pd);
                                                                        *proj_sig.write() = Some(p);
                                                                        fs::settings::save_last_project_path(Some(&path));
                                                                    }
                                                                    Err(e) => { *notif.write() = Some(format!("開くエラー: {}", e)); }
                                                                }
                                                            });
                                                        } else {
                                                            p_list.set(projects);
                                                            picker_visible.set(true);
                                                        }
                                                    }
                                                    #[cfg(not(target_os = "android"))]
                                                    {
                                                        let mut proj_sig = proj_sig.clone();
                                                        let mut notif = notif.clone();
                                                        let mut recent = recent.clone();
                                                        spawn(async move {
                                                            let dir = pick_folder("プロジェクトフォルダを選択");
                                                            if let Some(dir) = dir {
                                                                match crate::fs::project::load_project(&dir) {
                                                                    Ok(p) => {
                                                                        push_recent(&mut recent.write(), dir.to_string_lossy().to_string());
                                                                        *proj_sig.write() = Some(p);
                                                                        fs::settings::save_last_project_path(Some(&dir.to_string_lossy()));
                                                                    }
                                                                    Err(e) => { *notif.write() = Some(format!("開くエラー: {}", e)); }
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }, "プロジェクトを開く" }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    crate::model::MobilePage::Preview => rsx! {
                        div { class: "preview-pane",
                            Preview {
                                content: content,
                                writing_mode: writing_mode_str,
                                project: project,
                                global_settings: global_settings,
                            }
                        }
                    },
                }
            }
        }
    };
    // Status bar (desktop only)
    #[cfg(not(target_os = "android"))]
    let status_bar = rsx! {
        StatusBar {
            project: project,
            active_tab: active_tab,
            tab_content: tab_content,
            is_saved: is_saved,
            auto_save_enabled: auto_save_enabled,
            writing_mode: writing_mode_str,
            font_size: font_size,
            on_increase_font: on_increase_font,
            on_decrease_font: on_decrease_font,
        }
    };
    #[cfg(target_os = "android")]
    let status_bar = rsx! {};

    // Bottom navigation (used on Android and narrow desktop)
    let bottom_nav = rsx! {
        nav { class: "mobile-nav",
            button {
                class: if *mobile_page.read() == crate::model::MobilePage::Files { "mobile-nav-btn active" } else { "mobile-nav-btn" },
                onclick: move |_| mobile_page.set(crate::model::MobilePage::Files),
                "\u{1F4C1}",
                span { class: "mobile-nav-label", "ファイル" }
            }
            button {
                class: if *mobile_page.read() == crate::model::MobilePage::Editor { "mobile-nav-btn active" } else { "mobile-nav-btn" },
                onclick: move |_| mobile_page.set(crate::model::MobilePage::Editor),
                "\u{270D}",
                span { class: "mobile-nav-label", "エディタ" }
            }
            button {
                class: if *mobile_page.read() == crate::model::MobilePage::Preview { "mobile-nav-btn active" } else { "mobile-nav-btn" },
                onclick: move |_| mobile_page.set(crate::model::MobilePage::Preview),
                "\u{1F441}",
                span { class: "mobile-nav-label", "プレビュー" }
            }
        }
    };

    rsx! {
        style { "{styles::MAIN_CSS}" }
        div { class: "app",
            Toolbar {
                writing_mode: writing_mode,
                content: content,
                daily_progress: daily_progress,
                project: project,
                on_new_project: on_new_project,
                on_open_project: on_open_project,
                on_close_project: on_close_project,
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
                on_settings: on_settings,
            }
            div { class: "desktop-only", {desktop_main_content} }
            div { class: "mobile-only", {mobile_main_content} }
            div { class: "desktop-only", {status_bar} }
            div { class: "mobile-only", {bottom_nav} }
            ProjectDialog {
                visible: dialog_visible,
                title: "新規プロジェクト".to_string(),
                on_confirm: on_confirm_new,
            }
            ProjectPickerDialog {
                visible: project_picker_visible,
                projects: project_list,
                on_select: on_select_project,
                on_cancel: move |_| {},
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
            if *settings_visible.read() {
                SettingsDialog {
                    visible: settings_visible,
                    project_name: project_name,
                    project_settings: project_settings,
                    global_settings: global_settings,
                    project_is_open: project.read().is_some(),
                    on_save: on_confirm_settings,
                }
            }

            ExportDialog {
                visible: export_dialog_visible,
                on_export: on_export_confirm,
            }
            ConfirmDialog {
                pending: pending_delete,
                on_confirm: on_confirm_delete,
            }
            if let Some(pending_doc) = tab_close_pending.read().clone() {
                div { class: "dialog-overlay",
                    onclick: move |_| tab_close_pending.set(None),
                    div { class: "dialog", onclick: |e| e.stop_propagation(),
                        h2 { "確認" }
                        div { class: "dialog-body",
                            p { "保存していない変更があります。" }
                            p { "保存してから閉じますか？" }
                        }
                        div { class: "dialog-actions",
                            button {
                                class: "dialog-btn",
                                onclick: move |_| tab_close_pending.set(None),
                                "キャンセル"
                            }
                            button {
                                class: "dialog-btn",
                                onclick: {
                                    let pd = pending_doc.clone();
                                    move |_| {
                                        tab_dirty.write().remove(&pd);
                                        execute_close_tab(pd.clone());
                                        tab_close_pending.set(None);
                                    }
                                },
                                "保存せずに閉じる"
                            }
                            button {
                                class: "dialog-btn primary",
                                onclick: {
                                    let pd = pending_doc.clone();
                                    let mut proj = project.clone();
                                    let mut nf = save_notification.clone();
                                    let mut ec = execute_close_tab.clone();
                                    let mut tcp = tab_close_pending.clone();
                                    move |_| {
                                        let mut p = proj.write();
                                        if let Some(ref mut proj2) = p.as_mut() {
                                            if let Some(text) = tab_content.read().get(&pd).cloned() {
                                                let _ = save_doc_content(proj2, &pd, &text);
                                                *nf.write() = Some("保存しました".to_string());
                                            }
                                        }
                                        drop(p);
                                        tab_dirty.write().remove(&pd);
                                        ec(pd.clone());
                                        tcp.set(None);
                                    }
                                },
                                "保存して閉じる"
                            }
                        }
                    }
                }
            }
            if close_pending.read().is_some() {
                div { class: "dialog-overlay",
                    onclick: move |_| close_pending.set(None),
                    div { class: "dialog", onclick: |e| e.stop_propagation(),
                        h2 { "確認" }
                        div { class: "dialog-body",
                            p { "保存していない変更があります。" }
                            p { "保存してから閉じますか？" }
                        }
                        div { class: "dialog-actions",
                            button {
                                class: "dialog-btn",
                                onclick: move |_| close_pending.set(None),
                                "キャンセル"
                            }
                            button {
                                class: "dialog-btn",
                                onclick: {
                                    let cd = close_desktop.clone();
                                    let mut cp = close_pending.clone();
                                    let mut proj = project.clone();
                                    let mut tabs = open_tabs.clone();
                                    let mut atab = active_tab.clone();
                                    let mut cont = content.clone();
                                    let mut sv = is_saved.clone();
                                    move |_| {
                                        let action = cp.read().clone();
                                        cp.set(None);
                                        match action {
                                            Some(CloseAction::CloseProject) => {
                                                proj.set(None);
                                                tabs.set(Vec::new());
                                                atab.set(None);
                                                cont.set(String::new());
                                                sv.set(true);
                                                fs::settings::save_last_project_path(None);
                                            }
                                            Some(CloseAction::CloseWindow) => {
                                                cd();
                                            }
                                            None => {}
                                        }
                                    }
                                },
                                "保存せずに閉じる"
                            }
                            button {
                                class: "dialog-btn primary",
                                onclick: {
                                    let cd = close_desktop.clone();
                                    let mut cp = close_pending.clone();
                                    let mut proj = project.clone();
                                    let mut tabs = open_tabs.clone();
                                    let mut atab = active_tab.clone();
                                    let mut cont = content.clone();
                                    let mut sv = is_saved.clone();
                                    let tc = tab_content.clone();
                                    let at = active_tab.clone();
                                    let mut nf = save_notification.clone();
                                    move |_| {
                                        let action = cp.read().clone();
                                        let doc = at.read().clone();
                                        let mut p = proj.write();
                                        if let (Some(d), Some(ref mut proj2)) = (doc, p.as_mut()) {
                                            if let Some(text) = tc.read().get(&d).cloned() {
                                                let _ = save_doc_content(proj2, &d, &text);
                                                *nf.write() = Some("保存しました".to_string());
                                            }
                                        }
                                        drop(p);
                                        sv.set(true);
                                        cp.set(None);
                                        match action {
                                            Some(CloseAction::CloseProject) => {
                                                proj.set(None);
                                                tabs.set(Vec::new());
                                                atab.set(None);
                                                cont.set(String::new());
                                                fs::settings::save_last_project_path(None);
                                            }
                                            Some(CloseAction::CloseWindow) => {
                                                cd();
                                            }
                                            None => {}
                                        }
                                    }
                                },
                                "保存して閉じる"
                            }
                        }
                    }
                }
            }
            if let Some(msg) = save_notification.read().as_ref() {
                div { class: "notification", "{msg}" }
            }
            {resize_grip}
        }
    }
}
