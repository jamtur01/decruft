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
    fatitem_ids.first().copied()?;

    let is_comment_page = detect_comment_page(html);

    if is_comment_page {
        extract_comment_page(html, url, include_replies)
    } else {
        extract_story_page(html, url, include_replies)
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
    let _published = extract_date(html, fatitem);
    let post_content = extract_story_content(html, fatitem);
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
