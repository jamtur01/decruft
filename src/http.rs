/// Fetch a URL and return the response body as a string.
///
/// Returns `None` on network errors, non-200 status, or body read
/// failures.
pub(crate) fn get(url: &str) -> Option<String> {
    let response = ureq::get(url)
        .header("User-Agent", "decruft/0.1")
        .call()
        .ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}

/// Fetch a URL with custom headers and return the response body.
///
/// Returns `None` on network errors, non-200 status, or body read
/// failures.
pub(crate) fn get_with_headers(url: &str, headers: &[(&str, &str)]) -> Option<String> {
    let mut request = ureq::get(url).header("User-Agent", "decruft/0.1");
    for &(name, value) in headers {
        request = request.header(name, value);
    }
    let response = request.call().ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}
