use std::fmt::Write;

use scraper::{Html, Selector};

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment, build_comment_tree, build_content_html};

/// Detect whether this page is a Hacker News page.
#[must_use]
pub fn is_hackernews(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("news.ycombinator.com")) {
        return true;
    }
    Selector::parse(".fatitem")
        .ok()
        .is_some_and(|sel| html.select(&sel).next().is_some())
}

/// Extract content from a Hacker News page.
///
/// When `include_replies` is false, comments are omitted.
/// Falls back to the HN Firebase API when HTML extraction yields
/// fewer than 10 words (e.g., JS-rendered or empty pages).
#[must_use]
pub fn extract_hackernews(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    if !is_hackernews(html, url) {
        return None;
    }

    let fatitem_ids = dom::select_ids(html, ".fatitem");
    let has_fatitem = fatitem_ids.first().copied().is_some();

    let result = if has_fatitem {
        let is_comment_page = detect_comment_page(html);
        if is_comment_page {
            extract_comment_page(html, url, include_replies)
        } else {
            extract_story_page(html, url, include_replies)
        }
    } else {
        None
    };

    match result {
        Some(ref r) if dom::count_words_html(&r.content) >= 10 => result,
        _ => try_api_fetch(url, include_replies).or(result),
    }
}

fn detect_comment_page(html: &Html) -> bool {
    let fatitem_ids = dom::select_ids(html, ".fatitem");
    let Some(&fatitem) = fatitem_ids.first() else {
        return false;
    };
    let has_onstory = !dom::select_within(html, fatitem, ".onstory").is_empty();
    let has_titleline = !dom::select_within(html, fatitem, ".titleline").is_empty();
    has_onstory && !has_titleline
}

// --- Story page extraction ---

fn extract_story_page(
    html: &Html,
    _url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let fatitem_ids = dom::select_ids(html, ".fatitem");
    let fatitem = fatitem_ids.first().copied()?;

    let title = extract_story_title(html, fatitem);
    let author = extract_author(html, fatitem);
    let post_content = extract_story_content(html, fatitem);
    let comments = if include_replies {
        extract_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("hackernews", &post_content, &comments);

    let published = extract_date(html, fatitem);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Hacker News".to_string()),
        published: if published.is_empty() {
            None
        } else {
            Some(published)
        },
        image: None,
        description: None,
    })
}

fn extract_story_title(html: &Html, fatitem: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, fatitem, ".titleline");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_author(html: &Html, container: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, container, ".hnuser");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_date(html: &Html, container: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, container, ".age");
    ids.first()
        .and_then(|&id| dom::get_attr(html, id, "title"))
        .and_then(|dt| dt.split('T').next().map(String::from))
        .unwrap_or_default()
}

fn extract_story_content(html: &Html, fatitem: ego_tree::NodeId) -> String {
    use std::fmt::Write;
    let mut content = String::new();

    // Story link
    let link_ids = dom::select_within(html, fatitem, ".titleline a");
    if let Some(&link_id) = link_ids.first()
        && let Some(href) = dom::get_attr(html, link_id, "href")
    {
        let escaped = html_attr_escape(&href);
        let _ = write!(
            content,
            "<p><a href=\"{escaped}\" target=\"_blank\">{escaped}</a></p>"
        );
    }

    // Self-text (Ask HN, Show HN)
    let text_ids = dom::select_within(html, fatitem, ".toptext");
    if let Some(&text_id) = text_ids.first() {
        let text_html = dom::inner_html(html, text_id);
        let _ = write!(
            content,
            "<div class=\"post-text\">{}</div>",
            text_html.trim()
        );
    }

    content
}

// --- Comment page extraction ---

