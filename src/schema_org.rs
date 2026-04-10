use scraper::{Html, Selector};
use serde_json::Value;

/// Extract all JSON-LD schema.org data from the document.
/// Finds all `<script type="application/ld+json">` elements, parses them,
/// flattens @graph arrays, and returns combined data.
#[must_use]
pub fn extract_schema_org(html: &Html) -> Option<Value> {
    let Ok(selector) = Selector::parse(r#"script[type="application/ld+json"]"#) else {
        return None;
    };

    let mut items: Vec<Value> = Vec::new();

    for element in html.select(&selector) {
        let raw = element.text().collect::<String>();
        let cleaned = strip_json_comments(&raw);
        let Ok(mut parsed) = serde_json::from_str::<Value>(&cleaned) else {
            continue;
        };
        decode_entities(&mut parsed);
        collect_items(&parsed, &mut items);
    }

    match items.len() {
        0 => None,
        1 => Some(items.remove(0)),
        _ => Some(Value::Array(items)),
    }
}

/// Get a property from schema.org data using dot-path notation.
/// Supports: `author.name`, `author.[].name` (array traversal),
/// `datePublished`. Searches recursively through the data structure.
#[cfg(test)]
#[must_use]
fn get_property(data: &Value, path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('.').collect();

    // Try direct path traversal first
    if let Some(result) = walk_path(data, &segments) {
        return Some(result);
    }

    // Fall back to recursive search
    recursive_search(data, &segments)
}

/// Extract article body text from schema.org data.
/// Looks for `text` or `articleBody` fields.
#[must_use]
pub fn get_text(data: &Value) -> Option<String> {
    let keys = ["text", "articleBody"];

    if let Value::Array(arr) = data {
        for item in arr {
            if let Some(text) = find_text_field(item, &keys) {
                return Some(text);
            }
        }
    } else {
        return find_text_field(data, &keys);
    }

    None
}

/// Decode HTML entities in all string values recursively.
fn decode_entities(value: &mut Value) {
    match value {
        Value::String(s) => {
            *s = decode_entity_str(s);
        }
        Value::Array(arr) => {
            for item in arr {
                decode_entities(item);
            }
        }
        Value::Object(map) => {
            for val in map.values_mut() {
                decode_entities(val);
            }
        }
        _ => {}
    }
}

fn decode_entity_str(s: &str) -> String {
    let mut result = s.to_string();
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");

    // Decode numeric character references: &#NNN; and &#xHHHH;
    let mut output = String::with_capacity(result.len());
    let mut chars = result.as_str();
    while let Some(pos) = chars.find("&#") {
        output.push_str(&chars[..pos]);
        let rest = &chars[pos + 2..];
        if let Some(semi) = rest.find(';') {
            let num_str = &rest[..semi];
            let decoded = if num_str.starts_with('x') || num_str.starts_with('X') {
                u32::from_str_radix(&num_str[1..], 16)
                    .ok()
                    .and_then(char::from_u32)
            } else {
                num_str.parse::<u32>().ok().and_then(char::from_u32)
            };
            if let Some(ch) = decoded {
                output.push(ch);
                chars = &rest[semi + 1..];
            } else {
                output.push_str("&#");
                chars = rest;
            }
        } else {
            output.push_str("&#");
            chars = rest;
        }
    }
    output.push_str(chars);
    output
}

