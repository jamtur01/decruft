use scraper::{Html, Selector};

use crate::dom;
use crate::types::Metadata;

/// Extract metadata from an HTML document following defuddle's
/// priority chains. Schema.org data, when present, participates
/// in several chains.
#[must_use]
pub fn extract_metadata(
    html: &Html,
    url: Option<&str>,
    schema: Option<&serde_json::Value>,
) -> Metadata {
    let site_name = extract_site_name(html, schema);
    let title = extract_title(html, schema, &site_name);
    let author = extract_author(html, schema);
    let published = extract_published(html, schema);
    let description = extract_description(html, schema);
    let image = extract_image(html, schema);
    let language = extract_language(html, schema);
    let domain = extract_domain(url);
    let favicon = extract_favicon(html, url);

    Metadata {
        title,
        description,
        domain,
        favicon,
        image,
        language,
        published,
        author,
        site_name,
    }
}

/// Query `meta[{attr}="{value}"]` and return the `content` attribute.
fn get_meta_content(html: &Html, attr: &str, value: &str) -> Option<String> {
    let selector_str = format!("meta[{attr}=\"{value}\"]");
    let Ok(sel) = Selector::parse(&selector_str) else {
        return None;
    };
    let element = html.select(&sel).next()?;
    let content = element.value().attr("content")?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Dig into a `serde_json::Value` by a dot-separated path.
/// Returns the leaf value as a trimmed, non-empty string.
fn schema_str(schema: Option<&serde_json::Value>, path: &str) -> Option<String> {
    let mut current = schema?;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    let text = current.as_str()?.trim();
    if text.is_empty() {
        return None;
    }
    Some(text.to_string())
}

// ------------------------------------------------------------------
// Title
// ------------------------------------------------------------------

fn extract_title(html: &Html, schema: Option<&serde_json::Value>, site_name: &str) -> String {
    let raw = get_meta_content(html, "property", "og:title")
        .or_else(|| get_meta_content(html, "name", "twitter:title"))
        .or_else(|| get_meta_content(html, "property", "twitter:title"))
        .or_else(|| schema_str(schema, "headline"))
        .or_else(|| get_meta_content(html, "name", "title"))
        .or_else(|| get_meta_content(html, "name", "sailthru.title"))
        .or_else(|| title_element_text(html));

    let Some(title) = raw else {
        return String::new();
    };
    clean_title(&title, site_name)
}

fn title_element_text(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("title") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Remove site-name suffixes/prefixes separated by common delimiters.
/// When a site name is known, only strips the part that matches it.
/// Otherwise falls back to a length heuristic (shorter part is likely
/// the site name).
fn clean_title(title: &str, site_name: &str) -> String {
    let separators = [" | ", " - ", " -- ", " / ", " · "];
    for sep in &separators {
        let Some(idx) = title.rfind(sep) else {
            continue;
        };
        let before = title[..idx].trim();
        let after = title[idx + sep.len()..].trim();
        if before.is_empty() || after.is_empty() {
            continue;
        }

        if !site_name.is_empty() {
            let site_lower = site_name.to_lowercase();
            if after.to_lowercase() == site_lower {
                return before.to_string();
            }
            if before.to_lowercase() == site_lower {
                return after.to_string();
            }
            // Site name known but doesn't match either side
            continue;
        }

        // No site name: use length heuristic
        if before.len() >= after.len() {
            return before.to_string();
        }
        return after.to_string();
    }
    title.to_string()
}

// ------------------------------------------------------------------
// Author
// ------------------------------------------------------------------

fn extract_author(html: &Html, schema: Option<&serde_json::Value>) -> String {
    if let Some(v) = get_meta_content(html, "property", "author")
        .or_else(|| get_meta_content(html, "name", "author"))
    {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "sailthru.author") {
        return v;
    }

    if let Some(v) = schema_author(schema) {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "byl") {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "authorList") {
        return v;
    }

    if let Some(v) =
        get_meta_content(html, "name", "citation_author").map(|s| reverse_citation_author(&s))
    {
        return v;
    }

    if let Some(v) = get_meta_content(html, "name", "dc.creator") {
        return v;
    }

    if let Some(v) = itemprop_author(html) {
        return v;
    }

    if let Some(v) = class_author(html) {
        return v;
    }

    if let Some(v) = author_href_elements(html) {
        return v;
    }

    if let Some(v) = authors_link_elements(html) {
        return v;
    }

    String::new()
}

/// Reverse "Last, First" to "First Last" for `citation_author` meta.
fn reverse_citation_author(name: &str) -> String {
    if let Some((last, first)) = name.split_once(',') {
        let first = first.trim();
        let last = last.trim();
        if !first.is_empty() && !last.is_empty() {
            return format!("{first} {last}");
        }
    }
    name.to_string()
}

/// Extract author names from `[href*="/author/"]` elements (max 3).
fn author_href_elements(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("[href*=\"/author/\"]") else {
        return None;
    };
    let mut names = Vec::new();
    for el in html.select(&sel) {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            names.push(trimmed);
        }
        if names.len() >= 3 {
            break;
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join(", "))
}

/// Extract author names from `.authors a` elements (max 3).
fn authors_link_elements(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse(".authors a") else {
        return None;
    };
    let mut names = Vec::new();
    for el in html.select(&sel) {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            names.push(trimmed);
        }
        if names.len() >= 3 {
            break;
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join(", "))
}

fn schema_author(schema: Option<&serde_json::Value>) -> Option<String> {
    let author = schema?.get("author")?;
    if let Some(name) = author.get("name").and_then(|v| v.as_str()) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(s) = author.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn itemprop_author(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("[itemprop=\"author\"]") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn class_author(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse(".author") else {
        return None;
    };
    let mut names = Vec::new();
    for el in html.select(&sel) {
        let text = dom::text_content(html, el.id());
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            names.push(trimmed);
        }
        if names.len() >= 3 {
            break;
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join(", "))
}

// ------------------------------------------------------------------
// Published date
// ------------------------------------------------------------------

fn extract_published(html: &Html, schema: Option<&serde_json::Value>) -> String {
    schema_str(schema, "datePublished")
        .or_else(|| get_meta_content(html, "name", "publishDate"))
        .or_else(|| get_meta_content(html, "property", "article:published_time"))
        .or_else(|| abbr_date_published(html))
        .or_else(|| get_meta_content(html, "name", "sailthru.date"))
        .or_else(|| first_time_element(html))
        .unwrap_or_default()
}

/// Extract date from `abbr[itemprop="datePublished"]` -- check
/// `datetime` attr first, then text content.
fn abbr_date_published(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("abbr[itemprop=\"datePublished\"]") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    if let Some(dt) = el.value().attr("datetime") {
        let trimmed = dt.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn first_time_element(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("time") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    if let Some(dt) = el.value().attr("datetime") {
        let trimmed = dt.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let text = dom::text_content(html, el.id());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

// ------------------------------------------------------------------
// Site name
// ------------------------------------------------------------------

fn extract_site_name(html: &Html, schema: Option<&serde_json::Value>) -> String {
    schema_str(schema, "publisher.name")
        .or_else(|| get_meta_content(html, "property", "og:site_name"))
        .or_else(|| schema_graph_website_name(schema))
        .or_else(|| schema_str(schema, "sourceOrganization.name"))
        .or_else(|| get_meta_content(html, "name", "copyright"))
        .or_else(|| schema_str(schema, "copyrightHolder.name"))
        .or_else(|| schema_str(schema, "isPartOf.name"))
        .or_else(|| get_meta_content(html, "name", "application-name"))
        .and_then(|name| {
            if name.split_whitespace().count() > 6 {
                None
            } else {
                Some(name)
            }
        })
        .unwrap_or_default()
}

/// Search Schema.org `@graph` for `@type: "WebSite"` and return its
/// `name`.
fn schema_graph_website_name(schema: Option<&serde_json::Value>) -> Option<String> {
    let graph = schema?.get("@graph")?.as_array()?;
    for item in graph {
        let Some(type_val) = item.get("@type") else {
            continue;
        };
        let is_website = type_val.as_str() == Some("WebSite")
            || type_val
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("WebSite")));
        if !is_website {
            continue;
        }
        let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

// ------------------------------------------------------------------
// Description
// ------------------------------------------------------------------

fn extract_description(html: &Html, schema: Option<&serde_json::Value>) -> String {
    get_meta_content(html, "name", "description")
        .or_else(|| get_meta_content(html, "property", "og:description"))
        .or_else(|| get_meta_content(html, "property", "twitter:description"))
        .or_else(|| get_meta_content(html, "name", "twitter:description"))
        .or_else(|| get_meta_content(html, "name", "sailthru.description"))
        .or_else(|| schema_str(schema, "description"))
        .unwrap_or_default()
}

// ------------------------------------------------------------------
// Image
// ------------------------------------------------------------------

fn extract_image(html: &Html, schema: Option<&serde_json::Value>) -> String {
    get_meta_content(html, "property", "og:image")
        .or_else(|| get_meta_content(html, "property", "twitter:image"))
        .or_else(|| get_meta_content(html, "name", "twitter:image"))
        .or_else(|| schema_image(schema))
        .unwrap_or_default()
}

fn schema_image(schema: Option<&serde_json::Value>) -> Option<String> {
    let image = schema?.get("image")?;
    if let Some(url) = image.get("url").and_then(|v| v.as_str()) {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(s) = image.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

// ------------------------------------------------------------------
// Language
// ------------------------------------------------------------------

fn extract_language(html: &Html, schema: Option<&serde_json::Value>) -> String {
    html_lang(html)
        .or_else(|| get_meta_content(html, "name", "content-language"))
        .or_else(|| get_meta_content(html, "http-equiv", "content-language"))
        .or_else(|| get_meta_content(html, "property", "og:locale"))
        .or_else(|| schema_str(schema, "inLanguage"))
        .map(|s| normalize_bcp47(&s))
        .unwrap_or_default()
}

/// Normalize BCP 47 language tags: replace `_` with `-`.
fn normalize_bcp47(lang: &str) -> String {
    lang.replace('_', "-")
}

fn html_lang(html: &Html) -> Option<String> {
    let Ok(sel) = Selector::parse("html") else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let lang = el.value().attr("lang")?.trim();
    if lang.is_empty() {
        return None;
    }
    Some(lang.to_string())
}

// ------------------------------------------------------------------
// Favicon
// ------------------------------------------------------------------

fn extract_favicon(html: &Html, url: Option<&str>) -> String {
    get_meta_content(html, "property", "og:image:favicon")
        .or_else(|| link_icon(html, "icon"))
        .or_else(|| link_icon(html, "shortcut icon"))
        .or_else(|| favicon_fallback(url))
        .unwrap_or_default()
}

fn link_icon(html: &Html, rel: &str) -> Option<String> {
    let selector_str = format!("link[rel=\"{rel}\"]");
    let Ok(sel) = Selector::parse(&selector_str) else {
        return None;
    };
    let el = html.select(&sel).next()?;
    let href = el.value().attr("href")?.trim();
    if href.is_empty() {
        return None;
    }
    Some(href.to_string())
}

fn favicon_fallback(url: Option<&str>) -> Option<String> {
    let base = base_url(url?)?;
    Some(format!("{base}/favicon.ico"))
}

// ------------------------------------------------------------------
// Domain
// ------------------------------------------------------------------

fn extract_domain(url: Option<&str>) -> String {
    let Some(raw) = url else {
        return String::new();
    };
    let Ok(parsed) = url::Url::parse(raw) else {
        return String::new();
    };
    parsed.host_str().unwrap_or_default().to_string()
}

/// Return scheme + host (+ port if non-default).
fn base_url(raw: &str) -> Option<String> {
    let parsed = url::Url::parse(raw).ok()?;
    let host = parsed.host_str()?;
    let scheme = parsed.scheme();
    match parsed.port() {
        Some(port) => Some(format!("{scheme}://{host}:{port}")),
        None => Some(format!("{scheme}://{host}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn title_from_og() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:title" content="OG Title">
            <title>Fallback</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "OG Title");
    }

    #[test]
    fn title_cleaned_of_suffix() {
        let doc = Html::parse_document(
            r"<html><head>
            <title>Article Name | Site Name</title>
            </head><body></body></html>",
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Article Name");
    }

    #[test]
    fn title_suffix_keeps_longer_part() {
        let doc = Html::parse_document(
            r"<html><head>
            <title>SN | A Much Longer Article Title</title>
            </head><body></body></html>",
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "A Much Longer Article Title");
    }

    #[test]
    fn title_stripped_when_site_name_matches() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="Wikipedia">
            <title>Bengaluru - Wikipedia</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Bengaluru");
    }

    #[test]
    fn title_not_stripped_when_site_name_mismatches() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="MyBlog">
            <title>Part A - Part B</title>
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.title, "Part A - Part B");
    }

    #[test]
    fn author_from_meta() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta name="author" content="Jane Doe">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.author, "Jane Doe");
    }

    #[test]
    fn author_from_schema_object() {
        let schema: serde_json::Value = serde_json::json!({
            "author": { "name": "Schema Author" }
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.author, "Schema Author");
    }

    #[test]
    fn domain_extracted_from_url() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, Some("https://example.com/page"), None);
        assert_eq!(m.domain, "example.com");
    }

    #[test]
    fn favicon_fallback_to_root() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, Some("https://example.com/a/b"), None);
        assert_eq!(m.favicon, "https://example.com/favicon.ico");
    }

    #[test]
    fn language_from_html_attr() {
        let doc = Html::parse_document(r#"<html lang="en-US"><body></body></html>"#);
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.language, "en-US");
    }

    #[test]
    fn description_from_meta() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta name="description" content="A page about things">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.description, "A page about things");
    }

    #[test]
    fn published_from_schema() {
        let schema: serde_json::Value = serde_json::json!({
            "datePublished": "2025-01-15"
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.published, "2025-01-15");
    }

    #[test]
    fn site_name_from_og() {
        let doc = Html::parse_document(
            r#"<html><head>
            <meta property="og:site_name" content="My Site">
            </head><body></body></html>"#,
        );
        let m = extract_metadata(&doc, None, None);
        assert_eq!(m.site_name, "My Site");
    }

    #[test]
    fn image_from_schema_string() {
        let schema: serde_json::Value = serde_json::json!({
            "image": "https://img.example.com/photo.jpg"
        });
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, Some(&schema));
        assert_eq!(m.image, "https://img.example.com/photo.jpg");
    }

    #[test]
    fn empty_metadata_for_blank_doc() {
        let doc = Html::parse_document("<html><body></body></html>");
        let m = extract_metadata(&doc, None, None);
        assert!(m.title.is_empty());
        assert!(m.author.is_empty());
        assert!(m.published.is_empty());
        assert!(m.domain.is_empty());
    }
}
