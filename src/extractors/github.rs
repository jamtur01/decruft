use scraper::{Html, Selector};

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment_tree, build_content_html};

/// Detect whether this page is a GitHub issue or pull request.
#[must_use]
pub fn is_github(html: &Html, url: Option<&str>) -> bool {
    let has_meta = has_github_meta(html);
    if !has_meta {
        return false;
    }
    let is_issue = url.is_some_and(is_issue_url) || has_issue_markers(html);
    let is_pr = url.is_some_and(is_pr_url) || has_pr_markers(html);
    is_issue || is_pr
}

/// Extract content from a GitHub issue or pull request page.
///
/// When `include_replies` is false, comments are omitted.
#[must_use]
pub fn extract_github(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    if !is_github(html, url) {
        return None;
    }

    let is_pr = url.is_some_and(is_pr_url) || has_pr_markers(html);
    if is_pr {
        Some(extract_pr(html, url, include_replies))
    } else {
        Some(extract_issue(html, url, include_replies))
    }
}

fn has_github_meta(html: &Html) -> bool {
    let selectors = [
        "meta[name=\"expected-hostname\"][content=\"github.com\"]",
        "meta[name=\"octolytics-url\"]",
        "meta[name=\"github-keyboard-shortcuts\"]",
    ];
    selectors.iter().any(|s| {
        Selector::parse(s)
            .ok()
            .is_some_and(|sel| html.select(&sel).next().is_some())
    })
}

fn is_issue_url(url: &str) -> bool {
    url.contains("/issues/")
}

fn is_pr_url(url: &str) -> bool {
    url.contains("/pull/")
}

fn has_issue_markers(html: &Html) -> bool {
    let selectors = [
        "[data-testid=\"issue-metadata-sticky\"]",
        "[data-testid=\"issue-title\"]",
    ];
    selectors.iter().any(|s| {
        Selector::parse(s)
            .ok()
            .is_some_and(|sel| html.select(&sel).next().is_some())
    })
}

fn has_pr_markers(html: &Html) -> bool {
    let selectors = [
        ".pull-discussion-timeline",
        ".discussion-timeline",
        ".gh-header-title",
    ];
    selectors.iter().any(|s| {
        Selector::parse(s)
            .ok()
            .is_some_and(|sel| html.select(&sel).next().is_some())
    })
}

fn extract_repo_info(url: Option<&str>) -> (String, String) {
    use std::sync::LazyLock;
    static REPO_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"github\.com/([^/]+)/([^/]+)").expect("github repo regex is valid")
    });

    let Some(u) = url else {
        return (String::new(), String::new());
    };
    REPO_RE
        .captures(u)
        .map(|caps| {
            (
                caps.get(1)
                    .map_or(String::new(), |m| m.as_str().to_string()),
                caps.get(2)
                    .map_or(String::new(), |m| m.as_str().to_string()),
            )
        })
        .unwrap_or_default()
}

// --- Issue extraction ---

fn extract_issue(html: &Html, url: Option<&str>, include_replies: bool) -> ExtractorResult {
    let (owner, repo) = extract_repo_info(url);
    let title = extract_title(html);
    let (body, author) = extract_issue_body(html);
    let comments = if include_replies {
        extract_issue_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("github", &body, &comments);

    ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some(format!("GitHub - {owner}/{repo}")),
    }
}

fn extract_title(html: &Html) -> String {
    let sel = Selector::parse("title").ok();
    sel.and_then(|s| html.select(&s).next())
        .map(|el| dom::text_content(html, el.id()).trim().to_string())
        .unwrap_or_default()
}

fn extract_issue_body(html: &Html) -> (String, String) {
    let container_sel = "[data-testid=\"issue-viewer-issue-container\"]";
    let container_ids = dom::select_ids(html, container_sel);
    let Some(&container_id) = container_ids.first() else {
        return (String::new(), String::new());
    };

    let author = extract_issue_author(html, container_id);

    let body_sel = "[data-testid=\"issue-body-viewer\"] [data-testid=\"markdown-body\"]";
    let body_ids = dom::select_ids(html, body_sel);
    let body = body_ids
        .first()
        .map(|&id| dom::inner_html(html, id))
        .unwrap_or_default();

    (body.trim().to_string(), author)
}

