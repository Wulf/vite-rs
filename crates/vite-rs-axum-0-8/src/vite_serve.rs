use axum::body::Body;
use axum::response::Response;
use vite_rs_interface::GetFromVite;

pub struct ViteServe {
    pub cache_strategy: CacheStrategy,
    pub assets: Box<dyn GetFromVite>,
}

impl Clone for ViteServe {
    fn clone(&self) -> Self {
        Self {
            cache_strategy: self.cache_strategy.clone(),
            assets: self.assets.clone_box(),
        }
    }
}

/// Caching strategies specify how the server sets the Control-Cache header.
/// In development, we always send 'no-cache' to ensure the latest files are served.
#[derive(Clone)]
pub enum CacheStrategy {
    /// Always up-to-date. Checks for new updates before serving files.
    /// Clients will always receive the latest version of served assets.
    /// (default in release builds)
    Eager,
    /// Faster initial render. Checks for new updates after cached files are served.
    /// Clients may be on older versions of served assets until the next request.
    Lazy,
    /// No caching. Always serves the latest files without any cache headers.
    /// Not recommended, use `Eager` instead.
    /// (default in debug builds)
    None,
    /// Custom caching strategy. Allows you to set your own Control-Cache header.
    Custom(&'static str),
}

impl ViteServe {
    pub fn new(assets: Box<dyn GetFromVite>) -> Self {
        Self {
            #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
            cache_strategy: CacheStrategy::None,
            #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
            cache_strategy: CacheStrategy::Eager,
            assets,
        }
    }

    pub fn with_cache_strategy(mut self, cache_strategy: CacheStrategy) -> Self {
        self.cache_strategy = cache_strategy;
        self
    }

    pub async fn serve<B>(&self, req: axum::http::request::Request<B>) -> Response
    where
        B: axum::body::HttpBody<Data = axum::body::Bytes> + Send + 'static,
    {
        // Extract the path from the request, removing the leading slash
        let path = req.uri().path().trim_start_matches('/');
        let query = req
            .uri()
            .query()
            .map(|q| format!("?{}", q))
            .unwrap_or_default();

        // If the path is empty, default to index.html
        let request_file_path = if path.is_empty() { "index.html" } else { path };

        match self.assets.get(&format!("{}{}", request_file_path, query)) {
            Some(file) => {
                let mut response = Response::builder();

                response = response.header("Content-Type", file.content_type);
                response = response.header("Content-Length", file.content_length);

                let etag = {
                    #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
                    {
                        file.content_hash
                    }

                    #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
                    {
                        &file.content_hash
                    }
                };

                response = response.status(200).header("ETag", etag);

                match self.cache_strategy {
                    CacheStrategy::Eager => {
                        response = response.header("Cache-Control", "max-age=0, must-revalidate");
                    }
                    CacheStrategy::Lazy => {
                        response = response
                            .header("Cache-Control", "max-age=0, stale-while-revalidate=604800");
                    }
                    CacheStrategy::None => {
                        response = response.header("Cache-Control", "no-cache");
                    }
                    CacheStrategy::Custom(header) => {
                        response = response.header("Cache-Control", header);
                    }
                };

                if let Some(last_modified) = file.last_modified {
                    response = response.header("Last-Modified", last_modified);
                }

                match req.headers().get(axum::http::header::IF_NONE_MATCH) {
                    Some(header) => {
                        let header_etag = header.to_str().expect(
                            "Could not read IF_NONE_MATCH header, it contained invalid characters.",
                        );

                        if etag.eq(header_etag) {
                            // If the ETag matches, return 304 Not Modified
                            #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
                            return response.status(304).body(Body::from(vec![])).unwrap();

                            #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
                            return response.status(304).body(Body::from(&[][..])).unwrap();
                        } else {
                            // If it doesn't match, return the full response
                            return response.body(Body::from(file.bytes)).unwrap();
                        }
                    }
                    None => {
                        // If no IF_NONE_MATCH header, return the full response
                        return response.body(Body::from(file.bytes)).unwrap();
                    }
                }
            }
            None => {
                // Return 404 Not Found with an empty body
                #[cfg(all(debug_assertions, not(feature = "debug-prod")))]
                return Response::builder()
                    .status(404)
                    .body(Body::from(vec![]))
                    .unwrap();

                #[cfg(any(not(debug_assertions), feature = "debug-prod"))]
                return Response::builder()
                    .status(404)
                    .body(Body::from(&[][..]))
                    .unwrap();
            }
        }
    }
}
