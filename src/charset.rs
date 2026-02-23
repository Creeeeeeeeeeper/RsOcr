use std::sync::LazyLock;

static CHARSET_JSON: &str = include_str!("../charset3.json");

pub static CHARSET: LazyLock<Vec<String>> = LazyLock::new(|| {
    serde_json::from_str(CHARSET_JSON).expect("Failed to parse charset.json")
});

/// 从指定路径加载自定义字符集
pub fn load_charset_from_file(path: &str) -> Vec<String> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read charset file {}: {}", path, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse charset file {}: {}", path, e))
}