fn extract_issue_author(html: &Html, container_id: ego_tree::NodeId) -> String {
    let author_selectors = [
        "[data-testid=\"issue-body-header-author\"]",
        "a[data-testid=\"avatar-link\"]",
    ];
    for sel_str in &author_selectors {
        let ids = dom::select_within(html, container_id, sel_str);
        if let Some(&id) = ids.first()
            && let Some(href) = dom::get_attr(html, id, "href")
        {
            let name = href.strip_prefix('/').unwrap_or(&href);
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    String::new()
}

fn extract_issue_comments(html: &Html) -> String {
    let timeline_sel = "[data-wrapper-timeline-id]";
    let timeline_ids = dom::select_ids(html, timeline_sel);
    let mut comments = Vec::new();

    for &timeline_id in &timeline_ids {
        if let Some(comment) = extract_single_issue_comment(html, timeline_id) {
            comments.push(comment);
        }
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn extract_single_issue_comment(html: &Html, timeline_id: ego_tree::NodeId) -> Option<CommentData> {
    let react_ids = dom::select_within(html, timeline_id, ".react-issue-comment");
    let comment_container = react_ids.first().copied().unwrap_or(timeline_id);

    let author = extract_comment_author(html, comment_container);
    let date = extract_relative_time(html, comment_container);

    let body_ids = dom::select_within(html, comment_container, "[data-testid=\"markdown-body\"]");
    let body = body_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default();

    if body.is_empty() {
        return None;
    }

    Some(CommentData {
        author,
        date,
        content: body,
        depth: 0,
        score: None,
        url: None,
    })
}

fn extract_comment_author(html: &Html, container_id: ego_tree::NodeId) -> String {
    let selectors = [
        "[data-testid=\"avatar-link\"]",
        "a[href^=\"/\"][data-hovercard-url]",
    ];
    for sel_str in &selectors {
        let ids = dom::select_within(html, container_id, sel_str);
        if let Some(&id) = ids.first()
            && let Some(href) = dom::get_attr(html, id, "href")
        {
            let name = href.strip_prefix('/').unwrap_or(&href);
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn extract_relative_time(html: &Html, container_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, container_id, "relative-time");
    ids.first()
        .and_then(|&id| dom::get_attr(html, id, "datetime"))
        .and_then(|dt| dt.split('T').next().map(String::from))
        .unwrap_or_default()
}

// --- PR extraction ---

fn extract_pr(html: &Html, url: Option<&str>, include_replies: bool) -> ExtractorResult {
    let (owner, repo) = extract_repo_info(url);
    let title = extract_title(html);
    let (body, author) = extract_pr_body(html);
    let comments = if include_replies {
        extract_pr_comments(html)
    } else {
        String::new()
    };
    let content = build_content_html("github", &body, &comments);

    ExtractorResult {
        content,
        title: Some(title),
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some(format!("GitHub - {owner}/{repo}")),
    }
}

fn extract_pr_body(html: &Html) -> (String, String) {
    let pr_sel = "[id^=\"pullrequest-\"]";
    let pr_ids = dom::select_ids(html, pr_sel);
    let pr_container = pr_ids.first().copied();

    let body_sel = ".comment-body.markdown-body";
    let body = if let Some(container_id) = pr_container {
        let ids = dom::select_within(html, container_id, body_sel);
        ids.first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
    } else {
        let ids = dom::select_ids(html, body_sel);
        ids.first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
    }
    .unwrap_or_default();

    let author = pr_container
        .map(|cid| extract_pr_author(html, cid))
        .unwrap_or_default();

    (body, author)
}

fn extract_pr_author(html: &Html, container_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, container_id, ".author");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_pr_comments(html: &Html) -> String {
    let comment_sel = ".timeline-comment, .review-comment";
    let all_ids = dom::select_ids(html, comment_sel);
    let pr_body_ids = dom::select_ids(html, "[id^=\"pullrequest-\"]");
    let pr_body_id = pr_body_ids.first().copied();

    let mut comments = Vec::new();
    for &cid in &all_ids {
        if pr_body_id.is_some_and(|pb| pb == cid || dom::is_ancestor(html, cid, pb)) {
            continue;
        }
        if let Some(comment) = extract_single_pr_comment(html, cid) {
            comments.push(comment);
        }
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn extract_single_pr_comment(html: &Html, comment_id: ego_tree::NodeId) -> Option<CommentData> {
    let author = extract_pr_author(html, comment_id);
    let date = extract_relative_time(html, comment_id);

    let body_ids = dom::select_within(html, comment_id, ".comment-body.markdown-body");
    let body = body_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default();

    if body.is_empty() {
        return None;
    }

    Some(CommentData {
        author,
        date,
        content: body,
        depth: 0,
        score: None,
        url: None,
    })
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

    fn url_from_fixture(html_str: &str) -> Option<String> {
        let start = html_str.find("<!-- {")?;
        let comment_start = start + "<!-- ".len();
        let end = html_str[comment_start..].find(" -->")?;
        let json_str = &html_str[comment_start..comment_start + end];
        let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
        val.get("url").and_then(|v| v.as_str()).map(String::from)
    }

    #[test]
    fn extract_github_issue() {
        let html_str = load_fixture("general--github.com-issue-56.html");
        let url = url_from_fixture(&html_str);
        let html = Html::parse_document(&html_str);

        assert!(is_github(&html, url.as_deref()));
        let result = extract_github(&html, url.as_deref(), true).unwrap();

        assert!(result.title.as_ref().unwrap().contains("Issue #56"));
        assert!(result.site.as_ref().unwrap().contains("GitHub"));
        // Issue body content (from the issue description)
        assert!(result.content.contains("defuddle-cloudflare"));
        // Should have comments
        assert!(result.content.contains("Comments"));
    }

    #[test]
    fn extract_github_pr() {
        let html_str = load_fixture("general--github.com-test-owner-test-repo-pull-42.html");
        let url = url_from_fixture(&html_str);
        let html = Html::parse_document(&html_str);

        assert!(is_github(&html, url.as_deref()));
        let result = extract_github(&html, url.as_deref(), true).unwrap();

        assert!(result.title.unwrap().contains("Pull Request #42"));
        assert_eq!(result.author.as_deref(), Some("author-one"));
        assert!(result.site.unwrap().contains("test-owner/test-repo"));
        // PR body should contain the summary
        assert!(result.content.contains("Summary"));
        assert!(result.content.contains("regression"));
        // Should have review comments
        assert!(result.content.contains("Comments"));
        assert!(result.content.contains("reviewer-bot"));
    }
}
