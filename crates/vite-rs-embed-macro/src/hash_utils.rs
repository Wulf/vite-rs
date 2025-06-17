use sha2::{Digest, Sha256};

pub fn get_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:X}", hash)
}
