pub mod bbcode;
pub mod c2wiki;
pub mod comments;
pub mod github;
pub mod hackernews;
pub mod reddit;
pub mod substack;

use scraper::Html;

/// Result of a site-specific extractor.
pub struct ExtractorResult {
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub site: Option<String>,
}

/// Try each site-specific extractor in order.
///
/// Returns the first successful extraction, or `None` if no extractor
/// matched the page.
#[must_use]
pub fn try_extract(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    // Note: BBCode and Substack are handled separately in decruft.rs
    // because they interact with the metadata pipeline differently.

    if let Some(result) = github::extract_github(html, url) {
        return Some(result);
    }
    if let Some(result) = reddit::extract_reddit(html, url) {
        return Some(result);
    }
    if let Some(result) = hackernews::extract_hackernews(html, url) {
        return Some(result);
    }
    // C2 Wiki: async-only, always returns None from sync extraction.
    // YouTube, Twitter/X, ChatGPT, Claude, Gemini, Grok: skipped
    // because they require JS rendering or API calls.
    if let Some(result) = c2wiki::extract_c2wiki(html, url) {
        return Some(result);
    }
    None
}
