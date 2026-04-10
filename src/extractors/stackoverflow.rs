use scraper::Html;

use crate::dom;

use super::ExtractorResult;
use super::comments::{CommentData, build_comment_tree, build_content_html};

/// Detect whether this page is a Stack Overflow or Stack Exchange page.
#[must_use]
pub fn is_stackoverflow(url: Option<&str>) -> bool {
    let Some(u) = url else { return false };
    u.contains("stackoverflow.com/questions/")
        || (u.contains("stackexchange.com/questions/") && u.contains(".stackexchange.com"))
        || is_known_se_site(u)
}

/// Known Stack Exchange network sites with custom domains.
fn is_known_se_site(url: &str) -> bool {
    let se_domains = [
        "serverfault.com/questions/",
        "superuser.com/questions/",
        "askubuntu.com/questions/",
        "mathoverflow.net/questions/",
    ];
    se_domains.iter().any(|d| url.contains(d))
}

/// Extract content from a Stack Overflow/Stack Exchange page.
///
/// When `include_replies` is false, answers are omitted.
/// Falls back to the Stack Exchange API when HTML extraction
/// yields fewer than 10 words.
#[must_use]
pub fn extract_stackoverflow(
    html: &Html,
    url: Option<&str>,
    include_replies: bool,
) -> Option<ExtractorResult> {
    if !is_stackoverflow(url) {
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
    let question_body = extract_question_body(html);

    if question_body.is_empty() {
        return None;
    }

    let author = extract_question_author(html);
    let answers_html = if include_replies {
        extract_answers(html)
    } else {
        String::new()
    };

    let content = build_content_html("stackoverflow", &question_body, &answers_html);

    Some(ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Stack Overflow".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

fn extract_title(html: &Html) -> String {
    let ids = dom::select_ids(html, "#question-header h1");
    ids.first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_question_body(html: &Html) -> String {
    let ids = dom::select_ids(html, ".question .js-post-body");
    ids.first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_question_author(html: &Html) -> String {
    let ids = dom::select_ids(html, ".question .user-details a");
    ids.last()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_answers(html: &Html) -> String {
    let answer_ids = dom::select_ids(html, ".answer");
    if answer_ids.is_empty() {
        return String::new();
    }

    let mut comments = Vec::new();
    for &aid in &answer_ids {
        let vote_count = extract_vote_count(html, aid);
        let accepted = is_accepted_answer(html, aid);
        let author = extract_answer_author(html, aid);

        let body_ids = dom::select_within(html, aid, ".js-post-body");
        let body = body_ids
            .first()
            .map(|&id| dom::inner_html(html, id).trim().to_string())
            .unwrap_or_default();

        if body.is_empty() {
            continue;
        }

        let mut score_parts = Vec::new();
        score_parts.push(format!("{vote_count} votes"));
        if accepted {
            score_parts.push("accepted".to_string());
        }

        comments.push(CommentData {
            author,
            date: String::new(),
            content: body,
            depth: 0,
            score: Some(score_parts.join(", ")),
            url: None,
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn extract_vote_count(html: &Html, container: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, container, ".js-vote-count");
    ids.first().map_or_else(
        || "0".to_string(),
        |&id| dom::text_content(html, id).trim().to_string(),
    )
}

fn is_accepted_answer(html: &Html, answer_id: ego_tree::NodeId) -> bool {
    dom::has_class(html, answer_id, "accepted-answer")
}

fn extract_answer_author(html: &Html, answer_id: ego_tree::NodeId) -> String {
    let ids = dom::select_within(html, answer_id, ".user-details a");
    ids.last()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default()
}

// --- API fallback ---

const SE_API_BASE: &str = "https://api.stackexchange.com/2.3";
const SE_MAX_ANSWERS: usize = 20;

/// Extract the question ID from a Stack Exchange URL.
fn parse_question_id(url: &str) -> Option<&str> {
    // .../questions/12345/... or .../questions/12345
    let after_q = url.split("/questions/").nth(1)?;
    let id = after_q.split('/').next()?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(id)
}

/// Extract the API site name from a URL.
fn parse_site_name(url: &str) -> Option<String> {
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = host.split('/').next()?;

    if host.contains("stackoverflow.com") {
        return Some("stackoverflow".to_string());
    }
    if host.contains("serverfault.com") {
        return Some("serverfault".to_string());
    }
    if host.contains("superuser.com") {
        return Some("superuser".to_string());
    }
    if host.contains("askubuntu.com") {
        return Some("askubuntu".to_string());
    }
    if host.contains("mathoverflow.net") {
        return Some("mathoverflow".to_string());
    }
    // {name}.stackexchange.com -> {name}
    if let Some(name) = host.strip_suffix(".stackexchange.com") {
        return Some(name.to_string());
    }
    None
}

/// Fetch content from the Stack Exchange API.
fn try_api_fetch(url: Option<&str>, include_replies: bool) -> Option<ExtractorResult> {
    let u = url?;
    let id = parse_question_id(u)?;
    let site = parse_site_name(u)?;

    let question_json = fetch_se_question(id, &site)?;
    let items = question_json
        .get("items")
        .and_then(serde_json::Value::as_array)?;
    let item = items.first()?;

    build_from_api(item, id, &site, include_replies)
}

fn build_from_api(
    item: &serde_json::Value,
    question_id: &str,
    site: &str,
    include_replies: bool,
) -> Option<ExtractorResult> {
    let title = se_json_str(item, "title");
    let body = item
        .get("body")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if body.trim().is_empty() {
        return None;
    }

    let author = item
        .get("owner")
        .and_then(|o| o.get("display_name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();

    let answers_html = if include_replies {
        fetch_answers_from_api(question_id, site)
    } else {
        String::new()
    };

    let content = build_content_html(
        "stackoverflow",
        &format!("<div class=\"post-text\">{body}</div>"),
        &answers_html,
    );

    Some(ExtractorResult {
        content,
        title: if title.is_empty() { None } else { Some(title) },
        author: if author.is_empty() {
            None
        } else {
            Some(author)
        },
        site: Some("Stack Overflow".to_string()),
        published: None,
        image: None,
        description: None,
    })
}

fn fetch_answers_from_api(question_id: &str, site: &str) -> String {
    let url = format!(
        "{SE_API_BASE}/questions/{question_id}/answers\
         ?site={site}&filter=withbody&order=desc&sort=votes\
         &pagesize={SE_MAX_ANSWERS}"
    );
    let Some(json) = fetch_se_json(&url) else {
        return String::new();
    };
    let Some(items) = json.get("items").and_then(serde_json::Value::as_array) else {
        return String::new();
    };

    let mut comments = Vec::new();
    for answer in items {
        let body = answer
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if body.trim().is_empty() {
            continue;
        }

        let author = answer
            .get("owner")
            .and_then(|o| o.get("display_name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        let score = answer
            .get("score")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        let accepted = answer
            .get("is_accepted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let mut score_str = format!("{score} votes");
        if accepted {
            score_str.push_str(", accepted");
        }

        comments.push(CommentData {
            author,
            date: String::new(),
            content: body.to_string(),
            depth: 0,
            score: Some(score_str),
            url: None,
        });
    }

    if comments.is_empty() {
        return String::new();
    }
    build_comment_tree(&comments)
}

fn fetch_se_question(id: &str, site: &str) -> Option<serde_json::Value> {
    let url = format!(
        "{SE_API_BASE}/questions/{id}\
         ?site={site}&filter=withbody&order=desc&sort=activity"
    );
    fetch_se_json(&url)
}

fn fetch_se_json(url: &str) -> Option<serde_json::Value> {
    let output = std::process::Command::new("curl")
        .args(["--compressed", "-sL", "--max-time", "10", url])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&body).ok()
}

fn se_json_str(json: &serde_json::Value, key: &str) -> String {
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
    fn detect_stackoverflow_url() {
        assert!(is_stackoverflow(Some(
            "https://stackoverflow.com/questions/12345/how-to-foo"
        )));
        assert!(is_stackoverflow(Some(
            "https://gaming.stackexchange.com/questions/999/bar"
        )));
        assert!(is_stackoverflow(Some(
            "https://serverfault.com/questions/42/baz"
        )));
    }

    #[test]
    fn reject_non_stackoverflow_url() {
        assert!(!is_stackoverflow(Some("https://example.com")));
        assert!(!is_stackoverflow(Some(
            "https://stackoverflow.com/users/123"
        )));
        assert!(!is_stackoverflow(None));
    }

    #[test]
    fn parse_question_id_valid() {
        assert_eq!(
            parse_question_id("https://stackoverflow.com/questions/12345/how-to"),
            Some("12345")
        );
        assert_eq!(
            parse_question_id("https://unix.stackexchange.com/questions/999"),
            Some("999")
        );
    }

    #[test]
    fn parse_question_id_invalid() {
        assert!(parse_question_id("https://example.com").is_none());
        assert!(parse_question_id("https://stackoverflow.com/users/123").is_none());
    }

    #[test]
    fn parse_site_name_variants() {
        assert_eq!(
            parse_site_name("https://stackoverflow.com/questions/1"),
            Some("stackoverflow".to_string())
        );
        assert_eq!(
            parse_site_name("https://serverfault.com/questions/1"),
            Some("serverfault".to_string())
        );
        assert_eq!(
            parse_site_name("https://gaming.stackexchange.com/questions/1"),
            Some("gaming".to_string())
        );
    }

    #[test]
    fn extract_from_html_basic() {
        let html_str = r#"
        <html>
        <body>
        <div id="question-header"><h1>How to foo?</h1></div>
        <div class="question">
            <div class="js-vote-count">42</div>
            <div class="js-post-body">
                <p>I want to know how to foo.</p>
            </div>
            <div class="user-details">
                <a href="/users/1/alice">Alice</a>
            </div>
        </div>
        <div class="answer accepted-answer">
            <div class="js-vote-count">10</div>
            <div class="js-post-body">
                <p>You can foo by doing bar.</p>
            </div>
            <div class="user-details">
                <a href="/users/2/bob">Bob</a>
            </div>
        </div>
        </body>
        </html>
        "#;
        let html = Html::parse_document(html_str);
        let result = extract_from_html(&html, true).unwrap();

        assert_eq!(result.title.as_deref(), Some("How to foo?"));
        assert_eq!(result.author.as_deref(), Some("Alice"));
        assert!(result.content.contains("how to foo"));
        assert!(result.content.contains("foo by doing bar"));
        assert!(result.content.contains("Comments"));
    }

    #[test]
    fn extract_from_html_no_answers() {
        let html_str = r#"
        <html>
        <body>
        <div id="question-header"><h1>Unanswered</h1></div>
        <div class="question">
            <div class="js-post-body">
                <p>Some question text here.</p>
            </div>
        </div>
        </body>
        </html>
        "#;
        let html = Html::parse_document(html_str);
        let result = extract_from_html(&html, true).unwrap();

        assert_eq!(result.title.as_deref(), Some("Unanswered"));
        assert!(result.content.contains("question text"));
        assert!(!result.content.contains("Comments"));
    }

    #[test]
    fn api_fetch_live() {
        // Real network call — skip in CI
        if std::env::var("CI").is_ok() {
            return;
        }
        let url = "https://stackoverflow.com/questions/927358/\
             how-do-i-undo-the-most-recent-local-commits-in-git";
        let result = try_api_fetch(Some(url), false);
        if let Some(r) = result {
            assert!(r.title.is_some());
            assert_eq!(r.site.as_deref(), Some("Stack Overflow"));
        }
        // Don't fail if network is unavailable
    }
}
