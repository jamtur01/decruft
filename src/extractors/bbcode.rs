use regex::Regex;
use scraper::{Html, Selector};

/// Result of `BBCode` extraction from a data attribute.
pub struct BbcodeContent {
    pub html: String,
    pub title: Option<String>,
    pub author: Option<String>,
}

/// Check for `div[data-partnereventstore]` and extract `BBCode` content.
///
/// Returns `None` if the attribute is missing or the JSON is malformed.
pub fn extract_bbcode_content(html: &Html) -> Option<BbcodeContent> {
    let sel = Selector::parse("div[data-partnereventstore]").ok()?;
    let el = html.select(&sel).next()?;
    let raw = el.value().attr("data-partnereventstore")?;

    let events: serde_json::Value = serde_json::from_str(raw).ok()?;
    let event = events.as_array()?.first()?;

    let body = event
        .pointer("/announcement_body/body")
        .and_then(|v| v.as_str())?;

    let title = event
        .get("event_name")
        .and_then(|v| v.as_str())
        .map(String::from);

    let author = extract_group_name(html);

    let converted = bbcode_to_html(body);
    Some(BbcodeContent {
        html: converted,
        title,
        author,
    })
}

/// Extract group name from `data-groupvanityinfo` attribute.
fn extract_group_name(html: &Html) -> Option<String> {
    let sel = Selector::parse("div[data-groupvanityinfo]").ok()?;
    let el = html.select(&sel).next()?;
    let raw = el.value().attr("data-groupvanityinfo")?;
    let info: serde_json::Value = serde_json::from_str(raw).ok()?;
    let entry = info.as_array()?.first()?;
    entry
        .get("group_name")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Convert basic `BBCode` markup to HTML.
fn bbcode_to_html(bbcode: &str) -> String {
    let mut out = bbcode.to_string();

    // Newlines to <br>
    out = out.replace('\n', "<br>");

    // Block tags
    out = out.replace("[p]", "<p>").replace("[/p]", "</p>");
    out = out.replace("[b]", "<strong>").replace("[/b]", "</strong>");
    out = out.replace("[i]", "<em>").replace("[/i]", "</em>");
    out = out.replace("[u]", "<u>").replace("[/u]", "</u>");
    out = out.replace("[h1]", "<h1>").replace("[/h1]", "</h1>");
    out = out.replace("[h2]", "<h2>").replace("[/h2]", "</h2>");
    out = out.replace("[h3]", "<h3>").replace("[/h3]", "</h3>");

    // Lists
    out = out.replace("[list]", "<ul>").replace("[/list]", "</ul>");
    out = out.replace("[olist]", "<ol>").replace("[/olist]", "</ol>");
    out = out.replace("[*]", "<li>");

    // URL tags: [url="X"]text[/url] and [url=X]text[/url]
    if let Ok(re) = Regex::new(r#"\[url="?([^"\]]+)"?\](.*?)\[/url\]"#) {
        out = re.replace_all(&out, r#"<a href="$1">$2</a>"#).to_string();
    }

    // YouTube preview: [previewyoutube="ID;params"][/previewyoutube]
    if let Ok(re) = Regex::new(r#"\[previewyoutube="?([^;"\]]+)[^"]*"?\]\[/previewyoutube\]"#) {
        out = re
            .replace_all(
                &out,
                r#"<iframe src="https://www.youtube.com/embed/$1"></iframe>"#,
            )
            .to_string();
    }

    // Image tags: [img]URL[/img]
    if let Ok(re) = Regex::new(r"\[img\](.*?)\[/img\]") {
        out = re.replace_all(&out, r#"<img src="$1">"#).to_string();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_basic_bbcode() {
        let input = "[p]Hello [b]world[/b]![/p]";
        let result = bbcode_to_html(input);
        assert_eq!(result, "<p>Hello <strong>world</strong>!</p>");
    }

    #[test]
    fn convert_url_tag() {
        let input = r#"[url="https://example.com"]link[/url]"#;
        let result = bbcode_to_html(input);
        assert_eq!(result, r#"<a href="https://example.com">link</a>"#);
    }

    #[test]
    fn convert_youtube_preview() {
        let input = r#"[previewyoutube="dQw4w9WgXcQ;full"][/previewyoutube]"#;
        let result = bbcode_to_html(input);
        assert!(result.contains("youtube.com/embed/dQw4w9WgXcQ"));
    }
}
