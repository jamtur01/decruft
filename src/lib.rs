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
//!
//! Or even simpler with [`parse_with_defaults`]:
//!
//! ```
//! let html = "<html><body><article><p>Hello</p></article></body></html>";
//! let result = decruft::parse_with_defaults(html);
//! assert!(result.content.contains("Hello"));
//! ```

pub(crate) mod callouts;
pub(crate) mod cleanup;
pub(crate) mod code_blocks;
pub(crate) mod content;
mod decruft;
pub(crate) mod dom;
pub(crate) mod extractors;
pub(crate) mod footnotes;
pub(crate) mod math;
pub(crate) mod metadata;
pub(crate) mod metadata_block;
pub(crate) mod noscript;
pub(crate) mod patterns;
pub(crate) mod schema_org;
pub(crate) mod scorer;
pub(crate) mod selectors;
pub(crate) mod standardize;
pub(crate) mod streaming_ssr;
pub(crate) mod types;

pub use decruft::parse;
pub use dom::strip_html_tags;
pub use types::{DebugInfo, DecruftOptions, DecruftResult, MetaTag, Removal};

/// Parse HTML with default options.
///
/// Equivalent to `parse(html, &DecruftOptions::default())`.
///
/// # Examples
///
/// ```
/// let html = "<html><body><article><p>Hello</p></article></body></html>";
/// let result = decruft::parse_with_defaults(html);
/// assert!(result.content.contains("Hello"));
/// ```
#[must_use]
pub fn parse_with_defaults(html: &str) -> DecruftResult {
    parse(html, &DecruftOptions::default())
}
