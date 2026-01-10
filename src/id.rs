use sha1::{Digest, Sha1};
use std::collections::HashSet;

pub fn generate_id(
    title: &str,
    description: &str,
    editor: &str,
    existing_ids: &HashSet<String>,
) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let mut length = 4;

    loop {
        let input = format!("{}{}{}{}{}", title, description, editor, timestamp, length);
        let hash = Sha1::digest(input.as_bytes());
        let hex = hex::encode(hash);
        let id = format!("st-{}", &hex[..length]);

        if !existing_ids.contains(&id) {
            return id;
        }
        length += 1;
        if length > 8 {
            panic!("ID collision after 8 chars - should not happen");
        }
    }
}
