// C2 Wiki extractor
//
// SKIPPED: The C2 Wiki extractor in defuddle is async-only -- it fetches
// content from the C2 API (https://c2.com/wiki/remodel/pages/) and renders
// its custom wiki markup to HTML. The sync `canExtract()` returns false.
//
// Since decruft operates on static HTML (no network requests during parsing),
// implementing the C2 Wiki extractor would require either:
// 1. An async extraction pipeline (architectural change)
// 2. Pre-fetching the API content before calling decruft
//
// For now, this module provides detection only, which can be used if
// the async pipeline is added in the future.

use scraper::Html;

use super::ExtractorResult;

/// Detect whether this page is a C2 Wiki page.
///
/// Detection is URL-based only since the actual content comes from
/// an API, not the page HTML.
#[must_use]
pub fn is_c2wiki(_html: &Html, url: Option<&str>) -> bool {
    url.is_some_and(|u| u.contains("wiki.c2.com"))
}

/// Attempt to extract C2 Wiki content.
///
/// Always returns `None` because C2 Wiki extraction requires
/// fetching content from the C2 API asynchronously. See module
/// documentation for details.
#[must_use]
pub fn extract_c2wiki(_html: &Html, _url: Option<&str>) -> Option<ExtractorResult> {
    // C2 Wiki content is fetched from an API, not present in the page HTML.
    // This would require async support to implement.
    None
}
