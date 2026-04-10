//! Conversation extractors for `ChatGPT`, Claude, Gemini, and Grok.
//!
//! These sites share the same pattern: extract user/assistant message
//! pairs from site-specific DOM structures and format them as a
//! readable conversation.

use scraper::Html;

use crate::dom;

use super::ExtractorResult;

/// A single message in a conversation.
struct Message {
    author: String,
    role: &'static str,
    content: String,
}

/// Which conversation site we detected.
enum ConversationSite {
    ChatGpt,
    Claude,
    Gemini,
    Grok,
}

/// Try to extract conversation content from a supported AI chat site.
#[must_use]
pub fn extract_conversation(html: &Html, url: Option<&str>) -> Option<ExtractorResult> {
    let site = detect_site(html, url)?;
    let messages = extract_messages(html, &site);
    if messages.is_empty() {
        return None;
    }

    let title = extract_title(html, &site, &messages);
    let site_name = site_name(&site);
    let content = build_conversation_html(&messages);

    Some(ExtractorResult {
        content,
        title: Some(title),
        author: None,
        site: Some(site_name.to_string()),
        published: None,
        image: None,
        description: None,
    })
}

fn detect_site(html: &Html, url: Option<&str>) -> Option<ConversationSite> {
    if is_chatgpt(html, url) {
        return Some(ConversationSite::ChatGpt);
    }
    if is_claude(html, url) {
        return Some(ConversationSite::Claude);
    }
    if is_gemini(html, url) {
        return Some(ConversationSite::Gemini);
    }
    if is_grok(html, url) {
        return Some(ConversationSite::Grok);
    }
    None
}

fn site_name(site: &ConversationSite) -> &'static str {
    match site {
        ConversationSite::ChatGpt => "ChatGPT",
        ConversationSite::Claude => "Claude",
        ConversationSite::Gemini => "Gemini",
        ConversationSite::Grok => "Grok",
    }
}

// --- Site detection ---

fn is_chatgpt(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("chatgpt.com") || u.contains("chat.openai.com")) {
        return true;
    }
    has_selector(html, "article[data-testid^=\"conversation-turn-\"]")
}

fn is_claude(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("claude.ai")) {
        return true;
    }
    has_selector(html, "[data-testid=\"user-message\"]")
        || has_selector(html, "div.font-claude-response")
}

fn is_gemini(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("gemini.google.com")) {
        return true;
    }
    has_selector(html, "div.conversation-container") || has_selector(html, "user-query")
}

fn is_grok(html: &Html, url: Option<&str>) -> bool {
    if url.is_some_and(|u| u.contains("grok.com") || u.contains("x.com/i/grok")) {
        return true;
    }
    // Grok uses utility classes; check for its characteristic structure
    has_selector(html, "div.items-end")
        && has_selector(html, "div.items-start")
        && url.is_some_and(|u| u.contains("grok"))
}

fn has_selector(html: &Html, sel: &str) -> bool {
    scraper::Selector::parse(sel)
        .ok()
        .is_some_and(|s| html.select(&s).next().is_some())
}

// --- Message extraction per site ---

fn extract_messages(html: &Html, site: &ConversationSite) -> Vec<Message> {
    match site {
        ConversationSite::ChatGpt => extract_chatgpt_messages(html),
        ConversationSite::Claude => extract_claude_messages(html),
        ConversationSite::Gemini => extract_gemini_messages(html),
        ConversationSite::Grok => extract_grok_messages(html),
    }
}

fn extract_chatgpt_messages(html: &Html) -> Vec<Message> {
    let turn_ids = dom::select_ids(html, "article[data-testid^=\"conversation-turn-\"]");
    if turn_ids.is_empty() {
        // Fallback: older ChatGPT layout uses .group elements
        return extract_chatgpt_group_messages(html);
    }

    let mut messages = Vec::new();
    for &turn_id in &turn_ids {
        let (author, role) = detect_chatgpt_role(html, turn_id);
        let content = extract_turn_content(html, turn_id);
        if !content.is_empty() {
            messages.push(Message {
                author,
                role,
                content,
            });
        }
    }
    messages
}

fn detect_chatgpt_role(html: &Html, turn_id: ego_tree::NodeId) -> (String, &'static str) {
    // ChatGPT uses sr-only headings to label turns
    let heading_ids = dom::select_within(html, turn_id, "h5.sr-only, h6.sr-only");
    let heading_text = heading_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_lowercase())
        .unwrap_or_default();

    if heading_text.contains("you") {
        ("You".to_string(), "user")
    } else {
        ("ChatGPT".to_string(), "assistant")
    }
}

fn extract_turn_content(html: &Html, turn_id: ego_tree::NodeId) -> String {
    // Try markdown-body first, then fall back to inner HTML
    let md_ids = dom::select_within(html, turn_id, ".markdown-body, .markdown");
    if let Some(&md_id) = md_ids.first() {
        return dom::inner_html(html, md_id).trim().to_string();
    }
    // Fallback: get the prose content div
    let prose_ids = dom::select_within(html, turn_id, ".prose, .text-message");
    prose_ids
        .first()
        .map(|&id| dom::inner_html(html, id).trim().to_string())
        .unwrap_or_default()
}

