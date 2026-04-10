use scraper::{Html, Selector};

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment_tree, build_content_html};

/// Detect whether this page is a Reddit page (old.reddit.com or new).
#[must_use]
pub fn is_reddit(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("reddit.com")) {
        return true;
    }
    has_shreddit_post(html) || is_old_reddit(html)
}

/// Extract content from a Reddit page.
///
/// When `include_replies` is false, comments are omitted.
#[must_use]
pub fn extract_reddit(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    if !is_reddit(html, url) {
        return None;
    }
    if is_old_reddit(html) {
        extract_old_reddit(html, url, include_replies)
    } else if has_shreddit_post(html) {
        extract_new_reddit(html, url, include_replies)
    } else {
        None
    }
}

fn has_shreddit_post(html: &Html) -> bool {
    Selector::parse("shreddit-post")
        .ok()
        .is_some_and(|sel| html.select(&sel).next().is_some())
}

fn is_old_reddit(html: &Html) -> bool {
    Selector::parse(".thing.link")
        .ok()
        .is_some_and(|sel| html.select(&sel).next().is_some())
}

fn get_subreddit(url: Option<&str>) -> String {
    use std::sync::LazyLock;
    static SUBREDDIT_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"/r/([^/]+)").expect("subreddit regex is valid"));

    let Some(u) = url else {
        return String::new();
    };
    SUBREDDIT_RE
        .captures(u)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_default()
}

// --- Old Reddit extraction ---

fn extract_old_reddit(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let thing_ids = dom::select_ids(html, ".thing.link");
    let thing_id = thing_ids.first().copied()?;

    let title = extract_old_reddit_title(html, thing_id);
    let author = dom::get_attr(html, thing_id, "data-author").unwrap_or_default();
    let subreddit =
        dom::get_attr(html, thing_id, "data-subreddit").unwrap_or_else(|| get_subreddit(url));

    let body_ids = dom::select_within(html, thing_id, ".usertext-body .md");
    let body = body_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default();

    let comments = if include_replies {
        extract_old_reddit_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("reddit", &body, &comments);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some(format!("r/{subreddit}")),
    })
}

fn extract_old_reddit_title(html: &Html, thing_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, thing_id, "a.title");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_old_reddit_comments(html: &Html) -> String {
    let area_ids = dom::select_ids(html, ".commentarea .sitetable");
    let Some(&area_id) = area_ids.first() else {
        return String::new();
    };
    let comments = collect_old_reddit_comments(html, area_id, 0);
    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn collect_old_reddit_comments(
    html: &Html,
    container_id: ego_tree::NodeId,
    depth: usize,
) -> Vec<CommentData> {
    // select_within doesn't support :scope; find direct child comments manually
    let comment_ids = find_direct_child_comments(html, container_id);
    let mut result = Vec::new();

    for cid in comment_ids {
        let author = dom::get_attr(html, cid, "data-author").unwrap_or_default();
        let permalink = dom::get_attr(html, cid, "data-permalink").unwrap_or_default();

        let (date, score) = extract_old_comment_meta(html, cid);

        let body_ids = dom::select_within(html, cid, ".usertext-body .md");
        let body = body_ids
            .first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
            .unwrap_or_default();

        if !body.is_empty() {
            let url = if permalink.is_empty() {
                None
            } else {
                Some(format!("https://reddit.com{permalink}"))
            };
            result.push(CommentData {
                author,
                date,
                content: body,
                depth,
                score: if score.is_empty() { None } else { Some(score) },
                url,
            });
        }

        // Recurse into child comments
        let child_ids = dom::select_within(html, cid, ".child > .sitetable");
        if let Some(&child_container) = child_ids.first() {
            result.extend(collect_old_reddit_comments(
                html,
                child_container,
                depth + 1,
            ));
        }
    }

    result
}

/// Find direct `.thing.comment` children of a container.
fn find_direct_child_comments(
    html: &Html,
    container_id: ego_tree::NodeId,
) -> Vec<ego_tree::NodeId> {
    let Some(node) = html.tree.get(container_id) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for child in node.children() {
        if let scraper::Node::Element(el) = child.value() {
            let classes = el.attr("class").unwrap_or("");
            if classes.contains("thing") && classes.contains("comment") {
                result.push(child.id());
            }
        }
    }
    result
}

fn extract_old_comment_meta(html: &Html, comment_id: ego_tree::NodeId) -> (String, String) {
    let score_ids = dom::select_within(html, comment_id, ".score.unvoted");
    let score = score_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default();

    let time_ids = dom::select_within(html, comment_id, "time[datetime]");
    let date = time_ids
        .first()
        .and_then(|&id| dom::get_attr(html, id, "datetime"))
        .and_then(|dt| dt.split('T').next().map(String::from))
        .unwrap_or_default();

    (date, score)
}

// --- New Reddit extraction (shreddit) ---

fn extract_new_reddit(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let title_ids = dom::select_ids(html, "h1");
    let title = title_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default();

    let subreddit = get_subreddit(url);

    let post_ids = dom::select_ids(html, "shreddit-post");
    let post_id = post_ids.first().copied()?;
    let author = dom::get_attr(html, post_id, "author").unwrap_or_default();

    let text_ids = dom::select_within(html, post_id, "[slot=\"text-body\"]");
    let body = text_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default();

    let comments = if include_replies {
        extract_new_reddit_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("reddit", &body, &comments);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some(format!("r/{subreddit}")),
    })
}

fn extract_new_reddit_comments(html: &Html) -> String {
    let comment_ids = dom::select_ids(html, "shreddit-comment");
    if comment_ids.is_empty() {
        return String::new();
    }

    let mut comments = Vec::new();
    for &cid in &comment_ids {
        let depth = dom::get_attr(html, cid, "depth")
            .and_then(|d| d.parse::<usize>().ok())
            .unwrap_or(0);
        let author = dom::get_attr(html, cid, "author").unwrap_or_default();
        let score = dom::get_attr(html, cid, "score").unwrap_or_default();
        let permalink = dom::get_attr(html, cid, "permalink").unwrap_or_default();

        let slot_ids = dom::select_within(html, cid, "[slot=\"comment\"]");
        let body = slot_ids
            .first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
            .unwrap_or_default();

        if body.is_empty() {
            continue;
        }

        comments.push(CommentData {
            author,
            date: String::new(),
            content: body,
            depth,
            score: if score.is_empty() {
                None
            } else {
                Some(format!("{score} points"))
            },
            url: if permalink.is_empty() {
                None
            } else {
                Some(format!("https://reddit.com{permalink}"))
            },
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
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
    fn extract_old_reddit_post_and_comments() {
        let html_str =
            load_fixture("comments--old.reddit.com-r-test-comments-abc123-test_post.html");
        let url = Some("https://old.reddit.com/r/test/comments/abc123/test_post");
        let html = Html::parse_document(&html_str);

        assert!(is_reddit(&html, url));
        let result = extract_reddit(&html, url, true).unwrap();

        assert_eq!(result.title.as_deref(), Some("Test Post"));
        assert_eq!(result.author.as_deref(), Some("poster_user"));
        assert_eq!(result.site.as_deref(), Some("r/test"));
        // Post body
        assert!(result.content.contains("post body with some content"));
        // Comments section
        assert!(result.content.contains("Comments"));
        assert!(result.content.contains("user_alpha"));
        assert!(result.content.contains("top-level comment"));
        // Nested comment
        assert!(result.content.contains("user_beta"));
        assert!(result.content.contains("Great point"));
    }
}
