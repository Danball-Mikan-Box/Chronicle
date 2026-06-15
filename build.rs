fn main() {
    let tag = std::env::var("GIT_TAG").ok();
    let version = tag
        .as_ref()
        .and_then(|t| t.strip_prefix('v'))
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=CHRONICLE_VERSION={}", version);
    if let Some(ref t) = tag {
        println!("cargo:rustc-env=CHRONICLE_BUILD_TAG={}", t);
    } else {
        println!("cargo:rustc-env=CHRONICLE_BUILD_TAG=dev");
    }
    println!("cargo:rerun-if-env-changed=GIT_TAG");

    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let png_path = manifest_dir.join("Chronicle.png");
    let ico_path = manifest_dir.join("Chronicle.ico");

    if png_path.exists()
        && (!ico_path.exists()
            || modified(&png_path) > modified(&ico_path))
    {
        let png_data = std::fs::read(&png_path).expect("Failed to read Chronicle.png");
        let ico = make_ico(&png_data);
        std::fs::write(&ico_path, &ico).expect("Failed to write Chronicle.ico");
        println!("cargo:rerun-if-changed=Chronicle.png");
    }

    #[cfg(target_os = "windows")]
    {
        let rc_path = manifest_dir.join("icon.rc");
        if !rc_path.exists() {
            std::fs::write(&rc_path, r#"1 ICON "Chronicle.ico""#)
                .expect("Failed to write icon.rc");
        }
        embed_resource::compile("icon.rc", std::iter::empty::<&str>());
    }
}

fn modified(path: &std::path::Path) -> std::time::SystemTime {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

fn make_ico(png_data: &[u8]) -> Vec<u8> {
    let w = u32::from_be_bytes([png_data[16], png_data[17], png_data[18], png_data[19]]);
    let h = u32::from_be_bytes([png_data[20], png_data[21], png_data[22], png_data[23]]);

    let w_byte = if w >= 256 { 0 } else { w as u8 };
    let h_byte = if h >= 256 { 0 } else { h as u8 };

    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.push(w_byte);
    ico.push(h_byte);
    ico.push(0);
    ico.push(0);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    let data_size = png_data.len() as u32;
    let data_offset: u32 = 6 + 16;
    ico.extend_from_slice(&data_size.to_le_bytes());
    ico.extend_from_slice(&data_offset.to_le_bytes());
    ico.extend_from_slice(png_data);
    ico
}