/// Strip JavaScript-style comments and CDATA wrappers from text
/// before parsing as JSON.
fn strip_json_comments(s: &str) -> String {
    // Strip CDATA wrappers
    let s = s.trim().strip_prefix("<![CDATA[").unwrap_or(s.trim());
    let s = s.strip_suffix("]]>").unwrap_or(s).trim();

    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut in_string = false;

    while let Some(&ch) = chars.peek() {
        if in_string {
            chars.next();
            if ch == '\\' {
                result.push(ch);
                if let Some(&next) = chars.peek() {
                    result.push(next);
                    chars.next();
                }
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            result.push(ch);
            continue;
        }

        if ch == '"' {
            in_string = true;
            result.push(ch);
            chars.next();
            continue;
        }

        // Line comment
        if ch == '/' {
            chars.next();
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    // Skip until newline
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
                if next == '*' {
                    chars.next();
                    // Skip until */
                    let mut prev = ' ';
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if prev == '*' && c == '/' {
                            break;
                        }
                        prev = c;
                    }
                    continue;
                }
            }
            result.push('/');
            continue;
        }

        result.push(ch);
        chars.next();
    }
    result
}

/// Flatten @graph arrays and collect top-level items.
fn collect_items(value: &Value, items: &mut Vec<Value>) {
    if let Value::Array(arr) = value {
        for item in arr {
            collect_items(item, items);
        }
        return;
    }

    let Value::Object(obj) = value else {
        return;
    };

    if let Some(Value::Array(graph)) = obj.get("@graph") {
        for item in graph {
            items.push(item.clone());
        }
        // Also include fields beyond @graph at the top level
        let mut top = obj.clone();
        top.remove("@graph");
        if top.len() > 1 || (top.len() == 1 && !top.contains_key("@context")) {
            items.push(Value::Object(top));
        }
    } else {
        items.push(value.clone());
    }
}

/// Walk a dot-path through JSON, returning the final string value.
#[cfg(test)]
fn walk_path(value: &Value, segments: &[&str]) -> Option<String> {
    if segments.is_empty() {
        return value_to_string(value);
    }

    let (segment, rest) = (segments[0], &segments[1..]);

    if segment == "[]" {
        let Value::Array(arr) = value else {
            return None;
        };
        for item in arr {
            if let Some(result) = walk_path(item, rest) {
                return Some(result);
            }
        }
        return None;
    }

    let Value::Object(obj) = value else {
        return None;
    };
    let child = obj.get(segment)?;
    walk_path(child, rest)
}

/// Search recursively for a path through nested objects.
#[cfg(test)]
fn recursive_search(value: &Value, segments: &[&str]) -> Option<String> {
    if segments.is_empty() {
        return value_to_string(value);
    }

    match value {
        Value::Object(obj) => {
            // Try direct key match at this level
            if let Some(child) = obj.get(segments[0])
                && let Some(result) = walk_path(child, &segments[1..])
            {
                return Some(result);
            }
            // Recurse into all values
            for val in obj.values() {
                if let Some(result) = recursive_search(val, segments) {
                    return Some(result);
                }
            }
            None
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(result) = recursive_search(item, segments) {
                    return Some(result);
                }
            }
            None
        }
        _ => None,
    }
}

