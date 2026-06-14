pub const VERSION: &str = match option_env!("CHRONICLE_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
pub const BUILD_TAG: &str = match option_env!("CHRONICLE_BUILD_TAG") {
    Some(v) => v,
    None => "dev",
};
