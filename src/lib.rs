//! # decruft
//!
//! Extract clean, readable content from web pages.
//!
//! Given a noisy HTML page (ads, navigation, sidebars, popups, cookie banners...),
//! decruft extracts the main content and metadata.
//!
//! ## Quick start
//!
//! ```
//! use decruft::{parse, DecruftOptions};
//!
//! let html = r#"<html>
//!   <head><title>My Post - Blog</title></head>
//!   <body>
//!     <nav><a href="/">Home</a></nav>
//!     <article><h1>My Post</h1><p>The content.</p></article>
//!     <footer>Copyright 2025</footer>
//!   </body>
//! </html>"#;
//!
//! let result = parse(html, &DecruftOptions::default());
//! assert!(result.content.contains("The content."));
//! assert!(!result.content.contains("Copyright"));
//! ```

pub mod callouts;
pub mod cleanup;
pub mod code_blocks;
pub mod content;
mod decruft;
pub mod dom;
pub mod extractors;
pub mod footnotes;
pub mod math;
pub mod metadata;
pub mod noscript;
pub mod patterns;
pub mod schema_org;
pub mod scorer;
pub mod selectors;
pub mod standardize;
pub mod types;

pub use decruft::parse;
pub use types::{DecruftOptions, DecruftResult};
