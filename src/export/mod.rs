use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use crate::model::project::Project;

#[derive(Debug, Clone)]
pub enum ExportFormat {
    ProjectZip,
    ManuscriptZipTxt,
    ManuscriptZipHtml,
    SiteZip,
}

pub fn export_project_zip(project: &Project, output_path: &Path) -> Result<(), String> {
    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let root = &project.root_dir;
    
    // Add chronicle.json
    let json_path = project.project_file_path();
    if json_path.exists() {
        zip.start_file("chronicle.json", options).map_err(|e| e.to_string())?;
        let content = fs::read(&json_path).map_err(|e| e.to_string())?;
        zip.write_all(&content).map_err(|e| e.to_string())?;
    }

    // Recursive helper to add directories
    fn add_dir_to_zip(
        zip: &mut zip::ZipWriter<fs::File>,
        root: &Path,
        current_dir: &Path,
        options: SimpleFileOptions,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = path.strip_prefix(root).map_err(|e| e.to_string())?;
            let name_str = name.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                zip.add_directory(&name_str, options).map_err(|e| e.to_string())?;
                add_dir_to_zip(zip, root, &path, options)?;
            } else {
                if name_str == "chronicle.json" { continue; } // Already added
                // Skip the output file itself if it happens to be in the project dir
                // (though rfd usually saves elsewhere, it's safer to check)
                // We can't easily check the output file handle here without passing it,
                // but usually output_path is what we want to avoid.
                
                zip.start_file(&name_str, options).map_err(|e| e.to_string())?;
                let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                zip.write_all(&buffer).map_err(|e| e.to_string())?;
            }
        }
    Ok(())
}

    add_dir_to_zip(&mut zip, root, root, options)?;
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn site_css() -> &'static str {
    r##":root {
  --bg-app: #f5f0eb;
  --bg-toolbar: #e8e0d8;
  --bg-sidebar: #f0ece6;
  --bg-editor: #faf8f5;
  --bg-paper: #faf8f5;
  --bg-btn: #f5f0eb;
  --bg-hover: #e0d8d0;
  --bg-active: #5a4a3a;
  --border: #d0c8c0;
  --text-primary: #2c2c2c;
  --text-secondary: #5a4a3a;
  --text-muted: #8a7a6a;
  --text-active: #f5f0eb;
  --notif-bg: #3a2a1a;
  --notif-text: #faf8f5;
}
* { box-sizing:border-box; margin:0; padding:0 }
body {
  background:var(--bg-app);
  color:var(--text-primary);
  font-family:"Noto Serif JP","Yu Mincho","游明朝",serif;
  line-height:2;
  min-height:100vh;
  display:flex;
  flex-direction:column;
}
/* ── Nav bar ── */
.nav-bar {
  background:var(--bg-toolbar);
  border-bottom:1px solid var(--border);
  padding:12px 24px;
  display:flex;
  align-items:center;
  justify-content:space-between;
  flex-wrap:wrap;
  gap:8px;
  position:sticky;top:0;z-index:10;
}
.nav-bar .site-title {
  font-size:16px;font-weight:700;
  color:var(--text-secondary);
  text-decoration:none;
}
.nav-bar .nav-links a {
  color:var(--text-muted);
  text-decoration:none;
  font-size:14px;
  margin-left:16px;
}
.nav-bar .nav-links a:hover { color:var(--text-primary) }
/* ── Reading area ── */
.reading {
  flex:1;
  background:var(--bg-paper);
  max-width:720px;
  margin:0 auto;
  padding:48px 32px 64px;
  width:100%;
  text-transform:full-width;
}
.reading code,.reading pre { text-transform:none }
.reading h1 {
  font-size:28px;
  color:var(--text-secondary);
  margin-bottom:32px;
  padding-bottom:12px;
  border-bottom:2px solid var(--border);
  font-weight:700;
  line-height:1.4;
}
.reading h2 {
  font-size:22px;
  color:var(--text-secondary);
  margin:28px 0 14px;
  font-weight:600;
}
.reading h3 {
  font-size:18px;
  color:var(--text-secondary);
  margin:24px 0 12px;
  font-weight:600;
}
.reading p {
  margin-bottom:14px;
  text-indent:1em;
  line-height:2;
  font-size:16px;
}
.reading blockquote {
  border-left:3px solid var(--border);
  padding-left:20px;
  color:var(--text-muted);
  margin:16px 0;
  font-style:italic;
}
.reading ul,.reading ol {
  margin:12px 0;
  padding-left:24px;
}
.reading li { margin-bottom:6px }
.reading hr {
  border:none;
  border-top:1px solid var(--border);
  margin:24px 0;
}
.reading code {
  background:var(--bg-toolbar);
  padding:2px 6px;
  border-radius:3px;
  font-size:14px;
  font-family:"Cascadia Code","Fira Code",monospace;
}
.reading pre {
  background:var(--bg-toolbar);
  padding:16px;
  border-radius:6px;
  overflow-x:auto;
  margin:16px 0;
}
.reading strong { font-weight:700 }
.reading a { color:#6a8aba;text-decoration:none }
.reading a:hover { text-decoration:underline }
.reading table { border-collapse:collapse;margin:16px 0;width:100% }
.reading th,.reading td {
  border:1px solid var(--border);
  padding:8px 12px;
  text-align:left;
}
.reading th { background:var(--bg-toolbar);font-weight:600 }
/* ── Index page ── */
.index-header {
  text-align:center;
  padding:48px 24px 32px;
  border-bottom:1px solid var(--border);
  margin-bottom:32px;
}
.index-header h1 {
  font-size:36px;
  color:var(--text-secondary);
  margin-bottom:8px;
}
.index-header p {
  color:var(--text-muted);
  font-size:14px;
}
.chapter-block {
  margin-bottom:28px;
}
.chapter-block h2 {
  font-size:20px;
  color:var(--text-secondary);
  padding-bottom:8px;
  border-bottom:1px solid var(--border-light,var(--border));
  margin-bottom:12px;
}
.tale-list { list-style:none;padding:0 }
.tale-list li {
  padding:6px 0;
}
.tale-list a {
  color:var(--text-primary);
  text-decoration:none;
  font-size:16px;
  display:block;
  padding:6px 8px;
  border-radius:4px;
  transition:background 0.15s;
}
.tale-list a:hover { background:var(--bg-hover) }
/* ── Tale navigation ── */
.tale-nav {
  margin-top:48px;
  padding-top:24px;
  border-top:1px solid var(--border);
  display:flex;
  justify-content:space-between;
  align-items:center;
  font-size:15px;
}
.tale-nav a {
  color:var(--text-secondary);
  text-decoration:none;
  padding:8px 16px;
  border-radius:4px;
  background:var(--bg-btn);
  border:1px solid var(--border);
  transition:background 0.15s;
}
.tale-nav a:hover { background:var(--bg-hover) }
.tale-nav .nav-prev { margin-right:auto }
.tale-nav .nav-next { margin-left:auto }
.tale-nav .nav-up {
  color:var(--text-muted);
  font-size:13px;
}
/* ── Footer ── */
.site-footer {
  text-align:center;
  padding:24px;
  color:var(--text-muted);
  font-size:12px;
  border-top:1px solid var(--border);
  background:var(--bg-toolbar);
}
/* ── Responsive ── */
@media (max-width:640px) {
  .nav-bar { padding:10px 16px }
  .nav-bar .nav-links a { margin-left:10px;font-size:13px }
  .reading { padding:24px 16px 48px }
  .reading h1 { font-size:22px;margin-bottom:20px }
  .reading p { font-size:15px }
  .index-header { padding:32px 16px 24px }
  .index-header h1 { font-size:28px }
  .chapter-block h2 { font-size:18px }
  .tale-list a { font-size:15px;padding:8px }
  .tale-nav { flex-wrap:wrap;gap:8px;justify-content:center }
  .tale-nav a { font-size:14px;padding:6px 12px;flex:1;text-align:center }
  .tale-nav .nav-up { width:100%;text-align:center;order:-1 }
}
"##
}

