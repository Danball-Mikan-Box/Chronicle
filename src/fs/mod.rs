pub mod project;
pub mod chapter;
pub mod material;
pub mod settings;

#[cfg(target_os = "android")]
pub fn android_storage_dir() -> std::path::PathBuf {
    // Prefer explicitly set directory via environment variable.
    if let Ok(data_dir) = std::env::var("CHRONICLE_DATA_DIR") {
        return std::path::PathBuf::from(data_dir);
    }

    // Try to obtain the app's internal files directory via the Android NDK context.
    if let Some(path) = get_android_files_dir() {
        return path;
    }

    // Fallback: use HOME environment variable which points to the app's data dir on most Android runtimes.
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home).join("files");
    }

    // As a last resort, use the legacy absolute path (may be invalid on newer Android versions).
    std::path::PathBuf::from("/data/data/com.chronicle.app/files")
}

#[cfg(target_os = "android")]
fn get_android_files_dir() -> Option<std::path::PathBuf> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let files_dir = env.call_method(&context, "getFilesDir", "()Ljava/io/File;", &[]).ok()?.l().ok()?;
    let path_obj = env.call_method(&files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[]).ok()?.l().ok()?;
    let path_jstr = jni::objects::JString::from(path_obj);
    let path_str: String = env.get_string(&path_jstr).ok()?.into();
    Some(std::path::PathBuf::from(path_str))
}
