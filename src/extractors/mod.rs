pub mod bbcode;
pub mod c2wiki;
pub mod comments;
pub mod conversations;
pub mod github;
pub mod hackernews;
pub mod reddit;
pub mod substack;
pub mod twitter;

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
/// Returns the first successful extraction along with the extractor
/// type name, or `None` if no extractor matched the page.
#[must_use]
pub fn try_extract(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<(ExtractorResult, &'static str)> {
    // Note: BBCode and Substack are handled separately in decruft.rs
    // because they interact with the metadata pipeline differently.

    if let Some(result) = github::extract_github(html, url, include_replies) {
        return Some((result, "github"));
    }
    if let Some(result) = reddit::extract_reddit(html, url, include_replies) {
        return Some((result, "reddit"));
    }
    if let Some(result) = hackernews::extract_hackernews(html, url, include_replies) {
        return Some((result, "hackernews"));
    }
    if let Some(result) = twitter::extract_x_article(html, url) {
        return Some((result, "twitter"));
    }
    if let Some(result) = c2wiki::extract_c2wiki(html, url) {
        return Some((result, "c2wiki"));
    }
    if let Some(result) = conversations::extract_conversation(html, url) {
        return Some((result, "conversation"));
    }
    None
}
