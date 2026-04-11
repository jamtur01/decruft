//! X/Twitter extractor.
//!
//! Handles two page types:
//! - **Articles**: long-form content at `x.com/user/article/ID`,
//!   extracted from static HTML containers.
//! - **Tweets**: individual posts at `x.com/user/status/ID`. Static
//!   HTML rarely contains the tweet text, so we fall back to the
//!   Twitter oEmbed API (`publish.twitter.com/oembed`).

use scraper::Html;

use crate::dom;

use super::ExtractorResult;

const ARTICLE_CONTAINER_SEL: &str = "[data-testid=\"twitterArticleRichTextView\"]";
const ARTICLE_READ_VIEW_SEL: &str = "[data-testid=\"twitterArticleReadView\"]";
const TITLE_SEL: &str = "[data-testid=\"twitter-article-title\"]";
const AUTHOR_SEL: &str = "[itemprop=\"author\"]";
const AUTHOR_NAME_SEL: &str = "meta[itemprop=\"name\"]";
const AUTHOR_HANDLE_SEL: &str = "meta[itemprop=\"additionalName\"]";
const IMAGES_SEL: &str = "[data-testid=\"tweetPhoto\"] img";

/// Detect whether this page is an X/Twitter article.
#[must_use]
pub fn is_x_article(html: &Html, url: Option<&str>) -> bool {
    let is_x_domain = url.is_some_and(|u| {
        (u.contains("x.com/") || u.contains("twitter.com/")) && u.contains("/article/")
    });
    if !is_x_domain {
        return false;
    }
    has_selector(html, ARTICLE_CONTAINER_SEL)
}

/// Check whether a URL points to an individual tweet.
#[must_use]
pub fn is_tweet_url(url: Option<&str>) -> bool {
    use std::sync::LazyLock;
    static TWEET_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?:x\.com|twitter\.com)/[a-zA-Z0-9_]{1,15}/status/\d+")
            .expect("tweet url regex is valid")
    });
    url.is_some_and(|u| TWEET_RE.is_match(u))
}

/// Extract content from an X/Twitter page (article or tweet).
///
/// Tries the article extractor first. If the URL is a tweet and
/// article extraction fails, falls back to the oEmbed API.
#[must_use]
pub fn extract_x_article(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    if let Some(result) = try_article(html, url) {
        return Some(result);
    }
    if let Some(u) = url
        && is_tweet_url(Some(u))
    {
        return try_oembed(u);
    }
    None
}

/// Try extracting a long-form X/Twitter article.
fn try_article(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    if !is_x_article(html, url) {
        return None;
    }

    let container_ids = dom::select_ids(html, ARTICLE_CONTAINER_SEL);
    let container_id = container_ids.first().copied()?;

    let title = extract_title(html);
    let author = extract_author(html, url);
    let header_image = extract_header_image(html, container_id);
    let body = dom::inner_html(html, container_id);

    let content = format!(
        "<article class=\"x-article\">{header_image}{}</article>",
        body.trim()
    );

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("X (Twitter)".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

/// Fetch tweet content via the Twitter oEmbed API.
///
/// The API returns JSON with `html` (a blockquote), `author_name`,
/// and `author_url`. We use `html` as content and `author_name` as
/// author.
fn try_oembed(tweet_url: &str) -> Option<ExtractorResult> {
    let api_url = format!(
        "https://publish.twitter.com/oembed?url={}",
        urlencoding(tweet_url),
    );

    let body = crate::http::get(&api_url)?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let html = json.get("html")?.as_str()?;
    if html.trim().is_empty() {
        return None;
    }

    let author_name = json
        .get("author_name")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(ExtractorResult {
        content: html.to_string(),
        title: None,
        author: author_name,
        site: Some("X (Twitter)".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

/// Percent-encode a URL for use as a query parameter value.
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(result, "%{byte:02X}");
            }
        }
    }
    result
}

fn extract_title(html: &Html) -> String {
    let ids = dom::select_ids(html, TITLE_SEL);
    ids.first().map_or_else(
        || "Untitled X Article".to_string(),
        |&id| dom::text_content(html, id).trim().to_string(),
    )
}

fn extract_author(html: &Html, url: Option<&str>) -> String {
    let author_ids = dom::select_ids(html, AUTHOR_SEL);
    if let Some(&author_id) = author_ids.first() {
        let name = extract_meta_content(html, author_id, AUTHOR_NAME_SEL);
        let handle = extract_meta_content(html, author_id, AUTHOR_HANDLE_SEL);
        if !name.is_empty() && !handle.is_empty() {
            return format!("{name} (@{handle})");
        }
        if !name.is_empty() {
            return name;
        }
        if !handle.is_empty() {
            return handle;
        }
    }
    author_from_url(url).unwrap_or_default()
}

fn extract_meta_content(html: &Html, container_id: ego_tree::NodeId, sel: &str) -> String {
    let ids = dom::select_within(html, container_id, sel);
    ids.first()
        .and_then(|&id| dom::get_attr(html, id, "content"))
        .unwrap_or_default()
}

fn author_from_url(url: Option<&str>) -> Option<String> {
    use std::sync::LazyLock;
    static AUTHOR_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"/([a-zA-Z0-9_]{1,15})/(article|status)/\d+")
            .expect("x author regex is valid")
    });

    let u = url?;
    AUTHOR_RE
        .captures(u)
        .and_then(|caps| caps.get(1))
        .map(|m| format!("@{}", m.as_str()))
}

