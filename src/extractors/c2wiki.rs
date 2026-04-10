//! C2 Wiki extractor.
//!
//! Extracts content from wiki.c2.com pages. C2 Wiki renders content
//! client-side from a JSON API at `c2.com/wiki/remodel/pages/{title}`.
//! When the static HTML has no rendered content, we fetch the JSON API
//! directly and convert the wiki text to HTML.

use scraper::Html;

use crate::dom;

use super::ExtractorResult;

/// Detect whether this page is a C2 Wiki page.
#[must_use]
pub fn is_c2wiki(_html: &Html, url: Option<&str>) -> bool {
    url.is_some_and(|u| u.contains("wiki.c2.com"))
}

/// Extract content from a C2 Wiki page.
///
/// First tries the rendered `.page[data-title]` container (for saved
/// pages). Falls back to fetching from the C2 Wiki JSON API.
#[must_use]
pub fn extract_c2wiki(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    if !is_c2wiki(html, url) {
        return None;
    }

    // Try rendered content first
    if let Some(result) = try_rendered(html, url) {
        return Some(result);
    }

    // Fall back to API fetch
    let page_name = extract_page_name(url?)?;
    try_api_fetch(&page_name)
}

/// Try extracting from rendered `.page[data-title]` (saved HTML).
fn try_rendered(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    let page_ids = dom::select_ids(html, ".page[data-title]");
    let page_id = page_ids.first().copied()?;

    let title = dom::get_attr(html, page_id, "data-title")
        .map(|t| expand_wiki_word(&t))
        .unwrap_or_default();
    let content = dom::inner_html(html, page_id);

    if content.trim().is_empty() {
        return None;
    }

    Some(ExtractorResult {
        content,
        title: if title.is_empty() {
            extract_page_name(url.unwrap_or("")).map(|n| expand_wiki_word(&n))
        } else {
            Some(title)
        },
        author: None,
        site: Some("C2 Wiki".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

/// Fetch content from the C2 Wiki JSON API.
fn try_api_fetch(page_name: &str) -> Option<ExtractorResult> {
    let api_url = format!("https://c2.com/wiki/remodel/pages/{page_name}");

    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "10", &api_url])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let text = json.get("text")?.as_str()?;
    if text.trim().is_empty() {
        return None;
    }

    let date = json.get("date").and_then(|d| d.as_str()).map(String::from);

    let html_content = wiki_text_to_html(text);
    let title = expand_wiki_word(page_name);

    Some(ExtractorResult {
        content: html_content,
        title: Some(title),
        author: None,
        site: Some("C2 Wiki".to_string()),
        published: date,
        image: None,
        description: None,
    })
}

/// Convert C2 Wiki text format to HTML.
///
/// C2 Wiki text uses CamelCase for links, blank lines for paragraph
/// breaks, and `''text''` for italics.
fn wiki_text_to_html(text: &str) -> String {
    let mut html = String::with_capacity(text.len() * 2);
    let mut in_paragraph = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_paragraph {
                html.push_str("</p>\n");
                in_paragraph = false;
            }
            continue;
        }

        if in_paragraph {
            html.push(' ');
        } else {
            html.push_str("<p>");
            in_paragraph = true;
        }

        // Convert ''text'' to <em>text</em>
        let processed = convert_wiki_italics(trimmed);
        // Convert CamelCase words to links
        let processed = convert_wiki_links(&processed);
        html.push_str(&processed);
    }

    if in_paragraph {
        html.push_str("</p>\n");
    }

    html
}

/// Convert `''text''` wiki markup to `<em>text</em>`.
fn convert_wiki_italics(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_italic = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\'' && chars.peek() == Some(&'\'') {
            chars.next();
            if in_italic {
                result.push_str("</em>");
            } else {
                result.push_str("<em>");
            }
            in_italic = !in_italic;
            continue;
        }
        result.push(ch);
    }

    // Close unclosed italic
    if in_italic {
        result.push_str("</em>");
    }
    result
}

/// Convert CamelCase words to wiki links.
fn convert_wiki_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut word = String::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            word.push(ch);
        } else {
            flush_word(&word, &mut result);
            word.clear();
            result.push(ch);
        }
    }
    flush_word(&word, &mut result);
    result
}

fn flush_word(word: &str, result: &mut String) {
    if is_wiki_word(word) {
        use std::fmt::Write;
        let display = expand_wiki_word(word);
        let _ = write!(
            result,
            "<a href=\"https://wiki.c2.com/?{word}\">{display}</a>"
        );
    } else {
        result.push_str(word);
    }
}

/// Check if a word is CamelCase (at least 2 uppercase letters with
/// lowercase between them).
fn is_wiki_word(word: &str) -> bool {
    if word.len() < 3 {
        return false;
    }
    let mut upper_count = 0;
    let mut has_lower = false;
    for ch in word.chars() {
        if ch.is_uppercase() {
            upper_count += 1;
        } else if ch.is_lowercase() {
            has_lower = true;
        }
    }
    upper_count >= 2 && has_lower && word.chars().next().is_some_and(char::is_uppercase)
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

/// Extract the page name from a C2 Wiki URL.
fn extract_page_name(url: &str) -> Option<String> {
    // wiki.c2.com/?PageName
    if let Some(idx) = url.find('?') {
        let name = &url[idx + 1..];
        if !name.is_empty() && !name.contains('=') {
            return Some(name.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_c2wiki_url() {
        let html = Html::parse_document("<html><body></body></html>");
        assert!(is_c2wiki(&html, Some("https://wiki.c2.com/?TestPage")));
        assert!(!is_c2wiki(&html, Some("https://example.com")));
    }

    #[test]
    fn expands_wiki_words() {
        assert_eq!(expand_wiki_word("WelcomeVisitors"), "Welcome Visitors");
        assert_eq!(
            expand_wiki_word("ExtremeProgramming"),
            "Extreme Programming"
        );
        assert_eq!(expand_wiki_word("XP"), "XP");
    }

    #[test]
    fn converts_wiki_italics() {
        assert_eq!(
            convert_wiki_italics("This is ''italic'' text"),
            "This is <em>italic</em> text"
        );
    }

    #[test]
    fn identifies_wiki_words() {
        assert!(is_wiki_word("ExtremeProgramming"));
        assert!(is_wiki_word("WelcomeVisitors"));
        assert!(!is_wiki_word("the"));
        assert!(!is_wiki_word("XP"));
        assert!(!is_wiki_word("abc"));
    }

    #[test]
    fn extracts_page_name_from_url() {
        assert_eq!(
            extract_page_name("https://wiki.c2.com/?ExtremeProgramming"),
            Some("ExtremeProgramming".to_string())
        );
        assert_eq!(extract_page_name("https://wiki.c2.com/"), None);
    }

    #[test]
    fn wiki_text_converts_to_html() {
        let text = "First paragraph.\n\nSecond paragraph.";
        let html = wiki_text_to_html(text);
        assert!(html.contains("<p>First paragraph.</p>"));
        assert!(html.contains("<p>Second paragraph.</p>"));
    }

    #[test]
    fn api_fetch_on_live_page() {
        // This test makes a real network call — skip in CI
        if std::env::var("CI").is_ok() {
            return;
        }
        let result = try_api_fetch("ExtremeProgramming");
        if let Some(r) = result {
            assert!(r.content.len() > 100, "should have substantial content");
            assert_eq!(r.title, Some("Extreme Programming".to_string()));
            assert_eq!(r.site, Some("C2 Wiki".to_string()));
        }
        // Don't fail if network is unavailable
    }
}