fn build_nav_html(chapters: &[crate::model::project::ChapterEntry], current_ch: &str, current_tale: &str) -> String {
    let mut prev: Option<(String, String)> = None;
    let mut next: Option<(String, String)> = None;
    let mut found = false;
    for ch in chapters {
        for tale in &ch.tales {
            let fname = format!("{}-{}.html", ch.dir_name, tale.file_name.replace(".md", ""));
            if found {
                next = Some((fname, tale.title.clone()));
                break;
            }
            if ch.dir_name == current_ch && tale.file_name == current_tale {
                found = true;
                continue;
            }
            if !found {
                prev = Some((fname, tale.title.clone()));
            }
        }
        if next.is_some() { break; }
    }
    let has_prev = prev.is_some();
    let has_next = next.is_some();
    let prev_html = match prev.as_ref() {
        Some((href, title)) => format!("<a class=\"nav-prev\" href=\"{}\">← {}</a>", href, title),
        None => "<span></span>".to_string(),
    };
    let next_html = match next.as_ref() {
        Some((href, title)) => format!("<a class=\"nav-next\" href=\"{}\">{} →</a>", href, title),
        None => "<span></span>".to_string(),
    };
    format!("{}{}<a class=\"nav-up\" href=\"index.html\">目次に戻る</a>{}", prev_html, if has_prev && has_next { "" } else { "" }, next_html)
}