fn extract_comment_page(
    html: &Html,
    _url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let fatitem_ids = dom::select_ids(html, ".fatitem");
    let fatitem = fatitem_ids.first().copied()?;

    let main_comment = extract_main_comment(html, fatitem)?;
    let author = main_comment.author.clone();
    let title = build_comment_title(&main_comment);
    let post_content = build_comment(&main_comment);
    let comments = if include_replies {
        extract_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("hackernews", &post_content, &comments);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Hacker News".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

fn extract_main_comment(html: &Html, fatitem: ego_tree::NodeId) -> Option<CommentData> {
    let athing_ids = dom::select_within(html, fatitem, "tr.athing");
    let athing = athing_ids.first().copied()?;

    let author = extract_author(html, athing);
    let date = extract_date(html, athing);

    let commtext_ids = dom::select_within(html, athing, ".commtext");
    let content = commtext_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default();

    if content.is_empty() {
        return None;
    }

    Some(CommentData {
        author,
        date,
        content,
        depth: 0,
        score: None,
        url: None,
    })
}

fn build_comment_title(comment: &CommentData) -> String {
    let text = dom::strip_html_tags(&comment.content);
    let trimmed = text.trim();
    let preview = match trimmed.char_indices().nth(50) {
        Some((i, _)) => format!("{}...", &trimmed[..i]),
        None => trimmed.to_string(),
    };
    format!("Comment by {}: {preview}", comment.author)
}

fn html_attr_escape(s: &str) -> String {
    dom::html_attr_escape(s)
}

// --- Comment extraction (shared) ---

fn extract_comments(html: &Html) -> String {
    let comment_ids = dom::select_ids(html, "tr.comtr");
    if comment_ids.is_empty() {
        return String::new();
    }

    let mut comments = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for &cid in &comment_ids {
        let Some(id_attr) = dom::get_attr(html, cid, "id") else {
            continue;
        };
        if !processed.insert(id_attr.clone()) {
            continue;
        }

        let depth = extract_indent_depth(html, cid);
        let author = extract_comment_author(html, cid);
        let date = extract_comment_date(html, cid);
        let score = extract_comment_score(html, cid);

        let commtext_ids = dom::select_within(html, cid, ".commtext");
        let body = commtext_ids
            .first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
            .unwrap_or_default();

        if body.is_empty() {
            continue;
        }

        let comment_url = format!("https://news.ycombinator.com/item?id={id_attr}");

        comments.push(CommentData {
            author,
            date,
            content: body,
            depth,
            score: if score.is_empty() { None } else { Some(score) },
            url: Some(comment_url),
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn extract_indent_depth(html: &Html, comment_id: ego_tree::NodeId) -> usize {
    let img_ids = dom::select_within(html, comment_id, ".ind img");
    img_ids
        .first()
        .and_then(|&id| dom::get_attr(html, id, "width"))
        .and_then(|w| w.parse::<usize>().ok())
        .map_or(0, |w| w / 40)
}

fn extract_comment_author(html: &Html, comment_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, comment_id, ".hnuser");
    ids.first().map_or_else(
        || "[deleted]".to_string(),
        |&id| dom::text_content(html, id).trim().to_string(),
    )
}

fn extract_comment_date(html: &Html, comment_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, comment_id, ".age");
    ids.first()
        .and_then(|&id| dom::get_attr(html, id, "title"))
        .and_then(|dt| dt.split('T').next().map(String::from))
        .unwrap_or_default()
}

fn extract_comment_score(html: &Html, comment_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, comment_id, ".score");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

// --- API fallback ---

const HN_API_BASE: &str = "https://hacker-news.firebaseio.com/v0/item";
const HN_MAX_COMMENT_DEPTH: usize = 3;
const HN_MAX_COMMENTS: usize = 20;

/// Extract the item ID from a Hacker News URL.
fn parse_hn_item_id(url: &str) -> Option<&str> {
    // news.ycombinator.com/item?id=12345
    let query = url.split("id=").nth(1)?;
    let id = query.split('&').next()?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(id)
}

/// Fetch content from the HN Firebase API.
fn try_api_fetch(url: Option<&str>, include_replies: bool) -> Option<ExtractorResult> {
    let id = parse_hn_item_id(url?)?;
    let json = fetch_hn_json(id)?;

    let item_type = json
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match item_type {
        "story" | "job" | "poll" => Some(build_story_from_api(&json, include_replies)),
        "comment" => build_comment_from_api(&json, include_replies),
        _ => None,
    }
}

/// Build an `ExtractorResult` for a story item from the API.
fn build_story_from_api(json: &serde_json::Value, include_replies: bool) -> ExtractorResult {
    let title = hn_json_str(json, "title");
    let author = hn_json_str(json, "by");
    let published = format_unix_timestamp(json);

    let mut post_html = String::new();
    if let Some(link) = json.get("url").and_then(serde_json::Value::as_str) {
        let escaped = dom::html_attr_escape(link);
        let _ = write!(
            post_html,
            "<p><a href=\"{escaped}\" target=\"_blank\">{escaped}</a></p>"
        );
    }
    if let Some(text) = json.get("text").and_then(serde_json::Value::as_str) {
        let _ = write!(post_html, "<div class=\"post-text\">{text}</div>");
    }

    let comments_html = if include_replies {
        let mut count = 0;
        fetch_comment_kids(json, 0, &mut count)
    } else {
        String::new()
    };

    let content = build_content_html("hackernews", &post_html, &comments_html);

    ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Hacker News".to_string()),
        published: if published.is_empty() {
            None
        } else {
            Some(published)
        },
        image: None,
        description: None,
    }
}

/// Build an `ExtractorResult` for a single comment from the API.
fn build_comment_from_api(
    json: &serde_json::Value,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let text = json.get("text").and_then(serde_json::Value::as_str)?;
    if text.trim().is_empty() {
        return None;
    }
    let author = hn_json_str(json, "by");

    let plain = dom::strip_html_tags(text);
    let trimmed = plain.trim();
    let preview = match trimmed.char_indices().nth(50) {
        Some((i, _)) => format!("{}...", &trimmed[..i]),
        None => trimmed.to_string(),
    };
    let title = format!("Comment by {author}: {preview}");

    let main_comment = CommentData {
        author: author.clone(),
        date: format_unix_timestamp(json),
        content: text.to_string(),
        depth: 0,
        score: None,
        url: None,
    };
    let post_html = build_comment(&main_comment);

    let comments_html = if include_replies {
        let mut count = 0;
        fetch_comment_kids(json, 0, &mut count)
    } else {
        String::new()
    };

    let content = build_content_html("hackernews", &post_html, &comments_html);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Hacker News".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

/// Recursively fetch child comments from the HN API into a flat
/// list, then build a comment tree.
fn fetch_comment_kids(parent: &serde_json::Value, depth: usize, count: &mut usize) -> String {
    let mut comments = Vec::new();
    collect_kids_flat(parent, depth, count, &mut comments);
    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

/// Collect nested comments into a flat Vec with depth info.
fn collect_kids_flat(
    parent: &serde_json::Value,
    depth: usize,
    count: &mut usize,
    out: &mut Vec<CommentData>,
) {
    if depth >= HN_MAX_COMMENT_DEPTH || *count >= HN_MAX_COMMENTS {
        return;
    }

    let Some(kids) = parent.get("kids").and_then(serde_json::Value::as_array) else {
        return;
    };

    for kid_val in kids {
        if *count >= HN_MAX_COMMENTS {
            break;
        }
        let Some(kid_id) = kid_val.as_u64() else {
            continue;
        };
        let id_str = kid_id.to_string();
        let Some(child_json) = fetch_hn_json(&id_str) else {
            continue;
        };
        if child_json
            .get("deleted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }

        let text = child_json
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if text.trim().is_empty() {
            continue;
        }

        *count += 1;
        out.push(CommentData {
            author: hn_json_str(&child_json, "by"),
            date: format_unix_timestamp(&child_json),
            content: text.to_string(),
            depth,
            score: None,
            url: Some(format!("https://news.ycombinator.com/item?id={kid_id}")),
        });

        collect_kids_flat(&child_json, depth + 1, count, out);
    }
}

/// Fetch a single item from the HN Firebase API.
fn fetch_hn_json(id: &str) -> Option<serde_json::Value> {
    let url = format!("{HN_API_BASE}/{id}.json");
    let body = crate::http::get(&url)?;
    serde_json::from_str(&body).ok()
}

fn hn_json_str(json: &serde_json::Value, key: &str) -> String {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Format a unix timestamp field into a `YYYY-MM-DD` date string.
fn format_unix_timestamp(json: &serde_json::Value) -> String {
    let Some(ts) = json.get("time").and_then(serde_json::Value::as_i64) else {
        return String::new();
    };
    // Manual conversion from unix timestamp to YYYY-MM-DD
    // Using days-since-epoch arithmetic to avoid pulling in chrono
    let secs_per_day: i64 = 86400;
    let days = ts / secs_per_day;
    // Algorithm from Howard Hinnant's civil_from_days
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn load_fixture(name: &str) -> String {
        let path = format!(
            "{}/tests/fixtures/defuddle/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("fixture not found at {path}: {e}"))
    }

    #[test]
    fn parse_hn_item_id_valid() {
        assert_eq!(
            parse_hn_item_id("https://news.ycombinator.com/item?id=12345678"),
            Some("12345678")
        );
    }

    #[test]
    fn parse_hn_item_id_with_extra_params() {
        assert_eq!(
            parse_hn_item_id("https://news.ycombinator.com/item?id=999&p=2"),
            Some("999")
        );
    }

    #[test]
    fn parse_hn_item_id_invalid() {
        assert!(parse_hn_item_id("https://news.ycombinator.com/").is_none());
        assert!(parse_hn_item_id("https://example.com").is_none());
    }

    #[test]
    fn format_timestamp_known_date() {
        // 2024-01-15 00:00:00 UTC = 1705276800
        let json = serde_json::json!({"time": 1_705_276_800});
        assert_eq!(format_unix_timestamp(&json), "2024-01-15");
    }

    #[test]
    fn format_timestamp_missing() {
        let json = serde_json::json!({});
        assert_eq!(format_unix_timestamp(&json), "");
    }

    #[test]
    #[ignore = "real network call"]
    fn api_fetch_live_story() {
        // Item 1 is the first HN post
        let url = "https://news.ycombinator.com/item?id=1";
        let result = try_api_fetch(Some(url), false);
        if let Some(r) = result {
            assert!(r.title.is_some());
            assert_eq!(r.site.as_deref(), Some("Hacker News"));
        }
        // Don't fail if network is unavailable
    }

    #[test]
    fn extract_hn_comment_page() {
        let html_str = load_fixture("general--news.ycombinator.com-item-id=12345678.html");
        let url = Some("https://news.ycombinator.com/item?id=12345678");
        let html = Html::parse_document(&html_str);

        assert!(is_hackernews(&html, url));
        let result = extract_hackernews(&html, url, true).unwrap();

        assert!(result.title.unwrap().contains("testuser"));
        assert_eq!(result.author.as_deref(), Some("testuser"));
        assert!(result.content.contains("main comment text"));
    }

    #[test]
    fn build_story_from_api_canned_json() {
        let json = serde_json::json!({
            "type": "story",
            "title": "Show HN: A Rust parser",
            "by": "rustfan",
            "url": "https://example.com/parser",
            "text": "I built a parser in Rust. It's fast.",
            "time": 1_705_276_800_i64,
            "kids": []
        });

        let result = build_story_from_api(&json, false);
        assert_eq!(result.title.as_deref(), Some("Show HN: A Rust parser"));
        assert_eq!(result.author.as_deref(), Some("rustfan"));
        assert_eq!(result.published.as_deref(), Some("2024-01-15"));
        assert_eq!(result.site.as_deref(), Some("Hacker News"));
        assert!(result.content.contains("example.com/parser"));
        assert!(result.content.contains("parser in Rust"));
    }

    #[test]
    fn build_story_from_api_no_text() {
        let json = serde_json::json!({
            "type": "story",
            "title": "Link-only story",
            "by": "poster",
            "url": "https://example.com/article",
            "time": 1_705_276_800_i64
        });

        let result = build_story_from_api(&json, false);
        assert_eq!(result.title.as_deref(), Some("Link-only story"));
        assert!(result.content.contains("example.com/article"));
    }

    #[test]
    fn extract_hn_story_with_comments() {
        let html_str = load_fixture("comments--news.ycombinator.com-item-id=12345678.html");
        let url = Some("https://news.ycombinator.com/item?id=12345678");
        let html = Html::parse_document(&html_str);

        assert!(is_hackernews(&html, url));
        let result = extract_hackernews(&html, url, true).unwrap();

        assert_eq!(result.title.as_deref(), Some("A Sample Article"));
        assert_eq!(result.author.as_deref(), Some("author_one"));
        assert_eq!(result.site.as_deref(), Some("Hacker News"));
        assert!(result.content.contains("example.com/article"));
        assert!(result.content.contains("Comments"));
        assert!(result.content.contains("commenter_one"));
        assert!(result.content.contains("distributed systems"));
    }
}
