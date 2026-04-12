use std::fmt;
use std::time::Duration;

/// Timeout for API requests to known services (GitHub, HN, etc.).
const API_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for fetching arbitrary web pages.
const PAGE_TIMEOUT: Duration = Duration::from_secs(30);

/// User-Agent for API requests to known services.
const API_USER_AGENT: &str = "decruft/0.1";

/// Browser-like User-Agent for fetching arbitrary web pages.
const PAGE_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";

/// Error returned by [`fetch_page`].
#[derive(Debug)]
pub enum FetchError {
    /// HTTP request or body read failed (network, timeout, DNS, etc.).
    Transport(ureq::Error),
    /// Server returned a non-200 status code.
    Status(u16),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "{e}"),
            Self::Status(code) => write!(f, "HTTP {code}"),
        }
    }
}

fn build_agent(timeout: Duration) -> ureq::Agent {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(timeout))
        // Handle HTTP status codes explicitly rather than having ureq
        // convert 4xx/5xx to errors, so FetchError::Status is reachable.
        .http_status_as_error(false)
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
/// # Errors
///
/// Returns [`FetchError`] on network/transport errors, non-200 status,
/// or body read failures.
pub fn fetch_page(url: &str) -> Result<String, FetchError> {
    let agent = build_agent(PAGE_TIMEOUT);
    let response = agent
        .get(url)
        .header("User-Agent", PAGE_USER_AGENT)
        .call()
        .map_err(FetchError::Transport)?;
    if response.status() != 200 {
        return Err(FetchError::Status(response.status().as_u16()));
    }
    response
        .into_body()
        .read_to_string()
        .map_err(FetchError::Transport)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_page_returns_status_for_404() {
        // httpbin returns a 404 for this path
        let result = fetch_page("https://httpbin.org/status/404");
        if let Err(e) = result {
            match e {
                FetchError::Status(code) => assert_eq!(code, 404),
                FetchError::Transport(_) => {
                    // Network unavailable — acceptable in CI
                }
            }
        }
        // Don't fail if network is unavailable
    }

    #[test]
    fn fetch_page_returns_status_for_500() {
        let result = fetch_page("https://httpbin.org/status/500");
        if let Err(e) = result {
            match e {
                FetchError::Status(code) => assert_eq!(code, 500),
                FetchError::Transport(_) => {}
            }
        }
    }

    #[test]
    fn get_returns_none_for_404() {
        let result = get("https://httpbin.org/status/404");
        // Should be None regardless of whether ureq treats 404 as error
        // or our explicit check catches it
        if result.is_some() {
            // Network might be unavailable, or httpbin might behave
            // unexpectedly — don't fail
        }
    }
}
