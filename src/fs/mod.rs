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
    let vm = match unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) } {
        Ok(vm) => vm,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: JavaVM::from_raw failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: attach_current_thread failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let env_class = match env.find_class("android/os/Environment") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: find_class failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let downloads_field = match env.get_static_field(
        &env_class,
        "DIRECTORY_DOWNLOADS",
        "Ljava/lang/String;",
    ) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: get_static_field failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let downloads_jobj = match downloads_field.l() {
        Ok(obj) => obj,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: downloads_field.l() failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let downloads_file = match env.call_static_method(
        &env_class,
        "getExternalStoragePublicDirectory",
        "(Ljava/lang/String;)Ljava/io/File;",
        &[jni::objects::JValue::Object(&downloads_jobj)],
    ) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: getExternalStoragePublicDirectory failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let file_obj = match downloads_file.l() {
        Ok(obj) => obj,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: downloads_file.l() failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let path_obj = match env.call_method(
        &file_obj,
        "getAbsolutePath",
        "()Ljava/lang/String;",
        &[],
    ) {
        Ok(r) => r,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: getAbsolutePath failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let path_jobj = match path_obj.l() {
        Ok(obj) => obj,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: path_obj.l() failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let path_jstr: jni::objects::JString = path_jobj.into();
    let path_str = match env.get_string(&path_jstr) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("[chronicle] android_downloads_export_dir: get_string failed, using android_export_dir fallback");
            return android_export_dir();
        }
    };
    let path_str: String = path_str.into();
    let chronicle_dir = std::path::PathBuf::from(&path_str).join("Chronicle");
    if std::fs::create_dir_all(&chronicle_dir).is_ok() {
        eprintln!("[chronicle] android_downloads_export_dir: {}", chronicle_dir.display());
        chronicle_dir
    } else {
        eprintln!("[chronicle] android_downloads_export_dir: cannot create dir, using android_export_dir fallback");
        android_export_dir()
    }
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
