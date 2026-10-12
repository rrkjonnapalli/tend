//! The few HTTP GETs Tend makes (a link's page title, a dictionary entry), through one `ureq`
//! setup: a short timeout, a user agent, and only the first `limit` bytes read.

use std::io::Read;
use std::time::Duration;

use crate::error::{AppResult, ErrorDef};

const TIMEOUT: Duration = Duration::from_secs(5);
const USER_AGENT: &str = "Tend";
const NOT_FOUND: u16 = 404;

/// The body's first `limit` bytes as text, or `None` when the server says 404. Anything else that
/// fails is `fail` with its own message: the transport's wording ("timeout: global") means nothing
/// to the person reading the toast.
pub fn get(url: &str, limit: u64, fail: ErrorDef) -> AppResult<Option<String>> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .into();
    let mut response = match agent.get(url).header("User-Agent", USER_AGENT).call() {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(NOT_FOUND)) => return Ok(None),
        Err(_) => return Err(fail.err()),
    };
    let mut body = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(limit)
        .read_to_end(&mut body)
        .map_err(|_| fail.err())?;
    Ok(Some(String::from_utf8_lossy(&body).into_owned()))
}
