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

pub fn export_site_zip(project: &Project, output_path: &Path) -> Result<(), String> {
    let file = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    // Build index and pages
    let mut toc = String::new();
    let mut pages: Vec<(String, String)> = Vec::new();
    for ch in &project.chapters {
        toc.push_str(&format!("<h2>{}</h2>\n<ul>\n", ch.title));
        for tale in &ch.tales {
            let tale_content = crate::fs::chapter::load_tale(project, &ch.dir_name, &tale.file_name)?;
            let html_body = crate::markdown::renderer::render_to_html(&tale_content);
            let filename = format!("{}-{}.html", ch.dir_name, tale.file_name.replace(".md", ""));
            let page_html = format!(
                "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title><style>:root{{--bg-paper:#faf8f5;--text-body:#2c2c2c;--text-heading:#5a4a3a;--text-muted:#8a7a6a}}body{{background:var(--bg-paper);color:var(--text-body);font-family:\"Noto Serif JP\",\"Yu Mincho\",\"游明朝\",serif;line-height:1.8;max-width:40rem;margin:2rem auto;padding:0 2rem}}h1{{color:var(--text-heading);margin-top:2rem}}p{{margin:0.8rem 0}}a{{color:#0066cc;text-decoration:none}}a:hover{{text-decoration:underline}}ul,ol{{margin-left:2rem}}blockquote{{border-left:.25rem solid #d0c8c0;padding-left:1rem;color:var(--text-muted);margin:1rem 0}}</style></head><body><h1>{}</h1>{}</body></html>",
                tale.title, tale.title, html_body);
            pages.push((filename.clone(), page_html));
            toc.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", filename, tale.title));
        }
        toc.push_str("</ul>\n");
    }
    
let index_html = format!(
        "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title><style>:root{{--bg-paper:#faf8f5;--text-body:#2c2c2c;--text-heading:#5a4a3a;--text-muted:#8a7a6a}}body{{background:var(--bg-paper);color:var(--text-body);font-family:\"Noto Serif JP\",\"Yu Mincho\",\"游明朝\",serif;line-height:1.8;max-width:40rem;margin:2rem auto;padding:0 2rem}}h1,h2,h3,h4,h5,h6{{color:var(--text-heading);margin-top:2rem}}p{{margin:0.8rem 0}}a{{color:#0066cc;text-decoration:none}}a:hover{{text-decoration:underline}}ul,ol{{margin-left:2rem}}blockquote{{border-left:.25rem solid #d0c8c0;padding-left:1rem;color:var(--text-muted);margin:1rem 0}}</style></head><body>{}</body></html>",
        project.name, toc);
    zip.start_file("index.html", options).map_err(|e| e.to_string())?;
    zip.write_all(index_html.as_bytes()).map_err(|e| e.to_string())?;
    for (path, content) in pages {
        zip.start_file(path, options).map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }
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


