use boa_engine::JsError;

// A helper to Format Boa Errors
pub fn format_js_error(err: JsError, action: &str) -> String {
    format!(
        "Action: {}\n{}",
        action,
        err.to_string()
    )
}
