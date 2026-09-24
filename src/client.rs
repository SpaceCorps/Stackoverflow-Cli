//! HTTP client for the Apify StackOverflow scraper API, and the translation from HTTP status to [`ErrorCode`].
//!
//! One blocking agent per process: a CLI makes a handful of requests, so an async runtime
//! would cost more in startup than it could save.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://api.apify.com/v2/";
const DEFAULT_ACTOR: &str = "sheshinmcfly~stackoverflow-scraper";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub site: String,
    pub include_answers: bool,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub site: String,
    pub include_answers: bool,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    actor: String,
    api_key: String,
}

enum Method {
    Get,
    Post,
}

impl Client {
    pub fn new(api_key: &str) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("stackoverflow-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("APIFY_API_URL")
            .or_else(|_| std::env::var("STACKOVERFLOW_API_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        let actor = std::env::var("APIFY_ACTOR_ID")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ACTOR.to_string());

        Client { agent, base, actor, api_key: api_key.trim().to_string() }
    }

    pub fn search(&self, input: &SearchInput) -> Result<Value> {
        let path = format!("acts/{}/run-sync-get-dataset-items", self.actor);
        let body = serde_json::to_value(input).map_err(|e| Error::other(e.to_string()))?;
        self.send(Method::Post, &path, Some(&body))
    }

    pub fn scrape(&self, input: &ScrapeInput) -> Result<Value> {
        let path = format!("acts/{}/run-sync-get-dataset-items", self.actor);
        let body = serde_json::to_value(input).map_err(|e| Error::other(e.to_string()))?;
        self.send(Method::Post, &path, Some(&body))
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    #[allow(dead_code)]
    pub fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.send(Method::Post, path, Some(body))
    }

    /// Verifies that an API key works by querying the Apify account endpoint.
    pub fn test_key(api_key: &str) -> Result<()> {
        let client = Client::new(api_key);
        client.get("users/me").map(|_| ())
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.trim_start_matches('/');
        let separator = if clean_path.contains('?') { "&" } else { "?" };
        let url = format!("{}{}{}token={}", self.base, clean_path, separator, self.api_key);

        let bearer = format!("Bearer {}", self.api_key);
        macro_rules! headers {
            ($req:expr) => {{ $req.header("Authorization", &bearer).header("Accept", "application/json") }};
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Post, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send(&json[..])
            }
            (Method::Post, None) => {
                headers!(self.agent.post(&url)).header("Content-Type", "application/json").send_empty()
            }
        };

        let response = result.map_err(transport_error)?;
        read(response)
    }
}

fn read(mut response: Response<ureq::Body>) -> Result<Value> {
    let status = response.status().as_u16();
    let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;

    if !(200..300).contains(&status) {
        let body = String::from_utf8_lossy(&bytes).trim().to_string();
        return Err(status_error(status, &body));
    }

    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Value::Array(Vec::new()));
    }

    serde_json::from_slice(&bytes).or_else(|_| Ok(Value::String(String::from_utf8_lossy(&bytes).into_owned())))
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Apify API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str) -> Error {
    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    let parsed_error = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.get("error")
            .and_then(|e| {
                if let Some(msg) = e.get("message").and_then(Value::as_str) {
                    Some(msg.to_string())
                } else {
                    e.as_str().map(str::to_string)
                }
            })
            .or_else(|| v.get("message").and_then(Value::as_str).map(str::to_string))
    });

    let e = match status {
        401 => Error::new(ErrorCode::AuthRequired, "The Apify API token was rejected.")
            .fix("Set APIFY_TOKEN, use --api-key <key>, or run: stackoverflow login"),
        403 => Error::new(ErrorCode::AuthRequired, "The Apify API token is not permitted to perform this action.")
            .fix("Verify permissions for this token at https://console.apify.com/account/integrations"),
        404 => Error::new(ErrorCode::NotFound, "The requested resource was not found on Apify."),
        429 => Error::new(ErrorCode::RateLimited, "Rate limited by the Apify API.")
            .fix("Wait before retrying or check your usage at https://console.apify.com"),
        400 | 422 => {
            let msg = parsed_error.unwrap_or_else(|| "The Apify API refused the request.".to_string());
            Error::new(ErrorCode::InvalidInput, msg)
        }
        s if s >= 500 => Error::new(ErrorCode::Network, "The Apify API returned a server error.")
            .fix("Retry; if it persists, check https://status.apify.com"),
        _ => Error::new(ErrorCode::Error, "The request failed."),
    };
    e.detail(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_codes() {
        assert_eq!(status_error(401, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(403, "").code, ErrorCode::AuthRequired);
        assert_eq!(status_error(404, "").code, ErrorCode::NotFound);
        assert_eq!(status_error(429, "").code, ErrorCode::RateLimited);
        assert_eq!(status_error(400, r#"{"error":{"message":"bad query"}}"#).code, ErrorCode::InvalidInput);
        assert_eq!(status_error(422, "").code, ErrorCode::InvalidInput);
        assert_eq!(status_error(500, "").code, ErrorCode::Network);
        assert_eq!(status_error(502, "").code, ErrorCode::Network);
    }
}
