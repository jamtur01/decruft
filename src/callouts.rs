use ego_tree::NodeId;
use scraper::Html;

use crate::dom;

/// Callout types recognized by the standardization pass.
const ADMONITION_TYPES: &[&str] = &[
    "info",
    "warning",
    "note",
    "tip",
    "danger",
    "caution",
    "important",
    "abstract",
    "success",
    "question",
    "failure",
    "bug",
    "example",
    "quote",
];

/// Standardize callout/alert/admonition elements from various
/// sources into a uniform `data-callout` / `data-callout-title`
/// attribute scheme.
///
/// Runs BEFORE selector-based removal so `.alert` and similar
/// classes aren't stripped before we can convert them.
pub fn standardize_callouts(html: &mut Html, main_content: NodeId) {
    standardize_obsidian_callouts(html, main_content);
    standardize_github_alerts(html, main_content);
    standardize_admonitions(html, main_content);
    standardize_bootstrap_alerts(html, main_content);
    standardize_github_blockquote_alerts(html, main_content);
}

/// Obsidian Publish callouts: already have `data-callout`, but
/// collapsed ones need the fold attribute cleaned up so
/// `removeHiddenElements` doesn't strip their content.
fn standardize_obsidian_callouts(html: &mut Html, main_content: NodeId) {
    let collapsed = dom::select_within(
        html,
        main_content,
        ".callout.is-collapsed, .callout.is-collapsible",
    );
    for id in collapsed {
        let fold = if has_class(html, id, "is-collapsed") {
            "-"
        } else {
            "+"
        };

        if dom::get_attr(html, id, "data-callout-fold").is_none() {
            dom::set_attr(html, id, "data-callout-fold", fold);
        }
    }
}

/// GitHub markdown alerts: `.markdown-alert` with type sub-class.
fn standardize_github_alerts(html: &mut Html, main_content: NodeId) {
    let alerts = dom::select_within(html, main_content, ".markdown-alert");
    for id in alerts {
        if dom::get_attr(html, id, "data-callout").is_some() {
            continue;
        }
        let callout_type = extract_type_from_class(html, id, "markdown-alert-");
        dom::set_attr(html, id, "data-callout", &callout_type);

        let title = capitalize(&callout_type);
        dom::set_attr(html, id, "data-callout-title", &title);
    }
}

/// Hugo/Docsy admonitions: `.admonition` with type class.
fn standardize_admonitions(html: &mut Html, main_content: NodeId) {
    let admonitions = dom::select_within(html, main_content, ".admonition");
    for id in admonitions {
        if dom::get_attr(html, id, "data-callout").is_some() {
            continue;
        }

        let callout_type = extract_admonition_type(html, id);
        dom::set_attr(html, id, "data-callout", &callout_type);

        let title = extract_admonition_title(html, id, &callout_type);
        dom::set_attr(html, id, "data-callout-title", &title);
    }
}

/// Bootstrap alerts: `.alert.alert-*` elements.
fn standardize_bootstrap_alerts(html: &mut Html, main_content: NodeId) {
    let selector = r#".alert[class*="alert-"]"#;
    let alerts = dom::select_within(html, main_content, selector);
    for id in alerts {
        if dom::get_attr(html, id, "data-callout").is_some() {
            continue;
        }
        let callout_type = extract_type_from_class(html, id, "alert-");
        if callout_type == "dismissible" {
            continue;
        }
        dom::set_attr(html, id, "data-callout", &callout_type);

        let title = extract_child_title(html, id, ".alert-heading, .alert-title");
        let title = title.unwrap_or_else(|| capitalize(&callout_type));
        dom::set_attr(html, id, "data-callout-title", &title);
    }
}

/// GitHub-style blockquote alerts containing `[!NOTE]` etc.
/// in the first paragraph.
fn standardize_github_blockquote_alerts(html: &mut Html, main_content: NodeId) {
    let blockquotes = dom::descendant_elements_by_tag(html, main_content, "blockquote");
    for bq_id in blockquotes {
        if dom::get_attr(html, bq_id, "data-callout").is_some() {
            continue;
        }
        let text = dom::text_content(html, bq_id);
        let Some(callout_type) = parse_blockquote_alert(&text) else {
            continue;
        };
        dom::set_attr(html, bq_id, "data-callout", &callout_type);
        let title = capitalize(&callout_type);
        dom::set_attr(html, bq_id, "data-callout-title", &title);
    }
}

/// Parse `[!TYPE]` from the start of blockquote text.
fn parse_blockquote_alert(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with("[!") {
        return None;
    }
    let end = trimmed.find(']')?;
    let tag = &trimmed[2..end];
    let lower = tag.to_ascii_lowercase();
    let valid = ["note", "warning", "tip", "important", "caution"];
    if valid.contains(&lower.as_str()) {
        Some(lower)
    } else {
        None
    }
}

