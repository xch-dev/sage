use base64::{Engine, prelude::BASE64_STANDARD};

pub fn base64_data_uri(blob: &[u8], mime_type: &str) -> String {
    format!("data:{mime_type};base64,{}", BASE64_STANDARD.encode(blob))
}