/// Convert a JSON value to a string if possible.
#[cfg(test)]
fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Find a text/article body field in an object.
fn find_text_field(value: &Value, keys: &[&str]) -> Option<String> {
    let Value::Object(obj) = value else {
        return None;
    };
    for key in keys {
        if let Some(Value::String(s)) = obj.get(*key)
            && !s.is_empty()
        {
            return Some(s.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_jsonld() {
        let html_str = r#"<html><head>
            <script type="application/ld+json">
            {"@type": "Article", "name": "Test"}
            </script>
            </head><body></body></html>"#;
        let doc = Html::parse_document(html_str);
        let data = extract_schema_org(&doc);
        assert!(data.is_some());
        let data = data.as_ref();
        assert_eq!(
            data.and_then(|d| d.get("name")).and_then(Value::as_str),
            Some("Test")
        );
    }

    #[test]
    fn test_extract_graph() {
        let html_str = r#"<html><head>
            <script type="application/ld+json">
            {"@context": "https://schema.org",
             "@graph": [
               {"@type": "Article", "name": "A"},
               {"@type": "Person", "name": "B"}
             ]}
            </script>
            </head><body></body></html>"#;
        let doc = Html::parse_document(html_str);
        let data = extract_schema_org(&doc);
        assert!(data.is_some());
        let arr = data.as_ref().and_then(Value::as_array);
        assert!(arr.is_some());
        assert_eq!(arr.map(Vec::len), Some(2));
    }

    #[test]
    fn test_get_property_simple() {
        let data: Value =
            serde_json::from_str(r#"{"author": {"name": "Alice"}, "datePublished": "2024"}"#)
                .ok()
                .unwrap_or_default();
        assert_eq!(
            get_property(&data, "author.name"),
            Some("Alice".to_string())
        );
        assert_eq!(
            get_property(&data, "datePublished"),
            Some("2024".to_string())
        );
    }

    #[test]
    fn test_get_property_array() {
        let data: Value = serde_json::from_str(r#"{"author": [{"name": "A"}, {"name": "B"}]}"#)
            .ok()
            .unwrap_or_default();
        assert_eq!(get_property(&data, "author.[].name"), Some("A".to_string()));
    }

    #[test]
    fn test_get_text() {
        let data: Value =
            serde_json::from_str(r#"{"@type": "Article", "articleBody": "Hello world"}"#)
                .ok()
                .unwrap_or_default();
        assert_eq!(get_text(&data), Some("Hello world".to_string()));
    }

    #[test]
    fn test_decode_entities() {
        let mut val = Value::String("A &amp; B &lt; C &gt; D &quot;E&#65;".into());
        decode_entities(&mut val);
        assert_eq!(val.as_str(), Some("A & B < C > D \"EA"));
    }

    #[test]
    fn test_decode_hex_entities() {
        // &#x2019; = right single quotation mark (U+2019)
        let mut val = Value::String("it&#x2019;s a test".into());
        decode_entities(&mut val);
        assert_eq!(val.as_str(), Some("it\u{2019}s a test"));
    }

    #[test]
    fn test_decode_hex_entity_ampersand() {
        // &#x26; = &
        let mut val = Value::String("A &#x26; B".into());
        decode_entities(&mut val);
        assert_eq!(val.as_str(), Some("A & B"));
    }

    #[test]
    fn test_decode_uppercase_hex() {
        // &#X41; = A
        let mut val = Value::String("&#X41;BC".into());
        decode_entities(&mut val);
        assert_eq!(val.as_str(), Some("ABC"));
    }

    #[test]
    fn test_strip_comments() {
        let input = r#"{
            // line comment
            "key": "value" /* block */
        }"#;
        let cleaned = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&cleaned).ok().unwrap_or_default();
        assert_eq!(parsed.get("key").and_then(Value::as_str), Some("value"));
    }

    #[test]
    fn test_no_jsonld() {
        let html_str = "<html><body>No JSON-LD here</body></html>";
        let doc = Html::parse_document(html_str);
        assert!(extract_schema_org(&doc).is_none());
    }

    #[test]
    fn test_cdata_stripping() {
        let input = r#"<![CDATA[{"name": "test"}]]>"#;
        let cleaned = strip_json_comments(input);
        let parsed: Value = serde_json::from_str(&cleaned).ok().unwrap_or_default();
        assert_eq!(parsed.get("name").and_then(Value::as_str), Some("test"));
    }

    #[test]
    fn test_recursive_search() {
        let data: Value = serde_json::from_str(
            r#"[{"@type": "WebPage"}, {"@type": "Article", "author": {"name": "Bob"}}]"#,
        )
        .ok()
        .unwrap_or_default();
        assert_eq!(get_property(&data, "author.name"), Some("Bob".to_string()));
    }

    #[test]
    fn test_get_text_from_array() {
        let data: Value = serde_json::from_str(
            r#"[{"@type": "WebPage"}, {"@type": "Article", "text": "Content"}]"#,
        )
        .ok()
        .unwrap_or_default();
        assert_eq!(get_text(&data), Some("Content".to_string()));
    }
}
