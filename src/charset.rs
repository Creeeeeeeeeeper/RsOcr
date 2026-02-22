use std::sync::LazyLock;

static CHARSET_JSON: &str = include_str!("../charset.json");

pub static CHARSET: LazyLock<Vec<String>> = LazyLock::new(|| {
    serde_json::from_str(CHARSET_JSON).expect("Failed to parse charset.json")
});
