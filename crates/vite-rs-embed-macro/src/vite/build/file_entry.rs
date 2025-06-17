use std::time::SystemTime;

pub struct FileEntry {
    /// The string used to lookup this file.
    /// It's either the original file name or the compiled file name.
    key: String,

    /// Path to the file on the filesystem.
    absolute_file_path: String,

    /// The last modified timestamp of the file. Useful for caching.
    last_modified: Option<String>,

    /// The MIME type of the file. Useful for serving the file.
    content_type: String,

    /// The length of the file in bytes. Useful for serving the file.
    content_length: u64,
}

impl FileEntry {
    pub fn new(key: String, absolute_file_path: String) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&absolute_file_path)?;
        let last_modified = metadata.modified().ok().map(|last_modified| {
            let last_modified_secs = last_modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time before the UNIX epoch is unsupported")
                .as_secs();

            // Unix timestamps conversion from u64 to i64 won't overflow for several billion years
            let last_modified_secs = last_modified_secs as i64;
            chrono::DateTime::<chrono::Utc>::from_timestamp(last_modified_secs, 0)
                .map(|dt| dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
                .expect("Failed to format last-modified date according to HTTP standards")
        });

        Ok(Self {
            key,
            last_modified: last_modified,
            content_type: mime_guess::from_path(&absolute_file_path)
                .first_or_octet_stream()
                .to_string(),
            content_length: metadata.len(),
            absolute_file_path,
        })
    }

    pub fn match_key(&self) -> &String {
        &self.key
    }

    pub fn match_value(&self, crate_path: &syn::Path) -> proc_macro2::TokenStream {
        self.code(crate_path)
    }

    fn code(&self, crate_path: &syn::Path) -> proc_macro2::TokenStream {
        use quote::quote;

        let absolute_file_path = &self.absolute_file_path;

        let last_modified = if let Some(last_modified) = &self.last_modified {
            quote! { ::std::option::Option::Some(#last_modified) }
        } else {
            quote! { ::std::option::Option::None }
        };

        let content_type = &self.content_type;
        let content_length = self.content_length;

        let content_hash = if cfg!(feature = "content-hash") {
            // We have to read the file here because it's currently not possible to use sha2 in const fns until https://github.com/RustCrypto/hashes/issues/288 is resolved.
            // And without a const fn, we cant generate a const HASH: &'static str = "..." for each FileEntry (which would be nice and in-line with the const BYTES array).
            // Once the above is resolved, we won't have to read the file here and in the include_bytes!.
            let bytes =
                std::fs::read(absolute_file_path).expect("Failed to read file to compute hash");
            let content_hash = crate::hash_utils::get_content_hash(&bytes);
            quote! { content_hash: #content_hash, }
        } else {
            quote! {}
        };

        quote! {
            {
                const BYTES: &'static [u8] = include_bytes!(#absolute_file_path);

                #crate_path::ViteFile {
                    bytes: ::std::borrow::Cow::Borrowed(&BYTES),
                    last_modified: #last_modified,
                    content_type: #content_type,
                    content_length: #content_length,
                    #content_hash
                }
            }
        }
    }
}
