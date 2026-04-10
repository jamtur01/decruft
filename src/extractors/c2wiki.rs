//! C2 Wiki extractor.
//!
//! Extracts content from wiki.c2.com pages. C2 Wiki renders content
//! client-side, so this extractor targets pages saved after JS
//! execution (e.g., via browser "Save As" or headless browsers) where
//! the rendered `.page[data-title]` element is present in the HTML.

use scraper::Html;

use crate::dom;

use super::ExtractorResult;

/// Detect whether this page is a C2 Wiki page.
#[must_use]
pub fn is_c2wiki(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("wiki.c2.com")) {
        return true;
    }
    has_page_element(html)
}

/// Extract content from a C2 Wiki page.
///
/// Looks for the rendered `.page[data-title]` container that C2 Wiki's
/// JS produces. Returns `None` if the page wasn't detected or has no
/// content.
#[must_use]
pub fn extract_c2wiki(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    if !is_c2wiki(html, url) {
        return None;
    }

    let page_ids = dom::select_ids(html, ".page[data-title]");
    let page_id = page_ids.first().copied()?;

    let title = extract_title(html, page_id, url);
    let content = extract_content(html, page_id);

    if content.is_empty() {
        return None;
    }

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: None,
        site: Some("C2 Wiki".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

/// Extract the page title from `data-title`, expanding `CamelCase`
/// into separate words.
fn extract_title(html: &Html, page_id: ego_tree::NodeId, url: Option<&str>) -> String {
    // Prefer data-title attribute on the .page element
    if let Some(data_title) = dom::get_attr(html, page_id, "data-title")
        && !data_title.is_empty()
    {
        return expand_wiki_word(&data_title);
    }

    // Fall back to extracting the title from the URL query string
    if let Some(title) = title_from_url(url) {
        return expand_wiki_word(&title);
    }

    "C2 Wiki Page".to_string()
}

/// Extract the main body content from the `.page` container, skipping
/// the `<h1>` heading (already captured as title).
fn extract_content(html: &Html, page_id: ego_tree::NodeId) -> String {
    let inner = dom::inner_html(html, page_id);
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Strip the leading <h1>...</h1> since we extract title separately
    let body = strip_leading_h1(trimmed);
    format!("<article class=\"c2-wiki\">{body}</article>")
}

/// Remove a leading `<h1>...</h1>` block from HTML content.
fn strip_leading_h1(html: &str) -> &str {
    let s = html.trim_start();
    if let Some(rest) = s.strip_prefix("<h1")
        && let Some(end_pos) = rest.find("</h1>")
    {
        return rest[end_pos + 5..].trim_start();
    }
    s
}

/// Expand a `CamelCase` wiki word into space-separated words.
///
/// `"WelcomeVisitors"` becomes `"Welcome Visitors"`.
fn expand_wiki_word(word: &str) -> String {
    let mut result = String::with_capacity(word.len() + 8);
    let mut prev_char: Option<char> = None;
    for ch in word.chars() {
        if ch.is_uppercase()
            && let Some(prev) = prev_char
            && prev.is_lowercase()
        {
            result.push(' ');
        }
        result.push(ch);
        prev_char = Some(ch);
    }
    result
}

/// Extract the wiki page name from a C2 Wiki URL query string.
///
/// `"https://wiki.c2.com/?WelcomeVisitors"` yields
/// `Some("WelcomeVisitors")`.
fn title_from_url(url: Option<&str>) -> Option<String> {
    let u = url?;
    let query = u.split('?').nth(1)?;
    let name = query.split('&').next()?;
    if name.is_empty() || name.contains('=') {
        return None;
    }
    Some(name.to_string())
}

fn has_page_element(html: &Html) -> bool {
    scraper::Selector::parse(".page[data-title]")
        .ok()
        .is_some_and(|s| html.select(&s).next().is_some())
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_by_url() {
        let html = Html::parse_document("<html><body></body></html>");
        assert!(is_c2wiki(&html, Some("https://wiki.c2.com/?TestPage")));
        assert!(!is_c2wiki(&html, Some("https://example.com")));
    }

    #[test]
    fn detect_by_page_element() {
        let doc = r#"<html><body>
            <div class="page" data-title="TestPage">
                <h1>Test Page</h1>
                <p>Some content</p>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        assert!(is_c2wiki(&html, None));
    }

    #[test]
    fn extract_basic_page() {
        let doc = r#"<html><body>
            <div class="page" data-title="WelcomeVisitors">
                <h1><span>Welcome Visitors</span></h1>
                <p>This is the C2 Wiki.</p>
                <p>Founded by Ward Cunningham.</p>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_c2wiki(&html, Some("https://wiki.c2.com/?WelcomeVisitors"));
        let result = result.unwrap();
        assert_eq!(result.title.as_deref(), Some("Welcome Visitors"));
        assert_eq!(result.site.as_deref(), Some("C2 Wiki"));
        assert!(result.content.contains("c2-wiki"));
        assert!(result.content.contains("Founded by Ward Cunningham"));
        assert!(!result.content.contains("<h1>"));
    }

    #[test]
    fn extract_returns_none_without_page_element() {
        let doc = "<html><body><p>Nothing here</p></body></html>";
        let html = Html::parse_document(doc);
        let result = extract_c2wiki(&html, Some("https://wiki.c2.com/?TestPage"));
        assert!(result.is_none());
    }

    #[test]
    fn extract_returns_none_for_empty_content() {
        let doc = r#"<html><body>
            <div class="page" data-title="Empty"></div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_c2wiki(&html, Some("https://wiki.c2.com/?Empty"));
        assert!(result.is_none());
    }

    #[test]
    fn expand_wiki_word_splits_camel_case() {
        assert_eq!(expand_wiki_word("WelcomeVisitors"), "Welcome Visitors");
        assert_eq!(
            expand_wiki_word("ExtremeProgramming"),
            "Extreme Programming"
        );
        assert_eq!(expand_wiki_word("Hello"), "Hello");
        assert_eq!(expand_wiki_word(""), "");
    }

    #[test]
    fn title_from_url_extracts_page_name() {
        assert_eq!(
            title_from_url(Some("https://wiki.c2.com/?WelcomeVisitors")),
            Some("WelcomeVisitors".to_string())
        );
        assert_eq!(title_from_url(Some("https://wiki.c2.com/")), None);
        assert_eq!(title_from_url(None), None);
    }

    #[test]
    fn title_falls_back_to_url() {
        let doc = r#"<html><body>
            <div class="page" data-title="">
                <p>Content here</p>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_c2wiki(&html, Some("https://wiki.c2.com/?DesignPatterns"));
        let result = result.unwrap();
        assert_eq!(result.title.as_deref(), Some("Design Patterns"));
    }
}
