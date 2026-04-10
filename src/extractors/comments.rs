//! Standardized comment HTML construction.
//!
//! Used by Reddit, Hacker News, GitHub, and other extractors to produce
//! consistent comment markup.

/// Data for a single comment.
pub struct CommentData {
    pub author: String,
    pub date: String,
    pub content: String,
    pub depth: usize,
    pub score: Option<String>,
    pub url: Option<String>,
}

/// Build full content HTML for a post with optional comments section.
#[must_use]
pub fn build_content_html(site: &str, post_content: &str, comments: &str) -> String {
    let post_section = format!(
        "<div class=\"{site} post\"><div class=\"post-content\">{post_content}</div></div>"
    );
    if comments.is_empty() {
        return post_section;
    }
    format!("{post_section}<hr><div class=\"{site} comments\"><h2>Comments</h2>{comments}</div>")
}

/// Build a nested comment tree from a flat list of comments with depth.
///
/// Uses `<blockquote>` elements to represent reply hierarchy.
#[must_use]
pub fn build_comment_tree(comments: &[CommentData]) -> String {
    let mut parts = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for comment in comments {
        let depth = comment.depth;

        if depth == 0 {
            while !stack.is_empty() {
                parts.push("</blockquote>".to_string());
                stack.pop();
            }
            parts.push("<blockquote>".to_string());
            stack.push(0);
        } else {
            let current_depth = stack.last().copied().unwrap_or(0);
            if depth < current_depth {
                while stack.last().is_some_and(|&d| d >= depth) {
                    parts.push("</blockquote>".to_string());
                    stack.pop();
                }
            }
            let new_depth = stack.last().copied().unwrap_or(0);
            if depth > new_depth {
                parts.push("<blockquote>".to_string());
                stack.push(depth);
            }
        }

        parts.push(build_comment(comment));
    }

    while !stack.is_empty() {
        parts.push("</blockquote>".to_string());
        stack.pop();
    }

    parts.join("")
}

/// Build a single comment div with metadata and content.
#[must_use]
pub fn build_comment(comment: &CommentData) -> String {
    let author_html = format!(
        "<span class=\"comment-author\"><strong>{}</strong></span>",
        html_escape(&comment.author)
    );

    let date_html = if let Some(ref url) = comment.url {
        format!(
            "<a href=\"{}\" class=\"comment-link\">{}</a>",
            html_escape(url),
            html_escape(&comment.date)
        )
    } else {
        format!(
            "<span class=\"comment-date\">{}</span>",
            html_escape(&comment.date)
        )
    };

    let score_html = comment.score.as_ref().map_or_else(String::new, |s| {
        format!(
            " · <span class=\"comment-points\">{}</span>",
            html_escape(s)
        )
    });

    format!(
        "<div class=\"comment\">\
         <div class=\"comment-metadata\">{author_html} · {date_html}{score_html}</div>\
         <div class=\"comment-content\">{}</div>\
         </div>",
        comment.content
    )
}

fn html_escape(s: &str) -> String {
    crate::dom::html_escape(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_comment_tree_nested_blockquotes() {
        let comments = vec![
            CommentData {
                author: "alice".into(),
                date: "2025-01-01".into(),
                content: "<p>Top level</p>".into(),
                depth: 0,
                score: None,
                url: None,
            },
            CommentData {
                author: "bob".into(),
                date: "2025-01-02".into(),
                content: "<p>Reply to alice</p>".into(),
                depth: 1,
                score: None,
                url: None,
            },
        ];
        let tree = build_comment_tree(&comments);
        // Should contain nested blockquotes
        assert!(tree.contains("<blockquote>"));
        assert!(tree.contains("</blockquote>"));
        assert!(tree.contains("alice"));
        assert!(tree.contains("bob"));
        assert!(tree.contains("Reply to alice"));
        // The reply should be inside a nested blockquote
        let inner_start = tree.find("Reply to alice").unwrap_or(0);
        let blockquote_count = tree[..inner_start].matches("<blockquote>").count();
        assert!(
            blockquote_count >= 2,
            "reply should be nested in at least 2 blockquotes, found {blockquote_count}"
        );
    }
}
