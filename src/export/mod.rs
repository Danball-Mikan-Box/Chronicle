use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use crate::model::project::Project;

#[derive(Debug, Clone)]
pub enum ExportFormat {
    ProjectZip,
    ManuscriptSingleTxt,
    ManuscriptSingleHtml,
    ManuscriptZipTxt,
    ManuscriptZipHtml,
    CurrentFileTxt,
    CurrentFileHtml,
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
                "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title></head><body><h1>{}</h1>{}</body></html>",
                tale.title, tale.title, html_body);
            pages.push((filename.clone(), page_html));
            toc.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", filename, tale.title));
        }
        toc.push_str("</ul>\n");
    }
    
    let index_html = format!(
        "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title></head><body>{}</body></html>",
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



pub fn export_manuscript_single(
    project: &Project,
    format: ExportFormat,
    output_path: &Path,
) -> Result<(), String> {
    let mut content = String::new();
    
    for (i, ch) in project.chapters.iter().enumerate() {
        if i > 0 { content.push_str("\n\n---\n\n"); }
        content.push_str(&format!("# {}\n\n", ch.title));
        
        for (j, tale) in ch.tales.iter().enumerate() {
            if j > 0 { content.push_str("\n\n"); }
            let tale_content = crate::fs::chapter::load_tale(project, &ch.dir_name, &tale.file_name)?;
            content.push_str(&format!("## {}\n\n", tale.title));
            content.push_str(&tale_content);
        }
    }

    match format {
        ExportFormat::ManuscriptSingleTxt => {
            fs::write(output_path, content).map_err(|e| e.to_string())?;
        }
        ExportFormat::ManuscriptSingleHtml => {
            let html_body = crate::markdown::renderer::render_to_html(&content);
            let html = format!(
                "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{max-width:800px;margin:2em auto;padding:0 1em;line-height:1.6;font-family:sans-serif;}}</style></head><body>{}</body></html>",
                project.name, html_body
            );
            fs::write(output_path, html).map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid format for single file export".to_string()),
    }
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
                _ => continue,
            };
            
            let path_in_zip = format!("{}/{}", ch_dir, file_name);
            zip.start_file(path_in_zip, options).map_err(|e| e.to_string())?;
            zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn export_current_file(
    _project: &Project,
    format: ExportFormat,
    doc_ref: &crate::model::DocRef,
    content: &str,
    output_path: &Path,
) -> Result<(), String> {
    match format {
        ExportFormat::CurrentFileTxt => {
            fs::write(output_path, content).map_err(|e| e.to_string())?;
        }
        ExportFormat::CurrentFileHtml => {
            let title = doc_ref.tab_label();
            let html_body = crate::markdown::renderer::render_to_html(content);
            let html = format!(
                "<!DOCTYPE html><html lang=\"ja\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{max-width:800px;margin:2em auto;padding:0 1em;line-height:1.6;font-family:sans-serif;}}</style></head><body><h1>{}</h1>{}</body></html>",
                title, title, html_body
            );
            fs::write(output_path, html).map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid format for current file export".to_string()),
    }
    Ok(())
}
