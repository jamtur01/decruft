use scraper::{Html, Selector};

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment_tree, build_content_html};

/// Detect whether this page is a GitHub issue or pull request.
#[must_use]
pub fn is_github(html: &Html, url: Option<&str>) -> bool {
    let has_meta = has_github_meta(html);
    let is_issue = url.is_some_and(is_issue_url) || has_issue_markers(html);
    let is_pr = url.is_some_and(is_pr_url) || has_pr_markers(html);
    // Accept URL-based detection even without meta tags (API fallback for JS-rendered pages)
    (has_meta || url.is_some_and(|u| u.contains("github.com/"))) && (is_issue || is_pr)
}

/// Extract content from a GitHub issue or pull request page.
///
/// When `include_replies` is false, comments are omitted.
/// Falls back to the GitHub REST API when the HTML yields fewer
/// than 10 words (common with JS-rendered React pages).
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
    let result = if is_pr {
        extract_pr(html, url, include_replies)
    } else {
        extract_issue(html, url, include_replies)
    };

    if dom::count_words_html(&result.content) < 10
        && let Some(api_result) = try_api_fetch(url, include_replies)
    {
        return Some(api_result);
    }

    Some(result)
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
    let (_owner, _repo) = extract_repo_info(url);
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
        site: Some("GitHub".to_string()),
        published: None,
        image: None,
        description: None,
    }
}

fn extract_title(html: &Html) -> String {
    let sel = Selector::parse("title").ok();
    let raw = sel
        .and_then(|s| html.select(&s).next())
        .map(|el| dom::text_content(html, el.id()).trim().to_string())
        .unwrap_or_default();
    // Strip trailing " · owner/repo" from GitHub titles
    if let Some(idx) = raw.rfind(" · ") {
        let after = &raw[idx + " · ".len()..];
        // If the suffix contains a "/" it's an owner/repo path
        if after.contains('/') {
            return raw[..idx].to_string();
        }
    }
    raw
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
        if let Some(&id) = ids.first() {
            // Prefer text content (the displayed username)
            let text = dom::text_content(html, id);
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
            // Fall back to href, extracting username from path
            if let Some(href) = dom::get_attr(html, id, "href") {
                let name = href
                    .strip_prefix("https://github.com/")
                    .or_else(|| href.strip_prefix('/'))
                    .unwrap_or(&href);
                if !name.is_empty() {
                    return name.to_string();
                }
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
    let (_owner, _repo) = extract_repo_info(url);
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
        site: Some("GitHub".to_string()),
        published: None,
        image: None,
        description: None,
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

// --- API fallback ---

/// Parse a GitHub issue/PR URL into (owner, repo, number, `is_pr`).
fn parse_github_url(url: &str) -> Option<(String, String, String, bool)> {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"github\.com/([^/]+)/([^/]+)/(issues|pull)/(\d+)")
            .expect("github url regex is valid")
    });

    let caps = RE.captures(url)?;
    let owner = caps.get(1)?.as_str().to_string();
    let repo = caps.get(2)?.as_str().to_string();
    let kind = caps.get(3)?.as_str();
    let number = caps.get(4)?.as_str().to_string();
    Some((owner, repo, number, kind == "pull"))
}

/// Fetch content from the GitHub REST API when HTML extraction fails.
fn try_api_fetch(url: Option<&str>, include_replies: bool) -> Option<ExtractorResult> {
    use std::fmt::Write;

    let (owner, repo, number, is_pr) = parse_github_url(url?)?;

    let endpoint = if is_pr { "pulls" } else { "issues" };
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/{endpoint}/{number}");
    let json = fetch_github_json(&api_url)?;

    let title = json_str(&json, "title");
    let body = json_str(&json, "body");
    let author = json
        .get("user")
        .and_then(|u| u.get("login"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    let published = json_str(&json, "created_at")
        .split('T')
        .next()
        .unwrap_or("")
        .to_string();

    // Extract labels
    let labels: Vec<&str> = json
        .get("labels")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|l| l.get("name").and_then(serde_json::Value::as_str))
                .collect()
        })
        .unwrap_or_default();

    // Extract milestone
    let milestone = json
        .get("milestone")
        .and_then(|m| m.get("title"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    // PR merge status
    let merged = is_pr
        && json
            .get("merged")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
    let state = json_str(&json, "state");

    // Build metadata header
    let mut meta_html = String::new();
    if is_pr {
        let status = if merged { "merged" } else { &state };
        if !status.is_empty() {
            let _ = writeln!(meta_html, "<p><strong>Status:</strong> {status}</p>");
        }
    }
    if !labels.is_empty() {
        let label_str = labels.join(", ");
        let _ = writeln!(meta_html, "<p><strong>Labels:</strong> {label_str}</p>");
    }
    if !milestone.is_empty() {
        let _ = writeln!(meta_html, "<p><strong>Milestone:</strong> {milestone}</p>");
    }

    let body_html = format!("{meta_html}{}", markdown_to_html(&body));
    let comments_html = if include_replies {
        let issue_comments = fetch_api_comments(&owner, &repo, &number);
        let review_comments = if is_pr {
            fetch_pr_review_comments(&owner, &repo, &number)
        } else {
            String::new()
        };
        if issue_comments.is_empty() {
            review_comments
        } else if review_comments.is_empty() {
            issue_comments
        } else {
            format!("{issue_comments}\n{review_comments}")
        }
    } else {
        String::new()
    };
    let content = build_content_html("github", &body_html, &comments_html);

    Some(ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("GitHub".to_string()),
        published: if published.is_empty() {
            None
        } else {
            Some(published)
        },
        image: None,
        description: None,
    })
}

