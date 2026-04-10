use std::fmt::Write;

use scraper::Html;

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment_tree, build_content_html};

/// Detect whether this page is a Lobste.rs story page.
#[must_use]
pub fn is_lobsters(url: Option<&str>) -> bool {
    url.is_some_and(|u| u.contains("lobste.rs/s/"))
}

/// Extract content from a Lobste.rs story page.
///
/// When `include_replies` is false, comments are omitted.
/// Falls back to the Lobste.rs JSON API when HTML extraction
/// yields fewer than 10 words.
#[must_use]
pub fn extract_lobsters(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    if !is_lobsters(url) {
        return None;
    }

    let result = extract_from_html(html, include_replies);

    match result {
        Some(ref r) if dom::count_words_html(&r.content) >= 10 => result,
        _ => try_api_fetch(url, include_replies).or(result),
    }
}

fn extract_from_html(html: &Html, include_replies: bool) -> Option<ExtractorResult> {
    let title = extract_title(html);
    let story_body = extract_story_body(html);
    let story_link = extract_story_link(html);

    if story_body.is_empty() && story_link.is_empty() {
        return None;
    }

    let mut post_html = String::new();
    if !story_link.is_empty() {
        let escaped = dom::html_attr_escape(&story_link);
        let _ = write!(
            post_html,
            "<p><a href=\"{escaped}\" target=\"_blank\">\
             {escaped}</a></p>"
        );
    }
    if !story_body.is_empty() {
        let _ = write!(post_html, "<div class=\"post-text\">{story_body}</div>");
    }

    let comments_html = if include_replies {
        extract_comments(html)
    } else {
        String::new()
    };

    let content = build_content_html("lobsters", &post_html, &comments_html);

    Some(ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: None,
        site: Some("Lobsters".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

fn extract_title(html: &Html) -> String {
    let ids = dom::select_ids(html, ".u-repost-of");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_story_body(html: &Html) -> String {
    // Lobsters stories with text have a .description div
    let ids = dom::select_ids(html, ".story_text");
    ids.first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_story_link(html: &Html) -> String {
    let ids = dom::select_ids(html, ".u-repost-of");
    ids.first()
        .and_then(|&id| dom::get_attr(html, id, "href"))
        .unwrap_or_default()
}

fn extract_comments(html: &Html) -> String {
    let comment_ids = dom::select_ids(html, ".comment_text");
    if comment_ids.is_empty() {
        return String::new();
    }

    let mut comments = Vec::new();
    for &cid in &comment_ids {
        let body = dom::inner_html(html, cid).trim().to_string();
        if body.is_empty() {
            continue;
        }

        // Walk up to the .comment container to get metadata
        let container = find_comment_container(html, cid);
        let (author, depth, score) = container.map_or_else(
            || (String::new(), 0, String::new()),
            |c| extract_comment_meta(html, c),
        );

        comments.push(CommentData {
            author,
            date: String::new(),
            content: body,
            depth,
            score: if score.is_empty() { None } else { Some(score) },
            url: None,
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

/// Walk up from a `.comment_text` node to the nearest `.comment`
/// ancestor (via `li.comments_subtree`).
fn find_comment_container(html: &Html, node_id: ego_tree::NodeId) -> Option<ego_tree::NodeId> {
    let mut current = Some(node_id);
    while let Some(nid) = current {
        if dom::has_class(html, nid, "comment") {
            return Some(nid);
        }
        current = dom::parent_element(html, nid);
    }
    None
}

fn extract_comment_meta(html: &Html, container: ego_tree::NodeId) -> (String, usize, String) {
    let author_ids = dom::select_within(html, container, ".comment_author a");
    let author = author_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default();

    // Lobsters uses inline style for depth indentation;
    // fall back to 0 if we can't determine it
    let depth = 0;

    let score_ids = dom::select_within(html, container, ".comment_score");
    let score = score_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default();

    (author, depth, score)
}

// --- API fallback ---

/// Parse the short ID from a Lobste.rs URL.
///
/// `https://lobste.rs/s/abc123/story_title` -> `abc123`
fn parse_short_id(url: &str) -> Option<&str> {
    let after_s = url.split("lobste.rs/s/").nth(1)?;
    let id = after_s.split('/').next()?;
    if id.is_empty() {
        return None;
    }
    Some(id)
}

/// Build the JSON API URL from the page URL.
fn api_url(url: &str) -> Option<String> {
    let id = parse_short_id(url)?;
    Some(format!("https://lobste.rs/s/{id}.json"))
}

fn try_api_fetch(url: Option<&str>, include_replies: bool) -> Option<ExtractorResult> {
    let u = url?;
    let api = api_url(u)?;
    let json = fetch_lobsters_json(&api)?;

    build_from_api(&json, include_replies)
}

fn build_from_api(json: &serde_json::Value, include_replies: bool) -> Option<ExtractorResult> {
    let title = lobsters_json_str(json, "title");
    let description = json
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let story_url = lobsters_json_str(json, "url");
    let submitter = json
        .get("submitter_user")
        .and_then(|u| u.get("username"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();

    let mut post_html = String::new();
    if !story_url.is_empty() {
        let escaped = dom::html_attr_escape(&story_url);
        let _ = write!(
            post_html,
            "<p><a href=\"{escaped}\" target=\"_blank\">\
             {escaped}</a></p>"
        );
    }
    if !description.trim().is_empty() {
        let _ = write!(post_html, "<div class=\"post-text\">{description}</div>");
    }

    if post_html.is_empty() {
        return None;
    }

    let comments_html = if include_replies {
        build_api_comments(json)
    } else {
        String::new()
    };

    let content = build_content_html("lobsters", &post_html, &comments_html);

    Some(ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: if submitter.is_empty() {
            None
        } else {
            Some(submitter)
        },
        site: Some("Lobsters".to_string()),
        published: json
            .get("created_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|d| d.split('T').next())
            .map(String::from),
        image: None,
        description: None,
    })
}

fn build_api_comments(json: &serde_json::Value) -> String {
    let Some(comments_arr) = json.get("comments").and_then(serde_json::Value::as_array) else {
        return String::new();
    };

    let mut comments = Vec::new();
    for c in comments_arr {
        let body = c
            .get("comment")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if body.trim().is_empty() {
            continue;
        }

        let author = c
            .get("commenting_user")
            .and_then(|u| u.get("username"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        let depth = c
            .get("depth")
            .and_then(serde_json::Value::as_u64)
            .and_then(|d| usize::try_from(d).ok())
            .unwrap_or(0);

        let score = c
            .get("score")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        comments.push(CommentData {
            author,
            date: String::new(),
            content: body.to_string(),
            depth,
            score: Some(format!("{score} points")),
            url: None,
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn fetch_lobsters_json(url: &str) -> Option<serde_json::Value> {
    let body = crate::http::get(url)?;
    serde_json::from_str(&body).ok()
}

fn lobsters_json_str(json: &serde_json::Value, key: &str) -> String {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_lobsters_url() {
        assert!(is_lobsters(Some("https://lobste.rs/s/abc123/some_story")));
        assert!(is_lobsters(Some("https://lobste.rs/s/xyz")));
    }

    #[test]
    fn reject_non_lobsters_url() {
        assert!(!is_lobsters(Some("https://example.com")));
        assert!(!is_lobsters(Some("https://lobste.rs/")));
        assert!(!is_lobsters(None));
    }

    #[test]
    fn parse_short_id_valid() {
        assert_eq!(
            parse_short_id("https://lobste.rs/s/abc123/some_title"),
            Some("abc123")
        );
        assert_eq!(parse_short_id("https://lobste.rs/s/xyz"), Some("xyz"));
    }

    #[test]
    fn parse_short_id_invalid() {
        assert!(parse_short_id("https://example.com").is_none());
    }

    #[test]
    fn api_url_construction() {
        assert_eq!(
            api_url("https://lobste.rs/s/abc123/story"),
            Some("https://lobste.rs/s/abc123.json".to_string())
        );
    }

    #[test]
    fn extract_from_html_basic() {
        let html_str = r#"
        <html>
        <body>
        <a class="u-repost-of" href="https://example.com/article">
            A Cool Story
        </a>
        <div class="comment">
            <span class="comment_author">
                <a href="/u/alice">alice</a>
            </span>
            <div class="comment_text">
                <p>This is a great article.</p>
            </div>
        </div>
        </body>
        </html>
        "#;
        let html = Html::parse_document(html_str);
        let result = extract_from_html(&html, true).unwrap();

        assert_eq!(result.title.as_deref(), Some("A Cool Story"));
        assert!(result.content.contains("example.com/article"));
        assert!(result.content.contains("great article"));
    }

    #[test]
    fn build_from_api_basic() {
        let json = serde_json::json!({
            "title": "Test Story",
            "description": "<p>Story description</p>",
            "url": "https://example.com/test",
            "score": 42,
            "submitter_user": {"username": "alice"},
            "tags": ["rust"],
            "created_at": "2025-01-15T12:00:00Z",
            "comments": [
                {
                    "comment": "<p>Nice!</p>",
                    "commenting_user": {"username": "bob"},
                    "depth": 0,
                    "score": 5,
                    "created_at": "2025-01-15T13:00:00Z"
                },
                {
                    "comment": "<p>Agreed</p>",
                    "commenting_user": {"username": "carol"},
                    "depth": 1,
                    "score": 2,
                    "created_at": "2025-01-15T14:00:00Z"
                }
            ]
        });

        let result = build_from_api(&json, true).unwrap();
        assert_eq!(result.title.as_deref(), Some("Test Story"));
        assert_eq!(result.author.as_deref(), Some("alice"));
        assert_eq!(result.published.as_deref(), Some("2025-01-15"));
        assert!(result.content.contains("example.com/test"));
        assert!(result.content.contains("Nice!"));
        assert!(result.content.contains("bob"));
        assert!(result.content.contains("Agreed"));
    }

    #[test]
    #[ignore = "real network call"]
    fn api_fetch_live() {
        let url = "https://lobste.rs/s/abc123";
        let result = try_api_fetch(Some(url), false);
        // Don't assert success — just verify it doesn't crash
        drop(result);
    }
}
