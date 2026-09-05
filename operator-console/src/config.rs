use std::{env, net::IpAddr, time::Duration};

use anyhow::{Context, Result, bail};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Environment {
    fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            "prod" | "production" => Ok(Self::Production),
            other => bail!("unsupported APP_ENV value: {other}"),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Test => "test",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub host: String,
    pub port: u16,
    pub api_base_url: Url,
    pub tenant_id: Uuid,
    pub request_timeout: Duration,
    pub response_limit_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let environment = Environment::parse(
            &env::var("APP_ENV").unwrap_or_else(|_| "development".to_owned()),
        )?;
        if environment == Environment::Production {
            bail!(
                "production startup blocked: Shared Auth session propagation is not certified for the operator console"
            );
        }

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8081".to_owned())
            .parse::<u16>()
            .context("PORT must be a valid TCP port")?;
        let api_base_url = validate_api_url(
            &env::var("EAL_API_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned()),
            environment,
        )?;
        let tenant_id = env::var("EAL_TENANT_ID")
            .context("EAL_TENANT_ID is required outside Shared Auth")?
            .parse::<Uuid>()
            .context("EAL_TENANT_ID must be a UUID")?;
        let timeout_seconds = env::var("EAL_API_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "20".to_owned())
            .parse::<u64>()
            .context("EAL_API_TIMEOUT_SECONDS must be an integer")?;
        if !(1..=120).contains(&timeout_seconds) {
            bail!("EAL_API_TIMEOUT_SECONDS must be between 1 and 120");
        }
        let response_limit_bytes = env::var("EAL_API_RESPONSE_LIMIT_BYTES")
            .unwrap_or_else(|_| (2 * 1024 * 1024).to_string())
            .parse::<usize>()
            .context("EAL_API_RESPONSE_LIMIT_BYTES must be an integer")?;
        if !(64 * 1024..=16 * 1024 * 1024).contains(&response_limit_bytes) {
            bail!("EAL_API_RESPONSE_LIMIT_BYTES must be between 65536 and 16777216");
        }

        Ok(Self {
            environment,
            host,
            port,
            api_base_url,
            tenant_id,
            request_timeout: Duration::from_secs(timeout_seconds),
            response_limit_bytes,
        })
    }
}

fn validate_api_url(value: &str, environment: Environment) -> Result<Url> {
    let mut url = Url::parse(value.trim()).context("EAL_API_BASE_URL must be an absolute URL")?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        bail!("EAL_API_BASE_URL must not contain credentials, query parameters, or fragments");
    }
    let host = url
        .host_str()
        .context("EAL_API_BASE_URL must contain a host")?;
    match url.scheme() {
        "https" => {}
        "http" if environment != Environment::Production && is_loopback_host(host) => {}
        "http" => bail!("plain HTTP is allowed only for loopback development APIs"),
        _ => bail!("EAL_API_BASE_URL must use HTTPS"),
    }
    if url.path() != "/" && !url.path().is_empty() {
        bail!("EAL_API_BASE_URL must not contain a path prefix");
    }
    url.set_path("/");
    Ok(url)
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_allows_loopback_http() {
        assert!(validate_api_url("http://127.0.0.1:8080", Environment::Development).is_ok());
    }

    #[test]
    fn development_rejects_remote_http() {
        assert!(validate_api_url("http://example.com", Environment::Development).is_err());
    }

    #[test]
    fn api_url_rejects_embedded_credentials() {
        assert!(
            validate_api_url(
                "https://operator:secret@example.com",
                Environment::Development,
            )
            .is_err()
        );
    }
}
