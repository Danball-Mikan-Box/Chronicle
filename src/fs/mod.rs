pub mod project;
pub mod chapter;
pub mod material;
pub mod settings;

#[cfg(target_os = "android")]
pub fn android_storage_dir() -> std::path::PathBuf {
    // Prefer explicitly set directory via environment variable.
    if let Ok(data_dir) = std::env::var("CHRONICLE_DATA_DIR") {
        eprintln!("[chronicle] android_storage_dir: using CHRONICLE_DATA_DIR={}", data_dir);
        return std::path::PathBuf::from(data_dir);
    }

    // Try to obtain the app's internal files directory via the Android NDK context.
    if let Some(path) = get_android_files_dir() {
        eprintln!("[chronicle] android_storage_dir: JNI getFilesDir={}", path.display());
        return path;
    }
    eprintln!("[chronicle] android_storage_dir: JNI getFilesDir failed, trying fallbacks");

    // Fallback: create a writable directory in the app's home if available.
    if let Ok(home) = std::env::var("HOME") {
        let p = std::path::PathBuf::from(home).join("files");
        eprintln!("[chronicle] android_storage_dir: HOME fallback={}", p.display());
        return p;
    }

    // As a last resort, use the legacy absolute path.
    let p = std::path::PathBuf::from("/data/data/com.chronicle.app/files");
    eprintln!("[chronicle] android_storage_dir: legacy fallback={}", p.display());
    p
}

#[cfg(target_os = "android")]
fn get_android_files_dir() -> Option<std::path::PathBuf> {
    let ctx = ndk_context::android_context();
    eprintln!("[chronicle] get_android_files_dir: ctx.vm={:p}, ctx.context={:p}", ctx.vm(), ctx.context());
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let files_dir = env.call_method(&context, "getFilesDir", "()Ljava/io/File;", &[]).ok()?.l().ok()?;
    let path_obj = env.call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[]).ok()?.l().ok()?;
    let path_jstr = jni::objects::JString::from(path_obj);
    let path_str: String = env.get_string(&path_jstr).ok()?.into();
    eprintln!("[chronicle] get_android_files_dir: success={}", path_str);
    Some(std::path::PathBuf::from(path_str))
}

/// Returns the exports directory on Android – uses external (user-accessible)
/// storage via `context.getExternalFilesDir(null)`, falling back to internal.
#[cfg(target_os = "android")]
pub fn android_export_dir() -> std::path::PathBuf {
    if let Some(path) = get_android_external_files_dir() {
        let d = path.join("exports");
        eprintln!("[chronicle] android_export_dir: external={}", d.display());
        let _ = std::fs::create_dir_all(&d);
        return d;
    }
    eprintln!("[chronicle] android_export_dir: external failed, using internal fallback");
    let d = android_storage_dir().join("exports");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Returns the public Downloads/Chronicle/ directory via JNI.
/// Falls back to `android_export_dir()` if the public directory is not writable.
#[cfg(target_os = "android")]
pub fn android_downloads_export_dir() -> std::path::PathBuf {
    let ctx = ndk_context::android_context();
    if let Ok(vm) = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) } {
        if let Ok(mut env) = vm.attach_current_thread() {
            if let Ok(env_class) = env.find_class("android/os/Environment") {
                if let Ok(downloads_field) = env.get_static_field(
                    &env_class,
                    "DIRECTORY_DOWNLOADS",
                    "Ljava/lang/String;",
                ) {
                    if let Ok(downloads_jobj) = downloads_field.l() {
                        let downloads_jstr: jni::objects::JString = downloads_jobj.into();
                        if let Ok(dir_name) = env.get_string(&downloads_jstr) {
                            let dir_name: String = dir_name.into();
                            if let Ok(downloads_file) = env.call_static_method(
                                &env_class,
                                "getExternalStoragePublicDirectory",
                                "(Ljava/lang/String;)Ljava/io/File;",
                                &[jni::objects::JValue::Object(&downloads_jobj)],
                            ) {
                                if let Ok(file_obj) = downloads_file.l() {
                                    if let Ok(path_obj) = env.call_method(
                                        &file_obj,
                                        "getAbsolutePath",
                                        "()Ljava/lang/String;",
                                        &[],
                                    ) {
                                        if let Ok(path_jobj) = path_obj.l() {
                                            let path_jstr: jni::objects::JString = path_jobj.into();
                                            if let Ok(path_str) = env.get_string(&path_jstr) {
                                                let path_str: String = path_str.into();
                                                let chronicle_dir = std::path::PathBuf::from(&path_str).join("Chronicle");
                                                if std::fs::create_dir_all(&chronicle_dir).is_ok() {
                                                    eprintln!("[chronicle] android_downloads_export_dir: {}", chronicle_dir.display());
                                                    return chronicle_dir;
                                                }
                                                eprintln!("[chronicle] android_downloads_export_dir: cannot create dir, falling back");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("[chronicle] android_downloads_export_dir: JNI failed, using android_export_dir fallback");
    android_export_dir()
}

#[cfg(target_os = "android")]
fn get_android_external_files_dir() -> Option<std::path::PathBuf> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let null_obj = jni::objects::JObject::null();
    let files_dir = env
        .call_method(
            &context,
            "getExternalFilesDir",
            "(Ljava/lang/String;)Ljava/io/File;",
            &[jni::objects::JValue::Object(&null_obj)],
        )
        .ok()?
        .l()
        .ok()?;
    let path_obj = env
        .call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    let path_jstr = jni::objects::JString::from(path_obj);
    let path_str: String = env.get_string(&path_jstr).ok()?.into();
    eprintln!("[chronicle] get_android_external_files_dir: success={}", path_str);
    Some(std::path::PathBuf::from(path_str))
}
