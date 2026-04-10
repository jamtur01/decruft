use std::time::Duration;

/// Connection and read timeout for API requests (10 seconds).
const TIMEOUT: Duration = Duration::from_secs(10);

/// Fetch a URL and return the response body as a string.
///
/// Times out after 10 seconds. Returns `None` on network errors,
/// non-200 status, timeout, or body read failures.
pub(crate) fn get(url: &str) -> Option<String> {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(TIMEOUT))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let response = agent
        .get(url)
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
/// Times out after 10 seconds. Returns `None` on network errors,
/// non-200 status, timeout, or body read failures.
pub(crate) fn get_with_headers(url: &str, headers: &[(&str, &str)]) -> Option<String> {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(TIMEOUT))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let mut request = agent.get(url).header("User-Agent", "decruft/0.1");
    for &(name, value) in headers {
        request = request.header(name, value);
    }
    let response = request.call().ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}
