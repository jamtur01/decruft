use std::time::Duration;

/// Timeout for API requests to known services (GitHub, HN, etc.).
const API_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for fetching arbitrary web pages.
const PAGE_TIMEOUT: Duration = Duration::from_secs(30);

/// User-Agent for API requests to known services.
const API_USER_AGENT: &str = "decruft/0.1";

/// Browser-like User-Agent for fetching arbitrary web pages.
const PAGE_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

fn build_agent(timeout: Duration) -> ureq::Agent {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(timeout))
        .build();
    ureq::Agent::new_with_config(config)
}

/// Fetch a URL using API defaults (10s timeout, `decruft/0.1` UA).
///
/// Returns `None` on network errors, non-200 status, timeout, or
/// body read failures.
pub(crate) fn get(url: &str) -> Option<String> {
    let agent = build_agent(API_TIMEOUT);
    let response = agent
        .get(url)
        .header("User-Agent", API_USER_AGENT)
        .call()
        .ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}

/// Fetch a URL with custom headers using API defaults.
///
/// Returns `None` on network errors, non-200 status, timeout, or
/// body read failures.
pub(crate) fn get_with_headers(url: &str, headers: &[(&str, &str)]) -> Option<String> {
    let agent = build_agent(API_TIMEOUT);
    let mut request = agent.get(url).header("User-Agent", API_USER_AGENT);
    for &(name, value) in headers {
        request = request.header(name, value);
    }
    let response = request.call().ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}

/// Fetch a web page (30s timeout, browser-like UA).
///
/// Returns `None` on network errors, non-200 status, timeout, or
/// body read failures.
#[must_use]
pub fn fetch_page(url: &str) -> Option<String> {
    let agent = build_agent(PAGE_TIMEOUT);
    let response = agent
        .get(url)
        .header("User-Agent", PAGE_USER_AGENT)
        .call()
        .ok()?;
    if response.status() != 200 {
        return None;
    }
    response.into_body().read_to_string().ok()
}
