pub const VERSION: &str = match option_env!("CHRONICLE_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