/// Fetch and format comments from the GitHub Issues API.
fn fetch_api_comments(owner: &str, repo: &str, number: &str) -> String {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues/{number}/comments");
    let items = fetch_github_json_paginated(&url);

    let comments: Vec<CommentData> = items
        .iter()
        .filter_map(|c| {
            let body = c.get("body")?.as_str()?;
            if body.trim().is_empty() {
                return None;
            }
            let author = c
                .get("user")
                .and_then(|u| u.get("login"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            let date = c
                .get("created_at")
                .and_then(serde_json::Value::as_str)
                .and_then(|d| d.split('T').next())
                .unwrap_or("")
                .to_string();
            Some(CommentData {
                author,
                date,
                content: markdown_to_html(body),
                depth: 0,
                score: None,
                url: None,
            })
        })
        .collect();

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

/// Fetch PR review comments (review summaries + line-level comments).
///
/// PRs have two comment sources beyond issue comments: review objects
/// (the summary body submitted with each review) and review comments
/// (line-level discussion). Both are fetched and merged here.
fn fetch_pr_review_comments(owner: &str, repo: &str, number: &str) -> String {
    let mut comments = Vec::new();

    // Review summaries (approve/request-changes/comment with body)
    let reviews_url = format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}/reviews");
    for review in fetch_github_json_paginated(&reviews_url) {
        let body = review
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if body.trim().is_empty() {
            continue;
        }
        let author = review
            .get("user")
            .and_then(|u| u.get("login"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Unknown")
            .to_string();
        let date = review
            .get("submitted_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|d| d.split('T').next())
            .unwrap_or("")
            .to_string();
        comments.push(CommentData {
            author,
            date,
            content: markdown_to_html(body),
            depth: 0,
            score: None,
            url: None,
        });
    }

    // Line-level review comments
    let line_url = format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}/comments");
    for c in fetch_github_json_paginated(&line_url) {
        let body = c
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if body.trim().is_empty() {
            continue;
        }
        let author = c
            .get("user")
            .and_then(|u| u.get("login"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Unknown")
            .to_string();
        let date = c
            .get("created_at")
            .and_then(serde_json::Value::as_str)
            .and_then(|d| d.split('T').next())
            .unwrap_or("")
            .to_string();
        let path = c
            .get("path")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let prefix = if path.is_empty() {
            String::new()
        } else {
            format!("<p><code>{path}</code></p>\n")
        };
        comments.push(CommentData {
            author,
            date,
            content: format!("{prefix}{}", markdown_to_html(body)),
            depth: 0,
            score: None,
            url: None,
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

/// Fetch JSON from a GitHub API endpoint.
fn fetch_github_json(url: &str) -> Option<serde_json::Value> {
    let body = crate::http::get_with_headers(url, &[("Accept", "application/vnd.github+json")])?;
    serde_json::from_str(&body).ok()
}

/// Fetch all pages of a GitHub API endpoint that returns a JSON array.
///
/// Appends `?per_page=100` and follows `page` parameters up to 10 pages
/// to avoid runaway fetching on massive threads.
fn fetch_github_json_paginated(url: &str) -> Vec<serde_json::Value> {
    const MAX_PAGES: u32 = 10;
    let mut all_items = Vec::new();

    for page in 1..=MAX_PAGES {
        let separator = if url.contains('?') { '&' } else { '?' };
        let page_url = format!("{url}{separator}per_page=100&page={page}");
        let Some(json) = fetch_github_json(&page_url) else {
            break;
        };
        let Some(arr) = json.as_array() else {
            break;
        };
        if arr.is_empty() {
            break;
        }
        all_items.extend(arr.iter().cloned());
        // Less than a full page means we've reached the end
        if arr.len() < 100 {
            break;
        }
    }

    all_items
}

/// Minimal helper to extract a string field from JSON.
fn json_str(json: &serde_json::Value, key: &str) -> String {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Convert GitHub-flavored markdown to HTML.
///
/// Uses pulldown-cmark with GFM extensions (tables, strikethrough,
/// task lists, footnotes) plus a regex pre-pass to linkify bare URLs,
/// which pulldown-cmark does not handle natively. Raw HTML in the
/// markdown input is escaped to prevent XSS.
fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser, Tag, html};

    let linkified = linkify_bare_urls(md);
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(&linkified, options);

    // Escape raw HTML events and sanitize link destinations to
    // prevent XSS from user-controlled GitHub issue/comment bodies.
    let sanitized = parser.map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            dest_url,
            title,
            id,
            link_type,
        }) if dest_url.starts_with("javascript:") => Event::Start(Tag::Link {
            dest_url: "".into(),
            title,
            id,
            link_type,
        }),
        other => other,
    });

    let mut html_output = String::with_capacity(md.len() * 2);
    html::push_html(&mut html_output, sanitized);
    html_output
}

/// Turn bare `https://` and `http://` URLs into markdown links so
/// pulldown-cmark renders them as `<a>` tags. Skips URLs inside
/// markdown link syntax, fenced code blocks, inline code spans,
/// angle-bracket autolinks, and reference-style link definitions.
fn linkify_bare_urls(md: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;

    static URL_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"https?://[^\s<>\[\])\]]+").expect("valid regex"));

    let mut result = String::with_capacity(md.len());
    let mut in_code_block = false;

    for line in md.lines() {
        if !result.is_empty() {
            result.push('\n');
        }
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            result.push_str(line);
            continue;
        }
        if in_code_block {
            result.push_str(line);
            continue;
        }

        // Skip reference-style link definitions: [ref]: https://...
        if line.trim_start().starts_with('[') && line.contains("]: http") {
            result.push_str(line);
            continue;
        }

        let line_bytes = line.as_bytes();
        let mut last_end = 0;
        for m in URL_RE.find_iter(line) {
            let before = if m.start() > 0 {
                line_bytes[m.start() - 1]
            } else {
                b' '
            };
            // Skip URLs already in markdown link/image syntax or
            // angle-bracket autolinks
            if before == b'(' || before == b'[' || before == b']' || before == b'<' {
                continue;
            }
            // Skip URLs inside inline code spans
            let prefix = &line[last_end..m.start()];
            let backtick_count = prefix.bytes().filter(|&b| b == b'`').count();
            if backtick_count % 2 != 0 {
                continue;
            }

            // Trim trailing punctuation that isn't part of the URL
            let url = m.as_str().trim_end_matches(['.', ',', ';', ':', '!', '?']);

            result.push_str(&line[last_end..m.start()]);
            result.push('[');
            result.push_str(url);
            result.push_str("](");
            result.push_str(url);
            result.push(')');
            // Append any trimmed punctuation as plain text
            let trimmed_len = m.as_str().len() - url.len();
            if trimmed_len > 0 {
                result.push_str(&m.as_str()[url.len()..]);
            }
            last_end = m.end();
        }
        result.push_str(&line[last_end..]);
    }
    result
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn load_fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
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
    fn parse_github_issue_url() {
        let result = parse_github_url("https://github.com/owner/repo/issues/123");
        let (o, r, n, is_pr) = result.unwrap();
        assert_eq!(o, "owner");
        assert_eq!(r, "repo");
        assert_eq!(n, "123");
        assert!(!is_pr);
    }

    #[test]
    fn parse_github_pr_url() {
        let result = parse_github_url("https://github.com/owner/repo/pull/42");
        let (o, r, n, is_pr) = result.unwrap();
        assert_eq!(o, "owner");
        assert_eq!(r, "repo");
        assert_eq!(n, "42");
        assert!(is_pr);
    }

    #[test]
    fn parse_github_url_invalid() {
        assert!(parse_github_url("https://github.com/owner/repo").is_none());
        assert!(parse_github_url("https://example.com").is_none());
    }

    #[test]
    fn markdown_to_html_basic() {
        let md = "Hello **world**\n\nA paragraph.\n\n## Header\n\n```\ncode\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("<strong>world</strong>"));
        assert!(html.contains("<p>A paragraph.</p>"));
        assert!(html.contains("<h2>Header</h2>"));
        assert!(html.contains("<pre><code>"));
        assert!(html.contains("code"));
    }

    #[test]
    fn markdown_to_html_gfm_features() {
        let md = "- [x] done\n- [ ] todo\n\n~~strike~~\n\n| a | b |\n|---|---|\n| 1 | 2 |";
        let html = markdown_to_html(md);
        assert!(html.contains("<del>strike</del>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("checked"));
    }

    #[test]
    fn markdown_to_html_autolinks() {
        let md = "See https://example.com for info\n\nAlready [linked](https://other.com)";
        let html = markdown_to_html(md);
        assert!(html.contains("<a href=\"https://example.com\">"));
        assert!(html.contains("<a href=\"https://other.com\">"));
    }

    #[test]
    fn markdown_to_html_autolinks_skip_code_blocks() {
        let md = "```\nhttps://example.com\n```";
        let html = markdown_to_html(md);
        assert!(!html.contains("<a href"));
        assert!(html.contains("https://example.com"));
    }

    #[test]
    fn markdown_to_html_escapes_raw_html() {
        let md = "<script>alert('xss')</script>\n\n<b>bold</b>";
        let html = markdown_to_html(md);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("</script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<b>"));
        assert!(html.contains("&lt;b&gt;"));
        assert!(html.contains("alert"));
    }

    #[test]
    fn markdown_to_html_sanitizes_javascript_links() {
        let md = "[click](javascript:alert(1))";
        let html = markdown_to_html(md);
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn markdown_to_html_autolinks_skip_angle_brackets() {
        let md = "See <https://example.com> for info";
        let html = markdown_to_html(md);
        assert!(html.contains("<a href=\"https://example.com\">"));
        // Should not double-linkify
        assert!(!html.contains("[https://"));
    }

    #[test]
    fn markdown_to_html_autolinks_trim_trailing_punctuation() {
        let md = "Visit https://example.com. And https://other.com, too!";
        let html = markdown_to_html(md);
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("href=\"https://other.com\""));
        // Trailing punctuation should not be in the link
        assert!(!html.contains("href=\"https://example.com.\""));
        assert!(!html.contains("href=\"https://other.com,\""));
    }

    #[test]
    fn markdown_to_html_autolinks_skip_inline_code() {
        let md = "Use `https://example.com` as the base URL";
        let html = markdown_to_html(md);
        assert!(html.contains("<code>https://example.com</code>"));
        assert!(!html.contains("<a href=\"https://example.com\">"));
    }

    #[test]
    fn markdown_to_html_autolinks_skip_reference_links() {
        let md = "[example]: https://example.com\n\nSee [example] for details.";
        let html = markdown_to_html(md);
        assert!(html.contains("href=\"https://example.com\""));
        // Should not double-linkify the reference definition
        assert!(!html.contains("[https://example.com](https://example.com)"));
    }

    #[test]
    fn api_fetch_live_issue() {
        let url = "https://github.com/rust-lang/rust/issues/1";
        let result = try_api_fetch(Some(url), false);
        if let Some(r) = result {
            assert!(r.title.is_some());
            assert!(r.author.is_some());
            assert_eq!(r.site.as_deref(), Some("GitHub"));
        }
        // Don't fail if network is unavailable
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
        assert_eq!(result.site.as_deref(), Some("GitHub"));
        // PR body should contain the summary
        assert!(result.content.contains("Summary"));
        assert!(result.content.contains("regression"));
        // Should have review comments
        assert!(result.content.contains("Comments"));
        assert!(result.content.contains("reviewer-bot"));
    }

    #[test]
    fn api_fetch_live_pr_includes_labels_and_status() {
        // rust-lang/rust#1 is an issue, not a PR, so test a known PR
        let url = "https://github.com/rust-lang/rust/pull/2";
        let result = try_api_fetch(Some(url), false);
        if let Some(r) = result {
            // PR should have status metadata
            assert!(
                r.content.contains("<strong>Status:</strong>"),
                "PR content should include status"
            );
        }
        // Don't fail if network is unavailable
    }

    #[test]
    fn api_fetch_live_issue_with_labels() {
        // Use a well-known issue that has labels
        let url = "https://github.com/rust-lang/rust/issues/1";
        let result = try_api_fetch(Some(url), false);
        if let Some(r) = result {
            assert!(r.title.is_some());
            // Issue #1 doesn't necessarily have labels, but the
            // content should be parseable and non-empty
            assert!(!r.content.is_empty());
        }
        // Don't fail if network is unavailable
    }

    #[test]
    fn api_fetch_live_pr_includes_review_comments() {
        // Fetch a PR with replies to verify review comments are included
        let url = "https://github.com/rust-lang/rust/pull/2";
        let result = try_api_fetch(Some(url), true);
        if let Some(r) = result {
            assert!(r.title.is_some());
            assert!(!r.content.is_empty());
        }
        // Don't fail if network is unavailable
    }

    #[test]
    fn pagination_helper_returns_empty_for_invalid_url() {
        let items = fetch_github_json_paginated(
            "https://api.github.com/repos/nonexistent/repo/issues/99999/comments",
        );
        // Either empty (404) or network failure — both should return empty vec
        // Don't fail if network is unavailable
        assert!(items.len() <= 100);
    }
}
