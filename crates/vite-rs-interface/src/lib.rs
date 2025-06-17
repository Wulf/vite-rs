// Production File
#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
#[derive(Debug, Clone)]
/// File retrieved from a ViteJS-compiled project
pub struct ViteFile {
    pub bytes: ::std::borrow::Cow<'static, [u8]>,
    pub last_modified: Option<&'static str>,
    pub content_type: &'static str,
    pub content_length: u64,
    #[cfg(feature = "content-hash")]
    /// SHA-256 hash of the file contents.
    pub content_hash: &'static str,
}

// Production Struct Trait
/// Note: this is used to allow dynamic usage of embedded asset structs.
#[cfg(any(not(debug_assertions), feature = "debug-prod"))]
pub trait GetFromVite: Send + Sync + 'static {
    fn get(&self, file_path: &str) -> Option<ViteFile>;
    fn clone_box(&self) -> Box<dyn GetFromVite>;
}

// Development File
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[derive(Debug, Clone)]
/// File retreived from the ViteJS dev server
pub struct ViteFile {
    pub bytes: Vec<u8>,
    pub last_modified: Option<String>,
    pub content_type: String,
    pub content_length: u64,
    #[cfg(feature = "content-hash")]
    /// Note: in development mode, this is a weak hash returned by the ViteJS dev server.
    pub content_hash: String,
}

// Development Struct Trait
/// Note: this is used to allow dynamic usage of embedded asset structs.
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub trait GetFromVite: Send + Sync + 'static {
    fn get(&self, file_path: &str) -> Option<ViteFile>;
    fn clone_box(&self) -> Box<dyn GetFromVite>;
}
