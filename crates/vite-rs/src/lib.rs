#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
#[cfg(feature = "ctrlc")]
#[cfg(not(doctest))] // for some reason, the cfgs above don't apply to doc tests
pub use vite_rs_dev_server::ctrlc;
#[cfg(all(debug_assertions, not(feature = "debug-prod")))]
pub use vite_rs_dev_server::{self, ViteProcess};
pub use vite_rs_embed_macro::Embed;

pub use vite_rs_interface::*;