fn extract_chatgpt_group_messages(html: &Html) -> Vec<Message> {
    let group_ids = dom::select_ids(html, ".group");
    let mut messages = Vec::new();
    for (i, &gid) in group_ids.iter().enumerate() {
        let role = if i % 2 == 0 { "user" } else { "assistant" };
        let author = if role == "user" { "You" } else { "ChatGPT" };
        let content = dom::inner_html(html, gid).trim().to_string();
        if !content.is_empty() {
            messages.push(Message {
                author: author.to_string(),
                role,
                content,
            });
        }
    }
    messages
}

fn extract_claude_messages(html: &Html) -> Vec<Message> {
    let mut messages = Vec::new();

    // User messages
    let user_ids = dom::select_ids(html, "[data-testid=\"user-message\"]");
    // Assistant messages
    let assistant_ids = dom::select_ids(
        html,
        "[data-testid=\"assistant-message\"], div.font-claude-response",
    );

    // Interleave by document order: collect all with roles
    let mut all: Vec<(ego_tree::NodeId, &str, &str)> = Vec::new();
    for &id in &user_ids {
        all.push((id, "You", "user"));
    }
    for &id in &assistant_ids {
        all.push((id, "Claude", "assistant"));
    }
    // Sort by NodeId to approximate document order
    all.sort_by_key(|(id, _, _)| *id);

    for (id, author, role) in all {
        let content = dom::inner_html(html, id).trim().to_string();
        if !content.is_empty() {
            messages.push(Message {
                author: author.to_string(),
                role,
                content,
            });
        }
    }
    messages
}

fn extract_gemini_messages(html: &Html) -> Vec<Message> {
    let mut messages = Vec::new();

    // Try conversation-container approach
    let container_ids = dom::select_ids(html, "div.conversation-container");
    if !container_ids.is_empty() {
        for &cid in &container_ids {
            // User query
            let query_ids = dom::select_within(html, cid, "user-query");
            if let Some(&qid) = query_ids.first() {
                let content = dom::inner_html(html, qid).trim().to_string();
                if !content.is_empty() {
                    messages.push(Message {
                        author: "You".to_string(),
                        role: "user",
                        content,
                    });
                }
            }
            // Model response
            let resp_ids = dom::select_within(html, cid, "model-response");
            if let Some(&rid) = resp_ids.first() {
                let content = dom::inner_html(html, rid).trim().to_string();
                if !content.is_empty() {
                    messages.push(Message {
                        author: "Gemini".to_string(),
                        role: "assistant",
                        content,
                    });
                }
            }
        }
        return messages;
    }

    // Fallback: query-content / response-content pairs
    let query_ids = dom::select_ids(html, ".query-content");
    let response_ids = dom::select_ids(html, ".response-content");
    let pairs = query_ids.len().min(response_ids.len());
    for i in 0..pairs {
        let q_content = dom::inner_html(html, query_ids[i]).trim().to_string();
        if !q_content.is_empty() {
            messages.push(Message {
                author: "You".to_string(),
                role: "user",
                content: q_content,
            });
        }
        let r_content = dom::inner_html(html, response_ids[i]).trim().to_string();
        if !r_content.is_empty() {
            messages.push(Message {
                author: "Gemini".to_string(),
                role: "assistant",
                content: r_content,
            });
        }
    }
    messages
}

fn extract_grok_messages(html: &Html) -> Vec<Message> {
    let mut messages = Vec::new();

    // Grok uses flex containers with items-end (user) and items-start (assistant)
    let bubble_ids = dom::select_ids(html, ".relative.group.flex.flex-col.justify-center.w-full");

    for &bid in &bubble_ids {
        let Some(node) = html.tree.get(bid) else {
            continue;
        };
        let scraper::Node::Element(el) = node.value() else {
            continue;
        };
        let classes = el.attr("class").unwrap_or("");
        let (author, role) = if classes.contains("items-end") {
            ("You", "user")
        } else if classes.contains("items-start") {
            ("Grok", "assistant")
        } else {
            continue;
        };

        let content = dom::inner_html(html, bid).trim().to_string();
        if !content.is_empty() {
            messages.push(Message {
                author: author.to_string(),
                role,
                content,
            });
        }
    }
    messages
}

// --- Title extraction ---

fn extract_title(html: &Html, site: &ConversationSite, messages: &[Message]) -> String {
    // Try page title first
    let page_title = extract_page_title(html, site);
    if !page_title.is_empty() {
        return page_title;
    }

    // Fallback: first user message, truncated
    let first_user = messages.iter().find(|m| m.role == "user");
    if let Some(msg) = first_user {
        let plain = dom::strip_html_tags(&msg.content);
        let trimmed = plain.trim();
        return truncate_title(trimmed, 50);
    }

    format!("{} Conversation", site_name(site))
}