/// Extract a type from a class list by finding a class with the
/// given prefix and stripping it.
fn extract_type_from_class(html: &Html, node_id: NodeId, prefix: &str) -> String {
    let Some(class_val) = dom::get_attr(html, node_id, "class") else {
        return "note".to_string();
    };
    for cls in class_val.split_whitespace() {
        if let Some(suffix) = cls.strip_prefix(prefix)
            && !suffix.is_empty()
        {
            return suffix.to_string();
        }
    }
    "note".to_string()
}

/// Extract admonition type from class list by checking against
/// known types.
fn extract_admonition_type(html: &Html, node_id: NodeId) -> String {
    let Some(class_val) = dom::get_attr(html, node_id, "class") else {
        return "note".to_string();
    };
    for cls in class_val.split_whitespace() {
        if ADMONITION_TYPES.contains(&cls) {
            return cls.to_string();
        }
    }
    "note".to_string()
}

/// Extract title text from `.admonition-title` child, falling
/// back to the capitalized type.
fn extract_admonition_title(html: &Html, node_id: NodeId, fallback_type: &str) -> String {
    extract_child_title(html, node_id, ".admonition-title")
        .unwrap_or_else(|| capitalize(fallback_type))
}

/// Extract text from the first child matching a selector.
fn extract_child_title(html: &Html, node_id: NodeId, selector: &str) -> Option<String> {
    let matches = dom::select_within(html, node_id, selector);
    let first = matches.into_iter().next()?;
    let text = dom::text_content(html, first).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Check if an element has a specific class.
fn has_class(html: &Html, node_id: NodeId, class: &str) -> bool {
    let Some(class_val) = dom::get_attr(html, node_id, "class") else {
        return false;
    };
    class_val.split_whitespace().any(|c| c == class)
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let upper: String = first.to_uppercase().collect();
    format!("{upper}{}", chars.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_markdown_alert() {
        let input = r#"<html><body>
            <article>
                <div class="markdown-alert markdown-alert-warning">
                    <p>Watch out!</p>
                </div>
            </article>
        </body></html>"#;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_callouts(&mut html, article[0]);

        let alerts = dom::select_within(&html, article[0], "[data-callout]");
        assert_eq!(alerts.len(), 1);
        assert_eq!(
            dom::get_attr(&html, alerts[0], "data-callout").as_deref(),
            Some("warning")
        );
        assert_eq!(
            dom::get_attr(&html, alerts[0], "data-callout-title").as_deref(),
            Some("Warning")
        );
    }

    #[test]
    fn blockquote_alert() {
        let input = r"<html><body>
            <article>
                <blockquote>
                    <p>[!NOTE] Remember this.</p>
                </blockquote>
            </article>
        </body></html>";
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_callouts(&mut html, article[0]);

        let bqs = dom::select_within(&html, article[0], "blockquote[data-callout]");
        assert_eq!(bqs.len(), 1);
        assert_eq!(
            dom::get_attr(&html, bqs[0], "data-callout").as_deref(),
            Some("note")
        );
    }

    #[test]
    fn admonition_with_title() {
        let input = r#"<html><body>
            <article>
                <div class="admonition warning">
                    <div class="admonition-title">Be careful</div>
                    <p>This is dangerous.</p>
                </div>
            </article>
        </body></html>"#;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_callouts(&mut html, article[0]);

        let adm = dom::select_within(&html, article[0], "[data-callout]");
        assert_eq!(adm.len(), 1);
        assert_eq!(
            dom::get_attr(&html, adm[0], "data-callout").as_deref(),
            Some("warning")
        );
        assert_eq!(
            dom::get_attr(&html, adm[0], "data-callout-title").as_deref(),
            Some("Be careful")
        );
    }

    #[test]
    fn bootstrap_alert() {
        let input = r#"<html><body>
            <article>
                <div class="alert alert-info">
                    <p>Some info.</p>
                </div>
            </article>
        </body></html>"#;
        let mut html = Html::parse_document(input);
        let article = dom::select_ids(&html, "article");
        standardize_callouts(&mut html, article[0]);

        let alerts = dom::select_within(&html, article[0], "[data-callout]");
        assert_eq!(alerts.len(), 1);
        assert_eq!(
            dom::get_attr(&html, alerts[0], "data-callout").as_deref(),
            Some("info")
        );
    }

    #[test]
    fn capitalize_works() {
        assert_eq!(capitalize("note"), "Note");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("WARNING"), "WARNING");
    }

    #[test]
    fn parse_blockquote_alert_valid() {
        assert_eq!(
            parse_blockquote_alert("[!NOTE] Some text"),
            Some("note".to_string())
        );
        assert_eq!(
            parse_blockquote_alert("[!WARNING]\nDetails"),
            Some("warning".to_string())
        );
    }

    #[test]
    fn parse_blockquote_alert_invalid() {
        assert_eq!(parse_blockquote_alert("Just text"), None);
        assert_eq!(parse_blockquote_alert("[!RANDOM] Stuff"), None);
    }
}