fn extract_header_image(html: &Html, container_id: ego_tree::NodeId) -> String {
    let read_view_ids = dom::select_ids(html, ARTICLE_READ_VIEW_SEL);
    let Some(&read_view_id) = read_view_ids.first() else {
        return String::new();
    };

    let img_ids = dom::select_within(html, read_view_id, IMAGES_SEL);
    let Some(&img_id) = img_ids.first() else {
        return String::new();
    };

    // Skip if image is inside the article container
    if dom::is_ancestor(html, img_id, container_id) {
        return String::new();
    }

    let Some(src) = dom::get_attr(html, img_id, "src") else {
        return String::new();
    };

    let alt = dom::get_attr(html, img_id, "alt").unwrap_or_else(|| "Image".to_string());
    let upgraded_src = upgrade_image_src(&src);

    format!(
        "<img src=\"{}\" alt=\"{}\">",
        html_attr_escape(&upgraded_src),
        html_attr_escape(&alt)
    )
}

fn upgrade_image_src(src: &str) -> String {
    if src.contains("&name=") {
        use std::sync::LazyLock;
        static NAME_RE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"&name=\w+").expect("name param regex is valid"));
        NAME_RE.replace(src, "&name=large").into_owned()
    } else if src.contains('?') {
        format!("{src}&name=large")
    } else {
        format!("{src}?name=large")
    }
}

fn has_selector(html: &Html, sel: &str) -> bool {
    scraper::Selector::parse(sel)
        .ok()
        .is_some_and(|s| html.select(&s).next().is_some())
}

fn html_attr_escape(s: &str) -> String {
    dom::html_attr_escape(s)
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_x_article_by_url_and_selector() {
        let doc = r#"<html><body>
            <div data-testid="twitterArticleRichTextView">
                <p>Article content here</p>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        assert!(is_x_article(
            &html,
            Some("https://x.com/user/article/12345")
        ));
        assert!(is_x_article(
            &html,
            Some("https://twitter.com/user/article/12345")
        ));
        assert!(!is_x_article(
            &html,
            Some("https://x.com/user/status/12345")
        ));
    }

    #[test]
    fn no_detect_without_container() {
        let html = Html::parse_document("<html><body><p>Hello</p></body></html>");
        assert!(!is_x_article(
            &html,
            Some("https://x.com/user/article/12345")
        ));
    }

    #[test]
    fn extract_basic_article() {
        let doc = r#"<html><body>
            <div data-testid="twitter-article-title">My Article</div>
            <div data-testid="twitterArticleRichTextView">
                <p>Article body text</p>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_x_article(&html, Some("https://x.com/testuser/article/999"));
        let result = result.unwrap();
        assert_eq!(result.title.as_deref(), Some("My Article"));
        assert!(result.content.contains("Article body text"));
        assert!(result.content.contains("x-article"));
        assert_eq!(result.site.as_deref(), Some("X (Twitter)"));
        assert_eq!(result.author.as_deref(), Some("@testuser"));
    }

    #[test]
    fn author_from_url_extraction() {
        assert_eq!(
            author_from_url(Some("https://x.com/johndoe/article/123")),
            Some("@johndoe".to_string())
        );
        assert_eq!(author_from_url(Some("https://example.com")), None);
    }

    #[test]
    fn upgrade_image_src_variants() {
        assert_eq!(
            upgrade_image_src("https://pbs.twimg.com/img?fmt=jpg&name=small"),
            "https://pbs.twimg.com/img?fmt=jpg&name=large"
        );
        assert_eq!(
            upgrade_image_src("https://pbs.twimg.com/img?fmt=jpg"),
            "https://pbs.twimg.com/img?fmt=jpg&name=large"
        );
        assert_eq!(
            upgrade_image_src("https://pbs.twimg.com/img"),
            "https://pbs.twimg.com/img?name=large"
        );
    }

    #[test]
    fn is_tweet_url_detection() {
        assert!(is_tweet_url(Some("https://x.com/user/status/123456")));
        assert!(is_tweet_url(Some("https://twitter.com/user/status/123456")));
        assert!(!is_tweet_url(Some("https://x.com/user/article/123456")));
        assert!(!is_tweet_url(Some("https://example.com")));
        assert!(!is_tweet_url(None));
    }

    #[test]
    fn urlencoding_basics() {
        assert_eq!(
            urlencoding("https://x.com/user/status/123"),
            "https%3A%2F%2Fx.com%2Fuser%2Fstatus%2F123"
        );
        assert_eq!(urlencoding("hello"), "hello");
    }

    #[test]
    fn oembed_on_live_tweet() {
        let result = try_oembed("https://x.com/elikiris/status/1925627023102992830");
        if let Some(r) = result {
            assert!(!r.content.is_empty(), "oEmbed should return content");
            assert!(r.author.is_some(), "oEmbed should return author");
            assert_eq!(r.site.as_deref(), Some("X (Twitter)"));
        }
        // Don't fail if network is unavailable
    }
}
