#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[cfg(feature = "ctrlc")]
#[cfg(not(doctest))] // for some reason, the cfgs above don't apply to doc tests
pub use vite_rs_dev_server::ctrlc;
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub use vite_rs_dev_server::{self, ViteProcess};
pub use vite_rs_embed_macro::Embed;

// Production
#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
#[derive(Debug, Clone)]
/// File retrieved from a ViteJS-compiled project
pub struct ViteFile {
    pub bytes: ::std::borrow::Cow<'static, [u8]>,
    pub last_modified: Option<u64>,
    pub content_type: &'static str,
    pub content_length: u64,
}

// Development
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[derive(Debug, Clone)]
/// File retreived from the ViteJS dev server
pub struct ViteFile {
    pub bytes: Vec<u8>,
    pub last_modified: Option<u64>,
    pub content_type: String,
    pub content_length: u64,
}
