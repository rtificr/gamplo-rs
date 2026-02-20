pub fn get_error(val: &serde_json::Value) -> Option<String> {
    if let Some(error) = val.get("error") {
        if let Some(error_str) = error.as_str() {
            return Some(error_str.to_string());
        }
    }
    None
}