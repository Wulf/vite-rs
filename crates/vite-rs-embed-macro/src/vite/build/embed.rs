use std::time::SystemTime;

pub struct FileEntry {
    /// The string used to lookup this file.
    /// It's either the original file name or the compiled file name.
    key: String,

    /// Path to the file on the filesystem.
    absolute_file_path: String,

    /// The last modified timestamp of the file. Useful for caching.
    last_modified: Option<u64>,

    /// The MIME type of the file. Useful for serving the file.
    content_type: String,

    /// The length of the file in bytes. Useful for serving the file.
    content_length: u64,
}

impl FileEntry {
    pub fn new(key: String, absolute_file_path: String) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&absolute_file_path)?;
        let last_modified = metadata.modified().ok().map(|last_modified| {
            last_modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time before the UNIX epoch is unsupported")
                .as_secs()
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

    pub fn match_value(&self) -> proc_macro2::TokenStream {
        self.code()
    }

    fn code(&self) -> proc_macro2::TokenStream {
        use quote::quote;

        let absolute_file_path = &self.absolute_file_path;

        let last_modified = if let Some(last_modified) = self.last_modified {
            quote! { ::std::option::Option::Some(#last_modified) }
        } else {
            quote! { ::std::option::Option::None }
        };

        let content_type = &self.content_type;

        let content_length = self.content_length;

        quote! {
            {
                const BYTES: &'static [u8] = include_bytes!(#absolute_file_path);

                vite_rs::ViteFile {
                    bytes: ::std::borrow::Cow::Borrowed(&BYTES),
                    last_modified: #last_modified,
                    content_type: #content_type,
                    content_length: #content_length,
                }
            }
        }
    }
}