fn extract_page_title(html: &Html, site: &ConversationSite) -> String {
    let title_ids = dom::select_ids(html, "title");
    let raw = title_ids
        .first()
        .map(|&id| dom::text_content(html, id).trim().to_string())
        .unwrap_or_default();

    if raw.is_empty() {
        return String::new();
    }

    // Strip site suffixes
    let suffixes: &[&str] = match site {
        ConversationSite::ChatGpt => &[" - ChatGPT", " | ChatGPT"],
        ConversationSite::Claude => &[" - Claude", " | Claude"],
        ConversationSite::Gemini => &[" - Gemini", " - Google Gemini"],
        ConversationSite::Grok => &[" - Grok", " | Grok"],
    };

    let mut title = raw;
    for suffix in suffixes {
        if let Some(stripped) = title.strip_suffix(suffix) {
            title = stripped.to_string();
            break;
        }
    }

    // If the title is just the site name, treat as empty
    if title == site_name(site) {
        return String::new();
    }

    title
}

fn truncate_title(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        Some((i, _)) => format!("{}...", &s[..i]),
        None => s.to_string(),
    }
}

// --- HTML output ---

fn build_conversation_html(messages: &[Message]) -> String {
    use std::fmt::Write;

    let mut html = String::from("<div class=\"conversation\">");
    for msg in messages {
        let role = msg.role;
        let author = html_escape(&msg.author);
        let _ = write!(
            html,
            "<div class=\"message message-{role}\" data-role=\"{role}\">\
             <div class=\"message-header\"><strong>{author}</strong></div>\
             <div class=\"message-content\">{}</div>\
             </div>",
            msg.content
        );
    }
    html.push_str("</div>");
    html
}

fn html_escape(s: &str) -> String {
    crate::dom::html_escape(s)
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_chatgpt_by_url() {
        let html = Html::parse_document("<html><body></body></html>");
        assert!(is_chatgpt(&html, Some("https://chatgpt.com/c/abc")));
        assert!(is_chatgpt(&html, Some("https://chat.openai.com/c/abc")));
        assert!(!is_chatgpt(&html, Some("https://example.com")));
    }

    #[test]
    fn detect_claude_by_url() {
        let html = Html::parse_document("<html><body></body></html>");
        assert!(is_claude(&html, Some("https://claude.ai/chat/abc")));
        assert!(!is_claude(&html, Some("https://example.com")));
    }

    #[test]
    fn detect_gemini_by_url() {
        let html = Html::parse_document("<html><body></body></html>");
        assert!(is_gemini(&html, Some("https://gemini.google.com/app/abc")));
    }

    #[test]
    fn extract_chatgpt_from_dom() {
        let doc = r#"<html><body>
            <article data-testid="conversation-turn-1">
                <h6 class="sr-only">You said:</h6>
                <div class="markdown">Hello world</div>
            </article>
            <article data-testid="conversation-turn-2">
                <h6 class="sr-only">ChatGPT said:</h6>
                <div class="markdown">Hi there!</div>
            </article>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_conversation(&html, Some("https://chatgpt.com/c/abc"));
        let result = result.unwrap();
        assert!(result.content.contains("Hello world"));
        assert!(result.content.contains("Hi there!"));
        assert_eq!(result.site.as_deref(), Some("ChatGPT"));
    }

    #[test]
    fn extract_claude_from_dom() {
        let doc = r#"<html><body>
            <div data-testid="user-message">What is Rust?</div>
            <div data-testid="assistant-message">Rust is a systems language.</div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_conversation(&html, Some("https://claude.ai/chat/abc"));
        let result = result.unwrap();
        assert!(result.content.contains("What is Rust?"));
        assert!(result.content.contains("systems language"));
        assert_eq!(result.site.as_deref(), Some("Claude"));
    }

    #[test]
    fn extract_gemini_from_dom() {
        let doc = r#"<html><body>
            <div class="conversation-container">
                <user-query>Explain AI</user-query>
                <model-response>AI is artificial intelligence.</model-response>
            </div>
        </body></html>"#;
        let html = Html::parse_document(doc);
        let result = extract_conversation(&html, Some("https://gemini.google.com/app/x"));
        let result = result.unwrap();
        assert!(result.content.contains("Explain AI"));
        assert!(result.content.contains("artificial intelligence"));
    }

    #[test]
    fn build_conversation_html_format() {
        let messages = vec![
            Message {
                author: "You".to_string(),
                role: "user",
                content: "Hello".to_string(),
            },
            Message {
                author: "Bot".to_string(),
                role: "assistant",
                content: "Hi".to_string(),
            },
        ];
        let html = build_conversation_html(&messages);
        assert!(html.contains("message-user"));
        assert!(html.contains("message-assistant"));
        assert!(html.contains("data-role=\"user\""));
    }

    #[test]
    fn no_extraction_for_unrelated_page() {
        let html = Html::parse_document("<html><body><p>Hello</p></body></html>");
        assert!(extract_conversation(&html, Some("https://example.com")).is_none());
    }
}
