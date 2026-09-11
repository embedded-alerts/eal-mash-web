use std::{fmt, sync::Arc};

use futures_util::StreamExt;
use reqwest::{Client, Method, StatusCode, Url, redirect::Policy};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{SemanticEvaluatePayload, SemanticSearchPayload},
};

#[derive(Clone)]
pub struct ApiClient {
    inner: Arc<Inner>,
}

struct Inner {
    client: Client,
    base_url: Url,
    tenant_id: Uuid,
    response_limit_bytes: usize,
}

impl ApiClient {
    pub fn new(config: &Config) -> Result<Self, ApiClientError> {
        let client = Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            .connect_timeout(config.request_timeout.min(std::time::Duration::from_secs(10)))
            .timeout(config.request_timeout)
            .user_agent("embedded-alerts-operator-console/0.1")
            .build()
            .map_err(|error| ApiClientError::configuration(error.to_string()))?;
        Ok(Self {
            inner: Arc::new(Inner {
                client,
                base_url: config.api_base_url.clone(),
                tenant_id: config.tenant_id,
                response_limit_bytes: config.response_limit_bytes,
            }),
        })
    }

    pub async fn health(&self) -> Result<Value, ApiClientError> {
        self.request::<Value>(Method::GET, "/healthz", None)
            .await
    }

    pub async fn list_sources(&self) -> Result<Value, ApiClientError> {
        self.request::<Value>(Method::GET, "/v1/sources", None)
            .await
    }

    pub async fn create_source(&self, policy: Value) -> Result<Value, ApiClientError> {
        self.request(Method::POST, "/v1/sources", Some(&policy))
            .await
    }

    pub async fn semantic_search(
        &self,
        request: &SemanticSearchPayload,
    ) -> Result<Value, ApiClientError> {
        self.request(Method::POST, "/v1/semantic/search", Some(request))
            .await
    }

    pub async fn semantic_evaluate(
        &self,
        request: &SemanticEvaluatePayload,
    ) -> Result<Value, ApiClientError> {
        self.request(Method::POST, "/v1/semantic/evaluate", Some(request))
            .await
    }

    async fn request<T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<Value, ApiClientError>
    where
        T: Serialize + ?Sized,
    {
        let url = self
            .inner
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| ApiClientError::configuration(error.to_string()))?;
        let mut request = self
            .inner
            .client
            .request(method, url)
            .header("x-eal-tenant-id", self.inner.tenant_id.to_string())
            .header(reqwest::header::ACCEPT, "application/json");
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .map_err(|error| ApiClientError::transport(error.to_string()))?;
        let status = response.status();
        if status.is_redirection() {
            return Err(ApiClientError::response(
                status,
                "API redirects are blocked",
            ));
        }
        if response
            .content_length()
            .is_some_and(|length| length > self.inner.response_limit_bytes as u64)
        {
            return Err(ApiClientError::response(
                status,
                "API response exceeded the configured size limit",
            ));
        }

        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| ApiClientError::transport(error.to_string()))?;
            if body.len().saturating_add(chunk.len()) > self.inner.response_limit_bytes {
                return Err(ApiClientError::response(
                    status,
                    "API response exceeded the configured size limit",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let value = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice::<Value>(&body).map_err(|error| {
                ApiClientError::response(
                    status,
                    format!("API returned invalid JSON: {error}"),
                )
            })?
        };
        if !status.is_success() {
            return Err(ApiClientError::response(status, safe_error_message(&value)));
        }
        Ok(value)
    }
}

fn safe_error_message(value: &Value) -> String {
    let code = value
        .get("code")
        .and_then(Value::as_str)
        .map(sanitize)
        .unwrap_or_else(|| "api_error".to_owned());
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .map(sanitize)
        .unwrap_or_else(|| "the API rejected the request".to_owned());
    format!("{code}: {message}")
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(300)
        .collect()
}

#[derive(Debug)]
pub struct ApiClientError {
    kind: ApiClientErrorKind,
    status: Option<StatusCode>,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum ApiClientErrorKind {
    Configuration,
    Transport,
    Response,
}

impl ApiClientError {
    fn configuration(message: impl Into<String>) -> Self {
        Self {
            kind: ApiClientErrorKind::Configuration,
            status: None,
            message: message.into(),
        }
    }

    fn transport(message: impl Into<String>) -> Self {
        Self {
            kind: ApiClientErrorKind::Transport,
            status: None,
            message: message.into(),
        }
    }

    fn response(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            kind: ApiClientErrorKind::Response,
            status: Some(status),
            message: message.into(),
        }
    }

    pub fn status(&self) -> Option<StatusCode> {
        self.status
    }
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ApiClientErrorKind::Configuration => {
                write!(formatter, "operator API configuration error: {}", self.message)
            }
            ApiClientErrorKind::Transport => {
                write!(formatter, "operator API transport error: {}", self.message)
            }
            ApiClientErrorKind::Response => {
                if let Some(status) = self.status {
                    write!(formatter, "operator API returned {}: {}", status, self.message)
                } else {
                    formatter.write_str(&self.message)
                }
            }
        }
    }
}

impl std::error::Error for ApiClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages_are_bounded_and_control_free() {
        let value = Value::String("x\n".repeat(500));
        let sanitized = sanitize(value.as_str().expect("string"));
        assert!(sanitized.len() <= 300);
        assert!(!sanitized.contains('\n'));
    }
}
