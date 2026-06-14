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
}