pub fn export_site_zip(project: &Project, output_path: &Path) -> Result<(), String> {
    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let css = site_css();

    // Build tale pages
    let mut toc = String::new();
    let mut all_entries: Vec<(String, String, String)> = Vec::new(); // (filename, ch_dir, tale_file)
    for ch in &project.chapters {
        let ch_block = format!(
            "<div class=\"chapter-block\"><h2>{}</h2><ul class=\"tale-list\">\n",
            ch.title
        );
        toc.push_str(&ch_block);
        for tale in &ch.tales {
            let filename = format!("{}-{}.html", ch.dir_name, tale.file_name.replace(".md", ""));
            let tale_content = crate::fs::chapter::load_tale(project, &ch.dir_name, &tale.file_name)?;
            let html_body = crate::markdown::renderer::render_to_html(&tale_content);
            let nav = build_nav_html(&project.chapters, &ch.dir_name, &tale.file_name);
            let page_html = format!(
                "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\"><title>{} — {}</title><style>{}</style></head><body>\
                <nav class=\"nav-bar\"><a class=\"site-title\" href=\"index.html\">{}</a><div class=\"nav-links\"><a href=\"index.html\">目次</a></div></nav>\
                <article class=\"reading\"><h1>{}</h1>{}<div class=\"tale-nav\">{}</div></article>\
                <footer class=\"site-footer\">Generated by Chronicle</footer></body></html>",
                tale.title, project.name, css, project.name, tale.title, html_body, nav
            );
            all_entries.push((filename.clone(), ch.dir_name.clone(), tale.file_name.clone()));
            zip.start_file(&filename, options).map_err(|e| e.to_string())?;
            zip.write_all(page_html.as_bytes()).map_err(|e| e.to_string())?;
            toc.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", filename, tale.title));
        }
        toc.push_str("</ul></div>\n");
    }

    let author = &project.settings.author;
    let author_html = if author.is_empty() { "".to_string() } else { format!("<p>{}</p>", author) };
    // Build index page
    let index_html = format!(
        "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1.0\"><title>{}</title><style>{}</style></head><body>\
        <nav class=\"nav-bar\"><a class=\"site-title\" href=\"index.html\">{}</a></nav>\
        <main class=\"reading\"><div class=\"index-header\"><h1>{}</h1>{}</div>\
        {}</main><footer class=\"site-footer\">Generated by Chronicle</footer></body></html>",
        project.name, css, project.name, project.name, author_html, toc
    );

    zip.start_file("index.html", options).map_err(|e| e.to_string())?;
    zip.write_all(index_html.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map(|_| ()).map_err(|e| e.to_string())?;
    Ok(())
}





pub fn export_manuscript_zip(
    project: &Project,
    format: ExportFormat,
    output_path: &Path,
) -> Result<(), String> {
    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for ch in &project.chapters {
        let ch_dir = &ch.dir_name;
        zip.add_directory(ch_dir, options).map_err(|e| e.to_string())?;
        
        for tale in &ch.tales {
            let tale_content = crate::fs::chapter::load_tale(project, &ch.dir_name, &tale.file_name)?;
            let (file_name, content) = match format {
                ExportFormat::ManuscriptZipTxt => {
                    (tale.file_name.replace(".md", ".txt"), tale_content)
                }
                ExportFormat::ManuscriptZipHtml => {
                    let html_body = crate::markdown::renderer::render_to_html(&tale_content);
                    let html = format!(
                        "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{max-width:800px;margin:2em auto;padding:0 1em;line-height:1.6;font-family:sans-serif;}}</style></head><body><h1>{}</h1>{}</body></html>",
                        tale.title, tale.title, html_body
                    );
                    (tale.file_name.replace(".md", ".html"), html)
                }
                _ => unreachable!(),
            };
            
            let path_in_zip = format!("{}/{}", ch_dir, file_name);
            zip.start_file(path_in_zip, options).map_err(|e| e.to_string())?;
            zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}


